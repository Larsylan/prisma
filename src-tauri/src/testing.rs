//! Doublures des dépendances de l'app pour les tests : elles enregistrent ce qu'on leur demande.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use prisma_core::domain::Trophy;
use prisma_core::library::LibraryRepository;

use crate::events::{Announcer, Ui};
use crate::settings::SettingsStore;
use crate::store::LibraryStore;
use crate::sync::{StatusBoard, SyncStatus};

#[derive(Default)]
pub struct RecordingUi {
    library_changes: AtomicUsize,
    statuses: Mutex<Vec<SyncStatus>>,
}

impl RecordingUi {
    pub fn library_changes(&self) -> usize {
        self.library_changes.load(Ordering::SeqCst)
    }

    pub fn last_status(&self) -> Option<SyncStatus> {
        self.statuses.lock().unwrap().last().cloned()
    }
}

impl Ui for RecordingUi {
    fn library_changed(&self) {
        self.library_changes.fetch_add(1, Ordering::SeqCst);
    }

    fn status_changed(&self, status: &SyncStatus) {
        self.statuses.lock().unwrap().push(status.clone());
    }
}

/// Trophées annoncés, avec le titre du jeu.
#[derive(Default)]
pub struct RecordingAnnouncer {
    announced: Mutex<Vec<(String, Vec<String>)>>,
}

impl RecordingAnnouncer {
    pub fn announced(&self) -> Vec<(String, Vec<String>)> {
        self.announced.lock().unwrap().clone()
    }
}

impl Announcer for RecordingAnnouncer {
    fn announce(&self, trophies: Vec<Trophy>, game_title: &str) {
        let ids = trophies.into_iter().map(|trophy| trophy.id).collect();
        self.announced.lock().unwrap().push((game_title.to_owned(), ids));
    }
}

/// Store branché sur des doublures, qui enregistre sa bibliothèque dans `directory`.
pub struct TestApp {
    pub store: Arc<LibraryStore>,
    pub settings: Arc<SettingsStore>,
    pub ui: Arc<RecordingUi>,
    pub announcer: Arc<RecordingAnnouncer>,
    pub status: Arc<StatusBoard>,
}

impl TestApp {
    pub fn new(directory: &Path) -> Self {
        let settings = Arc::new(SettingsStore::load(directory.to_path_buf()));
        let ui = Arc::new(RecordingUi::default());
        let announcer = Arc::new(RecordingAnnouncer::default());
        let store = Arc::new(LibraryStore::new(
            LibraryRepository::new(directory),
            Arc::clone(&settings),
            Arc::clone(&announcer) as Arc<dyn Announcer>,
            Arc::clone(&ui) as Arc<dyn Ui>,
        ));
        let status = Arc::new(StatusBoard::new(Arc::clone(&ui) as Arc<dyn Ui>));
        TestApp { store, settings, ui, announcer, status }
    }
}
