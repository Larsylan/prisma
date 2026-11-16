use std::collections::BTreeSet;

use serde::Serialize;

use super::{GameId, Platform, Timestamp, Trophy, TrophyRef, TrophyTier};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: GameId,
    pub platform: Platform,
    pub title: String,
    pub systems: Vec<String>,
    pub cover_url: Option<String>,
    pub trophies: Vec<Trophy>,
    /// Vrai pour une entrée d'affichage qui réunit plusieurs versions d'un même jeu.
    pub is_group: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct TierTally {
    pub earned: usize,
    pub total: usize,
}

impl Game {
    pub fn new(id: GameId, platform: Platform, title: impl Into<String>, systems: Vec<String>, trophies: Vec<Trophy>) -> Self {
        Game { id, platform, title: title.into(), systems, cover_url: None, trophies, is_group: false }
    }

    pub fn system_label(&self) -> String {
        self.systems.join(" / ")
    }

    pub fn platforms(&self) -> BTreeSet<Platform> {
        let detected: BTreeSet<_> = self.systems.iter().filter_map(|system| Platform::from_system(system)).collect();
        if detected.is_empty() { BTreeSet::from([self.platform]) } else { detected }
    }

    pub fn earned_count(&self) -> usize {
        self.trophies.iter().filter(|trophy| trophy.is_earned).count()
    }

    pub fn last_earned_at(&self) -> Option<Timestamp> {
        self.trophies.iter().filter(|trophy| trophy.is_earned).filter_map(|trophy| trophy.earned_at).max()
    }

    pub fn is_platinum_earned(&self) -> bool {
        self.trophies.iter().any(|trophy| trophy.tier == TrophyTier::Platinum && trophy.is_earned)
    }

    /// Progression pondérée par les points, platine exclu, comme sur console.
    pub fn progress(&self) -> f64 {
        let weighted: Vec<_> = self.trophies.iter().filter(|trophy| trophy.tier != TrophyTier::Platinum).collect();
        let total: u32 = weighted.iter().map(|trophy| trophy.tier.points()).sum();
        if total == 0 {
            let is_complete = !self.trophies.is_empty() && self.trophies.iter().all(|trophy| trophy.is_earned);
            return if is_complete { 1.0 } else { 0.0 };
        }
        let earned: u32 = weighted.iter().filter(|trophy| trophy.is_earned).map(|trophy| trophy.tier.points()).sum();
        f64::from(earned) / f64::from(total)
    }

    pub fn tally(&self, tier: TrophyTier) -> TierTally {
        let of_tier = self.trophies.iter().filter(|trophy| trophy.tier == tier);
        let (earned, total) = of_tier.fold((0, 0), |(earned, total), trophy| (earned + usize::from(trophy.is_earned), total + 1));
        TierTally { earned, total }
    }

    /// Trophée stocké correspondant à un trophée affiché par ce jeu.
    pub fn reference(&self, trophy: &Trophy) -> TrophyRef {
        trophy.origin.clone().unwrap_or_else(|| TrophyRef::new(self.id.clone(), trophy.id.clone()))
    }

    pub fn rename(&mut self, new_title: &str) {
        self.title = new_title.to_owned();
        let (name, detail) = Trophy::generated_platinum_text(new_title);
        if let Some(platinum) = self.generated_platinum_mut() {
            platinum.name = name;
            platinum.detail = detail;
        }
    }

    pub fn regular_trophies(&self) -> impl Iterator<Item = &Trophy> {
        self.trophies.iter().filter(|trophy| !trophy.is_generated_platinum)
    }

    pub(crate) fn generated_platinum(&self) -> Option<&Trophy> {
        self.trophies.iter().find(|trophy| trophy.is_generated_platinum)
    }

    pub(crate) fn generated_platinum_mut(&mut self) -> Option<&mut Trophy> {
        self.trophies.iter_mut().find(|trophy| trophy.is_generated_platinum)
    }
}
