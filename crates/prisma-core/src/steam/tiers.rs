//! Steam n'a pas de grades : ils sont estimés pour uniformiser avec PlayStation.

use super::SteamAchievement;
use crate::domain::TrophyTier;

const GOLD_KEYWORDS: [&str; 21] = [
    "100%",
    "tous les",
    "toutes les",
    "all achievements",
    "every ",
    "complete the game",
    "finish the game",
    "terminer le jeu",
    "terminé le jeu",
    "insane",
    "nightmare",
    "cauchemar",
    "legendary",
    "légendaire",
    "hardest",
    "difficulté maximale",
    "master",
    "maître",
    "platinum",
    "perfect",
    "parfait",
];

/// La rareté, quand elle est connue, prime sur les mots-clés du nom et de la description.
pub(crate) fn estimate(achievement: &SteamAchievement, unlock_percentage: Option<f64>) -> TrophyTier {
    if let Some(percentage) = unlock_percentage {
        return tier_for_unlock_percentage(percentage);
    }
    let text = format!("{} {}", achievement.name, achievement.detail).to_lowercase();
    if GOLD_KEYWORDS.iter().any(|keyword| text.contains(keyword)) {
        TrophyTier::Gold
    } else if achievement.is_hidden {
        TrophyTier::Silver
    } else {
        TrophyTier::Bronze
    }
}

fn tier_for_unlock_percentage(percentage: f64) -> TrophyTier {
    match percentage {
        p if p < 7.0 => TrophyTier::Gold,
        p if p < 25.0 => TrophyTier::Silver,
        _ => TrophyTier::Bronze,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn achievement(name: &str, detail: &str, is_hidden: bool) -> SteamAchievement {
        SteamAchievement { api_name: name.into(), name: name.into(), detail: detail.into(), is_hidden, ..Default::default() }
    }

    #[test]
    fn rarity_decides_tier() {
        for (percentage, tier) in
            [(3.0, TrophyTier::Gold), (7.0, TrophyTier::Silver), (24.9, TrophyTier::Silver), (25.0, TrophyTier::Bronze)]
        {
            assert_eq!(estimate(&achievement("x", "", false), Some(percentage)), tier, "{percentage}");
        }
    }

    #[test]
    fn keywords_and_hidden_state_decide_tier_without_rarity() {
        assert_eq!(estimate(&achievement("Fin", "Terminer le jeu en Cauchemar", false), None), TrophyTier::Gold);
        assert_eq!(estimate(&achievement("Secret", "", true), None), TrophyTier::Silver);
        assert_eq!(estimate(&achievement("Premier pas", "", false), None), TrophyTier::Bronze);
    }
}
