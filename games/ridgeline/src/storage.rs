//! Versioned campaign progress, preferences and the resumable battle.
use crate::battle::{Battle, Phase, BASE_HEALTH, RULES};
use crate::data::{maps, Difficulty, LAYOUT};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

pub const VERSION: u32 = 1;
const LIMIT: usize = 1 << 20;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Medal {
    #[default]
    None,
    /// Victory.
    Clear,
    /// Victory with full base health remaining.
    Perfect,
}
impl Medal {
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "—",
            Self::Clear => "Clear",
            Self::Perfect => "Perfect",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub medal: Medal,
    /// Most base health remaining in a victory.
    pub best_health: u32,
    pub wins: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progress {
    /// Indexed by map, then difficulty.
    pub records: Vec<[Record; 2]>,
}
impl Default for Progress {
    fn default() -> Self {
        Self {
            records: vec![[Record::default(); 2]; maps().len()],
        }
    }
}
impl Progress {
    pub fn record(&self, map: usize, d: Difficulty) -> Record {
        self.records[map][d.index()]
    }
    /// Normal maps unlock in order; Hard unlocks per map after its Normal win.
    pub fn unlocked(&self, map: usize, d: Difficulty) -> bool {
        match d {
            Difficulty::Normal => {
                map == 0 || self.record(map - 1, Difficulty::Normal).medal != Medal::None
            }
            Difficulty::Hard => self.record(map, Difficulty::Normal).medal != Medal::None,
        }
    }
    fn valid(&self) -> bool {
        self.records.len() == maps().len()
            && self.records.iter().flatten().all(|r| {
                r.best_health <= BASE_HEALTH
                    && (r.medal == Medal::None) == (r.wins == 0)
                    && (r.medal == Medal::None) == (r.best_health == 0)
                    && (r.medal == Medal::Perfect) == (r.best_health == BASE_HEALTH)
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preferences {
    pub sound: bool,
    pub reduced_effects: bool,
    /// 1× or 2× simulation speed.
    pub speed: u32,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            sound: true,
            reduced_effects: false,
            speed: 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Save {
    pub version: u32,
    pub rules: u32,
    pub layout: u32,
    pub progress: Progress,
    pub preferences: Preferences,
    pub battle: Option<Battle>,
}
impl Default for Save {
    fn default() -> Self {
        Self {
            version: VERSION,
            rules: RULES,
            layout: LAYOUT,
            progress: Progress::default(),
            preferences: Preferences::default(),
            battle: None,
        }
    }
}

/// A result awarded to the campaign by `observe`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Award {
    pub medal: Medal,
    pub unlocked_next: bool,
    pub unlocked_hard: bool,
}

impl Save {
    /// Award medals, records and unlocks for a finished battle exactly once.
    pub fn observe(&mut self) -> Option<Award> {
        let battle = self.battle.as_mut()?;
        if !battle.finished() || battle.recorded {
            return None;
        }
        battle.recorded = true;
        if battle.phase != Phase::Victory {
            return None;
        }
        let (map, d, health) = (battle.map, battle.difficulty, battle.health);
        let next_before =
            map + 1 < maps().len() && self.progress.unlocked(map + 1, Difficulty::Normal);
        let hard_before = self.progress.unlocked(map, Difficulty::Hard);
        let medal = if health == BASE_HEALTH {
            Medal::Perfect
        } else {
            Medal::Clear
        };
        let r = &mut self.progress.records[map][d.index()];
        r.medal = r.medal.max(medal);
        r.best_health = r.best_health.max(health);
        r.wins += 1;
        Some(Award {
            medal,
            unlocked_next: !next_before
                && map + 1 < maps().len()
                && self.progress.unlocked(map + 1, Difficulty::Normal),
            unlocked_hard: !hard_before && self.progress.unlocked(map, Difficulty::Hard),
        })
    }
    pub fn valid(&self) -> bool {
        self.version == VERSION
            && self.rules == RULES
            && self.layout == LAYOUT
            && (1..=2).contains(&self.preferences.speed)
            && self.progress.valid()
            && self
                .battle
                .as_ref()
                .is_none_or(|b| b.valid() && self.progress.unlocked(b.map, b.difficulty))
    }
}

pub fn path() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/state")
        })
        .join("omarchy-retro-arcade/ridgeline.json")
}

/// Missing files start fresh. Anything unreadable, invalid or from another
/// version is rejected and left exactly as found.
pub fn load(path: &Path) -> Result<Save, String> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Save::default()),
        Err(e) => {
            return Err(format!(
                "Cannot open the saved campaign: {e}. Original retained."
            ))
        }
    };
    let mut bytes = vec![];
    file.take(LIMIT as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Cannot read the saved campaign: {e}. Original retained."))?;
    if bytes.len() > LIMIT {
        return Err("The saved campaign is too large. Original retained.".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("The saved campaign is damaged ({e}). Original retained."))?;
    let version = value.get("version").and_then(|v| v.as_u64());
    if version != Some(u64::from(VERSION)) {
        return Err(format!(
            "The saved campaign uses format {} but this build reads format {VERSION}. Original retained.",
            version.map_or("unknown".into(), |v| v.to_string())
        ));
    }
    let save: Save = serde_json::from_value(value)
        .map_err(|e| format!("The saved campaign is damaged ({e}). Original retained."))?;
    if !save.valid() {
        return Err(
            "The saved campaign does not match these rules or maps. Original retained.".into(),
        );
    }
    Ok(save)
}

pub fn write(path: &Path, save: &Save) -> Result<(), String> {
    if !save.valid() {
        return Err("Refusing to write an invalid campaign".into());
    }
    atomic_write(path, &serde_json::to_vec(save).map_err(|e| e.to_string())?)
}

#[cfg(feature = "desktop")]
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    arcade_platform::storage::atomic_write(path, bytes)
}
#[cfg(not(feature = "desktop"))]
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    // Engine-only builds keep equivalent private atomic replacement.
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

