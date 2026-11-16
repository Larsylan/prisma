mod playstation;
mod status;
mod steam_local;
mod steam_recorder;
mod steam_titles;
mod steam_web;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use prisma_core::domain::Platform;
use prisma_core::switch::{self, CatalogEntry};

pub use playstation::PlayStationSync;
pub use status::{StatusBoard, SyncStatus};
pub use steam_local::SteamLocalMonitor;
pub use steam_recorder::SteamRecorder;
pub use steam_titles::SteamTitles;
pub use steam_web::SteamWebImporter;

use crate::store::LibraryStore;

/// Jeux Switch, ajoutés et cochés à la main.
pub struct SwitchLibrary {
    store: Arc<LibraryStore>,
}

impl SwitchLibrary {
    pub fn new(store: Arc<LibraryStore>) -> Self {
        SwitchLibrary { store }
    }

    pub fn contains(&self, entry: &CatalogEntry) -> bool {
        self.store.game(&entry.game_id()).is_some()
    }

    pub fn add(&self, entry: &CatalogEntry) {
        if !self.contains(entry) {
            self.store.record(switch::make_game(entry), false);
        }
    }

    /// Applique la version actuelle du catalogue aux jeux déjà ajoutés, sans perdre les trophées cochés.
    pub fn refresh_from_catalog(&self) {
        for game in self.store.stored_games_on(Platform::Nintendo) {
            if let Some(entry) = switch::entry_for(&game) {
                self.store.record(switch::make_game(&entry), false);
            }
        }
    }
}

pub struct SyncCoordinator {
    steam_local: Arc<SteamLocalMonitor>,
    steam_web: Arc<SteamWebImporter>,
    playstation: Arc<PlayStationSync>,
    switch_library: Arc<SwitchLibrary>,
    status: Arc<StatusBoard>,
    has_started: AtomicBool,
    is_syncing: AtomicBool,
}

impl SyncCoordinator {
    pub fn new(
        steam_local: Arc<SteamLocalMonitor>,
        steam_web: Arc<SteamWebImporter>,
        playstation: Arc<PlayStationSync>,
        switch_library: Arc<SwitchLibrary>,
        status: Arc<StatusBoard>,
    ) -> Self {
        SyncCoordinator {
            steam_local,
            steam_web,
            playstation,
            switch_library,
            status,
            has_started: AtomicBool::new(false),
            is_syncing: AtomicBool::new(false),
        }
    }

    pub fn start(&self) {
        if self.has_started.swap(true, Ordering::AcqRel) {
            return;
        }
        self.switch_library.refresh_from_catalog();
        self.steam_local.start();
        let (playstation, steam_web) = (Arc::clone(&self.playstation), Arc::clone(&self.steam_web));
        tauri::async_runtime::spawn(async move {
            playstation.start().await;
            steam_web.import_history().await;
        });
    }

    pub async fn sync_all(&self) {
        if self.is_syncing.swap(true, Ordering::AcqRel) {
            return;
        }
        self.status.update(|status| status.syncing = true);
        self.steam_local.scan();
        self.playstation.sync(false).await;
        self.steam_web.import_history().await;
        self.status.update(|status| status.syncing = false);
        self.is_syncing.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use prisma_core::domain::TrophyRef;

    use super::*;
    use crate::testing::TestApp;

    #[test]
    fn refreshing_the_catalog_keeps_checked_trophies() {
        let directory = tempfile::tempdir().unwrap();
        let app = TestApp::new(directory.path());
        let library = SwitchLibrary::new(Arc::clone(&app.store));
        let entry = switch::custom_entry("Tiny Isles");
        library.add(&entry);
        let trophy_id = switch::make_game(&entry).trophies[1].id.clone();
        app.store.set_earned(true, &TrophyRef::new(entry.game_id(), trophy_id.clone()));

        library.refresh_from_catalog();

        let game = app.store.game(&entry.game_id()).unwrap();
        assert!(game.trophies.iter().find(|trophy| trophy.id == trophy_id).unwrap().is_earned);
        assert!(library.contains(&entry));
    }

    #[test]
    fn adding_a_game_twice_keeps_a_single_copy() {
        let directory = tempfile::tempdir().unwrap();
        let app = TestApp::new(directory.path());
        let library = SwitchLibrary::new(Arc::clone(&app.store));
        let entry = &switch::entries()[0];

        library.add(entry);
        library.add(entry);

        assert_eq!(app.store.stored_games_on(Platform::Nintendo).len(), 1);
    }
}
