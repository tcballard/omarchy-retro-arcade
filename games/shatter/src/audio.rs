//! Original synthesised cues; owned child process is reaped on pause and drop.
use std::{
    io::Write,
    process::{Child, Command, Stdio},
};
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
    pub fn play(&mut self, kind: u8) {
        self.stop();
        let Ok(mut file) = tempfile::NamedTempFile::new() else {
            return;
        };
        if file.write_all(&cue(kind)).is_err() {
            return;
        }
        self.child = Command::new("paplay")
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
pub fn cue(kind: u8) -> Vec<u8> {
    let rate = 22050u32;
    let n = if kind == 2 { 6600u32 } else { 2200 };
    let mut out = vec![];
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
        let t = i as f64 / rate as f64;
        let f = match kind {
            0 => 190. + t * 500.,
            1 => 420. + t * 1800.,
            3 => 880. + t * 1600.,
            4 => 170. - t * 400.,
            5 => 300. + t * 3000.,
            _ => [523.25, 659.25, 783.99][(i * 3 / n) as usize],
        };
        let envelope = (1. - i as f64 / n as f64).powi(2) * (i as f64 / 90.).min(1.);
        out.extend(
            (((t * f * std::f64::consts::TAU).sin() * envelope * 3500.) as i16).to_le_bytes(),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_cues_are_valid_mono_pcm() {
        for kind in 0..3 {
            let bytes = cue(kind);
            assert_eq!(&bytes[..4], b"RIFF");
            assert_eq!(&bytes[8..12], b"WAVE");
            assert_eq!(
                u32::from_le_bytes(bytes[40..44].try_into().unwrap()) as usize,
                bytes.len() - 44
            );
            assert_eq!(u16::from_le_bytes(bytes[22..24].try_into().unwrap()), 1);
            assert!(bytes[44..].iter().any(|b| *b != 0));
        }
    }
    #[test]
    fn stop_reaps_owned_process_and_removes_temporary_audio() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let path = file.path().to_owned();
        let child = Command::new("sleep").arg("30").spawn().unwrap();
        let mut audio = Audio {
            child: Some(child),
            file: Some(file),
        };
        audio.stop();
        assert!(audio.child.is_none());
        assert!(!path.exists());
    }
}
