use std::fmt;

use serde::{Deserialize, Serialize};

use super::Platform;

/// Identifiant stable d'un jeu, préfixé par sa plateforme : `psn:…`, `steam:…`, `switch:…`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GameId(String);

const GROUP_PREFIX: &str = "group:";

impl GameId {
    pub fn new(raw: impl Into<String>) -> Self {
        GameId(raw.into())
    }

    pub fn playstation(communication_id: &str) -> Self {
        Self::on(Platform::Playstation, communication_id)
    }

    pub fn steam(app_id: u32) -> Self {
        Self::on(Platform::Steam, &app_id.to_string())
    }

    pub fn nintendo(catalog_id: &str) -> Self {
        Self::on(Platform::Nintendo, catalog_id)
    }

    pub(crate) fn group(key: &str) -> Self {
        GameId(format!("{GROUP_PREFIX}{key}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn platform(&self) -> Option<Platform> {
        Platform::ALL.into_iter().find(|platform| self.0.starts_with(prefix(*platform)))
    }

    pub fn steam_app_id(&self) -> Option<u32> {
        self.local_part(Platform::Steam)?.parse().ok()
    }

    pub fn nintendo_catalog_id(&self) -> Option<&str> {
        self.local_part(Platform::Nintendo)
    }

    fn on(platform: Platform, local_part: &str) -> Self {
        GameId(format!("{}{local_part}", prefix(platform)))
    }

    fn local_part(&self, platform: Platform) -> Option<&str> {
        self.0.strip_prefix(prefix(platform))
    }
}

fn prefix(platform: Platform) -> &'static str {
    match platform {
        Platform::Playstation => "psn:",
        Platform::Steam => "steam:",
        Platform::Nintendo => "switch:",
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Désigne un trophée tel qu'il est stocké, indépendamment de son éventuel regroupement à l'affichage.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrophyRef {
    pub game_id: GameId,
    pub trophy_id: String,
}

impl TrophyRef {
    pub fn new(game_id: GameId, trophy_id: impl Into<String>) -> Self {
        TrophyRef { game_id, trophy_id: trophy_id.into() }
    }
}
