use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crate::events::{Announcer, Ui};
use crate::settings::SettingsStore;
use chrono::Utc;
use prisma_core::domain::{Game, GameId, Platform, TrophyRef, TrophyTier};
use prisma_core::library::{Change, GameGrouper, LibraryRepository, SourceCache, StoredLibrary};

const SAVE_DELAY: Duration = Duration::from_secs(1);
/// Regroupe les rafales de modifications, par exemple pendant un import, en une seule mise à jour de l'interface.
const PUBLISH_DELAY: Duration = Duration::from_millis(150);

/// Source de vérité de l'app : bibliothèque stockée, entrées regroupées pour l'affichage et sauvegarde.
pub struct LibraryStore {
    state: Mutex<State>,
    repository: LibraryRepository,
    settings: Arc<SettingsStore>,
    announcer: Arc<dyn Announcer>,
    ui: Arc<dyn Ui>,
    pending_save: Debounced,
    pending_publish: Debounced,
}

struct State {
    stored: StoredLibrary,
    grouper: GameGrouper,
    /// Entrées affichées, recalculées à la demande après une modification et partagées sans copie.
    entries: Option<Arc<Vec<Game>>>,
}

impl LibraryStore {
    pub fn new(repository: LibraryRepository, settings: Arc<SettingsStore>, announcer: Arc<dyn Announcer>, ui: Arc<dyn Ui>) -> Self {
        LibraryStore {
            state: Mutex::new(State { stored: StoredLibrary::default(), grouper: GameGrouper::default(), entries: None }),
            repository,
            settings,
            announcer,
            ui,
            pending_save: Debounced::new(SAVE_DELAY),
            pending_publish: Debounced::new(PUBLISH_DELAY),
        }
    }

    pub fn load(&self) {
        let stored = self.repository.load().unwrap_or_else(|error| {
            log::error!("{error}");
            self.quarantine_unreadable_library();
            StoredLibrary::default()
        });
        let mut state = self.state();
        state.stored = stored;
        state.entries = None;
    }

    /// Entrées affichées, des derniers trophées obtenus aux plus anciens.
    pub fn entries(&self) -> Arc<Vec<Game>> {
        let mode = self.settings.get().grouping_mode;
        let mut state = self.state();
        let State { stored, grouper, entries } = &mut *state;
        entries
            .get_or_insert_with(|| {
                let mut grouped = grouper.group(stored.library.games(), mode);
                grouped.sort_by(|lhs, rhs| {
                    let by_title = || lhs.title.to_lowercase().cmp(&rhs.title.to_lowercase());
                    rhs.last_earned_at().cmp(&lhs.last_earned_at()).then_with(by_title)
                });
                Arc::new(grouped)
            })
            .clone()
    }

    /// Jeu stocké ou entrée regroupée.
    pub fn game(&self, id: &GameId) -> Option<Game> {
        if let Some(game) = self.state().stored.library.game(id) {
            return Some(game.clone());
        }
        self.entries().iter().find(|entry| &entry.id == id).cloned()
    }

    pub fn stored_games_on(&self, platform: Platform) -> Vec<Game> {
        self.state().stored.library.games_on(platform).cloned().collect()
    }

    pub fn source_cache(&self) -> SourceCache {
        self.state().stored.cache.clone()
    }

    pub fn has_tier_override(&self, reference: &TrophyRef) -> bool {
        self.state().stored.library.has_tier_override(reference)
    }

    /// Enregistre un jeu lu depuis sa source ; `announcing` notifie les trophées obtenus depuis la dernière lecture.
    pub fn record(self: &Arc<Self>, game: Game, announcing: bool) {
        let id = game.id.clone();
        let change = self.state().stored.library.integrate(game, Utc::now());
        self.apply(change, &id, announcing);
    }

    pub fn set_earned(self: &Arc<Self>, earned: bool, reference: &TrophyRef) {
        let change = self.state().stored.library.set_earned(earned, reference, Utc::now());
        self.apply(change, &reference.game_id, true);
    }

    pub fn set_tier(self: &Arc<Self>, tier: Option<TrophyTier>, reference: &TrophyRef) {
        if self.state().stored.library.set_tier(tier, reference) {
            self.did_mutate();
        }
    }

    pub fn remove(self: &Arc<Self>, id: &GameId) {
        if self.state().stored.library.remove(id) {
            self.did_mutate();
        }
    }

    pub fn remove_all_on(self: &Arc<Self>, platform: Platform) {
        {
            let mut state = self.state();
            state.stored.library.remove_all_on(platform);
            if platform == Platform::Playstation {
                state.stored.cache.playstation_revisions.clear();
            }
        }
        self.did_mutate();
    }

    pub fn update_cache(self: &Arc<Self>, update: impl FnOnce(&mut SourceCache)) {
        update(&mut self.state().stored.cache);
        self.schedule_save();
    }

    /// Recalcule les entrées affichées, par exemple après un changement du mode de regroupement.
    pub fn regroup(self: &Arc<Self>) {
        self.state().entries = None;
        self.schedule_publish();
    }

    pub fn save_now(&self) {
        let stored = self.state().stored.clone();
        if let Err(error) = self.repository.save(&stored) {
            log::error!("Sauvegarde impossible : {error}");
        }
    }

    fn apply(self: &Arc<Self>, change: Change, id: &GameId, announcing: bool) {
        if !change.did_change {
            return;
        }
        self.did_mutate();
        if !announcing || change.newly_earned.is_empty() {
            return;
        }
        let title = self.state().stored.library.game(id).map(|game| game.title.clone()).unwrap_or_default();
        self.announcer.announce(change.newly_earned, &title);
    }

