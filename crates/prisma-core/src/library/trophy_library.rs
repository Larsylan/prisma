use std::collections::HashMap;

use chrono::Duration;

use crate::domain::{Game, GameId, Platform, Timestamp, Trophy, TrophyRef, TrophyTier};

/// Bibliothèque des jeux tels que lus depuis chaque source, avec les grades choisis par l'utilisateur.
#[derive(Clone, Debug, Default)]
pub struct TrophyLibrary {
    games: HashMap<GameId, Game>,
    tier_overrides: HashMap<TrophyRef, TrophyTier>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Change {
    pub did_change: bool,
    pub newly_earned: Vec<Trophy>,
}

impl Change {
    fn unchanged() -> Self {
        Change::default()
    }

    fn changed(newly_earned: Vec<Trophy>) -> Self {
        Change { did_change: true, newly_earned }
    }
}

/// Un trophée sans historique n'est annoncé que s'il vient d'être obtenu :
/// l'import initial d'une collection ne doit pas déclencher de notifications.
fn fresh_unlock_window() -> Duration {
    Duration::seconds(120)
}

impl TrophyLibrary {
    pub fn new(games: impl IntoIterator<Item = Game>, tier_overrides: HashMap<TrophyRef, TrophyTier>) -> Self {
        let games = games.into_iter().map(|game| (game.id.clone(), game)).collect();
        TrophyLibrary { games, tier_overrides }
    }

    pub fn games(&self) -> impl Iterator<Item = &Game> {
        self.games.values()
    }

    pub fn game(&self, id: &GameId) -> Option<&Game> {
        self.games.get(id)
    }

    pub fn games_on(&self, platform: Platform) -> impl Iterator<Item = &Game> {
        self.games.values().filter(move |game| game.platform == platform)
    }

    pub fn tier_overrides(&self) -> &HashMap<TrophyRef, TrophyTier> {
        &self.tier_overrides
    }

    pub fn has_tier_override(&self, reference: &TrophyRef) -> bool {
        self.tier_overrides.contains_key(reference)
    }

    /// Intègre un jeu fraîchement lu depuis sa source. Un trophée déjà obtenu le reste,
    /// même si la source l'ignore (cache local vidé, catalogue réécrit…).
    pub fn integrate(&mut self, incoming: Game, now: Timestamp) -> Change {
        let previous = self.games.get(&incoming.id);
        let previous_trophies: HashMap<&str, &Trophy> =
            previous.map(|game| game.trophies.iter().map(|trophy| (trophy.id.as_str(), trophy)).collect()).unwrap_or_default();
        let mut game = incoming;
        let mut newly_earned = Vec::new();

        for trophy in game.trophies.iter_mut().filter(|trophy| !trophy.is_generated_platinum) {
            let is_new = reconcile(trophy, previous_trophies.get(trophy.id.as_str()).copied(), previous.is_some(), now);
            trophy.automatic_tier = trophy.tier;
            if let Some(tier) = self.tier_overrides.get(&TrophyRef::new(game.id.clone(), trophy.id.clone())) {
                trophy.tier = *tier;
            }
            if is_new {
                newly_earned.push(trophy.clone());
            }
        }

        let previous_platinum = previous.and_then(Game::generated_platinum).cloned();
        if let Some(platinum) = refresh_generated_platinum(&mut game, previous_platinum.as_ref(), now) {
            if !newly_earned.is_empty() {
                newly_earned.push(platinum);
            }
        }

        if previous == Some(&game) {
            return Change::unchanged();
        }
        self.games.insert(game.id.clone(), game);
        Change::changed(newly_earned)
    }

    /// Coche ou décoche un trophée à la main, pour les jeux dont la progression n'est pas détectable.
    pub fn set_earned(&mut self, earned: bool, reference: &TrophyRef, now: Timestamp) -> Change {
        let Some(game) = self.games.get_mut(&reference.game_id) else { return Change::unchanged() };
        let previous_platinum = game.generated_platinum().cloned();
        let Some(trophy) = game.trophies.iter_mut().find(|trophy| trophy.id == reference.trophy_id) else {
            return Change::unchanged();
        };
        if trophy.is_generated_platinum || trophy.is_earned == earned {
            return Change::unchanged();
        }

        if earned {
            trophy.mark_earned(Some(now));
        } else {
            trophy.mark_locked();
        }
        let mut newly_earned = if earned { vec![trophy.clone()] } else { Vec::new() };
        if let Some(platinum) = refresh_generated_platinum(game, previous_platinum.as_ref(), now) {
            newly_earned.push(platinum);
        }
        Change::changed(newly_earned)
    }

    /// Choisit le grade d'un trophée ; `None` rétablit le grade automatique.
    pub fn set_tier(&mut self, tier: Option<TrophyTier>, reference: &TrophyRef) -> bool {
        let Some(trophy) = self
            .games
            .get_mut(&reference.game_id)
            .and_then(|game| game.trophies.iter_mut().find(|trophy| trophy.id == reference.trophy_id))
        else {
            return false;
        };
        trophy.tier = tier.unwrap_or(trophy.automatic_tier);
        match tier {
            Some(tier) => self.tier_overrides.insert(reference.clone(), tier),
            None => self.tier_overrides.remove(reference),
        };
        true
    }

    pub fn remove(&mut self, id: &GameId) -> bool {
        if self.games.remove(id).is_none() {
            return false;
        }
        self.tier_overrides.retain(|reference, _| &reference.game_id != id);
        true
    }

    pub fn remove_all_on(&mut self, platform: Platform) {
        self.games.retain(|_, game| game.platform != platform);
    }
}

/// Retourne vrai si le trophée vient d'être obtenu et doit être annoncé.
fn reconcile(trophy: &mut Trophy, previous: Option<&Trophy>, game_is_known: bool, now: Timestamp) -> bool {
    let Some(previous) = previous else {
        let is_fresh = trophy.earned_at.is_some_and(|earned_at| now - earned_at < fresh_unlock_window());
        return trophy.is_earned && (game_is_known || is_fresh);
    };
    if previous.is_earned {
        let earned_at = if trophy.is_earned { trophy.earned_at.or(previous.earned_at) } else { previous.earned_at };
        trophy.mark_earned(earned_at);
        return false;
    }
    trophy.is_earned
}

/// Le platine généré est obtenu dès que tous les autres trophées le sont.
/// Retourne le platine lorsqu'il vient d'être obtenu.
fn refresh_generated_platinum(game: &mut Game, previous: Option<&Trophy>, now: Timestamp) -> Option<Trophy> {
    let regular: Vec<_> = game.regular_trophies().collect();
    let is_complete = !regular.is_empty() && regular.iter().all(|trophy| trophy.is_earned);
    let latest_unlock = regular.iter().filter_map(|trophy| trophy.earned_at).max();

    let platinum = game.generated_platinum_mut()?;
    if is_complete {
        platinum.mark_earned(Some(previous.and_then(|trophy| trophy.earned_at).or(latest_unlock).unwrap_or(now)));
    } else {
        platinum.mark_locked();
    }
    platinum.automatic_tier = TrophyTier::Platinum;

    let was_earned = previous.is_some_and(|trophy| trophy.is_earned);
    (is_complete && !was_earned).then(|| platinum.clone())
}
