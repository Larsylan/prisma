use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// Couleur sRGB, échangée avec l'interface au format `#rrggbb`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const fn rgb(hex: u32) -> Self {
        Color { red: (hex >> 16) as u8, green: (hex >> 8) as u8, blue: hex as u8 }
    }

    pub fn parse(text: &str) -> Option<Self> {
        let digits = text.strip_prefix('#')?;
        if digits.len() != 6 {
            return None;
        }
        u32::from_str_radix(digits, 16).ok().map(Self::rgb)
    }

    /// Luminance relative, de 0 (noir) à 1 (blanc), au sens des recommandations d'accessibilité WCAG.
    pub fn relative_luminance(self) -> f64 {
        let linear = |channel: u8| {
            let value = f64::from(channel) / 255.0;
            if value <= 0.039_28 { value / 12.92 } else { ((value + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * linear(self.red) + 0.7152 * linear(self.green) + 0.0722 * linear(self.blue)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Color::parse(&text).ok_or_else(|| de::Error::custom(format!("couleur invalide : {text}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats_hexadecimal() {
        let color = Color::parse("#2d7cf6").expect("couleur valide");
        assert_eq!(color, Color::rgb(0x2d7cf6));
        assert_eq!(color.to_string(), "#2d7cf6");
        assert_eq!(Color::parse("2d7cf6"), None);
        assert_eq!(Color::parse("#2d7cf"), None);
    }

    #[test]
    fn luminance_spans_black_to_white() {
        assert_eq!(Color::rgb(0x000000).relative_luminance(), 0.0);
        assert!((Color::rgb(0xffffff).relative_luminance() - 1.0).abs() < 1e-9);
    }
}
