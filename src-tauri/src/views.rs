//! Données préparées pour l'interface.

use prisma_core::domain::{Game, Platform, Timestamp, Trophy, TrophyRef, TrophyStatistics, TrophyTier};
use prisma_core::switch::CatalogEntry;
use serde::Serialize;

use crate::store::LibraryStore;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryOverview {
    pub profile: Profile,
    pub games: Vec<GameSummary>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String,
    pub level: u32,
    pub level_progress: f64,
    pub next_level: u32,
    pub level_tier: TrophyTier,
    pub points: u64,
    pub earned_by_tier: Vec<TierQuantity>,
    pub earned_total: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TierQuantity {
    pub tier: TrophyTier,
    pub count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TierCount {
    pub tier: TrophyTier,
    pub earned: usize,
    pub total: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSummary {
    pub id: String,
    pub title: String,
    pub platform: Platform,
    pub platforms: Vec<Platform>,
    pub systems: Vec<String>,
    pub system_label: String,
    pub cover_url: Option<String>,
    pub progress: f64,
    pub tier_counts: Vec<TierCount>,
    pub is_platinum_earned: bool,
    pub is_group: bool,
    pub earned_count: usize,
    pub trophy_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameDetail {
    #[serde(flatten)]
    pub summary: GameSummary,
    /// Seuls les jeux Switch sont ajoutés à la main, et donc retirables.
    pub is_removable: bool,
    pub trophies: Vec<TrophyView>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrophyView {
    pub id: String,
    pub reference: TrophyRef,
    pub name: String,
    pub detail: String,
    pub tier: TrophyTier,
    pub is_hidden: bool,
    /// Platine ajouté par Prisma aux jeux Steam et Switch ; l'interface traduit sa description.
    pub is_generated_platinum: bool,
    pub icon_url: Option<String>,
    pub is_earned: bool,
    pub earned_at: Option<Timestamp>,
    pub earned_on: Vec<String>,
    /// Trophée Switch, coché à la main.
    pub can_toggle_earned: bool,
    /// Succès Steam, dont le grade estimé peut être corrigé.
    pub can_change_tier: bool,
    pub has_tier_override: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItem {
    pub id: String,
    pub title: String,
    pub trophy_counts: Vec<TierQuantity>,
    pub is_added: bool,
}

impl LibraryOverview {
    pub fn new(profile_name: String, games: &[Game]) -> Self {
        let level = games.level();
        let earned_by_tier: Vec<TierQuantity> =
            tiers_from_platinum().map(|tier| TierQuantity { tier, count: games.earned_count(tier) }).collect();
        LibraryOverview {
            profile: Profile {
                name: profile_name,
                level: level.value,
                level_progress: level.progress,
                next_level: level.next(),
                level_tier: level.tier(),
                points: games.points(),
                earned_total: earned_by_tier.iter().map(|earned| earned.count).sum(),
                earned_by_tier,
            },
            games: games.iter().map(GameSummary::new).collect(),
        }
    }
}

impl GameSummary {
    fn new(game: &Game) -> Self {
        GameSummary {
            id: game.id.to_string(),
            title: game.title.clone(),
            platform: game.platform,
            platforms: game.platforms().into_iter().collect(),
            systems: game.systems.clone(),
            system_label: game.system_label(),
            cover_url: game.cover_url.clone(),
            progress: game.progress(),
            tier_counts: tiers_from_platinum()
                .map(|tier| {
                    let tally = game.tally(tier);
                    TierCount { tier, earned: tally.earned, total: tally.total }
                })
                .filter(|count| count.total > 0)
                .collect(),
            is_platinum_earned: game.is_platinum_earned(),
            is_group: game.is_group,
            earned_count: game.earned_count(),
            trophy_count: game.trophies.len(),
        }
    }
}

impl GameDetail {
    pub fn new(game: &Game, store: &LibraryStore) -> Self {
        GameDetail {
            summary: GameSummary::new(game),
            is_removable: game.platform == Platform::Nintendo && !game.is_group,
            trophies: game.trophies.iter().map(|trophy| TrophyView::new(trophy, game, store)).collect(),
        }
    }
}

impl TrophyView {
    fn new(trophy: &Trophy, game: &Game, store: &LibraryStore) -> Self {
        let reference = game.reference(trophy);
        let is_editable_on = |platform| !trophy.is_generated_platinum && reference.game_id.platform() == Some(platform);
        TrophyView {
            id: trophy.id.clone(),
            name: trophy.name.clone(),
            detail: trophy.detail.clone(),
            tier: trophy.tier,
            is_hidden: trophy.is_hidden,
            is_generated_platinum: trophy.is_generated_platinum,
            icon_url: if trophy.is_earned { trophy.icon_url.clone() } else { trophy.locked_icon_url.clone().or(trophy.icon_url.clone()) },
            is_earned: trophy.is_earned,
            earned_at: trophy.earned_at,
            earned_on: if game.is_group { trophy.earned_on.clone() } else { Vec::new() },
            can_toggle_earned: is_editable_on(Platform::Nintendo),
            can_change_tier: is_editable_on(Platform::Steam),
            has_tier_override: store.has_tier_override(&reference),
            reference,
        }
    }
}

impl CatalogItem {
    pub fn new(entry: &CatalogEntry, is_added: bool) -> Self {
        CatalogItem {
            id: entry.id.to_string(),
            title: entry.title.to_string(),
            trophy_counts: tiers_from_platinum().map(|tier| TierQuantity { tier, count: entry.trophy_count(tier) }).collect(),
            is_added,
        }
    }
}

fn tiers_from_platinum() -> impl Iterator<Item = TrophyTier> {
    TrophyTier::ALL.into_iter().rev()
}

#[cfg(test)]
mod tests {
    use prisma_core::domain::{Game, GameId, Platform, Trophy, TrophyTier, system};
    use prisma_core::{steam, switch};

    use super::*;
    use crate::testing::TestApp;

    fn trophy_views(game: &Game) -> Vec<TrophyView> {
        let directory = tempfile::tempdir().unwrap();
        let app = TestApp::new(directory.path());
        GameDetail::new(game, &app.store).trophies
    }

    #[test]
    fn switch_trophies_can_be_checked_but_not_the_generated_platinum() {
        let views = trophy_views(&switch::make_game(&switch::custom_entry("Tiny Isles")));

        assert!(!views[0].can_toggle_earned);
        assert!(views[1..].iter().all(|view| view.can_toggle_earned && !view.can_change_tier));
    }

    #[test]
    fn steam_achievements_can_change_tier() {
        let achievements: Vec<_> = (0..5)
            .map(|index| steam::SteamAchievement { api_name: format!("A{index}"), name: format!("A{index}"), ..Default::default() })
            .collect();
        let views = trophy_views(&steam::make_game(1, "Celeste", &achievements, None));

        assert!(!views[0].can_change_tier);
        assert!(views[1..].iter().all(|view| view.can_change_tier && !view.can_toggle_earned));
    }

    #[test]
    fn playstation_trophies_are_read_only() {
        let game = Game::new(
            GameId::playstation("A"),
            Platform::Playstation,
            "Astro",
            vec![system::PS5.into()],
            vec![Trophy::new("1", "Départ", "", TrophyTier::Bronze)],
        );

        let views = trophy_views(&game);

        assert!(!views[0].can_toggle_earned && !views[0].can_change_tier);
    }

    #[test]
    fn overview_counts_earned_trophies_by_tier_from_platinum_down() {
        let game = Game::new(
            GameId::playstation("A"),
            Platform::Playstation,
            "Astro",
            vec![system::PS5.into()],
            vec![Trophy::new("1", "Départ", "", TrophyTier::Gold).earned(None), Trophy::new("2", "Arrivée", "", TrophyTier::Bronze)],
        );

        let overview = LibraryOverview::new("Alex".into(), &[game]);

        let tiers: Vec<_> = overview.profile.earned_by_tier.iter().map(|count| (count.tier, count.count)).collect();
        assert_eq!(tiers, [(TrophyTier::Platinum, 0), (TrophyTier::Gold, 1), (TrophyTier::Silver, 0), (TrophyTier::Bronze, 0)]);
        assert_eq!(overview.profile.points, 90);
        assert_eq!(overview.games[0].tier_counts.len(), 2);
    }
}
