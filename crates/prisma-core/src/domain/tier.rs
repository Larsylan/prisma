use serde::{Deserialize, Serialize};

/// Les grades sont déclarés du moins au plus précieux : l'ordre dérivé les classe ainsi.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrophyTier {
    #[default]
    Bronze,
    Silver,
    Gold,
    Platinum,
}

impl TrophyTier {
    pub const ALL: [TrophyTier; 4] = [TrophyTier::Bronze, TrophyTier::Silver, TrophyTier::Gold, TrophyTier::Platinum];
    /// Grades qu'un trophée ordinaire peut recevoir ; le platine récompense un jeu complet.
    pub const REGULAR: [TrophyTier; 3] = [TrophyTier::Bronze, TrophyTier::Silver, TrophyTier::Gold];

    pub fn points(self) -> u32 {
        match self {
            TrophyTier::Bronze => 15,
            TrophyTier::Silver => 30,
            TrophyTier::Gold => 90,
            TrophyTier::Platinum => 300,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tier| tier.name() == name)
    }

    pub fn name(self) -> &'static str {
        match self {
            TrophyTier::Bronze => "bronze",
            TrophyTier::Silver => "silver",
            TrophyTier::Gold => "gold",
            TrophyTier::Platinum => "platinum",
        }
    }
}
