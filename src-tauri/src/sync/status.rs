use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::events::Ui;

/// État des sources, affiché en bas de la bibliothèque et dans les réglages.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub steam_local: String,
    pub steam_watching: bool,
    pub steam_web: String,
    pub steam_web_importing: bool,
    pub playstation: String,
    pub playstation_connected: bool,
    pub syncing: bool,
}

pub struct StatusBoard {
    status: Mutex<SyncStatus>,
    ui: Arc<dyn Ui>,
}

impl StatusBoard {
    pub fn new(ui: Arc<dyn Ui>) -> Self {
        StatusBoard { status: Mutex::default(), ui }
    }

    pub fn current(&self) -> SyncStatus {
        self.status.lock().expect("état accessible").clone()
    }

    pub fn update(&self, change: impl FnOnce(&mut SyncStatus)) {
        let status = {
            let mut status = self.status.lock().expect("état accessible");
            change(&mut status);
            status.clone()
        };
        self.ui.status_changed(&status);
    }
}
