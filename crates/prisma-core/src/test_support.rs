//! Outils réservés aux tests : un faux dossier Steam écrit au format du client.
//! Disponible dans les tests de ce crate et, via la fonctionnalité `test-support`, dans ceux de l'app.

use std::fs;
use std::io;
use std::path::Path;

const INDIVIDUAL_ACCOUNT_BASE: u64 = 76_561_197_960_265_728;

/// Succès d'un jeu factice ; ils occupent, dans l'ordre, les bits d'une même statistique.
pub struct FixtureAchievement {
    pub api_name: &'static str,
    pub name: &'static str,
    /// Date de déblocage en secondes depuis 1970 ; `None` pour un succès non obtenu.
    pub unlocked_at: Option<i64>,
}

/// Écrit un client Steam avec un compte connecté « Alex » et un jeu doté de ces succès.
pub fn write_steam_directory(
    root: &Path,
    account_id: u32,
    app_id: u32,
    game_name: &str,
    achievements: &[FixtureAchievement],
) -> io::Result<()> {
    let config = root.join("config");
    fs::create_dir_all(&config)?;
    fs::create_dir_all(stats_directory(root))?;
    let steam_id64 = INDIVIDUAL_ACCOUNT_BASE + u64::from(account_id);
    fs::write(config.join("loginusers.vdf"), format!(r#""users" {{ "{steam_id64}" {{ "PersonaName" "Alex" "MostRecent" "1" }} }}"#))?;

    let mut schema = BinaryWriter::default();
    schema.begin(&app_id.to_string()).text("gamename", game_name).begin("stats").begin("1").begin("bits");
    for (bit, achievement) in achievements.iter().enumerate() {
        schema.begin(&bit.to_string()).text("name", achievement.api_name).begin("display").begin("name");
        schema
            .text("french", achievement.name)
            .text("token", "#ACH")
            .end()
            .text("icon", &format!("{}.jpg", achievement.api_name))
            .end()
            .end();
    }
    schema.end().end().end().end().end();
    fs::write(stats_directory(root).join(format!("UserGameStatsSchema_{app_id}.bin")), &schema.bytes)?;

    write_user_stats(root, account_id, app_id, achievements)
}

/// Réécrit l'état de déblocage, comme le fait le client Steam quand un jeu débloque un succès.
pub fn write_user_stats(root: &Path, account_id: u32, app_id: u32, achievements: &[FixtureAchievement]) -> io::Result<()> {
    let unlocked_mask = achievements
        .iter()
        .enumerate()
        .filter(|(_, achievement)| achievement.unlocked_at.is_some())
        .fold(0_i32, |mask, (bit, _)| mask | (1 << bit));
    let mut stats = BinaryWriter::default();
    stats.begin("cache").begin("1").int32("data", unlocked_mask).begin("AchievementTimes");
    for (bit, achievement) in achievements.iter().enumerate() {
        if let Some(unlocked_at) = achievement.unlocked_at {
            stats.int32(&bit.to_string(), unlocked_at as i32);
        }
    }
    stats.end().end().end().end();
    fs::write(stats_directory(root).join(format!("UserGameStats_{account_id}_{app_id}.bin")), &stats.bytes)
}

fn stats_directory(root: &Path) -> std::path::PathBuf {
    root.join("appcache").join("stats")
}

/// Écrit des fichiers KeyValues binaires, au format du cache de statistiques Steam.
#[derive(Default)]
pub struct BinaryWriter {
    pub bytes: Vec<u8>,
}

impl BinaryWriter {
    pub fn begin(&mut self, key: &str) -> &mut Self {
        self.bytes.push(0x00);
        self.c_string(key)
    }

    pub fn end(&mut self) -> &mut Self {
        self.bytes.push(0x08);
        self
    }

    pub fn text(&mut self, key: &str, value: &str) -> &mut Self {
        self.bytes.push(0x01);
        self.c_string(key).c_string(value)
    }

    pub fn int32(&mut self, key: &str, value: i32) -> &mut Self {
        self.bytes.push(0x02);
        self.c_string(key);
        self.bytes.extend(value.to_le_bytes());
        self
    }

    fn c_string(&mut self, text: &str) -> &mut Self {
        self.bytes.extend(text.as_bytes());
        self.bytes.push(0);
        self
    }
}
