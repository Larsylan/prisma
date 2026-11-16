use serde::Serialize;

use super::{Timestamp, TrophyRef, TrophyTier};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Trophy {
    pub id: String,
    pub name: String,
    pub detail: String,
    pub tier: TrophyTier,
    /// Grade attribué par la source, conservé lorsque l'utilisateur en choisit un autre.
    pub automatic_tier: TrophyTier,
    pub is_hidden: bool,
    pub icon_url: Option<String>,
    pub locked_icon_url: Option<String>,
    pub is_earned: bool,
    pub earned_at: Option<Timestamp>,
    /// Platine ajouté par l'app aux jeux Steam et Switch, qui n'en ont pas nativement.
    pub is_generated_platinum: bool,
    /// Systèmes sur lesquels le trophée a été obtenu, renseigné pour les jeux regroupés.
    pub earned_on: Vec<String>,
    /// Trophée stocké d'où provient ce trophée affiché, renseigné pour les jeux regroupés.
    pub origin: Option<TrophyRef>,
}

pub(crate) const GENERATED_PLATINUM_ID: &str = "__platinum__";

impl Trophy {
    pub fn new(id: impl Into<String>, name: impl Into<String>, detail: impl Into<String>, tier: TrophyTier) -> Self {
        Trophy {
            id: id.into(),
            name: name.into(),
            detail: detail.into(),
            tier,
            automatic_tier: tier,
            is_hidden: false,
            icon_url: None,
            locked_icon_url: None,
            is_earned: false,
            earned_at: None,
            is_generated_platinum: false,
            earned_on: Vec::new(),
            origin: None,
        }
    }

    pub(crate) fn generated_platinum(game_title: &str, artwork_url: Option<String>) -> Self {
        let (name, detail) = Self::generated_platinum_text(game_title);
        Trophy {
            icon_url: artwork_url.clone(),
            locked_icon_url: artwork_url,
            is_generated_platinum: true,
            ..Trophy::new(GENERATED_PLATINUM_ID, name, detail, TrophyTier::Platinum)
        }
    }

    pub(crate) fn generated_platinum_text(game_title: &str) -> (String, String) {
        if game_title.is_empty() {
            ("Platine".into(), "Obtenir tous les trophées de ce jeu.".into())
        } else {
            (game_title.into(), format!("Obtenir tous les trophées de {game_title}."))
        }
    }

    pub fn earned(mut self, at: Option<Timestamp>) -> Self {
        self.mark_earned(at);
        self
    }

    pub(crate) fn mark_earned(&mut self, at: Option<Timestamp>) {
        self.is_earned = true;
        self.earned_at = at;
    }

    pub(crate) fn mark_locked(&mut self) {
        self.is_earned = false;
        self.earned_at = None;
    }
}
