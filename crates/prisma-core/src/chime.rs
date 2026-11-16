//! Carillon cristallin synthétisé : cloches en modulation de fréquence suivies d'un scintillement aigu.

use std::f64::consts::TAU;

use crate::domain::TrophyTier;

pub const SAMPLE_RATE: u32 = 44_100;
const TAIL_SECONDS: f64 = 0.1;

struct Note {
    frequency: f64,
    start: f64,
    duration: f64,
    gain: f64,
}

impl Note {
    fn end(&self) -> f64 {
        self.start + self.duration
    }
}

/// Carillon d'un grade, prêt à être joué, au format WAV.
pub fn wav(tier: TrophyTier) -> Vec<u8> {
    encode_wav(&samples(tier))
}

fn samples(tier: TrophyTier) -> Vec<f64> {
    let notes = if tier == TrophyTier::Platinum { platinum_fanfare() } else { unlock_triad() };
    let duration = notes.iter().map(Note::end).fold(0.0, f64::max) + TAIL_SECONDS;
    let mut samples = vec![0.0; (f64::from(SAMPLE_RATE) * duration) as usize];
    for note in &notes {
        add_bell(note, &mut samples);
    }
    let sparkle_start = notes.last().map_or(0.0, |note| note.start);
    add_sparkle(&mut samples, sparkle_start, if tier == TrophyTier::Platinum { 14 } else { 7 });
    samples
}

/// La, mi, la en montant.
fn unlock_triad() -> Vec<Note> {
    vec![
        Note { frequency: 880.0, start: 0.0, duration: 1.0, gain: 0.6 },
        Note { frequency: 1318.5, start: 0.07, duration: 1.1, gain: 0.7 },
        Note { frequency: 1760.0, start: 0.14, duration: 1.5, gain: 1.0 },
    ]
}

/// Arpège ascendant en ré majeur, puis accord tenu.
fn platinum_fanfare() -> Vec<Note> {
    let arpeggio = [587.3, 740.0, 880.0, 1174.7, 1480.0, 1760.0].into_iter().enumerate().map(|(index, frequency)| Note {
        frequency,
        start: index as f64 * 0.075,
        duration: 0.9,
        gain: 0.55,
    });
    let chord = [1174.7, 1760.0, 2349.3].into_iter().map(|frequency| Note { frequency, start: 0.5, duration: 2.6, gain: 0.7 });
    arpeggio.chain(chord).collect()
}

fn add_bell(note: &Note, samples: &mut [f64]) {
    let rate = f64::from(SAMPLE_RATE);
    let first = (note.start * rate) as usize;
    for offset in 0..(note.duration * rate) as usize {
        let Some(sample) = samples.get_mut(first + offset) else { break };
        let time = offset as f64 / rate;
        let modulation_index = 1.8 * (-time * 7.0).exp();
        let modulator = (TAU * note.frequency * 3.5 * time).sin() * modulation_index;
        let overtone = 0.25 * (TAU * note.frequency * 2.01 * time).sin() * (-time * 5.0).exp();
        let tone = (TAU * note.frequency * time + modulator).sin() + overtone;
        let envelope = (time / 0.003).min(1.0) * (-time * 3.2 / note.duration).exp();
        *sample += tone * envelope * note.gain;
    }
}

fn add_sparkle(samples: &mut [f64], start: f64, count: usize) {
    let rate = f64::from(SAMPLE_RATE);
    let mut random = DeterministicRandom(0x5EED);
    for index in 0..count {
        let frequency = 3200.0 + random.next() * 2600.0;
        let first = ((start + 0.03 + index as f64 * 0.045 + random.next() * 0.02) * rate) as usize;
        let gain = 0.12 * (1.0 - index as f64 / (count + 2) as f64);
        for offset in 0..(0.25 * rate) as usize {
            let Some(sample) = samples.get_mut(first + offset) else { break };
            let time = offset as f64 / rate;
            *sample += (TAU * frequency * time).sin() * (-time * 22.0).exp() * gain;
        }
    }
}

/// Générateur congruentiel linéaire : le scintillement sonne identique à chaque trophée.
struct DeterministicRandom(u64);

impl DeterministicRandom {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) as f64 / (1u64 << 31) as f64
    }
}

/// WAV PCM 16 bits mono, normalisé pour laisser de la marge.
fn encode_wav(samples: &[f64]) -> Vec<u8> {
    const HEADROOM: f64 = 0.55;
    const BYTES_PER_SAMPLE: u16 = 2;
    const HEADER_SIZE_AFTER_RIFF_SIZE: u32 = 36;
    const FORMAT_CHUNK_SIZE: u32 = 16;
    const PCM_FORMAT: u16 = 1;
    const CHANNEL_COUNT: u16 = 1;

    let peak = samples.iter().fold(0.0_f64, |peak, sample| peak.max(sample.abs()));
    let gain = if peak > 0.0 { HEADROOM / peak } else { 0.0 };
    let pcm: Vec<u8> =
        samples.iter().flat_map(|sample| (((sample * gain).clamp(-1.0, 1.0) * f64::from(i16::MAX)) as i16).to_le_bytes()).collect();
    let pcm_size = pcm.len() as u32;

    let mut wav = Vec::with_capacity(pcm.len() + 44);
    wav.extend(b"RIFF");
    wav.extend((HEADER_SIZE_AFTER_RIFF_SIZE + pcm_size).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(FORMAT_CHUNK_SIZE.to_le_bytes());
    wav.extend(PCM_FORMAT.to_le_bytes());
    wav.extend(CHANNEL_COUNT.to_le_bytes());
    wav.extend(SAMPLE_RATE.to_le_bytes());
    wav.extend((SAMPLE_RATE * u32::from(BYTES_PER_SAMPLE)).to_le_bytes());
    wav.extend(BYTES_PER_SAMPLE.to_le_bytes());
    wav.extend((8 * BYTES_PER_SAMPLE).to_le_bytes());
    wav.extend(b"data");
    wav.extend(pcm_size.to_le_bytes());
    wav.extend(pcm);
    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_valid_wav_of_expected_length() {
        let wav = wav(TrophyTier::Gold);
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        let expected_samples = (f64::from(SAMPLE_RATE) * (0.14 + 1.5 + TAIL_SECONDS)) as usize;
        assert_eq!(wav.len(), 44 + expected_samples * 2);
    }

    #[test]
    fn platinum_chime_lasts_longer() {
        assert!(wav(TrophyTier::Platinum).len() > wav(TrophyTier::Bronze).len());
    }
}
