use crate::engine::Game;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Save {
    pub version: u32,
    pub game: Game,
    pub reduced_motion: bool,
}
impl Save {
    pub fn new(game: Game) -> Self {
        Self {
            version: 1,
            game,
            reduced_motion: false,
        }
    }
    fn valid(&self) -> bool {
        self.version == 1 && self.game.valid()
    }
}
pub fn path() -> Result<PathBuf, String> {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")))
        .map(|p| p.join("omarchy-retro-arcade/2048.json"))
        .ok_or("No save directory".into())
}
pub fn load(path: &Path) -> Result<Option<Save>, String> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let mut bytes = Vec::new();
    file.take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65536 {
        return Err("2048 save exceeds 64 KiB; original retained".into());
    }
    let save: Save = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Invalid 2048 save; original retained: {e}"))?;
    if !save.valid() {
        return Err("Unsupported or invalid 2048 save; original retained".into());
    }
    Ok(Some(save))
}
pub fn save(path: &Path, state: &Save) -> Result<(), String> {
    if !state.valid() {
        return Err("Refusing to save invalid 2048 state".into());
    }
    let parent = path.parent().ok_or("Missing parent directory")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut f = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    f.write_all(&serde_json::to_vec(state).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    f.as_file().sync_all().map_err(|e| e.to_string())?;
    f.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_board_history_best_and_preferences() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("2048.json");
        assert_eq!(load(&p).unwrap(), None);
        let mut s = Save::new(Game::new(&mut rand::thread_rng(), 123));
        s.game
            .play(crate::engine::Direction::Left, &mut rand::thread_rng());
        s.reduced_motion = true;
        save(&p, &s).unwrap();
        assert_eq!(load(&p).unwrap(), Some(s));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&p).unwrap().permissions().mode() & 0o077, 0);
        }
    }
    #[test]
    fn bad_and_future_files_are_not_changed() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("2048.json");
        let mut future = Save::new(Game::new(&mut rand::thread_rng(), 0));
        future.version = 2;
        for bytes in [
            b"broken".to_vec(),
            serde_json::to_vec(&future).unwrap(),
            vec![b' '; 65537],
        ] {
            fs::write(&p, &bytes).unwrap();
            assert!(load(&p).is_err());
            assert_eq!(fs::read(&p).unwrap(), bytes);
        }
    }
}
