use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::SteamAchievement;
use super::keyvalue::{self, KeyValue};

const STATS_PREFIX: &str = "UserGameStats";
const SCHEMA_PREFIX: &str = "UserGameStatsSchema_";
const FILE_EXTENSION: &str = ".bin";
const PREFERRED_LANGUAGES: [&str; 2] = ["french", "english"];
const INDIVIDUAL_ACCOUNT_BASE: u64 = 76_561_197_960_265_728;

#[derive(Clone, Debug, PartialEq)]
pub struct SteamUser {
    pub steam_id64: u64,
    pub persona_name: String,
    is_most_recent: bool,
}

impl SteamUser {
    pub fn account_id(&self) -> Option<u32> {
        let offset = self.steam_id64.checked_sub(INDIVIDUAL_ACCOUNT_BASE).filter(|offset| *offset > 0)?;
        Some(offset as u32)
    }
}

#[derive(Clone, Debug)]
pub struct SteamLocalGame {
    pub app_id: u32,
    pub schema_name: String,
    pub achievements: Vec<SteamAchievement>,
}

/// Lecture directe des fichiers du client Steam, sans réseau.
/// Le client réécrit `UserGameStats_<compte>_<appid>.bin` dès qu'un jeu débloque un succès.
#[derive(Clone, Debug)]
pub struct SteamClientCache {
    root: PathBuf,
}

