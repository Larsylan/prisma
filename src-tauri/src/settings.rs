use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use prisma_core::library::GroupingMode;
use prisma_core::theme::Preset;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToastPosition {
    #[default]
    TopRight,
    TopLeft,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub profile_name: String,
    /// Renseigné uniquement pour suivre un autre compte que celui connecté dans Steam.
    pub steam_id64: String,
    pub use_steam_rarity: bool,
    pub playstation_polling: bool,
    pub grouping_mode: GroupingMode,
    pub sound_enabled: bool,
    pub toast_duration_seconds: u64,
    pub toast_position: ToastPosition,
    pub theme: Preset,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            profile_name: default_profile_name(),
            steam_id64: String::new(),
            use_steam_rarity: false,
            playstation_polling: true,
            grouping_mode: GroupingMode::All,
            sound_enabled: true,
            toast_duration_seconds: 6,
            toast_position: ToastPosition::TopRight,
            theme: Preset::default(),
        }
    }
}

fn default_profile_name() -> String {
    ["USER", "USERNAME"].iter().find_map(|variable| std::env::var(variable).ok()).unwrap_or_else(|| "Joueur".into())
}

/// Réglages conservés dans `settings.json`, à côté de la bibliothèque.
pub struct SettingsStore {
    file: PathBuf,
    current: Mutex<Settings>,
}

impl SettingsStore {
    pub fn load(directory: PathBuf) -> Self {
        let file = directory.join("settings.json");
        let current = fs::read(&file).ok().and_then(|data| serde_json::from_slice(&data).ok()).unwrap_or_default();
        SettingsStore { file, current: Mutex::new(current) }
    }

    pub fn get(&self) -> Settings {
        self.current.lock().expect("réglages accessibles").clone()
    }

    /// Enregistre les nouveaux réglages et retourne les précédents.
    pub fn replace(&self, settings: Settings) -> Settings {
        let previous = std::mem::replace(&mut *self.current.lock().expect("réglages accessibles"), settings.clone());
        if let Err(error) = self.write(&settings) {
            log::error!("Sauvegarde des réglages impossible : {error}");
        }
        previous
    }

    fn write(&self, settings: &Settings) -> std::io::Result<()> {
        if let Some(directory) = self.file.parent() {
            fs::create_dir_all(directory)?;
        }
        fs::write(&self.file, serde_json::to_vec_pretty(settings)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_gives_defaults() {
        let directory = tempfile::tempdir().unwrap();
        let settings = SettingsStore::load(directory.path().to_path_buf()).get();

        assert_eq!(settings.theme, Preset::Night);
        assert!(settings.sound_enabled);
        assert_eq!(settings.toast_duration_seconds, 6);
    }

    #[test]
    fn replaced_settings_are_saved_and_the_previous_ones_returned() {
        let directory = tempfile::tempdir().unwrap();
        let store = SettingsStore::load(directory.path().to_path_buf());
        let changed = Settings { theme: Preset::Synthwave, profile_name: "Alex".into(), ..store.get() };

        let previous = store.replace(changed.clone());

        assert_eq!(previous.theme, Preset::Night);
        assert_eq!(SettingsStore::load(directory.path().to_path_buf()).get(), changed);
    }

    #[test]
    fn missing_fields_take_their_default_value() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("settings.json"), r#"{"profileName": "Alex"}"#).unwrap();

        let settings = SettingsStore::load(directory.path().to_path_buf()).get();

        assert_eq!(settings.profile_name, "Alex");
        assert_eq!(settings.grouping_mode, GroupingMode::All);
    }
}
