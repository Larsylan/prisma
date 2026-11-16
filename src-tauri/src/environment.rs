use std::path::PathBuf;
use std::sync::Arc;

use prisma_core::steam::SteamClientCache;
use prisma_core::support::secrets::{EphemeralSecretStore, SecretStore};

use crate::secrets::KeyringSecretStore;

pub const APP_IDENTIFIER: &str = "io.github.larsylan.prisma";
const DATA_DIRECTORY_VARIABLE: &str = "PRISMA_DATA_DIR";
const STEAM_ROOT_VARIABLE: &str = "PRISMA_STEAM_ROOT";

/// Emplacements utilisés par l'app, que des variables d'environnement peuvent rediriger pour le diagnostic.
pub struct Environment {
    pub data_directory: PathBuf,
    /// Une bibliothèque isolée sert au diagnostic : elle ne touche ni aux données ni au compte PSN réels.
    pub is_isolated: bool,
    steam_root: Option<PathBuf>,
}

impl Environment {
    pub fn from_process() -> Self {
        let isolated_directory = std::env::var_os(DATA_DIRECTORY_VARIABLE).map(PathBuf::from);
        Environment {
            is_isolated: isolated_directory.is_some(),
            data_directory: isolated_directory.unwrap_or_else(|| dirs::data_dir().unwrap_or_default().join("Prisma")),
            steam_root: std::env::var_os(STEAM_ROOT_VARIABLE).map(PathBuf::from),
        }
    }

    /// Client Steam de cet ordinateur, s'il est installé.
    pub fn steam_cache(&self) -> Option<SteamClientCache> {
        match &self.steam_root {
            Some(root) => Some(SteamClientCache::new(root)),
            None => SteamClientCache::locate(),
        }
    }

    pub fn secret_store(&self) -> Arc<dyn SecretStore> {
        if self.is_isolated { Arc::new(EphemeralSecretStore::default()) } else { Arc::new(KeyringSecretStore::new(APP_IDENTIFIER)) }
    }
}
