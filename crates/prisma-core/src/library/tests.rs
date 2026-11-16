use chrono::{DateTime, Duration};

use super::*;
use crate::domain::{Game, GameId, Platform, Timestamp, Trophy, TrophyRef, TrophyTier, system};
use crate::steam;
use crate::switch;

fn reference_date() -> Timestamp {
    DateTime::from_timestamp(1_700_000_000, 0).expect("date valide")
}

fn long_ago() -> Timestamp {
    reference_date() - Duration::days(1)
}

fn trophy(id: &str, tier: TrophyTier, earned_at: Option<Timestamp>) -> Trophy {
    let trophy = Trophy::new(id, id, "", tier);
    match earned_at {
        Some(date) => trophy.earned(Some(date)),
        None => trophy,
    }
}

fn named(id: &str, name: &str, earned_at: Option<Timestamp>) -> Trophy {
    Trophy { name: name.into(), ..trophy(id, TrophyTier::Bronze, earned_at) }
}

fn playstation(id: &str, title: &str, systems: &[&str], trophies: Vec<Trophy>) -> Game {
    Game::new(GameId::playstation(id), Platform::Playstation, title, systems.iter().map(|system| system.to_string()).collect(), trophies)
}

fn with_generated_platinum(regular: Vec<Trophy>) -> Game {
    let mut trophies = vec![Trophy::generated_platinum("Test", None)];
    trophies.extend(regular);
    Game::new(GameId::nintendo("test"), Platform::Nintendo, "Test", vec![system::SWITCH.into()], trophies)
}

fn ids(trophies: &[Trophy]) -> Vec<&str> {
    trophies.iter().map(|trophy| trophy.id.as_str()).collect()
}

mod trophy_library {
    use super::*;

    #[test]
    fn initial_import_does_not_announce_old_trophies() {
        let mut library = TrophyLibrary::default();
        let change = library
            .integrate(playstation("A", "Jeu", &[system::PS5], vec![trophy("1", TrophyTier::Bronze, Some(long_ago()))]), reference_date());

        assert!(change.did_change);
        assert!(change.newly_earned.is_empty());
    }

    #[test]
    fn initial_import_announces_trophy_earned_moments_ago() {
        let mut library = TrophyLibrary::default();
        let just_now = reference_date() - Duration::seconds(30);
        let change = library
            .integrate(playstation("A", "Jeu", &[system::PS5], vec![trophy("1", TrophyTier::Bronze, Some(just_now))]), reference_date());

        assert_eq!(ids(&change.newly_earned), ["1"]);
    }

    #[test]
    fn trophy_earned_since_last_read_is_announced() {
        let mut library = TrophyLibrary::default();
        library.integrate(playstation("A", "Jeu", &[system::PS5], vec![trophy("1", TrophyTier::Bronze, None)]), reference_date());

        let change = library.integrate(
            playstation("A", "Jeu", &[system::PS5], vec![trophy("1", TrophyTier::Bronze, Some(reference_date()))]),
            reference_date(),
        );

        assert_eq!(ids(&change.newly_earned), ["1"]);
    }

    #[test]
    fn earned_trophy_stays_earned_when_source_forgets_it() {
        let mut library = TrophyLibrary::default();
        library
            .integrate(playstation("A", "Jeu", &[system::PS5], vec![trophy("1", TrophyTier::Bronze, Some(long_ago()))]), reference_date());
        library.integrate(playstation("A", "Jeu", &[system::PS5], vec![trophy("1", TrophyTier::Bronze, None)]), reference_date());

        let stored = &library.game(&GameId::playstation("A")).expect("jeu stocké").trophies[0];
        assert!(stored.is_earned);
        assert_eq!(stored.earned_at, Some(long_ago()));
    }

    #[test]
    fn identical_read_reports_no_change() {
        let mut library = TrophyLibrary::default();
        let game = playstation("A", "Jeu", &[system::PS5], vec![trophy("1", TrophyTier::Bronze, None)]);
        library.integrate(game.clone(), reference_date());

        assert!(!library.integrate(game, reference_date()).did_change);
    }

    #[test]
    fn generated_platinum_is_earned_and_announced_with_last_trophy() {
        let mut library = TrophyLibrary::default();
        library.integrate(
            with_generated_platinum(vec![trophy("a", TrophyTier::Bronze, Some(long_ago())), trophy("b", TrophyTier::Bronze, None)]),
            reference_date(),
        );

        let change = library.integrate(
            with_generated_platinum(vec![
                trophy("a", TrophyTier::Bronze, Some(long_ago())),
                trophy("b", TrophyTier::Bronze, Some(reference_date())),
            ]),
            reference_date(),
        );

        assert_eq!(ids(&change.newly_earned), ["b", "__platinum__"]);
        assert_eq!(change.newly_earned[1].earned_at, Some(reference_date()));
    }

