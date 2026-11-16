use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use prisma_core::steam::{SteamClientCache, SteamLocalGame};
use tokio::sync::mpsc;

use super::status::StatusBoard;
use super::steam_recorder::SteamRecorder;

/// Vérification de secours au cas où un événement du système de fichiers serait manqué.
const POLL_INTERVAL: Duration = Duration::from_secs(2);
/// Steam peut être en train d'écrire le fichier au moment de la lecture.
const READ_RETRY_DELAY: Duration = Duration::from_millis(100);
const READ_ATTEMPTS: u32 = 7;

/// Détecte en temps réel les succès Steam en surveillant le cache local du client.
pub struct SteamLocalMonitor {
    steam_cache: Option<SteamClientCache>,
    recorder: Arc<SteamRecorder>,
    status: Arc<StatusBoard>,
    state: Mutex<MonitorState>,
    watcher: Mutex<Option<RecommendedWatcher>>,
}

#[derive(Default)]
struct MonitorState {
    account_id: Option<u32>,
    known_modification_dates: HashMap<u32, SystemTime>,
}

impl SteamLocalMonitor {
    pub fn new(steam_cache: Option<SteamClientCache>, recorder: Arc<SteamRecorder>, status: Arc<StatusBoard>) -> Self {
        SteamLocalMonitor { steam_cache, recorder, status, state: Mutex::default(), watcher: Mutex::default() }
    }

    pub fn start(self: &Arc<Self>) {
        self.scan();
        self.start_watching();
    }

    pub fn scan(self: &Arc<Self>) {
        let Some(cache) = self.steam_cache.as_ref() else {
            self.status.update(|status| status.steam_local = "Steam n'est pas installé sur cet ordinateur.".into());
            return;
        };
        let account_id = cache.current_account_id();
        if account_id.is_some() {
            self.state().account_id = account_id;
        }
        let app_ids = cache.app_ids_with_schema();
        for app_id in app_ids.iter().copied() {
            let monitor = Arc::clone(self);
            tauri::async_runtime::spawn(async move { monitor.refresh(app_id, false).await });
        }
        let persona = cache.current_user().map(|user| user.persona_name).filter(|name| !name.is_empty());
        let summary = format!("{} jeu(x) Steam détecté(s) en local", app_ids.len());
        self.status.update(|status| status.steam_local = persona.map_or(summary.clone(), |persona| format!("{summary} · {persona}")));
    }

    fn start_watching(self: &Arc<Self>) {
        let Some(cache) = self.steam_cache.as_ref().filter(|cache| cache.is_installed()) else { return };
        let (changes, mut changed_app_ids) = mpsc::unbounded_channel::<HashSet<u32>>();
        let watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            let Ok(event) = event else { return };
            let app_ids: HashSet<u32> = event
                .paths
                .iter()
                .filter_map(|path| SteamClientCache::app_id_from_file_name(&path.file_name()?.to_string_lossy()))
                .collect();
            if !app_ids.is_empty() {
                let _ = changes.send(app_ids);
            }
        });
        let mut watcher = match watcher {
            Ok(watcher) => watcher,
            Err(error) => return log::error!("Surveillance de Steam impossible : {error}"),
        };
        if let Err(error) = watcher.watch(&cache.stats_directory(), RecursiveMode::NonRecursive) {
            return log::error!("Surveillance de Steam impossible : {error}");
        }
        *self.watcher.lock().expect("surveillance accessible") = Some(watcher);
        self.status.update(|status| status.steam_watching = true);

        let monitor = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            while let Some(app_ids) = changed_app_ids.recv().await {
                for app_id in app_ids {
                    monitor.refresh(app_id, true).await;
                }
            }
        });

        let monitor = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            loop {
                tokio::time::sleep(POLL_INTERVAL).await;
                monitor.refresh_modified_games().await;
            }
        });
    }

    async fn refresh_modified_games(&self) {
        let (Some(cache), Some(account_id)) = (self.steam_cache.as_ref(), self.state().account_id) else { return };
        for app_id in cache.app_ids_with_schema() {
            let modified = cache.stats_modified_at(app_id, account_id);
            if modified.is_some() && self.state().known_modification_dates.get(&app_id) != modified.as_ref() {
                self.refresh(app_id, true).await;
            }
        }
    }

    async fn refresh(&self, app_id: u32, announcing: bool) {
        let Some(cache) = self.steam_cache.as_ref() else { return };
        let Some(account_id) = self.state().account_id.or_else(|| cache.current_account_id()) else { return };
        if let Some(modified) = cache.stats_modified_at(app_id, account_id) {
            self.state().known_modification_dates.insert(app_id, modified);
        }

        for attempt in 1..=READ_ATTEMPTS {
            if let Some(local) = cache.load_game(app_id, account_id) {
                return self.record(local, announcing);
            }
            if attempt < READ_ATTEMPTS {
                tokio::time::sleep(READ_RETRY_DELAY).await;
            }
        }
    }

    fn record(&self, local: SteamLocalGame, announcing: bool) {
        let title = self.recorder.titles.title(local.app_id, &local.schema_name);
        self.recorder.record(local.app_id, &title, &local.achievements, announcing);
    }

    fn state(&self) -> std::sync::MutexGuard<'_, MonitorState> {
        self.state.lock().expect("surveillance accessible")
    }
}

