use std::collections::HashMap;

use super::{SteamAchievement, tiers};
use crate::domain::{Game, GameId, Platform, Trophy, system};

/// En dessous, la liste est trop courte pour justifier un platine.
const MINIMUM_ACHIEVEMENTS_FOR_PLATINUM: usize = 5;

pub fn make_game(app_id: u32, title: &str, achievements: &[SteamAchievement], rarity: Option<&HashMap<String, f64>>) -> Game {
    let cover = cover_url(app_id);
    let mut trophies: Vec<Trophy> = achievements
        .iter()
        .map(|achievement| {
            let unlock_percentage = rarity.and_then(|rarity| rarity.get(&achievement.api_name)).copied();
            Trophy {
                is_hidden: achievement.is_hidden,
                icon_url: achievement.icon_url.clone(),
                locked_icon_url: achievement.locked_icon_url.clone(),
                is_earned: achievement.is_earned,
                earned_at: achievement.earned_at,
                ..Trophy::new(
                    &achievement.api_name,
                    &achievement.name,
                    &achievement.detail,
                    tiers::estimate(achievement, unlock_percentage),
                )
            }
        })
        .collect();
    if trophies.len() >= MINIMUM_ACHIEVEMENTS_FOR_PLATINUM {
        trophies.insert(0, Trophy::generated_platinum(title, Some(cover.clone())));
    }
    Game { cover_url: Some(cover), ..Game::new(GameId::steam(app_id), Platform::Steam, title, vec![system::STEAM.into()], trophies) }
}

fn cover_url(app_id: u32) -> String {
    format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{app_id}/header.jpg")
}

pub mod title {
    /// Les jeux sans nom public n'exposent qu'un nom de code interne du type « ValveTestApp260 ».
    pub fn is_meaningful(title: &str) -> bool {
        !title.is_empty() && !title.starts_with("ValveTestApp")
    }

    pub fn placeholder(app_id: u32) -> String {
        format!("Jeu Steam {app_id}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn achievements(count: usize) -> Vec<SteamAchievement> {
        (0..count)
            .map(|index| SteamAchievement { api_name: format!("A{index}"), name: format!("A{index}"), ..Default::default() })
            .collect()
    }

    #[test]
    fn platinum_is_added_only_to_long_enough_lists() {
        let short = make_game(1, "Court", &achievements(4), None);
        let long = make_game(1, "Long", &achievements(5), None);

        assert!(short.trophies.iter().all(|trophy| !trophy.is_generated_platinum));
        assert!(long.trophies[0].is_generated_platinum);
        assert_eq!(long.trophies[0].name, "Long");
    }

    #[test]
    fn placeholder_titles_are_detected() {
        assert!(!title::is_meaningful("ValveTestApp260"));
        assert!(!title::is_meaningful(""));
        assert!(title::is_meaningful("Borderlands"));
    }
}
