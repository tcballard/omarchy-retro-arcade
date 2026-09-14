//! Original mono PCM cues, played by an owned and reaped process.
use std::{
    io::Write,
    process::{Child, Command, Stdio},
};
#[derive(Clone, Copy)]
pub enum Cue {
    Launch,
    Shell,
    Heavy,
    Digger,
    Round,
    Match,
}
#[derive(Default)]
pub struct Audio {
    child: Option<Child>,
    file: Option<tempfile::NamedTempFile>,
}
impl Audio {
    pub fn stop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        self.file = None;
    }
    pub fn poll(&mut self) -> Result<(), String> {
        let Some(child) = &mut self.child else {
            return Ok(());
        };
        let result = match child.try_wait() {
            Ok(None) => return Ok(()),
            Ok(Some(status)) if status.success() => Ok(()),
            Ok(Some(_)) => Err("Playback failed; check your audio output.".into()),
            Err(e) => Err(e.to_string()),
        };
        self.stop();
        result
    }
    pub fn play(&mut self, cue: Cue) -> Result<(), String> {
        self.stop();
        let mut f = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
        f.write_all(&pcm(cue)).map_err(|e| e.to_string())?;
        self.child = Some(
            Command::new("paplay")
                .arg(f.path())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| e.to_string())?,
        );
        self.file = Some(f);
        Ok(())
    }
}
impl Drop for Audio {
    fn drop(&mut self) {
        self.stop();
    }
}
pub fn pcm(cue: Cue) -> Vec<u8> {
    let rate = 22050u32;
    let (duration, freq, noise) = match cue {
        Cue::Launch => (0.18, 170.0, 0.3),
        Cue::Shell => (0.36, 100.0, 0.65),
        Cue::Heavy => (0.50, 65.0, 0.7),
        Cue::Digger => (0.42, 130.0, 0.85),
        Cue::Round => (0.48, 440.0, 0.0),
        Cue::Match => (0.72, 523.25, 0.0),
    };
    let n = (duration * f64::from(rate)) as u32;
    let mut bytes = Vec::with_capacity(44 + n as usize * 2);
    bytes.extend(b"RIFF");
    bytes.extend((36 + n * 2).to_le_bytes());
    bytes.extend(b"WAVEfmt ");
    bytes.extend(16u32.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(1u16.to_le_bytes());
    bytes.extend(rate.to_le_bytes());
    bytes.extend((rate * 2).to_le_bytes());
    bytes.extend(2u16.to_le_bytes());
    bytes.extend(16u16.to_le_bytes());
    bytes.extend(b"data");
    bytes.extend((n * 2).to_le_bytes());
    let mut seed = 0x54414e4bu32;
    let mut phase = 0.0;
    for k in 0..n {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let t = f64::from(k) / f64::from(n);
        let env = (1.0 - t).powi(2) * (f64::from(k) / 100.0).min(1.0);
        let pitch = if noise == 0.0 {
            [1.0, 1.25, 1.5][(t * 3.0).floor().min(2.0) as usize] * freq
        } else {
            freq * (1.0 - 0.65 * t)
        };
        phase += std::f64::consts::TAU * pitch / f64::from(rate);
        let sample = (phase.sin() * (1.0 - noise)
            + (f64::from(seed) / f64::from(u32::MAX) * 2.0 - 1.0) * noise)
            * env
            * 6000.0;
        bytes.extend((sample as i16).to_le_bytes());
    }
    bytes
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cues_are_bounded_deterministic_and_distinct() {
        let cues = [
            Cue::Launch,
            Cue::Shell,
            Cue::Heavy,
            Cue::Digger,
            Cue::Round,
            Cue::Match,
        ];
        let mut previous = Vec::new();
        for c in cues {
            let b = pcm(c);
            assert_eq!(b, pcm(c));
            assert_eq!(&b[..4], b"RIFF");
            assert_eq!(
                b.len() - 44,
                u32::from_le_bytes(b[40..44].try_into().unwrap()) as usize
            );
            assert!(b.len() < 40000);
            assert_ne!(b, previous);
            previous = b;
        }
    }
    #[test]
    fn stopping_reaps_child_and_removes_pcm() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let p = file.path().to_owned();
        let child = Command::new("sleep").arg("30").spawn().unwrap();
        let mut audio = Audio {
            child: Some(child),
            file: Some(file),
        };
        audio.stop();
        assert!(audio.child.is_none());
        assert!(!p.exists());
    }
}