impl SteamClientCache {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        SteamClientCache { root: root.into() }
    }

    /// Emplacements habituels du client Steam sur chaque système, du plus probable au moins probable.
    pub fn default_roots() -> Vec<PathBuf> {
        let home = dirs::home_dir().unwrap_or_default();
        if cfg!(target_os = "macos") {
            vec![home.join("Library/Application Support/Steam")]
        } else if cfg!(target_os = "windows") {
            let program_files =
                std::env::var_os("ProgramFiles(x86)").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"));
            registered_windows_root().into_iter().chain([program_files.join("Steam")]).collect()
        } else {
            vec![
                home.join(".steam/steam"),
                home.join(".local/share/Steam"),
                home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
            ]
        }
    }

    /// Premier emplacement par défaut où le client Steam est réellement installé.
    pub fn locate() -> Option<Self> {
        Self::default_roots().into_iter().map(Self::new).find(Self::is_installed)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn stats_directory(&self) -> PathBuf {
        self.root.join("appcache").join("stats")
    }

    pub fn is_installed(&self) -> bool {
        self.stats_directory().is_dir()
    }

    pub fn current_user(&self) -> Option<SteamUser> {
        let users = self.login_users();
        users.iter().find(|user| user.is_most_recent).or(users.first()).cloned()
    }

    /// Compte connecté, ou à défaut celui qui possède le plus de fichiers de statistiques.
    pub fn current_account_id(&self) -> Option<u32> {
        if let Some(account_id) = self.current_user().and_then(|user| user.account_id()) {
            return Some(account_id);
        }
        let mut files_per_account: HashMap<u32, usize> = HashMap::new();
        for name in self.stats_file_names() {
            let parts: Vec<_> = name.trim_end_matches(FILE_EXTENSION).split('_').collect();
            if let [STATS_PREFIX, account, _] = parts.as_slice() {
                if let Ok(account) = account.parse() {
                    *files_per_account.entry(account).or_default() += 1;
                }
            }
        }
        files_per_account.into_iter().max_by_key(|(_, count)| *count).map(|(account, _)| account)
    }

    /// Jeux dont la liste de succès est présente sur cet ordinateur.
    pub fn app_ids_with_schema(&self) -> Vec<u32> {
        let mut app_ids: Vec<u32> = self
            .stats_file_names()
            .iter()
            .filter_map(|name| name.strip_prefix(SCHEMA_PREFIX)?.strip_suffix(FILE_EXTENSION)?.parse().ok())
            .collect();
        app_ids.sort_unstable();
        app_ids
    }

    /// Appid d'un fichier `UserGameStats_<compte>_<appid>.bin` ou `UserGameStatsSchema_<appid>.bin`.
    pub fn app_id_from_file_name(name: &str) -> Option<u32> {
        if !name.starts_with(STATS_PREFIX) {
            return None;
        }
        name.strip_suffix(FILE_EXTENSION)?.rsplit('_').next()?.parse().ok()
    }

    pub fn stats_file(&self, app_id: u32, account_id: u32) -> PathBuf {
        self.stats_directory().join(format!("{STATS_PREFIX}_{account_id}_{app_id}{FILE_EXTENSION}"))
    }

    pub fn stats_modified_at(&self, app_id: u32, account_id: u32) -> Option<SystemTime> {
        fs::metadata(self.stats_file(app_id, account_id)).and_then(|metadata| metadata.modified()).ok()
    }

    /// Noms des jeux installés, lus dans les `appmanifest_*.acf` de chaque bibliothèque Steam.
    pub fn installed_game_names(&self) -> HashMap<u32, String> {
        let mut names = HashMap::new();
        for folder in self.library_folders() {
            let Ok(entries) = fs::read_dir(&folder) else { continue };
            for entry in entries.flatten() {
                if !entry.file_name().to_string_lossy().starts_with("appmanifest_") {
                    continue;
                }
                let manifest = read_text_key_values(&entry.path());
                let state = manifest.as_ref().and_then(|manifest| manifest.get("AppState"));
                let app_id = state.and_then(|state| state.get("appid")).and_then(KeyValue::integer);
                let name = state.and_then(|state| state.get("name")).and_then(KeyValue::text);
                if let (Some(app_id), Some(name)) = (app_id.and_then(|id| u32::try_from(id).ok()), name) {
                    names.insert(app_id, name);
                }
            }
        }
        names
    }

    /// Retourne `None` si les fichiers sont absents ou en cours d'écriture par Steam : la lecture est à retenter.
    pub fn load_game(&self, app_id: u32, account_id: u32) -> Option<SteamLocalGame> {
        let schema_file = self.stats_directory().join(format!("{SCHEMA_PREFIX}{app_id}{FILE_EXTENSION}"));
        let schema_root = keyvalue::parse_binary(&fs::read(schema_file).ok()?).ok()?;
        let (_, schema) = schema_root.entries().first()?;

        let user_stats = match fs::read(self.stats_file(app_id, account_id)) {
            Ok(bytes) => keyvalue::parse_binary(&bytes).ok()?.get("cache").cloned(),
            Err(_) => None,
        };

        let achievements = schema
            .get("stats")
            .map(|stats| sorted_numerically(stats.entries()))
            .unwrap_or_default()
            .into_iter()
            .flat_map(|(stat_id, stat)| achievements_in(stat, user_stats.as_ref().and_then(|stats| stats.get(stat_id)), app_id))
            .collect();

        Some(SteamLocalGame { app_id, schema_name: schema.get("gamename").and_then(KeyValue::text).unwrap_or_default(), achievements })
    }

    fn login_users(&self) -> Vec<SteamUser> {
        let Some(file) = read_text_key_values(&self.root.join("config").join("loginusers.vdf")) else { return Vec::new() };
        file.get("users")
            .map(KeyValue::entries)
            .unwrap_or_default()
            .iter()
            .filter_map(|(steam_id, user)| {
                Some(SteamUser {
                    steam_id64: steam_id.parse().ok()?,
                    persona_name: user.get("PersonaName").and_then(KeyValue::text).unwrap_or_default(),
                    is_most_recent: user.get("MostRecent").and_then(KeyValue::text).as_deref() == Some("1"),
                })
            })
            .collect()
    }

    fn library_folders(&self) -> HashSet<PathBuf> {
        let default_folder = self.root.join("steamapps");
        let declared = read_text_key_values(&default_folder.join("libraryfolders.vdf"));
        let additional = declared
            .as_ref()
            .and_then(|file| file.get("libraryfolders"))
            .map(KeyValue::entries)
            .unwrap_or_default()
            .iter()
            .filter_map(|(_, folder)| folder.get("path").and_then(KeyValue::text))
            .map(|path| PathBuf::from(path).join("steamapps"));
        std::iter::once(default_folder).chain(additional).collect()
    }

    fn stats_file_names(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(self.stats_directory()) else { return Vec::new() };
        entries
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(FILE_EXTENSION))
            .collect()
    }
}

/// Le client Steam inscrit son dossier d'installation dans le registre de Windows.
#[cfg(windows)]
fn registered_windows_root() -> Option<PathBuf> {
    let key = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER).open_subkey(r"Software\Valve\Steam").ok()?;
    key.get_value::<String, _>("SteamPath").ok().map(PathBuf::from)
}

#[cfg(not(windows))]
fn registered_windows_root() -> Option<PathBuf> {
    None
}