#[cfg(test)]
mod tests {
    use prisma_core::domain::GameId;
    use prisma_core::steam::SteamWebApi;
    use prisma_core::test_support::{FixtureAchievement, write_steam_directory, write_user_stats};

    use super::*;
    use crate::sync::SteamTitles;
    use crate::testing::TestApp;

    const ACCOUNT_ID: u32 = 12_345_678;
    const APP_ID: u32 = 49_520;

    fn achievements(second_unlocked_at: Option<i64>) -> [FixtureAchievement; 2] {
        [
            FixtureAchievement { api_name: "ACH_0", name: "Premier sang", unlocked_at: Some(1_600_000_000) },
            FixtureAchievement { api_name: "ACH_1", name: "Pourfendeur", unlocked_at: second_unlocked_at },
        ]
    }

    /// Le titre est déjà connu : aucune recherche n'est lancée sur la boutique Steam pendant le test.
    fn monitor(app: &TestApp, steam_root: &std::path::Path) -> Arc<SteamLocalMonitor> {
        app.store.update_cache(|cache| {
            cache.steam_titles.insert(APP_ID, "Borderlands 2".into());
        });
        let cache = SteamClientCache::new(steam_root);
        let titles = Arc::new(SteamTitles::new(Some(cache.clone()), SteamWebApi::default(), Arc::clone(&app.store)));
        let recorder = Arc::new(SteamRecorder::new(titles, SteamWebApi::default(), Arc::clone(&app.store), Arc::clone(&app.settings)));
        Arc::new(SteamLocalMonitor::new(Some(cache), recorder, Arc::clone(&app.status)))
    }

    #[tokio::test]
    async fn achievement_unlocked_while_watching_is_announced() {
        let (data, steam) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        write_steam_directory(steam.path(), ACCOUNT_ID, APP_ID, "ValveTestApp49520", &achievements(None)).unwrap();
        let app = TestApp::new(data.path());
        let monitor = monitor(&app, steam.path());
        monitor.refresh(APP_ID, false).await;
        assert!(app.announcer.announced().is_empty());

        write_user_stats(steam.path(), ACCOUNT_ID, APP_ID, &achievements(Some(chrono::Utc::now().timestamp()))).unwrap();
        monitor.refresh(APP_ID, true).await;

        assert_eq!(app.announcer.announced(), [("Borderlands 2".to_owned(), vec!["ACH_1".to_owned()])]);
        assert_eq!(app.store.game(&GameId::steam(APP_ID)).unwrap().earned_count(), 2);
    }

    #[test]
    fn scan_reports_detected_games_and_player() {
        let (data, steam) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        write_steam_directory(steam.path(), ACCOUNT_ID, APP_ID, "Borderlands", &achievements(None)).unwrap();
        let app = TestApp::new(data.path());

        monitor(&app, steam.path()).scan();

        assert_eq!(app.ui.last_status().unwrap().steam_local, "1 jeu(x) Steam détecté(s) en local · Alex");
    }

    #[test]
    fn missing_steam_is_reported() {
        let data = tempfile::tempdir().unwrap();
        let app = TestApp::new(data.path());
        let titles = Arc::new(SteamTitles::new(None, SteamWebApi::default(), Arc::clone(&app.store)));
        let recorder = Arc::new(SteamRecorder::new(titles, SteamWebApi::default(), Arc::clone(&app.store), Arc::clone(&app.settings)));

        Arc::new(SteamLocalMonitor::new(None, recorder, Arc::clone(&app.status))).scan();

        assert_eq!(app.ui.last_status().unwrap().steam_local, "Steam n'est pas installé sur cet ordinateur.");
    }
}
