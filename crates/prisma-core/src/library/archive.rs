//! Format du fichier `library.json`. Les noms de clés sont figés : le fichier reste lisible
//! par les versions précédentes de l'app.

use std::collections::HashMap;

use chrono::DateTime;
use serde::{Deserialize, Serialize};

use super::{SourceCache, StoredLibrary, TrophyLibrary};
use crate::domain::{Game, GameId, Platform, Timestamp, Trophy, TrophyRef, TrophyTier};

const OVERRIDE_SEPARATOR: char = '|';
const SYSTEM_SEPARATOR: &str = " / ";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LibraryArchive {
    games: Vec<GameRecord>,
    overrides: HashMap<String, TrophyTier>,
    psn_stamps: HashMap<String, String>,
    steam_rarity: HashMap<String, HashMap<String, f64>>,
    steam_names: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameRecord {
    id: String,
    platform: Platform,
    title: String,
    system: String,
    #[serde(rename = "coverURL", default, skip_serializing_if = "Option::is_none")]
    cover_url: Option<String>,
    trophies: Vec<TrophyRecord>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrophyRecord {
    id: String,
    name: String,
    detail: String,
    tier: TrophyTier,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    auto_tier: Option<TrophyTier>,
    hidden: bool,
    #[serde(rename = "iconURL", default, skip_serializing_if = "Option::is_none")]
    icon_url: Option<String>,
    #[serde(rename = "lockedIconURL", default, skip_serializing_if = "Option::is_none")]
    locked_icon_url: Option<String>,
    earned: bool,
    /// Secondes depuis le 1er janvier 1970.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    earned_at: Option<f64>,
    #[serde(default)]
    is_synthetic: bool,
}

impl LibraryArchive {
    pub(super) fn new(stored: &StoredLibrary) -> Self {
        let library = &stored.library;
        LibraryArchive {
            games: library.games().map(GameRecord::from).collect(),
            overrides: library
                .tier_overrides()
                .iter()
                .map(|(reference, tier)| (format!("{}{OVERRIDE_SEPARATOR}{}", reference.game_id, reference.trophy_id), *tier))
                .collect(),
            psn_stamps: stored.cache.playstation_revisions.clone(),
            steam_rarity: stored.cache.steam_rarity.iter().map(|(app_id, rarity)| (app_id.to_string(), rarity.clone())).collect(),
            steam_names: stored.cache.steam_titles.iter().map(|(app_id, title)| (app_id.to_string(), title.clone())).collect(),
        }
    }

    pub(super) fn into_stored(self) -> StoredLibrary {
        let tier_overrides = self
            .overrides
            .into_iter()
            .filter_map(|(key, tier)| {
                let (game_id, trophy_id) = key.split_once(OVERRIDE_SEPARATOR)?;
                Some((TrophyRef::new(GameId::new(game_id), trophy_id), tier))
            })
            .collect();
        StoredLibrary {
            library: TrophyLibrary::new(self.games.into_iter().map(Game::from), tier_overrides),
            cache: SourceCache {
                playstation_revisions: self.psn_stamps,
                steam_rarity: by_app_id(self.steam_rarity),
                steam_titles: by_app_id(self.steam_names),
            },
        }
    }
}

fn by_app_id<T>(entries: HashMap<String, T>) -> HashMap<u32, T> {
    entries.into_iter().filter_map(|(key, value)| Some((key.parse().ok()?, value))).collect()
}

impl From<&Game> for GameRecord {
    fn from(game: &Game) -> Self {
        GameRecord {
            id: game.id.to_string(),
            platform: game.platform,
            title: game.title.clone(),
            system: game.systems.join(SYSTEM_SEPARATOR),
            cover_url: game.cover_url.clone(),
            trophies: game.trophies.iter().map(TrophyRecord::from).collect(),
        }
    }
}

impl From<GameRecord> for Game {
    fn from(record: GameRecord) -> Self {
        let systems = record.system.split(SYSTEM_SEPARATOR).filter(|system| !system.is_empty()).map(str::to_owned).collect();
        Game {
            cover_url: record.cover_url,
            ..Game::new(
                GameId::new(record.id),
                record.platform,
                record.title,
                systems,
                record.trophies.into_iter().map(Trophy::from).collect(),
            )
        }
    }
}

impl From<&Trophy> for TrophyRecord {
    fn from(trophy: &Trophy) -> Self {
        TrophyRecord {
            id: trophy.id.clone(),
            name: trophy.name.clone(),
            detail: trophy.detail.clone(),
            tier: trophy.tier,
            auto_tier: Some(trophy.automatic_tier),
            hidden: trophy.is_hidden,
            icon_url: trophy.icon_url.clone(),
            locked_icon_url: trophy.locked_icon_url.clone(),
            earned: trophy.is_earned,
            earned_at: trophy.earned_at.map(seconds_since_epoch),
            is_synthetic: trophy.is_generated_platinum,
        }
    }
}

impl From<TrophyRecord> for Trophy {
    fn from(record: TrophyRecord) -> Self {
        Trophy {
            automatic_tier: record.auto_tier.unwrap_or(record.tier),
            is_hidden: record.hidden,
            icon_url: record.icon_url,
            locked_icon_url: record.locked_icon_url,
            is_earned: record.earned,
            earned_at: record.earned_at.and_then(timestamp),
            is_generated_platinum: record.is_synthetic,
            ..Trophy::new(record.id, record.name, record.detail, record.tier)
        }
    }
}

const MICROSECONDS_PER_SECOND: f64 = 1_000_000.0;

fn seconds_since_epoch(date: Timestamp) -> f64 {
    date.timestamp_micros() as f64 / MICROSECONDS_PER_SECOND
}

fn timestamp(seconds: f64) -> Option<Timestamp> {
    DateTime::from_timestamp_micros((seconds * MICROSECONDS_PER_SECOND).round() as i64)
}
