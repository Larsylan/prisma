use prisma_core::theme::{Preset, ThemePalette};
use serde::Serialize;

/// Une entrée de la liste des thèmes, avec de quoi la prévisualiser.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeOption {
    pub preset: Preset,
    pub palette: ThemePalette,
}

pub fn options() -> Vec<ThemeOption> {
    Preset::ALL.into_iter().map(|preset| ThemeOption { preset, palette: preset.palette() }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_is_offered_once() {
        let options = options();

        assert_eq!(options.len(), Preset::ALL.len());
        assert_eq!(options[0].preset, Preset::Night);
    }
}
