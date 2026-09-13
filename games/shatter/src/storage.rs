use crate::rules::{Phase, Run};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub score: u64,
    pub level: usize,
    pub complete: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Save {
    pub version: u32,
    pub campaign: Run,
    pub unlocked: usize,
    pub records: Vec<Record>,
    pub practice_best: [u64; 20],
    pub sound: bool,
    pub reduced_effects: bool,
}
impl Default for Save {
    fn default() -> Self {
        Self {
            version: 1,
            campaign: Run::new(0, false, 0x53484154544552),
            unlocked: 0,
            records: vec![],
            practice_best: [0; 20],
            sound: true,
            reduced_effects: false,
        }
    }
}
impl Save {
    pub fn observe(&mut self) {
        let run = &mut self.campaign;
        if matches!(run.phase, Phase::Clear | Phase::Complete) {
            self.unlocked = self.unlocked.max((run.level + 1).min(19));
        }
        if matches!(run.phase, Phase::Over | Phase::Complete) && !run.recorded {
            self.records.push(Record {
                score: run.score,
                level: run.level,
                complete: run.phase == Phase::Complete,
            });
            self.records.sort_by_key(|a| std::cmp::Reverse(a.score));
            self.records.truncate(20);
            run.recorded = true;
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.unlocked < 20
            && self.records.len() <= 20
            && self
                .records
                .iter()
                .all(|r| r.level < 20 && (!r.complete || r.level == 19))
            && !self.campaign.practice
            && self.campaign.valid()
            && self.campaign.level <= self.unlocked
    }
}
pub fn path() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/state")
        })
        .join("omarchy-retro-arcade/shatter.json")
}
pub fn load(path: &Path) -> Result<Save, String> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Save::default()),
        Err(e) => return Err(e.to_string()),
    };
    let mut bytes = vec![];
    file.take(262145)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 262144 {
        return Err("Save exceeds the supported size. Original retained.".into());
    }
    let save: Save = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Cannot read campaign: {e}. Original retained."))?;
    if !save.valid() {
        return Err("Incompatible rules/layout or invalid campaign. Original retained.".into());
    }
    Ok(save)
}
pub fn write(path: &Path, save: &Save) -> Result<(), String> {
    if !save.valid() {
        return Err("Refusing to write an invalid campaign".into());
    }
    atomic_write(
        path,
        &serde_json::to_vec_pretty(save).map_err(|e| e.to_string())?,
    )
}
#[cfg(feature = "desktop")]
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    omarchy_chess::storage::atomic_write(path, bytes)
}
#[cfg(not(feature = "desktop"))]
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    // Keep engine-only persistence equivalent without pulling in the desktop stack.
    let dir = path.parent().ok_or("Missing state directory")?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    file.write_all(bytes)
        .and_then(|_| file.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    fs::File::open(dir)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
}
/// Explicit recovery: retain the exact original under a new, never-overwritten name.
pub fn archive(path: &Path) -> Result<PathBuf, String> {
    let dir = path.parent().ok_or("Missing state directory")?;
    let mut temp = tempfile::Builder::new()
        .prefix("shatter-recovery-")
        .suffix(".json")
        .tempfile_in(dir)
        .map_err(|e| e.to_string())?;
    let mut source = fs::File::open(path).map_err(|e| e.to_string())?;
    std::io::copy(&mut source, &mut temp).map_err(|e| e.to_string())?;
    temp.as_file().sync_all().map_err(|e| e.to_string())?;
    let (_, backup) = temp.keep().map_err(|e| e.to_string())?;
    fs::File::open(dir)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(backup)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_roundtrip_and_private_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("save.json");
        let mut s = Save::default();
        s.campaign.step(crate::rules::Input {
            launch: true,
            ..Default::default()
        });
        s.campaign.pickup(crate::rules::Power::Multiball);
        s.campaign.pickup(crate::rules::Power::Expand);
        for _ in 0..123 {
            s.campaign.step(Default::default());
        }
        write(&path, &s).unwrap();
        assert_eq!(s, load(&path).unwrap());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
    #[test]
    fn invalid_files_are_unchanged_and_archives_unique() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("s.json");
        for bytes in [
            b"broken".to_vec(),
            serde_json::to_vec(&Save {
                version: 99,
                ..Save::default()
            })
            .unwrap(),
        ] {
            fs::write(&p, &bytes).unwrap();
            assert!(load(&p).is_err());
            assert_eq!(fs::read(&p).unwrap(), bytes);
            let a = archive(&p).unwrap();
            let b = archive(&p).unwrap();
            assert_ne!(a, b);
            assert_eq!(fs::read(a).unwrap(), bytes);
        }
    }
    #[test]
    fn record_once_and_practice_is_independent() {
        let mut s = Save::default();
        s.campaign.phase = Phase::Over;
        s.campaign.lives = 0;
        s.observe();
        s.observe();
        assert_eq!(s.records.len(), 1);
        let before = s.clone();
        let mut practice = Run::new(0, true, 3);
        practice.score = 99;
        assert_eq!(s, before);
    }
}
