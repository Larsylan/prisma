use std::sync::Arc;

use prisma_core::library::LibraryRepository;
use prisma_core::playstation::PsnAuthenticator;
use prisma_core::steam::SteamWebApi;
use tauri::AppHandle;

use crate::environment::Environment;
use crate::events::{TauriUi, Ui};
use crate::notifications::Notifier;
use crate::settings::SettingsStore;
use crate::store::LibraryStore;
use crate::sync::{
    PlayStationSync, StatusBoard, SteamLocalMonitor, SteamRecorder, SteamTitles, SteamWebImporter, SwitchLibrary, SyncCoordinator,
};

/// Racine de composition : crée une seule fois chaque service et relie leurs dépendances.
pub struct Services {
    pub environment: Environment,
    pub settings: Arc<SettingsStore>,
    pub store: Arc<LibraryStore>,
    pub notifier: Notifier,
    pub status: Arc<StatusBoard>,
    pub steam_recorder: Arc<SteamRecorder>,
    pub steam_web: Arc<SteamWebImporter>,
    pub playstation: Arc<PlayStationSync>,
    pub switch_library: Arc<SwitchLibrary>,
    pub coordinator: SyncCoordinator,
}

impl Services {
    pub fn new(app: &AppHandle) -> Self {
        let environment = Environment::from_process();
        let settings = Arc::new(SettingsStore::load(environment.data_directory.clone()));
        let notifier = Notifier::start(app.clone(), Arc::clone(&settings));
        let ui: Arc<dyn Ui> = Arc::new(TauriUi::new(app.clone()));
        let status = Arc::new(StatusBoard::new(Arc::clone(&ui)));
        let repository = LibraryRepository::new(environment.data_directory.clone());
        let store = Arc::new(LibraryStore::new(repository, Arc::clone(&settings), Arc::new(notifier.clone()), ui));

        let secrets = environment.secret_store();
        let steam_cache = environment.steam_cache();
        let steam_api = SteamWebApi::default();
        let titles = Arc::new(SteamTitles::new(steam_cache.clone(), steam_api.clone(), Arc::clone(&store)));
        let steam_recorder = Arc::new(SteamRecorder::new(titles, steam_api.clone(), Arc::clone(&store), Arc::clone(&settings)));
        let steam_local = Arc::new(SteamLocalMonitor::new(steam_cache.clone(), Arc::clone(&steam_recorder), Arc::clone(&status)));
        let steam_web = Arc::new(SteamWebImporter::new(
            steam_api,
            steam_cache,
            Arc::clone(&steam_recorder),
            Arc::clone(&settings),
            Arc::clone(&secrets),
            Arc::clone(&status),
        ));

        let authenticator = Arc::new(PsnAuthenticator::new(secrets));
        let playstation = Arc::new(PlayStationSync::new(authenticator, Arc::clone(&store), Arc::clone(&settings), Arc::clone(&status)));
        let switch_library = Arc::new(SwitchLibrary::new(Arc::clone(&store)));
        let coordinator = SyncCoordinator::new(
            steam_local,
            Arc::clone(&steam_web),
            Arc::clone(&playstation),
            Arc::clone(&switch_library),
            Arc::clone(&status),
        );

        Services { environment, settings, store, notifier, status, steam_recorder, steam_web, playstation, switch_library, coordinator }
    }

    pub fn start(&self) {
        self.store.load();
        self.coordinator.start();
    }
}
