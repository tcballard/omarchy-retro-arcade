fn sound_default() -> bool {
    true
}
fn started_default() -> bool {
    true
}
use crate::{
    ai::Difficulty,
    rules::{Game, Phase, RULES_VERSION},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Solo,
    Local,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Save {
    version: u32,
    rules: String,
    pub game: Game,
    pub mode: Mode,
    pub difficulty: Difficulty,
    pub ai_seed: u64,
    pub recorded: bool,
    pub solo_wins: u64,
    pub solo_losses: u64,
    pub local_matches: u64,
    pub reduced_effects: bool,
    #[serde(default = "sound_default")]
    pub sound: bool,
    #[serde(default)]
    pub resolution: Option<crate::effects::Resolution>,
    #[serde(default = "started_default")]
    pub started: bool,
}
impl Default for Save {
    fn default() -> Self {
        Self {
            version: 1,
            rules: RULES_VERSION.into(),
            game: Game::new(42),
            mode: Mode::Solo,
            difficulty: Difficulty::Normal,
            ai_seed: 123,
            recorded: false,
            solo_wins: 0,
            solo_losses: 0,
            local_matches: 0,
            reduced_effects: false,
            sound: true,
            resolution: None,
            started: false,
        }
    }
}
impl Save {
    pub fn observe(&mut self) {
        if let Phase::MatchOver { winner } = self.game.phase() {
            if !self.recorded {
                let count = match self.mode {
                    Mode::Local => &mut self.local_matches,
                    Mode::Solo if winner == 0 => &mut self.solo_wins,
                    Mode::Solo => &mut self.solo_losses,
                };
                *count = count.saturating_add(1);
                self.recorded = true;
            }
        }
    }
    pub fn new_match(&mut self, seed: u64) {
        self.game = Game::new(seed);
        self.ai_seed = seed ^ 0x54414e4b53;
        self.recorded = false;
        self.resolution = None;
        self.started = true;
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.rules == RULES_VERSION
            && self.game.valid()
            && self.resolution.as_ref().is_none_or(|r| r.valid(&self.game))
            && (!self.recorded || matches!(self.game.phase(), Phase::MatchOver { .. }))
    }
}
pub fn path() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/state")
        })
        .join("omarchy-retro-arcade/tanks.json")
}
pub fn load(path: &Path) -> Result<Save, String> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Save::default()),
        Err(e) => return Err(e.to_string()),
    };
    let mut bytes = Vec::new();
    file.take(524289)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 524288 {
        return Err("Save is too large; original retained.".into());
    }
    let save: Save = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Cannot read saved match: {e}. Original retained."))?;
    if !save.valid() {
        return Err("Unsupported or invalid saved match; original retained.".into());
    }
    Ok(save)
}
pub fn write(path: &Path, save: &Save) -> Result<(), String> {
    if !save.valid() {
        return Err("Refusing to write invalid match".into());
    }
    let bytes = serde_json::to_vec(save).map_err(|e| e.to_string())?;
    #[cfg(feature = "desktop")]
    {
        omarchy_chess::storage::atomic_write(path, &bytes)
    }
    #[cfg(not(feature = "desktop"))]
    {
        let dir = path.parent().ok_or("Missing save directory")?;
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let mut file = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
        file.write_all(&bytes)
            .and_then(|_| file.as_file().sync_all())
            .map_err(|e| e.to_string())?;
        file.persist(path).map_err(|e| e.to_string())?;
        fs::File::open(dir)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())
    }
}
/// Explicit recovery copies to a fresh private archive before replacement.
pub fn archive(path: &Path) -> Result<PathBuf, String> {
    let dir = path.parent().ok_or("Missing save directory")?;
    let mut archive = tempfile::Builder::new()
        .prefix("tanks-recovery-")
        .suffix(".json")
        .tempfile_in(dir)
        .map_err(|e| e.to_string())?;
    let mut source = fs::File::open(path).map_err(|e| e.to_string())?;
    std::io::copy(&mut source, &mut archive).map_err(|e| e.to_string())?;
    archive
        .flush()
        .and_then(|_| archive.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    let (_, name) = archive.keep().map_err(|e| e.to_string())?;
    fs::File::open(dir)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())?;
    Ok(name)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mid_flight_roundtrip_preserves_future_and_private_permissions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tanks.json");
        let mut save = Save::default();
        save.game.ready().unwrap();
        save.game.fire().unwrap();
        for _ in 0..37 {
            save.game.tick();
        }
        write(&path, &save).unwrap();
        let mut restored = load(&path).unwrap();
        assert_eq!(save, restored);
        for _ in 0..2400 {
            save.game.tick();
            restored.game.tick();
        }
        assert_eq!(save, restored);
        assert!(save.valid());
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
    fn finished_match_records_are_counted_once_across_reopen() {
        let mut value = serde_json::to_value(Save::default()).unwrap();
        value["game"]["tanks"][1]["health"] = serde_json::json!(0);
        value["game"]["wins"] = serde_json::json!([2, 0]);
        value["game"]["phase"] = serde_json::json!({"MatchOver":{"winner":0}});
        let mut save: Save = serde_json::from_value(value).unwrap();
        assert!(save.valid());
        save.observe();
        assert_eq!(save.solo_wins, 1);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tanks.json");
        write(&path, &save).unwrap();
        let mut restored = load(&path).unwrap();
        restored.observe();
        assert_eq!(restored, save);
        restored.new_match(100);
        assert_eq!(restored.solo_wins, 1);
        assert!(!restored.recorded);
    }
    #[test]
    fn invalid_future_and_oversized_saves_survive_recovery() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tanks.json");
        let future = Save {
            version: 99,
            ..Save::default()
        };
        for bytes in [
            b"broken".to_vec(),
            serde_json::to_vec(&future).unwrap(),
            vec![b' '; 524289],
        ] {
            fs::write(&path, &bytes).unwrap();
            assert!(load(&path).is_err());
            let a = archive(&path).unwrap();
            let b = archive(&path).unwrap();
            assert_ne!(a, b);
            assert_eq!(fs::read(a).unwrap(), bytes);
            assert_eq!(fs::read(&path).unwrap(), bytes);
        }
    }
    #[test]
    fn older_preview_defaults_resume_without_replacing_the_match() {
        let mut value = serde_json::to_value(Save::default()).unwrap();
        let object = value.as_object_mut().unwrap();
        for key in ["started", "sound", "resolution"] {
            object.remove(key);
        }
        let save: Save = serde_json::from_value(value).unwrap();
        assert!(save.valid());
        assert!(save.started);
        assert!(save.sound);
        assert!(save.resolution.is_none());
        assert_eq!(save.game, Save::default().game);
    }
}
