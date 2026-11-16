//! Liste les succès Steam lus dans le cache local : `cargo run -p prisma-core --example dump [dossier Steam]`.

use prisma_core::steam::{self, SteamClientCache};

fn main() {
    let cache = match std::env::args().nth(1) {
        Some(root) => SteamClientCache::new(root),
        None => match SteamClientCache::locate() {
            Some(cache) => cache,
            None => return println!("Steam introuvable"),
        },
    };
    let Some(account_id) = cache.current_account_id() else { return println!("Aucun compte Steam") };
    println!("Compte Steam : {account_id}");
    for app_id in cache.app_ids_with_schema() {
        let Some(local) = cache.load_game(app_id, account_id) else {
            println!("{app_id} : illisible");
            continue;
        };
        let game = steam::make_game(app_id, &local.schema_name, &local.achievements, None);
        let earned: Vec<_> = local.achievements.iter().filter(|achievement| achievement.is_earned).collect();
        let percent = (game.progress() * 100.0).floor();
        println!("{app_id} {} — {}/{} · {percent} %", local.schema_name, earned.len(), local.achievements.len());
        for achievement in earned.iter().take(3) {
            let date = achievement.earned_at.map(|date| date.with_timezone(&chrono::Local).format("%d/%m/%Y %H:%M").to_string());
            println!("   ✓ {} — {}", achievement.name, date.unwrap_or_else(|| "?".into()));
        }
    }
}
