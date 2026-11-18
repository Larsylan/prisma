mod chime_player;
mod toast;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use prisma_core::domain::{Trophy, TrophyTier};
use serde::Serialize;
use tauri::AppHandle;
use tokio::sync::mpsc;

use crate::events::Announcer;
use crate::settings::SettingsStore;
pub use toast::ToastWindow;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Announcement {
    pub trophy_name: String,
    pub tier: TrophyTier,
    pub icon_url: Option<String>,
    pub game_title: String,
}

/// File d'attente des cartes « Vous avez obtenu un trophée », présentées une à une.
#[derive(Clone)]
pub struct Notifier {
    queue: mpsc::UnboundedSender<Announcement>,
    test_count: Arc<AtomicUsize>,
}

impl Notifier {
    pub fn start(app: AppHandle, settings: Arc<SettingsStore>) -> Self {
        let (queue, mut announcements) = mpsc::unbounded_channel::<Announcement>();
        let chime = chime_player::ChimePlayer::start();
        tauri::async_runtime::spawn(async move {
            let toast = ToastWindow::new(app);
            while let Some(announcement) = announcements.recv().await {
                let settings = settings.get();
                if settings.sound_enabled {
                    chime.play(announcement.tier);
                }
                toast.present(&announcement, Duration::from_secs(settings.toast_duration_seconds), settings.toast_position).await;
            }
        });
        Notifier { queue, test_count: Arc::default() }
    }

    /// Sans grade précisé, les notifications de test passent en revue chaque grade.
    pub fn announce_test(&self, tier: Option<TrophyTier>) {
        let tier = tier.unwrap_or_else(|| TrophyTier::ALL[self.test_count.fetch_add(1, Ordering::Relaxed) % TrophyTier::ALL.len()]);
        let trophy_name = if tier == TrophyTier::Platinum { "Légende vivante" } else { "Premier pas" };
        self.enqueue(Announcement { trophy_name: trophy_name.into(), tier, icon_url: None, game_title: "Prisma".into() });
    }

    fn enqueue(&self, announcement: Announcement) {
        if self.queue.send(announcement).is_err() {
            log::error!("File des notifications fermée");
        }
    }
}

impl Announcer for Notifier {
    fn announce(&self, mut trophies: Vec<Trophy>, game_title: &str) {
        trophies.sort_by(presentation_order);
        for trophy in trophies {
            log::info!("Trophée obtenu « {} » ({}) — {game_title}", trophy.name, trophy.tier.name());
            self.enqueue(Announcement {
                trophy_name: trophy.name,
                tier: trophy.tier,
                icon_url: trophy.icon_url,
                game_title: game_title.to_owned(),
            });
        }
    }
}

/// Le platine arrive toujours en dernier, comme sur console.
fn presentation_order(lhs: &Trophy, rhs: &Trophy) -> std::cmp::Ordering {
    let is_platinum = |trophy: &Trophy| trophy.tier == TrophyTier::Platinum;
    is_platinum(lhs).cmp(&is_platinum(rhs)).then_with(|| lhs.earned_at.cmp(&rhs.earned_at))
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::*;

    #[test]
    fn platinum_comes_last_and_others_by_unlock_date() {
        let now = Utc::now();
        let mut trophies = [
            Trophy::new("platine", "", "", TrophyTier::Platinum).earned(Some(now - Duration::minutes(5))),
            Trophy::new("recent", "", "", TrophyTier::Bronze).earned(Some(now)),
            Trophy::new("ancien", "", "", TrophyTier::Gold).earned(Some(now - Duration::minutes(1))),
        ];

        trophies.sort_by(presentation_order);

        let ids: Vec<_> = trophies.iter().map(|trophy| trophy.id.as_str()).collect();
        assert_eq!(ids, ["ancien", "recent", "platine"]);
    }
}
