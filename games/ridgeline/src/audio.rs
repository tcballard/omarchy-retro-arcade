//! Original synthesized cues through the shared session-owned player.
//! Stopping drops the owner, which cancels, reaps and joins its worker.
use arcade_platform::audio::Playback;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    Build,
    Upgrade,
    Sell,
    Wave,
    Leak,
    Clear,
    Victory,
    Defeat,
}

#[derive(Default)]
pub struct Audio {
    playback: Option<Playback>,
}
impl Audio {
    pub fn play(&mut self, cue: Cue) {
        self.playback
            .get_or_insert_with(Playback::default)
            .play(move || wave(cue));
    }
    pub fn stop(&mut self) {
        self.playback = None;
    }
}

/// 16-bit mono PCM WAV. Pitches and envelopes are original to Ridgeline.
pub fn wave(cue: Cue) -> Vec<u8> {
    let rate = 22050u32;
    let (notes, length): (&[f64], f64) = match cue {
        Cue::Build => (&[330., 440.], 0.12),
        Cue::Upgrade => (&[440., 554.37, 659.25], 0.16),
        Cue::Sell => (&[392., 261.63], 0.14),
        Cue::Wave => (&[196., 196., 293.66], 0.35),
        Cue::Leak => (&[146.83, 110.], 0.25),
        Cue::Clear => (&[523.25, 659.25, 783.99], 0.3),
        Cue::Victory => (&[392., 523.25, 659.25, 783.99, 1046.5], 0.7),
        Cue::Defeat => (&[293.66, 261.63, 220., 164.81], 0.7),
    };
    let n = (f64::from(rate) * length) as u32;
    let mut out = Vec::with_capacity(44 + n as usize * 2);
    out.extend(b"RIFF");
    out.extend((36 + n * 2).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(rate.to_le_bytes());
    out.extend((rate * 2).to_le_bytes());
    out.extend(2u16.to_le_bytes());
    out.extend(16u16.to_le_bytes());
    out.extend(b"data");
    out.extend((n * 2).to_le_bytes());
    for i in 0..n {
        let t = f64::from(i) / f64::from(rate);
        let step = (i as usize * notes.len() / n as usize).min(notes.len() - 1);
        let f = notes[step];
        let local = (f64::from(i) * notes.len() as f64 / f64::from(n)).fract();
        let envelope = (1. - local).powf(1.5) * (f64::from(i) / 60.).min(1.);
        // A square-ish blend reads as a machined, retro tone without harshness.
        let s = (t * f * std::f64::consts::TAU).sin();
        let sample = (s * 0.7 + s.signum() * 0.12) * envelope * 3800.;
        out.extend((sample as i16).to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_cue_is_valid_mono_pcm() {
        for cue in [
            Cue::Build,
            Cue::Upgrade,
            Cue::Sell,
            Cue::Wave,
            Cue::Leak,
            Cue::Clear,
            Cue::Victory,
            Cue::Defeat,
        ] {
            let bytes = wave(cue);
            assert_eq!(&bytes[..4], b"RIFF");
            assert_eq!(&bytes[8..12], b"WAVE");
            assert_eq!(
                u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize,
                bytes.len() - 44
            );
            assert!(bytes[44..].iter().any(|b| *b != 0));
        }
    }
    #[test]
    fn stopping_drops_the_owned_player() {
        let mut audio = Audio::default();
        audio.stop();
        assert!(audio.playback.is_none());
    }
}
