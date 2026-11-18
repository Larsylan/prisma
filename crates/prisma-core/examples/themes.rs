//! Écrit la liste des thèmes au format JSON, pour le mode démo de l'interface :
//! `cargo run -p prisma-core --example themes > src/lib/demo-themes.json`.

use prisma_core::theme::Preset;
use serde_json::json;

fn main() {
    let themes: Vec<_> = Preset::ALL.into_iter().map(|preset| json!({ "preset": preset, "palette": preset.palette() })).collect();
    println!("{}", serde_json::to_string_pretty(&themes).expect("thèmes sérialisables"));
}
