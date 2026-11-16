use chrono::DateTime;

use crate::domain::{Timestamp, Trophy};

/// Succès Steam tel que fourni par le cache local ou par la Steam Web API.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SteamAchievement {
    pub api_name: String,
    pub name: String,
    pub detail: String,
    pub is_hidden: bool,
    pub icon_url: Option<String>,
    pub locked_icon_url: Option<String>,
    pub is_earned: bool,
    pub earned_at: Option<Timestamp>,
}

impl SteamAchievement {
    /// Un succès non obtenu n'a jamais de date, même si la source en fournit une.
    pub fn unlocked_at(mut self, is_earned: bool, earned_at: Option<Timestamp>) -> Self {
        self.is_earned = is_earned;
        self.earned_at = earned_at.filter(|_| is_earned);
        self
    }

    pub(crate) fn unlock_date(timestamp: Option<i64>) -> Option<Timestamp> {
        timestamp.filter(|seconds| *seconds > 0).and_then(|seconds| DateTime::from_timestamp(seconds, 0))
    }
}

impl From<&Trophy> for SteamAchievement {
    fn from(trophy: &Trophy) -> Self {
        SteamAchievement {
            api_name: trophy.id.clone(),
            name: trophy.name.clone(),
            detail: trophy.detail.clone(),
            is_hidden: trophy.is_hidden,
            icon_url: trophy.icon_url.clone(),
            locked_icon_url: trophy.locked_icon_url.clone(),
            ..SteamAchievement::default()
        }
        .unlocked_at(trophy.is_earned, trophy.earned_at)
    }
}
