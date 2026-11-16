use serde::Serialize;

use super::TrophyTier;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct TrophyLevel {
    pub value: u32,
    /// Avancement vers le niveau suivant, de 0 à 1.
    pub progress: f64,
}

struct Band {
    last_level: u32,
    points_per_level: u64,
}

const BANDS: [Band; 10] = [
    Band { last_level: 99, points_per_level: 60 },
    Band { last_level: 199, points_per_level: 90 },
    Band { last_level: 299, points_per_level: 450 },
    Band { last_level: 399, points_per_level: 900 },
    Band { last_level: 499, points_per_level: 1350 },
    Band { last_level: 599, points_per_level: 1800 },
    Band { last_level: 699, points_per_level: 2250 },
    Band { last_level: 799, points_per_level: 2700 },
    Band { last_level: 899, points_per_level: 3150 },
    Band { last_level: 998, points_per_level: 3600 },
];

impl TrophyLevel {
    pub const MAXIMUM: u32 = 999;

    pub fn from_points(points: u64) -> Self {
        let mut level = 1;
        let mut remaining = points;
        for band in &BANDS {
            while level <= band.last_level {
                if remaining < band.points_per_level {
                    return TrophyLevel { value: level, progress: remaining as f64 / band.points_per_level as f64 };
                }
                remaining -= band.points_per_level;
                level += 1;
            }
        }
        TrophyLevel { value: Self::MAXIMUM, progress: 1.0 }
    }

    pub fn next(&self) -> u32 {
        (self.value + 1).min(Self::MAXIMUM)
    }

    pub fn tier(&self) -> TrophyTier {
        match self.value {
            ..100 => TrophyTier::Bronze,
            100..300 => TrophyTier::Silver,
            300..600 => TrophyTier::Gold,
            _ => TrophyTier::Platinum,
        }
    }
}