    #[test]
    fn tier_override_survives_new_reads_and_can_be_reset() {
        let mut library = TrophyLibrary::default();
        let game = playstation("A", "Jeu", &[system::PS5], vec![trophy("1", TrophyTier::Bronze, None)]);
        let reference = TrophyRef::new(game.id.clone(), "1");
        library.integrate(game.clone(), reference_date());

        assert!(library.set_tier(Some(TrophyTier::Gold), &reference));
        library.integrate(game.clone(), reference_date());
        assert_eq!(library.game(&game.id).unwrap().trophies[0].tier, TrophyTier::Gold);
        assert!(library.has_tier_override(&reference));

        assert!(library.set_tier(None, &reference));
        assert_eq!(library.game(&game.id).unwrap().trophies[0].tier, TrophyTier::Bronze);
        assert!(!library.has_tier_override(&reference));
    }

    #[test]
    fn checking_last_trophy_by_hand_earns_platinum() {
        let mut library = TrophyLibrary::default();
        let game = with_generated_platinum(vec![trophy("a", TrophyTier::Bronze, Some(long_ago())), trophy("b", TrophyTier::Bronze, None)]);
        library.integrate(game.clone(), long_ago());

        let change = library.set_earned(true, &TrophyRef::new(game.id.clone(), "b"), reference_date());

        assert_eq!(ids(&change.newly_earned), ["b", "__platinum__"]);
        assert!(library.game(&game.id).unwrap().is_platinum_earned());
    }

    #[test]
    fn unchecking_trophy_locks_platinum_again() {
        let mut library = TrophyLibrary::default();
        let game = with_generated_platinum(vec![trophy("a", TrophyTier::Bronze, Some(long_ago()))]);
        library.integrate(game.clone(), reference_date());
        assert!(library.game(&game.id).unwrap().is_platinum_earned());

        let change = library.set_earned(false, &TrophyRef::new(game.id.clone(), "a"), reference_date());

        assert!(change.did_change);
        assert!(change.newly_earned.is_empty());
        assert!(!library.game(&game.id).unwrap().is_platinum_earned());
    }

    #[test]
    fn generated_platinum_cannot_be_checked_by_hand() {
        let mut library = TrophyLibrary::default();
        let game = with_generated_platinum(vec![trophy("a", TrophyTier::Bronze, None)]);
        library.integrate(game.clone(), reference_date());

        assert!(!library.set_earned(true, &TrophyRef::new(game.id, "__platinum__"), reference_date()).did_change);
    }

    #[test]
    fn removing_game_forgets_its_tier_overrides() {
        let mut library = TrophyLibrary::default();
        let game = playstation("A", "Jeu", &[system::PS5], vec![trophy("1", TrophyTier::Bronze, None)]);
        let reference = TrophyRef::new(game.id.clone(), "1");
        library.integrate(game.clone(), reference_date());
        library.set_tier(Some(TrophyTier::Gold), &reference);

        assert!(library.remove(&game.id));
        assert_eq!(library.games().count(), 0);
        assert!(!library.has_tier_override(&reference));
    }
}

mod grouper {
    use super::*;

    fn group(games: &[Game], mode: GroupingMode) -> Vec<Game> {
        GameGrouper::default().group(games, mode)
    }

    #[test]
    fn versions_of_same_game_are_combined() {
        let ps4 = playstation(
            "PS4",
            "Astro (PS4)",
            &[system::PS4],
            vec![trophy("p", TrophyTier::Platinum, None), named("1", "Départ", Some(reference_date()))],
        );
        let ps5 = playstation(
            "PS5",
            "Astro",
            &[system::PS5],
            vec![trophy("p", TrophyTier::Platinum, None), named("9", "départ !", None), named("10", "Arrivée", None)],
        );

        let entries = group(&[ps4, ps5.clone()], GroupingMode::All);
        let combined = &entries[0];

        assert_eq!(entries.len(), 1);
        assert!(combined.is_group);
        assert_eq!(combined.title, "Astro");
        assert_eq!(combined.systems, [system::PS4, system::PS5]);
        assert_eq!(combined.trophies.iter().filter(|trophy| trophy.tier == TrophyTier::Platinum).count(), 1);
        assert_eq!(combined.trophies.len(), 3);

        let departure = combined.trophies.iter().find(|trophy| trophy.name == "départ !").expect("trophée fusionné");
        assert!(departure.is_earned);
        assert_eq!(departure.earned_on, [system::PS4]);
        assert_eq!(combined.reference(departure), TrophyRef::new(ps5.id, "9"));
    }

