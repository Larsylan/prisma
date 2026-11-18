//! Langue de l'app. L'interface traduit elle-même ses textes ; le backend ne traduit que ce qu'il affiche
//! directement : le menu de l'icône et la notification de test.

use prisma_core::domain::TrophyTier;
use serde::{Deserialize, Serialize};

/// Langue choisie dans les réglages.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    /// Celle du système, ou l'anglais si le système n'est dans aucune des langues de l'app.
    #[default]
    System,
    French,
    English,
}

/// Langue effectivement affichée.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Locale {
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "en")]
    English,
}

impl Language {
    pub fn locale(self) -> Locale {
        match self {
            Language::System => Locale::from_tag(&sys_locale::get_locale().unwrap_or_default()),
            Language::French => Locale::French,
            Language::English => Locale::English,
        }
    }
}

impl Locale {
    /// `fr-FR`, `fr_CA`… sont en français ; toute autre langue passe à l'anglais.
    fn from_tag(tag: &str) -> Self {
        if tag.to_ascii_lowercase().starts_with("fr") { Locale::French } else { Locale::English }
    }

    pub fn texts(self) -> &'static Texts {
        match self {
            Locale::French => &FRENCH,
            Locale::English => &ENGLISH,
        }
    }
}

pub struct Texts {
    pub open: &'static str,
    pub settings: &'static str,
    pub test_notification: &'static str,
    pub sync: &'static str,
    pub quit: &'static str,
    pub level: &'static str,
    pub platinums: &'static str,
    pub test_trophy: &'static str,
    pub test_platinum: &'static str,
    /// Nom du profil quand le système n'en fournit aucun.
    pub player: &'static str,
    tiers: [&'static str; 4],
}

impl Texts {
    pub fn tier(&self, tier: TrophyTier) -> &'static str {
        // Les grades sont déclarés du bronze au platine, dans l'ordre du tableau.
        self.tiers[tier as usize]
    }
}

static FRENCH: Texts = Texts {
    open: "Ouvrir Prisma",
    settings: "Réglages…",
    test_notification: "Notification de test",
    sync: "Synchroniser",
    quit: "Quitter",
    level: "Niveau",
    platinums: "platine(s)",
    test_trophy: "Premier pas",
    test_platinum: "Légende vivante",
    player: "Joueur",
    tiers: ["Bronze", "Argent", "Or", "Platine"],
};

static ENGLISH: Texts = Texts {
    open: "Open Prisma",
    settings: "Settings…",
    test_notification: "Test notification",
    sync: "Sync",
    quit: "Quit",
    level: "Level",
    platinums: "platinum(s)",
    test_trophy: "First Steps",
    test_platinum: "Living Legend",
    player: "Player",
    tiers: ["Bronze", "Silver", "Gold", "Platinum"],
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_french_system_languages_switch_to_french() {
        assert_eq!(Locale::from_tag("fr-FR"), Locale::French);
        assert_eq!(Locale::from_tag("fr_CA"), Locale::French);
        assert_eq!(Locale::from_tag("en-US"), Locale::English);
        assert_eq!(Locale::from_tag("de-DE"), Locale::English);
        assert_eq!(Locale::from_tag(""), Locale::English);
    }

    #[test]
    fn tiers_follow_their_order() {
        assert_eq!(ENGLISH.tier(TrophyTier::Gold), "Gold");
        assert_eq!(FRENCH.tier(TrophyTier::Platinum), "Platine");
    }
}