    fn did_mutate(self: &Arc<Self>) {
        self.state().entries = None;
        self.schedule_publish();
        self.schedule_save();
    }

    fn schedule_save(self: &Arc<Self>) {
        let store = Arc::clone(self);
        self.pending_save.schedule(move || store.save_now());
    }

    fn schedule_publish(self: &Arc<Self>) {
        let ui = Arc::clone(&self.ui);
        self.pending_publish.schedule(move || ui.library_changed());
    }

    fn quarantine_unreadable_library(&self) {
        match self.repository.quarantine_unreadable_file(Utc::now()) {
            Ok(backup) => log::warn!("Bibliothèque illisible conservée dans {}", backup.display()),
            Err(error) => log::error!("Mise de côté impossible : {error}"),
        }
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().expect("bibliothèque accessible")
    }
}

/// Exécute une action après un délai, une seule fois quel que soit le nombre de demandes entre-temps.
struct Debounced {
    is_scheduled: Arc<AtomicBool>,
    delay: Duration,
}

impl Debounced {
    fn new(delay: Duration) -> Self {
        Debounced { is_scheduled: Arc::new(AtomicBool::new(false)), delay }
    }

    fn schedule(&self, action: impl FnOnce() + Send + 'static) {
        if self.is_scheduled.swap(true, Ordering::AcqRel) {
            return;
        }
        let is_scheduled = Arc::clone(&self.is_scheduled);
        let delay = self.delay;
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(delay).await;
            is_scheduled.store(false, Ordering::Release);
            action();
        });
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use prisma_core::domain::{Game, GameId, Platform, Trophy, TrophyRef, TrophyTier, system};
    use prisma_core::library::GroupingMode;
    use prisma_core::switch;

    use crate::testing::TestApp;

    fn playstation(id: &str, title: &str, earned: bool) -> Game {
        let trophy = Trophy::new("1", "Premier pas", "", TrophyTier::Bronze);
        let trophy = if earned { trophy.earned(Some(chrono::Utc::now())) } else { trophy };
        Game::new(GameId::playstation(id), Platform::Playstation, title, vec![system::PS5.into()], vec![trophy])
    }

    #[test]
    fn trophy_earned_since_last_read_is_announced() {
        let directory = tempfile::tempdir().unwrap();
        let app = TestApp::new(directory.path());
        app.store.record(playstation("A", "Astro", false), true);

        app.store.record(playstation("A", "Astro", true), true);

        assert_eq!(app.announcer.announced(), [("Astro".to_owned(), vec!["1".to_owned()])]);
    }

    #[test]
    fn silent_reads_announce_nothing() {
        let directory = tempfile::tempdir().unwrap();
        let app = TestApp::new(directory.path());
        app.store.record(playstation("A", "Astro", false), false);

        app.store.record(playstation("A", "Astro", true), false);

        assert!(app.announcer.announced().is_empty());
        assert!(app.store.game(&GameId::playstation("A")).unwrap().trophies[0].is_earned);
    }

    #[test]
    fn checking_last_switch_trophy_announces_it_with_the_platinum() {
        let directory = tempfile::tempdir().unwrap();
        let app = TestApp::new(directory.path());
        let entry = switch::custom_entry("Tiny Isles");
        let game = switch::make_game(&entry);
        app.store.record(game.clone(), false);

        for trophy in game.regular_trophies() {
            app.store.set_earned(true, &TrophyRef::new(game.id.clone(), trophy.id.clone()));
        }

        let announced = app.announcer.announced();
        assert_eq!(announced.len(), game.regular_trophies().count());
        assert_eq!(announced.last().unwrap().1.last().map(String::as_str), Some("__platinum__"));
    }

    #[test]
    fn a_burst_of_changes_updates_the_interface_once() {
        let directory = tempfile::tempdir().unwrap();
        let app = TestApp::new(directory.path());

        for index in 0..20 {
            app.store.record(playstation(&index.to_string(), "Jeu", false), false);
        }
        std::thread::sleep(super::PUBLISH_DELAY + Duration::from_millis(300));

        assert_eq!(app.ui.library_changes(), 1);
    }

    #[test]
    fn saved_library_is_read_back() {
        let directory = tempfile::tempdir().unwrap();
        let app = TestApp::new(directory.path());
        app.store.record(playstation("A", "Astro", true), false);
        app.store.save_now();

        let reopened = TestApp::new(directory.path());
        reopened.store.load();

        assert!(reopened.store.game(&GameId::playstation("A")).is_some());
    }

    #[test]
    fn unreadable_library_is_set_aside_instead_of_overwritten() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("library.json"), "{").unwrap();
        let app = TestApp::new(directory.path());

        app.store.load();

        assert!(app.store.entries().is_empty());
        let names: Vec<_> = std::fs::read_dir(directory.path()).unwrap().map(|entry| entry.unwrap().file_name()).collect();
        assert!(names.iter().any(|name| name.to_string_lossy().starts_with("library.unreadable-")));
    }

    #[test]
    fn entries_follow_the_grouping_setting() {
        let directory = tempfile::tempdir().unwrap();
        let app = TestApp::new(directory.path());
        app.store.record(playstation("A", "Astro (PS4)", false), false);
        app.store.record(playstation("B", "Astro", false), false);
        assert_eq!(app.store.entries().len(), 1);

        let mut settings = app.settings.get();
        settings.grouping_mode = GroupingMode::None;
        app.settings.replace(settings);
        app.store.regroup();

        assert_eq!(app.store.entries().len(), 2);
    }
}
