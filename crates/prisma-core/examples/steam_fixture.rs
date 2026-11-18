//! Écrit un faux client Steam pour essayer l'app sans compte : `cargo run -p prisma-core --features test-support --example steam_fixture -- <dossier> [unlock]`.
//! Avec `unlock`, le dernier succès du premier jeu est débloqué maintenant, comme le ferait Steam en pleine partie.

use std::path::PathBuf;

use prisma_core::test_support::{FixtureAchievement, write_steam_directory, write_user_stats};

const ACCOUNT_ID: u32 = 12_345_678;
const UNLOCKED_AT: i64 = 1_700_000_000;
const API_NAMES: [&str; 5] = ["ACH_0", "ACH_1", "ACH_2", "ACH_3", "ACH_4"];

struct FixtureGame {
    app_id: u32,
    name: &'static str,
    achievements: &'static [&'static str],
    earned: usize,
}

const GAMES: [FixtureGame; 3] = [
    FixtureGame {
        app_id: 9_900_001,
        name: "Abyssal Light",
        achievements: &["First Dive", "Forgotten Reef", "Bright Lantern", "Deep Current", "Ocean Floor"],
        earned: 4,
    },
    FixtureGame {
        app_id: 9_900_002,
        name: "Glass City",
        achievements: &["First Window", "Architect", "Crystal Tower", "Perfect Prism"],
        earned: 2,
    },
    FixtureGame { app_id: 9_900_003, name: "Low Orbit", achievements: &["Liftoff", "In Orbit", "Back to Earth"], earned: 3 },
];

fn main() -> std::io::Result<()> {
    let mut arguments = std::env::args().skip(1);
    let root = PathBuf::from(arguments.next().expect("dossier du faux client Steam"));
    if arguments.next().as_deref() == Some("unlock") {
        let game = &GAMES[0];
        let now = chrono::Utc::now().timestamp();
        return write_user_stats(&root, ACCOUNT_ID, game.app_id, &achievements(game, game.achievements.len(), Some(now)));
    }
    for game in &GAMES {
        write_steam_directory(&root, ACCOUNT_ID, game.app_id, game.name, &achievements(game, game.earned, None))?;
    }
    Ok(())
}

/// Les `earned` premiers succès sont obtenus ; le dernier d'entre eux peut porter une date choisie.
fn achievements(game: &FixtureGame, earned: usize, last_unlocked_at: Option<i64>) -> Vec<FixtureAchievement> {
    game.achievements
        .iter()
        .zip(API_NAMES)
        .enumerate()
        .map(|(index, (name, api_name))| FixtureAchievement {
            api_name,
            name,
            unlocked_at: (index < earned).then(|| if index + 1 == earned { last_unlocked_at.unwrap_or(UNLOCKED_AT) } else { UNLOCKED_AT }),
        })
        .collect()
}
