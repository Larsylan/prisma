use std::cmp::Reverse;

use super::{Game, Trophy, TrophyLevel, TrophyTier};

pub struct EarnedTrophy<'a> {
    pub trophy: &'a Trophy,
    pub game: &'a Game,
}

pub trait TrophyStatistics {
    fn points(&self) -> u64;
    fn level(&self) -> TrophyLevel;
    fn earned_count(&self, tier: TrophyTier) -> usize;
    fn recently_earned(&self, limit: usize) -> Vec<EarnedTrophy<'_>>;
}

impl TrophyStatistics for [Game] {
    fn points(&self) -> u64 {
        earned_trophies(self).map(|trophy| u64::from(trophy.tier.points())).sum()
    }

    fn level(&self) -> TrophyLevel {
        TrophyLevel::from_points(self.points())
    }

    fn earned_count(&self, tier: TrophyTier) -> usize {
        earned_trophies(self).filter(|trophy| trophy.tier == tier).count()
    }

    fn recently_earned(&self, limit: usize) -> Vec<EarnedTrophy<'_>> {
        let mut earned: Vec<_> = self
            .iter()
            .flat_map(|game| game.trophies.iter().filter(|trophy| trophy.is_earned).map(move |trophy| EarnedTrophy { trophy, game }))
            .collect();
        earned.sort_by_key(|earned| Reverse(earned.trophy.earned_at));
        earned.truncate(limit);
        earned
    }
}

fn earned_trophies(games: &[Game]) -> impl Iterator<Item = &Trophy> {
    games.iter().flat_map(|game| &game.trophies).filter(|trophy| trophy.is_earned)
}
