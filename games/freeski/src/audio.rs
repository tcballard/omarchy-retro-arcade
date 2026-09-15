//! Original PCM synthesis. One owned playback process, no queues or game clock.
use std::{
    io::Write,
    process::{Child, Command, Stdio},
};
#[derive(Clone, Copy)]
pub enum Cue {
    Carve,
    Jump,
    Crash,
    Gate,
    Miss,
    Warning,
    Caught,
    Finish,
}
#[derive(Default)]
pub struct Audio {
    child: Option<Child>,
    file: Option<tempfile::NamedTempFile>,
}
impl Audio {
    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.file = None;
    }
    pub fn play(&mut self, kind: Cue) {
        if cfg!(test) {
            return;
        }
        if matches!(kind, Cue::Carve)
            && self
                .child
                .as_mut()
                .is_some_and(|c| matches!(c.try_wait(), Ok(None)))
        {
            return;
        }
        self.stop();
        let Ok(mut file) = tempfile::NamedTempFile::new() else {
            return;
        };
        if file.write_all(&cue(kind)).is_err() {
            return;
        }
        self.child = Command::new("paplay")
            .arg("--client-name=Omarchy FreeSki")
            .arg(file.path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok();
        self.file = Some(file);
    }
}
impl Drop for Audio {
    fn drop(&mut self) {
        self.stop();
    }
}
fn cue(kind: Cue) -> Vec<u8> {
    let rate = 22050u32;
    let seconds = match kind {
        Cue::Carve => 0.16,
        Cue::Jump => 0.22,
        Cue::Crash => 0.28,
        Cue::Gate => 0.13,
        Cue::Miss => 0.18,
        Cue::Warning => 0.6,
        Cue::Caught => 0.55,
        Cue::Finish => 0.65,
    };
    let n = (seconds * f64::from(rate)) as u32;
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
    let mut noise = 0x9e3779b9u32;
    let mut phase = 0.;
    for i in 0..n {
        let t = f64::from(i) / f64::from(rate);
        let u = f64::from(i) / f64::from(n);
        noise ^= noise << 13;
        noise ^= noise >> 17;
        noise ^= noise << 5;
        let hiss = f64::from(noise) / f64::from(u32::MAX) * 2. - 1.;
        let frequency = match kind {
            Cue::Carve => 110.,
            Cue::Jump => 320. + 1200. * t,
            Cue::Crash => 100. - 45. * u,
            Cue::Gate => 880.,
            Cue::Miss => 210. - 80. * u,
            Cue::Warning => {
                if ((t * 6.) as u32).is_multiple_of(2) {
                    180.
                } else {
                    240.
                }
            }
            Cue::Caught => 220. - 140. * u,
            Cue::Finish => [523.25, 659.25, 783.99, 1046.5][((u * 4.) as usize).min(3)],
        };
        phase += std::f64::consts::TAU * frequency / f64::from(rate);
        let signal = match kind {
            Cue::Carve => hiss * 0.18,
            Cue::Crash => hiss * 0.65 + phase.sin() * 0.35,
            _ => phase.sin() * 0.7 + (phase * 2.).sin() * 0.15,
        };
        let envelope = (t / 0.012).min(1.) * (1. - u).powi(2);
        out.extend(((signal * envelope * 6000.) as i16).to_le_bytes());
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cues_are_bounded_original_pcm() {
        for kind in [
            Cue::Carve,
            Cue::Jump,
            Cue::Crash,
            Cue::Gate,
            Cue::Miss,
            Cue::Warning,
            Cue::Caught,
            Cue::Finish,
        ] {
            let b = cue(kind);
            assert_eq!(&b[..4], b"RIFF");
            assert_eq!(&b[8..12], b"WAVE");
            assert_eq!(
                u32::from_le_bytes(b[40..44].try_into().unwrap()) as usize,
                b.len() - 44
            );
            assert!(b.len() < 30000);
            assert!(b[44..].iter().any(|b| *b != 0));
            assert!(b[44..]
                .as_chunks::<2>()
                .0
                .iter()
                .all(|v| i16::from_le_bytes(*v).unsigned_abs() < 7000));
        }
    }
    #[test]
    fn stop_reaps_only_owned_process_and_file() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let path = file.path().to_owned();
        let child = Command::new("sleep").arg("30").spawn().unwrap();
        let mut a = Audio {
            child: Some(child),
            file: Some(file),
        };
        a.stop();
        assert!(a.child.is_none());
        assert!(!path.exists());
    }
}
