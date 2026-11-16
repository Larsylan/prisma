use std::collections::HashMap;
use std::io::Cursor;
use std::sync::mpsc;
use std::thread;

use prisma_core::chime;
use prisma_core::domain::TrophyTier;
use rodio::{Decoder, OutputStream, Source};

/// Joue le carillon sur un fil dédié : la sortie audio du système ne peut pas changer de fil.
pub struct ChimePlayer {
    requests: mpsc::Sender<TrophyTier>,
}

impl ChimePlayer {
    pub fn start() -> Self {
        let (requests, tiers) = mpsc::channel();
        thread::Builder::new()
            .name("carillon".into())
            .spawn(move || play_forever(tiers))
            .inspect_err(|error| log::error!("Lecteur audio indisponible : {error}"))
            .ok();
        ChimePlayer { requests }
    }

    pub fn play(&self, tier: TrophyTier) {
        // Sans sortie audio, le fil s'est arrêté : la notification reste simplement silencieuse.
        let _ = self.requests.send(tier);
    }
}

fn play_forever(tiers: mpsc::Receiver<TrophyTier>) {
    let (_stream, output) = match OutputStream::try_default() {
        Ok(output) => output,
        Err(error) => return log::warn!("Aucune sortie audio : {error}"),
    };
    let mut wav_by_tier: HashMap<TrophyTier, Vec<u8>> = HashMap::new();
    for tier in tiers {
        let wav = wav_by_tier.entry(tier).or_insert_with(|| chime::wav(tier)).clone();
        match Decoder::new(Cursor::new(wav)) {
            Ok(source) => {
                if let Err(error) = output.play_raw(source.convert_samples()) {
                    log::error!("Lecture du carillon impossible : {error}");
                }
            }
            Err(error) => log::error!("Carillon illisible : {error}"),
        }
    }
}