    #[test]
    fn mode_decides_which_platforms_are_grouped() {
        let playstation = playstation("A", "Hades", &[system::PS5], vec![]);
        let steam = steam::make_game(1, "Hades", &[], None);
        let games = [playstation, steam];

        assert_eq!(group(&games, GroupingMode::Playstation).len(), 2);
        assert_eq!(group(&games, GroupingMode::All).len(), 1);
        assert_eq!(group(&games, GroupingMode::None).len(), 2);
    }

    #[test]
    fn steam_and_switch_are_listed_after_playstation() {
        let games = [
            steam::make_game(1, "Celeste", &[], None),
            playstation("B", "Celeste", &[system::PS5], vec![]),
            playstation("A", "Celeste", &[system::PS4], vec![]),
        ];

        let combined = &group(&games, GroupingMode::All)[0];

        assert_eq!(combined.system_label(), "PS4 / PS5 / Steam");
        assert_eq!(combined.platform, Platform::Playstation);
    }

    #[test]
    fn repeated_grouping_gives_same_entries() {
        let mut grouper = GameGrouper::default();
        let games = [playstation("A", "Hades", &[system::PS5], vec![]), steam::make_game(1, "Hades", &[], None)];

        assert_eq!(grouper.group(&games, GroupingMode::All), grouper.group(&games, GroupingMode::All));
    }
}

mod repository {
    use super::*;

    /// Extrait du format écrit par les versions précédentes de l'app.
    const LEGACY_JSON: &str = r#"{
      "games": [{
        "id": "steam:49520", "platform": "steam", "title": "Borderlands", "system": "Steam",
        "coverURL": "https://example.com/header.jpg",
        "trophies": [
          {"id": "__platinum__", "name": "Borderlands", "detail": "", "tier": "platinum", "autoTier": "platinum",
           "hidden": false, "earned": false, "isSynthetic": true},
          {"id": "ACH_1", "name": "Pourfendeur", "detail": "Vaincre le dragon", "tier": "gold", "autoTier": "bronze",
           "hidden": true, "earned": true, "earnedAt": 1600000000, "isSynthetic": false}
        ]
      }, {
        "id": "psn:NPWR1", "platform": "playstation", "title": "Astro", "system": "PS4 / PS5",
        "trophies": [], "mergedSystems": null
      }],
      "overrides": {"steam:49520|ACH_1": "gold"},
      "psnStamps": {"NPWR1": "2024-01-01T00:00:00Z"},
      "steamRarity": {"49520": {"ACH_1": 3.5}},
      "steamNames": {"49520": "Borderlands"}
    }"#;

    fn repository_with_legacy_file() -> (tempfile::TempDir, LibraryRepository) {
        let directory = tempfile::tempdir().expect("dossier temporaire");
        std::fs::write(directory.path().join("library.json"), LEGACY_JSON).unwrap();
        let repository = LibraryRepository::new(directory.path());
        (directory, repository)
    }

    #[test]
    fn reads_legacy_library() {
        let (_directory, repository) = repository_with_legacy_file();

        let stored = repository.load().expect("bibliothèque lisible");
        let steam_game = stored.library.game(&GameId::steam(49520)).expect("jeu Steam");
        let achievement = &steam_game.trophies[1];

        assert!(steam_game.trophies[0].is_generated_platinum);
        assert_eq!(achievement.tier, TrophyTier::Gold);
        assert_eq!(achievement.automatic_tier, TrophyTier::Bronze);
        assert!(achievement.is_hidden);
        assert_eq!(achievement.earned_at.map(|date| date.timestamp()), Some(1_600_000_000));
        assert_eq!(stored.library.game(&GameId::playstation("NPWR1")).unwrap().systems, [system::PS4, system::PS5]);
        assert!(stored.library.has_tier_override(&TrophyRef::new(steam_game.id.clone(), "ACH_1")));
        assert_eq!(stored.cache.steam_rarity[&49520]["ACH_1"], 3.5);
        assert_eq!(stored.cache.steam_titles[&49520], "Borderlands");
        assert_eq!(stored.cache.playstation_revisions["NPWR1"], "2024-01-01T00:00:00Z");
    }

    #[test]
    fn saved_library_reads_back_identically() {
        let (_directory, repository) = repository_with_legacy_file();
        let original = repository.load().unwrap();

        repository.save(&original).unwrap();
        let reloaded = repository.load().unwrap();

        let sorted = |stored: &StoredLibrary| {
            let mut games: Vec<Game> = stored.library.games().cloned().collect();
            games.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));
            games
        };
        assert_eq!(sorted(&reloaded), sorted(&original));
        assert_eq!(reloaded.library.tier_overrides(), original.library.tier_overrides());
        assert_eq!(reloaded.cache, original.cache);
    }

    #[test]
    fn missing_file_gives_empty_library() {
        let directory = tempfile::tempdir().unwrap();
        assert_eq!(LibraryRepository::new(directory.path()).load().unwrap().library.games().count(), 0);
    }

    #[test]
    fn unreadable_file_is_set_aside() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("library.json"), "{").unwrap();
        let repository = LibraryRepository::new(directory.path());

        assert!(repository.load().is_err());
        let backup = repository.quarantine_unreadable_file(reference_date()).unwrap();

        assert!(backup.exists());
        assert_eq!(repository.load().unwrap().library.games().count(), 0);
    }
}

