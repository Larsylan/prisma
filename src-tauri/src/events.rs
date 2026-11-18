//! Événements envoyés à l'interface.

pub const LIBRARY_CHANGED: &str = "library-changed";
pub const STATUS_CHANGED: &str = "status-changed";
pub const NAVIGATE: &str = "navigate";
pub const TOAST_SHOW: &str = "toast-show";
pub const TOAST_HIDE: &str = "toast-hide";
pub const THEME_CHANGED: &str = "theme-changed";
pub const LANGUAGE_CHANGED: &str = "language-changed";

use prisma_core::domain::Trophy;
use tauri::{AppHandle, Emitter};

use crate::sync::SyncStatus;

/// Ce que les services signalent à l'interface. Une doublure en mémoire le remplace dans les tests.
pub trait Ui: Send + Sync {
    fn library_changed(&self);
    fn status_changed(&self, status: &SyncStatus);
}

/// Présente les trophées tout juste obtenus.
pub trait Announcer: Send + Sync {
    fn announce(&self, trophies: Vec<Trophy>, game_title: &str);
}

/// Envoie les événements aux fenêtres de l'app.
pub struct TauriUi {
    app: AppHandle,
}

impl TauriUi {
    pub fn new(app: AppHandle) -> Self {
        TauriUi { app }
    }

    fn emit(&self, event: &str, payload: impl serde::Serialize + Clone) {
        if let Err(error) = self.app.emit(event, payload) {
            log::error!("Mise à jour de l'interface impossible : {error}");
        }
    }
}

impl Ui for TauriUi {
    fn library_changed(&self) {
        self.emit(LIBRARY_CHANGED, ());
    }

    fn status_changed(&self, status: &SyncStatus) {
        self.emit(STATUS_CHANGED, status.clone());
    }
}
