use crate::engine::{Difficulty, Game, Status};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub played: u64,
    pub wins: u64,
    pub best_ms: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Save {
    pub version: u32,
    pub game: Game,
    pub records: [Record; 3],
}
impl Default for Save {
    fn default() -> Self {
        Self {
            version: 1,
            game: Game::new(Difficulty::Beginner),
            records: Default::default(),
        }
    }
}
impl Save {
    pub fn account(&mut self, before: Status) {
        let r = &mut self.records[self.game.difficulty.index()];
        if before == Status::Ready && self.game.status != Status::Ready {
            r.played = r.played.saturating_add(1);
        }
        if before != Status::Won && self.game.status == Status::Won {
            r.wins = r.wins.saturating_add(1);
            r.best_ms = Some(
                r.best_ms
                    .map_or(self.game.elapsed_ms, |b| b.min(self.game.elapsed_ms)),
            );
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.game.valid()
            && self
                .records
                .iter()
                .all(|r| r.wins <= r.played && (r.wins > 0) == r.best_ms.is_some())
            && (self.game.status == Status::Ready
                || self.records[self.game.difficulty.index()].played > 0)
            && (self.game.status != Status::Won
                || self.records[self.game.difficulty.index()].wins > 0)
    }
}
pub fn path() -> Result<PathBuf, String> {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")))
        .map(|p| p.join("omarchy-retro-arcade/minesweeper.json"))
        .ok_or("No save directory".into())
}
pub fn load(path: &Path) -> Result<Option<Save>, String> {
    let f = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let mut bytes = vec![];
    f.take(131073)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 131072 {
        return Err("Save exceeds 128 KiB; original retained".into());
    }
    let state: Save = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Invalid save; original retained: {e}"))?;
    if !state.valid() {
        return Err("Unsupported or invalid save; original retained".into());
    }
    Ok(Some(state))
}
pub fn save(path: &Path, state: &Save) -> Result<(), String> {
    if !state.valid() {
        return Err("Refusing invalid Minesweeper save".into());
    }
    let dir = path.parent().ok_or("Missing save parent")?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let mut f = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    f.write_all(&serde_json::to_vec(state).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    f.as_file().sync_all().map_err(|e| e.to_string())?;
    f.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};
    #[test]
    fn exact_roundtrip_and_statistics_once() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("save.json");
        let mut s = Save::default();
        assert!(load(&p).unwrap().is_none());
        s.game.reveal(0, &mut StdRng::seed_from_u64(1));
        s.account(Status::Ready);
        s.game.elapsed_ms = 14567;
        save(&p, &s).unwrap();
        assert_eq!(load(&p).unwrap(), Some(s.clone()));
        for i in 0..s.game.cells.len() {
            if !s.game.cells[i].mine {
                let before = s.game.status;
                s.game.reveal(i, &mut StdRng::seed_from_u64(1));
                s.account(before);
            }
        }
        assert_eq!(s.records[0].wins, 1);
        assert_eq!(s.records[0].played, 1);
        assert_eq!(s.records[0].best_ms, Some(14567));
        s.account(Status::Won);
        assert_eq!(s.records[0].wins, 1);
        save(&p, &s).unwrap();
        assert_eq!(load(&p).unwrap(), Some(s));
    }
    #[test]
    fn corrupt_future_and_oversized_saves_retained() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("save.json");
        let future = Save {
            version: 999,
            ..Default::default()
        };
        for bytes in [
            b"broken".to_vec(),
            serde_json::to_vec(&future).unwrap(),
            vec![b' '; 131073],
        ] {
            fs::write(&p, &bytes).unwrap();
            assert!(load(&p).is_err());
            assert_eq!(fs::read(&p).unwrap(), bytes);
        }
        assert!(save(&p, &future).is_err());
    }
}