mod domain {
    use super::*;
    use crate::domain::{TrophyLevel, TrophyStatistics};

    #[test]
    fn levels_follow_point_bands() {
        for (points, level, tier) in [
            (0, 1, TrophyTier::Bronze),
            (59, 1, TrophyTier::Bronze),
            (60, 2, TrophyTier::Bronze),
            (5_880, 99, TrophyTier::Bronze),
            (5_940, 100, TrophyTier::Silver),
            (59_940, 300, TrophyTier::Gold),
            (464_940, 600, TrophyTier::Platinum),
        ] {
            let computed = TrophyLevel::from_points(points);
            assert_eq!((computed.value, computed.tier()), (level, tier), "{points} points");
        }
        assert_eq!(TrophyLevel::from_points(30).progress, 0.5);
        assert_eq!(TrophyLevel::from_points(u64::MAX).value, TrophyLevel::MAXIMUM);
        assert_eq!(TrophyLevel::from_points(u64::MAX).next(), TrophyLevel::MAXIMUM);
    }

    #[test]
    fn progress_is_weighted_by_points_and_excludes_platinum() {
        let game = playstation(
            "A",
            "Jeu",
            &[system::PS5],
            vec![
                trophy("p", TrophyTier::Platinum, None),
                trophy("g", TrophyTier::Gold, Some(reference_date())),
                trophy("b", TrophyTier::Bronze, None),
            ],
        );

        assert_eq!(game.progress(), 90.0 / 105.0);
        assert_eq!(game.tally(TrophyTier::Gold), crate::domain::TierTally { earned: 1, total: 1 });
    }

    #[test]
    fn renaming_updates_generated_platinum() {
        let mut game = with_generated_platinum(vec![]);
        game.rename("Metroid Dread");

        assert_eq!(game.title, "Metroid Dread");
        assert_eq!(game.trophies[0].name, "Metroid Dread");
        assert_eq!(game.trophies[0].detail, "Obtenir tous les trophées de Metroid Dread.");
    }

    #[test]
    fn statistics_sum_earned_trophies() {
        let games = [playstation(
            "A",
            "Jeu",
            &[system::PS5],
            vec![
                trophy("g", TrophyTier::Gold, Some(reference_date())),
                trophy("s", TrophyTier::Silver, Some(reference_date() + Duration::seconds(10))),
                trophy("b", TrophyTier::Bronze, None),
            ],
        )];

        assert_eq!(games.points(), 120);
        assert_eq!(games.earned_count(TrophyTier::Gold), 1);
        assert_eq!(games.recently_earned(1)[0].trophy.id, "s");
    }

    #[test]
    fn identifiers_carry_their_platform() {
        assert_eq!(GameId::steam(730).as_str(), "steam:730");
        assert_eq!(GameId::steam(730).steam_app_id(), Some(730));
        assert_eq!(GameId::steam(730).platform(), Some(Platform::Steam));
        assert_eq!(GameId::playstation("NPWR1").platform(), Some(Platform::Playstation));
        assert_eq!(GameId::nintendo("zelda").nintendo_catalog_id(), Some("zelda"));
        assert_eq!(GameId::group("zelda").platform(), None);
        assert_eq!(GameId::nintendo("zelda").steam_app_id(), None);
    }

    #[test]
    fn switch_catalog_games_keep_their_identifiers() {
        let game = switch::make_game(&switch::custom_entry("Hollow Knight"));
        assert_eq!(game.id.as_str(), "switch:custom-hollow-knight");
    }
}