/// Chaque statistique regroupe jusqu'à 32 succès, dont l'état est encodé bit à bit.
fn achievements_in(stat: &KeyValue, user_stats: Option<&KeyValue>, app_id: u32) -> Vec<SteamAchievement> {
    let Some(bits) = stat.get("bits") else { return Vec::new() };
    let unlocked_mask = user_stats.and_then(|stats| stats.get("data")).and_then(KeyValue::integer).unwrap_or(0) as u32;
    let unlock_times = user_stats.and_then(|stats| stats.get("AchievementTimes"));
    let icon_base = format!("https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps/{app_id}/");

    sorted_numerically(bits.entries())
        .into_iter()
        .filter_map(|(bit_id, bit)| {
            let bit_index: u32 = bit_id.parse().ok()?;
            let api_name = bit.get("name").and_then(KeyValue::text)?;
            let display = bit.get("display");
            let field = |key: &str| display.and_then(|display| display.get(key));
            let is_unlocked = bit_index < u32::BITS && (unlocked_mask >> bit_index) & 1 == 1;
            let unlock_time = unlock_times.and_then(|times| times.get(bit_id)).and_then(KeyValue::integer);
            Some(
                SteamAchievement {
                    name: localized(field("name")).unwrap_or_else(|| api_name.clone()),
                    detail: localized(field("desc")).unwrap_or_default(),
                    is_hidden: field("hidden").and_then(KeyValue::integer).unwrap_or(0) != 0,
                    icon_url: field("icon").and_then(KeyValue::text).map(|icon| format!("{icon_base}{icon}")),
                    locked_icon_url: field("icon_gray").and_then(KeyValue::text).map(|icon| format!("{icon_base}{icon}")),
                    api_name,
                    ..SteamAchievement::default()
                }
                .unlocked_at(is_unlocked, SteamAchievement::unlock_date(unlock_time)),
            )
        })
        .collect()
}

fn localized(value: Option<&KeyValue>) -> Option<String> {
    let value = value?;
    if let Some(text) = value.text() {
        return Some(text).filter(|text| !text.is_empty());
    }
    PREFERRED_LANGUAGES
        .iter()
        .find_map(|language| value.get(language).and_then(KeyValue::text).filter(|text| !text.is_empty()))
        .or_else(|| value.entries().iter().find(|(key, _)| key != "token").and_then(|(_, text)| text.text()))
}

fn sorted_numerically(entries: &[(String, KeyValue)]) -> Vec<(&String, &KeyValue)> {
    let mut sorted: Vec<_> = entries.iter().map(|(key, value)| (key, value)).collect();
    sorted.sort_by_key(|(key, _)| key.parse::<i64>().unwrap_or(0));
    sorted
}

fn read_text_key_values(path: &Path) -> Option<KeyValue> {
    fs::read_to_string(path).ok().map(|text| keyvalue::parse_text(&text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{FixtureAchievement, write_steam_directory};

    const ACCOUNT_ID: u32 = 12_345_678;
    const APP_ID: u32 = 49_520;

    fn make_steam_directory() -> tempfile::TempDir {
        let root = tempfile::tempdir().expect("dossier temporaire");
        let achievements = [
            FixtureAchievement { api_name: "ACH_0", name: "Premier sang", unlocked_at: None },
            FixtureAchievement { api_name: "ACH_1", name: "Pourfendeur", unlocked_at: Some(1_600_000_000) },
        ];
        write_steam_directory(root.path(), ACCOUNT_ID, APP_ID, "Borderlands", &achievements).expect("dossier Steam factice");
        root
    }

    #[test]
    fn reads_achievements_and_unlock_state() {
        let root = make_steam_directory();
        let cache = SteamClientCache::new(root.path());

        let game = cache.load_game(APP_ID, ACCOUNT_ID).expect("jeu lisible");

        assert!(cache.is_installed());
        assert_eq!(cache.current_account_id(), Some(ACCOUNT_ID));
        assert_eq!(cache.current_user().map(|user| user.persona_name), Some("Alex".into()));
        assert_eq!(cache.app_ids_with_schema(), vec![APP_ID]);
        assert_eq!(game.schema_name, "Borderlands");
        assert_eq!(
            game.achievements.iter().map(|achievement| achievement.name.as_str()).collect::<Vec<_>>(),
            ["Premier sang", "Pourfendeur"]
        );
        assert_eq!(game.achievements.iter().map(|achievement| achievement.is_earned).collect::<Vec<_>>(), [false, true]);
        assert_eq!(game.achievements[1].earned_at.map(|date| date.timestamp()), Some(1_600_000_000));
        assert!(game.achievements[1].icon_url.as_deref().is_some_and(|url| url.ends_with("ACH_1.jpg")));
    }

    #[test]
    fn extracts_app_id_from_file_name() {
        assert_eq!(SteamClientCache::app_id_from_file_name("UserGameStats_12345678_49520.bin"), Some(49520));
        assert_eq!(SteamClientCache::app_id_from_file_name("UserGameStatsSchema_730.bin"), Some(730));
        assert_eq!(SteamClientCache::app_id_from_file_name("loginusers.vdf"), None);
    }
}