/// Explicit recovery: copy the exact original to a new, never-overwritten name.
pub fn archive(path: &Path) -> Result<PathBuf, String> {
    let dir = path.parent().ok_or("Missing state directory")?;
    let mut temp = tempfile::Builder::new()
        .prefix("ridgeline-recovery-")
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
    use crate::battle::Command;
    use crate::data::TowerKind;

    fn mid_wave() -> Battle {
        let mut b = Battle::new(0, Difficulty::Normal);
        b.apply(Command::Build {
            x: 5,
            y: 1,
            kind: TowerKind::Cannon,
        })
        .unwrap();
        b.apply(Command::Build {
            x: 7,
            y: 7,
            kind: TowerKind::Cryo,
        })
        .unwrap();
        b.apply(Command::Build {
            x: 3,
            y: 4,
            kind: TowerKind::Mortar,
        })
        .unwrap();
        b.wave = 12;
        b.apply(Command::StartWave).unwrap();
        // Stop at a moment with shots in flight and an active slow.
        for _ in 0..3000 {
            b.step();
            if !b.shots.is_empty() && b.enemies.iter().any(|e| e.slow.iter().any(|s| *s > 0)) {
                break;
            }
        }
        b
    }

    #[test]
    fn active_wave_round_trips_exactly_with_private_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state/ridgeline.json");
        let b = mid_wave();
        assert_eq!(b.phase, Phase::Running);
        assert!(!b.shots.is_empty(), "exercise in-flight projectiles");
        assert!(
            b.enemies.iter().any(|e| e.slow.iter().any(|s| *s > 0)),
            "exercise slows"
        );
        let save = Save {
            battle: Some(b.clone()),
            ..Save::default()
        };
        write(&path, &save).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(loaded, save);
        // Continuing the restored battle matches continuing the original.
        let (mut a, mut c) = (b, loaded.battle.unwrap());
        while a.phase == Phase::Running {
            a.step();
            c.step();
        }
        assert_eq!(a, c);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn rejected_files_are_unchanged_and_archives_are_unique() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ridgeline.json");
        let mut future = serde_json::to_value(Save::default()).unwrap();
        future["version"] = 2.into();
        let mut tampered = Save::default();
        let mut b = mid_wave();
        b.towers[0].x = 0;
        b.towers[0].y = 2; // on the road
        tampered.battle = Some(b);
        let locked = Save {
            battle: Some(Battle::new(4, Difficulty::Normal)),
            ..Save::default()
        };
        for bytes in [
            b"not json".to_vec(),
            serde_json::to_vec(&future).unwrap(),
            serde_json::to_vec(&tampered).unwrap(),
            serde_json::to_vec(&locked).unwrap(),
            vec![b' '; LIMIT + 1],
        ] {
            fs::write(&path, &bytes).unwrap();
            assert!(load(&path).is_err());
            assert_eq!(fs::read(&path).unwrap(), bytes);
            let a = archive(&path).unwrap();
            let c = archive(&path).unwrap();
            assert_ne!(a, c);
            assert_eq!(fs::read(&a).unwrap(), bytes);
            assert!(write(&path, &tampered).is_err());
        }
        assert!(load(&dir.path().join("missing.json")).unwrap() == Save::default());
    }

    #[test]
    fn medals_and_unlocks_are_awarded_once() {
        let mut s = Save::default();
        assert!(s.progress.unlocked(0, Difficulty::Normal));
        assert!(!s.progress.unlocked(1, Difficulty::Normal));
        assert!(!s.progress.unlocked(0, Difficulty::Hard));
        let mut b = Battle::new(0, Difficulty::Normal);
        b.phase = Phase::Victory;
        b.health = 12;
        s.battle = Some(b);
        let award = s.observe().unwrap();
        assert_eq!(award.medal, Medal::Clear);
        assert!(award.unlocked_next && award.unlocked_hard);
        assert_eq!(s.observe(), None);
        assert_eq!(s.progress.record(0, Difficulty::Normal).wins, 1);
        assert!(s.progress.unlocked(1, Difficulty::Normal));
        assert!(s.progress.unlocked(0, Difficulty::Hard));
        assert!(s.valid());
        // A later perfect replay upgrades the medal without re-announcing unlocks.
        let mut b = Battle::new(0, Difficulty::Normal);
        b.phase = Phase::Victory;
        s.battle = Some(b);
        let award = s.observe().unwrap();
        assert_eq!(award.medal, Medal::Perfect);
        assert!(!award.unlocked_next && !award.unlocked_hard);
        let r = s.progress.record(0, Difficulty::Normal);
        assert_eq!((r.medal, r.best_health, r.wins), (Medal::Perfect, 20, 2));
        // Defeat awards nothing but is still marked recorded.
        let mut b = Battle::new(1, Difficulty::Normal);
        b.phase = Phase::Defeat;
        b.health = 0;
        s.battle = Some(b);
        assert_eq!(s.observe(), None);
        assert!(s.battle.as_ref().unwrap().recorded);
        assert_eq!(s.progress.record(1, Difficulty::Normal), Record::default());
        assert!(s.valid());
    }
}
