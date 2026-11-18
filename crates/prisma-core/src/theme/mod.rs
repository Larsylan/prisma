//! Thèmes de couleur de l'interface.

mod color;

use serde::{Deserialize, Serialize};

pub use color::Color;

/// Un thème clair affiche son texte en sombre ; un thème sombre, en blanc.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Appearance {
    Dark,
    Light,
}

impl Appearance {
    pub fn text_color(self) -> Color {
        match self {
            Appearance::Dark => Color::rgb(0xffffff),
            Appearance::Light => Color::rgb(0x141622),
        }
    }
}

/// Couleurs d'un thème.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemePalette {
    pub appearance: Appearance,
    /// Dégradé vertical du fond, du haut vers le bas.
    pub background: [Color; 3],
    /// Boutons principaux et champs actifs.
    pub accent: Color,
    /// Mentions « Obtenu » et cases cochées : une couleur lisible comme du texte.
    pub highlight: Color,
    /// Couleurs successives des barres de progression, de gauche à droite.
    pub progress: Vec<Color>,
    /// Les thèmes néon font briller les barres de progression et les mentions colorées.
    pub glows: bool,
}

/// Thèmes proposés ; le premier est celui par défaut.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Preset {
    #[default]
    Night,
    Indigo,
    Amethyst,
    Sakura,
    Ruby,
    Ember,
    Amber,
    Emerald,
    Ocean,
    Aurora,
    Twilight,
    Graphite,
    NeonCyan,
    NeonMagenta,
    NeonAcid,
    Synthwave,
    WhiteNeon,
    SpectralBlack,
}

impl Preset {
    pub const ALL: [Preset; 18] = [
        Preset::Night,
        Preset::Indigo,
        Preset::Amethyst,
        Preset::Sakura,
        Preset::Ruby,
        Preset::Ember,
        Preset::Amber,
        Preset::Emerald,
        Preset::Ocean,
        Preset::Aurora,
        Preset::Twilight,
        Preset::Graphite,
        Preset::NeonCyan,
        Preset::NeonMagenta,
        Preset::NeonAcid,
        Preset::Synthwave,
        Preset::WhiteNeon,
        Preset::SpectralBlack,
    ];

    pub fn palette(self) -> ThemePalette {
        match self {
            Preset::Night => classic([0x0b1d46, 0x050b1e, 0x02040c], 0x2d6bf6, 0x6fb1ff),
            Preset::Indigo => classic([0x1b1550, 0x0b0924, 0x04030d], 0x6366f1, 0xa5b4fc),
            Preset::Amethyst => classic([0x2a1446, 0x120a22, 0x06030c], 0x9b5cf6, 0xcda8ff),
            Preset::Sakura => classic([0x3d1230, 0x1b0715, 0x0a0308], 0xec5fa6, 0xf9aed3),
            Preset::Ruby => classic([0x430d16, 0x1e0509, 0x0b0204], 0xe5394f, 0xff9aa7),
            Preset::Ember => classic([0x46180b, 0x200b05, 0x0c0402], 0xf06a2c, 0xffb089),
            Preset::Amber => classic([0x3a2a08, 0x1a1203, 0x0a0701], 0xe0a91c, 0xf5d47a),
            Preset::Emerald => classic([0x0e3a2e, 0x061a15, 0x020806], 0x22b07a, 0x7de3b8),
            Preset::Ocean => classic([0x0a3440, 0x04171d, 0x01080a], 0x14b8c4, 0x7fe3ea),
            Preset::Aurora => classic([0x0b2f3a, 0x071624, 0x02070c], 0xec5fa6, 0x8ff0c8),
            Preset::Twilight => classic([0x3a1446, 0x1a0a22, 0x0a0408], 0xff8a4c, 0xffc29b),
            Preset::Graphite => classic([0x23262d, 0x111317, 0x060708], 0x9aa7bd, 0xd1d8e4),
            Preset::NeonCyan => neon([0x041a24, 0x020b12, 0x000306], 0x00e5ff, 0x7df9ff, &[0x7df9ff, 0x00e5ff]),
            Preset::NeonMagenta => neon([0x24041f, 0x11020f, 0x050005], 0xff2bd6, 0xff8ef0, &[0xff8ef0, 0xff2bd6]),
            Preset::NeonAcid => neon([0x0d1f04, 0x050d02, 0x010300], 0x9dff00, 0xd4ff7a, &[0xf0ff4f, 0x9dff00]),
            Preset::Synthwave => {
                neon([0x2b0b4a, 0x12052a, 0x05010f], 0xff2e88, 0xffb86b, &[0xffd319, 0xff901f, 0xff2975, 0xf222ff, 0x8c1eff])
            }
            Preset::WhiteNeon => ThemePalette {
                appearance: Appearance::Light,
                ..neon([0xffffff, 0xf4f2fb, 0xe9e6f5], 0xd6119a, 0xa4006f, &[0x00c8f0, 0x8f3dff, 0xff1fb4])
            },
            Preset::SpectralBlack => neon(
                [0x0a0a0c, 0x050506, 0x000000],
                0x8f7bff,
                0xd9d4ff,
                &[0xff3b3b, 0xffb13b, 0xf5ff3b, 0x3bff6f, 0x3bd8ff, 0x6b3bff, 0xff3bd5],
            ),
        }
    }
}

/// Thème sombre dont la progression va de la teinte claire à l'accent.
fn classic(background: [u32; 3], accent: u32, highlight: u32) -> ThemePalette {
    ThemePalette {
        appearance: Appearance::Dark,
        background: background.map(Color::rgb),
        accent: Color::rgb(accent),
        highlight: Color::rgb(highlight),
        progress: vec![Color::rgb(highlight), Color::rgb(accent)],
        glows: false,
    }
}

/// Thème aux couleurs vives qui brillent, comme un tube néon.
fn neon(background: [u32; 3], accent: u32, highlight: u32, progress: &[u32]) -> ThemePalette {
    ThemePalette { progress: progress.iter().copied().map(Color::rgb).collect(), glows: true, ..classic(background, accent, highlight) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rapport de contraste WCAG entre deux couleurs, de 1 à 21.
    fn contrast(lhs: Color, rhs: Color) -> f64 {
        let [light, dark] = {
            let (a, b) = (lhs.relative_luminance(), rhs.relative_luminance());
            if a > b { [a, b] } else { [b, a] }
        };
        (light + 0.05) / (dark + 0.05)
    }

    #[test]
    fn every_preset_keeps_text_readable_and_accents_visible() {
        for preset in Preset::ALL {
            let palette = preset.palette();
            let text = palette.appearance.text_color();
            for background in palette.background {
                assert!(contrast(text, background) >= 7.0, "{preset:?} : texte peu lisible sur {background}");
                assert!(contrast(palette.highlight, background) >= 4.5, "{preset:?} : mention « Obtenu » peu visible sur {background}");
                assert!(contrast(palette.accent, background) >= 3.0, "{preset:?} : accent peu visible sur {background}");
            }
            assert!(palette.progress.len() >= 2, "{preset:?} : la progression doit être un dégradé");
        }
    }

    #[test]
    fn white_neon_is_the_only_light_theme() {
        let light: Vec<_> = Preset::ALL.into_iter().filter(|preset| preset.palette().appearance == Appearance::Light).collect();
        assert_eq!(light, [Preset::WhiteNeon]);
    }

    #[test]
    fn night_is_the_default_and_historical_look() {
        assert_eq!(Preset::default(), Preset::Night);
        assert_eq!(Preset::Night.palette().background[0], Color::rgb(0x0b1d46));
    }
}
