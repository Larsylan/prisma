//! La Switch n'a pas de système de succès et aucune API ne l'expose : chaque jeu reçoit une liste
//! de trophées écrite sur le modèle PS5, cochée à la main au fil de la progression.

mod catalog_data;

use std::borrow::Cow;

use serde::Serialize;

use crate::domain::{Game, GameId, Platform, Trophy, TrophyTier, system};
use crate::support::text::slug;

const CUSTOM_PREFIX: &str = "custom-";

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrophyDefinition {
    pub tier: TrophyTier,
    pub name: &'static str,
    pub detail: &'static str,
    pub is_hidden: bool,
}

impl TrophyDefinition {
    const fn hidden(self) -> Self {
        TrophyDefinition { is_hidden: true, ..self }
    }
}

const fn bronze(name: &'static str, detail: &'static str) -> TrophyDefinition {
    TrophyDefinition { tier: TrophyTier::Bronze, name, detail, is_hidden: false }
}

const fn silver(name: &'static str, detail: &'static str) -> TrophyDefinition {
    TrophyDefinition { tier: TrophyTier::Silver, ..bronze(name, detail) }
}

const fn gold(name: &'static str, detail: &'static str) -> TrophyDefinition {
    TrophyDefinition { tier: TrophyTier::Gold, ..bronze(name, detail) }
}

#[derive(Clone, Debug, Serialize)]
pub struct CatalogEntry {
    pub id: Cow<'static, str>,
    pub title: Cow<'static, str>,
    #[serde(skip)]
    pub trophies: &'static [TrophyDefinition],
}

const fn entry(id: &'static str, title: &'static str, trophies: &'static [TrophyDefinition]) -> CatalogEntry {
    CatalogEntry { id: Cow::Borrowed(id), title: Cow::Borrowed(title), trophies }
}

impl CatalogEntry {
    pub fn game_id(&self) -> GameId {
        GameId::nintendo(&self.id)
    }

    pub fn trophy_count(&self, tier: TrophyTier) -> usize {
        if tier == TrophyTier::Platinum { 1 } else { self.trophies.iter().filter(|trophy| trophy.tier == tier).count() }
    }
}

pub fn entries() -> &'static [CatalogEntry] {
    catalog_data::ENTRIES
}

/// Entrée d'un jeu déjà ajouté, y compris un jeu hors catalogue.
pub fn entry_for(game: &Game) -> Option<CatalogEntry> {
    let catalog_id = game.id.nintendo_catalog_id()?;
    if catalog_id.starts_with(CUSTOM_PREFIX) {
        return Some(custom_entry(&game.title));
    }
    entries().iter().find(|entry| entry.id == catalog_id).cloned()
}

/// Jeu absent du catalogue, doté d'une liste générique.
pub fn custom_entry(title: &str) -> CatalogEntry {
    CatalogEntry {
        id: Cow::Owned(format!("{CUSTOM_PREFIX}{}", slug(title))),
        title: Cow::Owned(title.to_owned()),
        trophies: catalog_data::GENERIC_TROPHIES,
    }
}

pub fn make_game(entry: &CatalogEntry) -> Game {
    let trophies = std::iter::once(Trophy::generated_platinum(&entry.title, None))
        .chain(entry.trophies.iter().map(|definition| Trophy {
            is_hidden: definition.is_hidden,
            ..Trophy::new(slug(definition.name), definition.name, definition.detail, definition.tier)
        }))
        .collect();
    Game::new(entry.game_id(), Platform::Nintendo, entry.title.as_ref(), vec![system::SWITCH.into()], trophies)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn catalog_and_trophy_identifiers_are_unique() {
        assert_eq!(entries().iter().map(|entry| &entry.id).collect::<HashSet<_>>().len(), entries().len());
        for entry in entries() {
            let ids: Vec<_> = make_game(entry).trophies.into_iter().map(|trophy| trophy.id).collect();
            assert_eq!(ids.iter().collect::<HashSet<_>>().len(), ids.len(), "doublon dans {}", entry.title);
        }
    }

    #[test]
    fn stored_game_finds_its_catalog_entry() {
        let entry = &entries()[0];
        let custom = custom_entry("Hollow Knight");

        assert_eq!(entry_for(&make_game(entry)).map(|found| found.id), Some(entry.id.clone()));
        assert_eq!(entry_for(&make_game(&custom)).map(|found| found.id.into_owned()), Some("custom-hollow-knight".into()));
    }

    #[test]
    fn every_game_gets_generated_platinum() {
        let game = make_game(&entries()[0]);
        assert!(game.trophies[0].is_generated_platinum);
        assert!(game.trophies[1..].iter().all(|trophy| trophy.tier != TrophyTier::Platinum));
    }
}
