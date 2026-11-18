use std::sync::{Arc, Mutex};

use prisma_core::domain::Timestamp;
use prisma_core::playstation::PsnError;
use prisma_core::support::HttpError;
use serde::Serialize;

use crate::events::Ui;

/// État des sources, affiché en bas de la bibliothèque et dans les réglages. L'interface le formule dans sa langue.
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub steam_local: SteamLocalState,
    pub steam_watching: bool,
    pub steam_web: SteamWebState,
    pub playstation: PlayStationState,
    pub playstation_connected: bool,
    pub syncing: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum SteamLocalState {
    #[default]
    Unknown,
    NotInstalled,
    #[serde(rename_all = "camelCase")]
    Detected {
        game_count: usize,
        player: Option<String>,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum SteamWebState {
    #[default]
    Idle,
    #[serde(rename_all = "camelCase")]
    Importing {
        analyzed: usize,
        total: usize,
    },
    #[serde(rename_all = "camelCase")]
    Imported {
        game_count: usize,
    },
    Failed {
        failure: Failure,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum PlayStationState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Syncing,
    #[serde(rename_all = "camelCase")]
    Updating {
        updated: usize,
        total: usize,
    },
    #[serde(rename_all = "camelCase")]
    Synced {
        game_count: usize,
        at: Timestamp,
    },
    Failed {
        failure: Failure,
    },
}

/// Cause d'un échec, que l'interface explique à l'utilisateur.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "reason", rename_all = "camelCase")]
pub enum Failure {
    InvalidToken,
    NotConnected,
    Http {
        service: &'static str,
        status: u16,
    },
    /// Le détail vient de la bibliothèque réseau et n'est pas traduit.
    Network {
        service: &'static str,
        detail: String,
    },
}

impl From<&HttpError> for Failure {
    fn from(error: &HttpError) -> Self {
        match error {
            HttpError::Status { service, status } => Failure::Http { service, status: *status },
            HttpError::Transport { service, source } => Failure::Network { service, detail: source.to_string() },
        }
    }
}

impl From<&PsnError> for Failure {
    fn from(error: &PsnError) -> Self {
        match error {
            PsnError::InvalidNpsso => Failure::InvalidToken,
            PsnError::NotConnected => Failure::NotConnected,
            PsnError::Http(error) => error.into(),
        }
    }
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
