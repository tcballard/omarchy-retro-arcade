//! Deterministic 60 Hz combat. No UI, clock or randomness lives here.
//!
//! Each tick resolves, in order: due spawns; enemy movement and exits (base
//! damage); arriving projectiles by launch order; tower reload and firing by
//! build order; then wave completion. Base health reaching zero is always a
//! defeat, even when the same tick resolves the final wave.
use crate::data::*;
use serde::{Deserialize, Serialize};

pub const RULES: u32 = 1;
pub const BASE_HEALTH: u32 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// Between waves; nothing moves until the player starts the next wave.
    Waiting,
    Running,
    Victory,
    Defeat,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tower {
    pub id: u32,
    pub kind: TowerKind,
    pub tier: u8,
    pub x: i32,
    pub y: i32,
    pub spent: u32,
    /// Ticks until the next shot is allowed.
    pub reload: u32,
}
impl Tower {
    pub fn pos(&self) -> P {
        P::cell(self.x, self.y)
    }
    pub fn stats(&self) -> TowerStats {
        tower(self.kind, self.tier)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enemy {
    pub id: u32,
    pub kind: EnemyKind,
    pub route: usize,
    /// Distance along the route in hundredths of a world unit.
    pub progress: u64,
    pub hp: u32,
    pub max_hp: u32,
    /// Remaining slow ticks per Cryo tier; the strongest active tier applies.
    pub slow: [u32; TIERS],
}
impl Enemy {
    pub fn distance(&self) -> u32 {
        (self.progress / 100) as u32
    }
    pub fn slow_percent(&self) -> u32 {
        (0..TIERS)
            .rev()
            .find(|&t| self.slow[t] > 0)
            .map_or(0, |t| tower(TowerKind::Cryo, t as u8).slow)
    }
    /// Movement this tick in hundredths of a unit; slows never reach zero.
    fn step(&self) -> u64 {
        u64::from(self.kind.stats().speed) * u64::from(100 - self.slow_percent())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shot {
    pub id: u32,
    pub kind: TowerKind,
    pub tier: u8,
    pub from: P,
    /// Committed impact point for splash; latest target position for direct shots.
    pub to: P,
    /// Direct shots strike this enemy on arrival; they never retarget.
    pub target: Option<u32>,
    pub damage: u32,
    pub splash: i32,
    pub flight: u32,
    pub elapsed: u32,
}
impl Shot {
    pub fn position(&self) -> P {
        let f = i64::from(self.flight.max(1));
        let e = i64::from(self.elapsed.min(self.flight));
        P::new(
            self.from.x + (i64::from(self.to.x - self.from.x) * e / f) as i32,
            self.from.y + (i64::from(self.to.y - self.from.y) * e / f) as i32,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Build { x: i32, y: i32, kind: TowerKind },
    Upgrade { tower: u32 },
    Sell { tower: u32 },
    StartWave,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reject {
    Finished,
    Blocked,
    Occupied,
    Unaffordable { need: u32 },
    MaxTier,
    NoTower,
    NotWaiting,
}
impl std::fmt::Display for Reject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Finished => write!(f, "The battle is over"),
            Self::Blocked => write!(f, "Cannot build on the road or rock"),
            Self::Occupied => write!(f, "A tower already stands here"),
            Self::Unaffordable { need } => write!(f, "Need {need} more credits"),
            Self::MaxTier => write!(f, "Fully upgraded"),
            Self::NoTower => write!(f, "No tower selected"),
            Self::NotWaiting => write!(f, "A wave is already running"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Fire { tower: u32, kind: TowerKind },
    Hit { at: P, kind: TowerKind, splash: i32 },
    Fizzle { at: P },
    Kill { at: P, bounty: u32, air: bool },
    Leak { at: P, damage: u32 },
    WaveCleared { wave: usize, award: u32 },
    Victory,
    Defeat,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Battle {
    pub rules: u32,
    pub layout: u32,
    pub map: usize,
    pub difficulty: Difficulty,
    pub phase: Phase,
    /// Next wave to start while waiting; current wave while running.
    pub wave: usize,
    /// Combat ticks since the battle began (waiting time never advances this).
    pub tick: u64,
    pub wave_tick: u32,
    pub spawned: usize,
    pub credits: u32,
    pub health: u32,
    pub towers: Vec<Tower>,
    pub enemies: Vec<Enemy>,
    pub shots: Vec<Shot>,
    pub next_id: u32,
    pub kills: u32,
    pub leaked: u32,
    /// Set by the progress store once the result has been awarded.
    pub recorded: bool,
}

impl Battle {
    pub fn new(map: usize, difficulty: Difficulty) -> Self {
        let map = map.min(maps().len() - 1);
        Self {
            rules: RULES,
            layout: LAYOUT,
            map,
            difficulty,
            phase: Phase::Waiting,
            wave: 0,
            tick: 0,
            wave_tick: 0,
            spawned: 0,
            credits: maps()[map].credits[difficulty.index()],
            health: BASE_HEALTH,
            towers: vec![],
            enemies: vec![],
            shots: vec![],
            next_id: 1,
            kills: 0,
            leaked: 0,
            recorded: false,
        }
    }
    pub fn data(&self) -> &'static Map {
        &maps()[self.map]
    }
    pub fn finished(&self) -> bool {
        matches!(self.phase, Phase::Victory | Phase::Defeat)
    }
    /// Has the player invested anything that replacing it would discard?
    pub fn unfinished(&self) -> bool {
        !self.finished()
            && (self.wave > 0 || self.phase == Phase::Running || !self.towers.is_empty())
    }
    /// The wave shown in previews: the next one while waiting, else the current one.
    pub fn preview(&self) -> &'static Wave {
        self.data().wave(self.difficulty, self.wave)
    }
    pub fn tower_at(&self, x: i32, y: i32) -> Option<&Tower> {
        self.towers.iter().find(|t| t.x == x && t.y == y)
    }
    pub fn tower(&self, id: u32) -> Option<&Tower> {
        self.towers.iter().find(|t| t.id == id)
    }
    pub fn enemy_pos(&self, e: &Enemy) -> P {
        self.data().routes[e.route].at(e.distance())
    }
    fn remaining(&self, e: &Enemy) -> u32 {
        self.data().routes[e.route].length() - e.distance()
    }
    /// Price of a legal placement, without changing anything.
    pub fn check_build(&self, x: i32, y: i32, kind: TowerKind) -> Result<u32, Reject> {
        if self.finished() {
            return Err(Reject::Finished);
        }
        if self.data().terrain(x, y) != Terrain::Build {
            return Err(Reject::Blocked);
        }
        if self.tower_at(x, y).is_some() {
            return Err(Reject::Occupied);
        }
        let cost = tower(kind, 0).cost;
        if cost > self.credits {
            return Err(Reject::Unaffordable {
                need: cost - self.credits,
            });
        }
        Ok(cost)
    }
    pub fn check_upgrade(&self, id: u32) -> Result<u32, Reject> {
        if self.finished() {
            return Err(Reject::Finished);
        }
        let t = self.tower(id).ok_or(Reject::NoTower)?;
        if usize::from(t.tier) + 1 >= TIERS {
            return Err(Reject::MaxTier);
        }
        let cost = tower(t.kind, t.tier + 1).cost;
        if cost > self.credits {
            return Err(Reject::Unaffordable {
                need: cost - self.credits,
            });
        }
        Ok(cost)
    }
    /// Apply a player command. Rejections never change state.
    pub fn apply(&mut self, command: Command) -> Result<(), Reject> {
        match command {
            Command::Build { x, y, kind } => {
                let cost = self.check_build(x, y, kind)?;
                self.credits -= cost;
                let id = self.id();
                self.towers.push(Tower {
                    id,
                    kind,
                    tier: 0,
                    x,
                    y,
                    spent: cost,
                    reload: 0,
                });
            }
            Command::Upgrade { tower: id } => {
                let cost = self.check_upgrade(id)?;
                self.credits -= cost;
                let t = self.towers.iter_mut().find(|t| t.id == id).unwrap();
                // Stats change immediately; the remaining reload is kept.
                t.tier += 1;
                t.spent += cost;
            }
            Command::Sell { tower: id } => {
                if self.finished() {
                    return Err(Reject::Finished);
                }
                let i = self
                    .towers
                    .iter()
                    .position(|t| t.id == id)
                    .ok_or(Reject::NoTower)?;
                // Shots already in flight keep their snapshot and still land.
                let t = self.towers.remove(i);
                self.credits += refund(t.spent);
            }
            Command::StartWave => {
                if self.finished() {
                    return Err(Reject::Finished);
                }
                if self.phase != Phase::Waiting {
                    return Err(Reject::NotWaiting);
                }
                self.phase = Phase::Running;
                self.wave_tick = 0;
                self.spawned = 0;
            }
        }
        Ok(())
    }
    fn id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Advance one ordinary simulation tick. Does nothing unless a wave is running.
    pub fn step(&mut self) -> Vec<Event> {
        let mut events = vec![];
        if self.phase != Phase::Running {
            return events;
        }
        let map = self.data();
        let wave = map.wave(self.difficulty, self.wave);
        // 1. Spawns due this tick.
        while let Some(s) = wave.spawns.get(self.spawned) {
            if s.tick > self.wave_tick {
                break;
            }
            let stats = s.kind.stats();
            let hp = (stats.hp * wave.health / 100).max(1);
            let id = self.id();
            self.enemies.push(Enemy {
                id,
                kind: s.kind,
                route: s.route,
                progress: 0,
                hp,
                max_hp: hp,
                slow: [0; TIERS],
            });
            self.spawned += 1;
        }
        // 2. Movement and exits.
        let mut i = 0;
        while i < self.enemies.len() {
            let step = self.enemies[i].step();
            let e = &mut self.enemies[i];
            e.progress += step;
            for s in &mut e.slow {
                *s = s.saturating_sub(1);
            }
            let length = u64::from(map.routes[e.route].length()) * 100;
            if e.progress >= length {
                e.progress = length;
                let damage = e.kind.stats().damage;
                let at = map.routes[e.route].at(e.distance());
                self.health = self.health.saturating_sub(damage);
                self.leaked += 1;
                self.enemies.remove(i);
                events.push(Event::Leak { at, damage });
            } else {
                i += 1;
            }
        }
        // 3. Projectiles, in launch order.
        for shot in &mut self.shots {
            shot.elapsed += 1;
        }
        let mut s = 0;
        while s < self.shots.len() {
            if let Some(target) = self.shots[s].target {
                if let Some(e) = self.enemies.iter().find(|e| e.id == target) {
                    self.shots[s].to = map.routes[e.route].at(e.distance());
                }
            }
            if self.shots[s].elapsed < self.shots[s].flight {
                s += 1;
                continue;
            }
            let shot = self.shots.remove(s);
            self.resolve(&shot, &mut events);
        }
        // 4. Towers, in build order.
        for t in 0..self.towers.len() {
            if self.towers[t].reload > 0 {
                self.towers[t].reload -= 1;
            }
            if self.towers[t].reload > 0 {
                continue;
            }
            let Some(target) = self.target(&self.towers[t]) else {
                continue;
            };
            let (tower_id, kind, tier) =
                (self.towers[t].id, self.towers[t].kind, self.towers[t].tier);
            let stats = self.towers[t].stats();
            let from = self.towers[t].pos();
            let e = self.enemies.iter().find(|e| e.id == target).unwrap();
            let now = self.enemy_pos(e);
            let flight = (from.dist(now) as u32)
                .div_ceil(stats.shot_speed as u32)
                .max(1);
            let (to, target) = if stats.splash > 0 {
                // Lead the target by its current speed; the impact point is committed.
                let lead = e.progress + e.step() * u64::from(flight);
                let route = &map.routes[e.route];
                (route.at((lead / 100) as u32), None)
            } else {
                (now, Some(target))
            };
            let id = self.id();
            self.towers[t].reload = stats.reload;
            self.shots.push(Shot {
                id,
                kind,
                tier,
                from,
                to,
                target,
                damage: stats.damage,
                splash: stats.splash,
                flight,
                elapsed: 0,
            });
            events.push(Event::Fire {
                tower: tower_id,
                kind,
            });
        }
        self.tick += 1;
        self.wave_tick += 1;
        // 5. Results: defeat takes precedence over any clear on the same tick.
        if self.health == 0 {
            self.phase = Phase::Defeat;
            self.shots.clear();
            events.push(Event::Defeat);
        } else if self.spawned == wave.spawns.len() && self.enemies.is_empty() {
            let award = self.difficulty.wave_award();
            self.credits += award;
            events.push(Event::WaveCleared {
                wave: self.wave,
                award,
            });
            self.shots.clear();
            self.wave += 1;
            if self.wave >= WAVES {
                self.wave = WAVES - 1;
                self.phase = Phase::Victory;
                events.push(Event::Victory);
            } else {
                self.phase = Phase::Waiting;
            }
            self.wave_tick = 0;
            self.spawned = 0;
            for t in &mut self.towers {
                t.reload = 0;
            }
        }
        events
    }

    /// Least remaining route distance first; stable spawn ID breaks ties.
    fn target(&self, t: &Tower) -> Option<u32> {
        let stats = t.stats();
        let air = t.kind.targets() == Targets::Air;
        let pos = t.pos();
        let (max, min) = (
            i64::from(stats.range).pow(2),
            i64::from(stats.min_range).pow(2),
        );
        self.enemies
            .iter()
            .filter(|e| e.kind.stats().air == air)
            .filter(|e| {
                let d = pos.dist2(self.enemy_pos(e));
                d <= max && d >= min
            })
            .min_by_key(|e| (self.remaining(e), e.id))
            .map(|e| e.id)
    }

    fn resolve(&mut self, shot: &Shot, events: &mut Vec<Event>) {
        let map = self.data();
        let slow = (shot.kind == TowerKind::Cryo).then_some(shot.tier);
        let victims: Vec<u32> = if let Some(target) = shot.target {
            match self.enemies.iter().find(|e| e.id == target) {
                Some(e) => vec![e.id],
                None => {
                    events.push(Event::Fizzle { at: shot.to });
                    return;
                }
            }
        } else {
            // Splash reads enemy positions at impact, whether or not its aim survived.
            let r2 = i64::from(shot.splash).pow(2);
            self.enemies
                .iter()
                .filter(|e| !e.kind.stats().air)
                .filter(|e| map.routes[e.route].at(e.distance()).dist2(shot.to) <= r2)
                .map(|e| e.id)
                .collect()
        };
        events.push(Event::Hit {
            at: shot.to,
            kind: shot.kind,
            splash: shot.splash,
        });
        for id in victims {
            let Some(i) = self.enemies.iter().position(|e| e.id == id) else {
                continue;
            };
            let e = &mut self.enemies[i];
            let stats = e.kind.stats();
            // Fixed armour reduction, never below one damage per hit.
            let damage = shot.damage.saturating_sub(stats.armour).max(1);
            e.hp = e.hp.saturating_sub(damage);
            if let Some(tier) = slow {
                let tier = usize::from(tier);
                e.slow[tier] = e.slow[tier].max(tower(TowerKind::Cryo, tier as u8).slow_ticks);
            }
            if e.hp == 0 {
                // Removal here makes the bounty impossible to award twice.
                let at = map.routes[e.route].at(e.distance());
                self.enemies.remove(i);
                self.credits += stats.bounty;
                self.kills += 1;
                events.push(Event::Kill {
                    at,
                    bounty: stats.bounty,
                    air: stats.air,
                });
            }
        }
    }

    /// Structural validation of a restored battle against the current data.
    pub fn valid(&self) -> bool {
        let Some(map) = maps().get(self.map) else {
            return false;
        };
        let wave = map.wave(self.difficulty, self.wave);
        let mut cells: Vec<(i32, i32)> = self.towers.iter().map(|t| (t.x, t.y)).collect();
        cells.sort_unstable();
        cells.dedup();
        let mut ids: Vec<u32> = self
            .towers
            .iter()
            .map(|t| t.id)
            .chain(self.enemies.iter().map(|e| e.id))
            .chain(self.shots.iter().map(|s| s.id))
            .collect();
        ids.sort_unstable();
        ids.dedup();
        self.rules == RULES
            && self.layout == LAYOUT
            && self.wave < WAVES
            && self.health <= BASE_HEALTH
            && (self.health == 0) == (self.phase == Phase::Defeat)
            && self.spawned <= wave.spawns.len()
            && cells.len() == self.towers.len()
            && ids.len() == self.towers.len() + self.enemies.len() + self.shots.len()
            && ids.last().is_none_or(|id| *id < self.next_id)
            && self.towers.iter().all(|t| {
                usize::from(t.tier) < TIERS
                    && map.terrain(t.x, t.y) == Terrain::Build
                    && t.spent == invested(t.kind, t.tier)
                    && t.reload <= tower(t.kind, 0).reload
            })
            && self.enemies.iter().all(|e| {
                map.routes.get(e.route).is_some_and(|r| {
                    r.air == e.kind.stats().air && e.progress < u64::from(r.length()) * 100
                }) && e.hp > 0
                    && e.hp <= e.max_hp
                    && e.slow
                        .iter()
                        .zip(0u8..)
                        .all(|(s, t)| *s <= tower(TowerKind::Cryo, t).slow_ticks)
            })
            && self.shots.iter().all(|s| {
                usize::from(s.tier) < TIERS
                    && s.elapsed < s.flight
                    && s.damage == tower(s.kind, s.tier).damage
                    && s.splash == tower(s.kind, s.tier).splash
                    && (s.splash > 0) == s.target.is_none()
            })
            && (matches!(self.phase, Phase::Running | Phase::Defeat)
                || (self.enemies.is_empty() && self.shots.is_empty()))
    }
}

/// Converts real elapsed time to whole ticks; 2× runs twice as many ordinary ticks.
///
/// A frame hitch up to one second runs at most a quarter-second of ticks and
/// drops the rest, so the defence briefly slows rather than jumping ahead.
/// Longer gaps (suspend, a hung compositor) pause instead.
#[derive(Clone, Debug, Default)]
pub struct Clock {
    remainder: f64,
}
pub const MAX_FRAME: f64 = 0.25;
pub const STALL: f64 = 1.0;
impl Clock {
    pub fn reset(&mut self) {
        self.remainder = 0.;
    }
    pub fn advance(&mut self, seconds: f64, speed: u32) -> Result<u32, &'static str> {
        if !seconds.is_finite() || !(0. ..=STALL).contains(&seconds) {
            self.remainder = 0.;
            return Err("Rendering stalled, so the defence paused. Resume when ready.");
        }
        let seconds = seconds.min(MAX_FRAME);
        self.remainder += seconds * f64::from(TICKS_PER_SECOND) * f64::from(speed.clamp(1, 2));
        let ticks = (self.remainder + 1e-9).floor();
        self.remainder -= ticks;
        Ok(ticks as u32)
    }
}

/// A tick-stamped command list. Commands at tick N apply before tick N runs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Replay {
    pub map: usize,
    pub difficulty: Difficulty,
    pub rules: u32,
    pub layout: u32,
    pub commands: Vec<(u64, Command)>,
}
impl Replay {
    /// Run through the production engine until the result or the list is exhausted.
    pub fn run(&self) -> Result<Battle, String> {
        let mut b = Battle::new(self.map, self.difficulty);
        for (tick, command) in &self.commands {
            while b.tick < *tick && b.phase == Phase::Running {
                b.step();
            }
            if b.tick != *tick {
                return Err(format!("command scheduled at {tick} reached at {}", b.tick));
            }
            b.apply(*command).map_err(|e| format!("tick {tick}: {e}"))?;
        }
        while b.phase == Phase::Running {
            b.step();
        }
        Ok(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_cell(b: &Battle) -> (i32, i32) {
        (0..ROWS)
            .flat_map(|y| (0..COLS).map(move |x| (x, y)))
            .find(|&(x, y)| b.data().terrain(x, y) == Terrain::Build && b.tower_at(x, y).is_none())
            .unwrap()
    }
    fn enemy(b: &mut Battle, kind: EnemyKind, route: usize, distance: u32) -> u32 {
        let id = b.next_id;
        b.next_id += 1;
        let hp = kind.stats().hp;
        b.enemies.push(Enemy {
            id,
            kind,
            route,
            progress: u64::from(distance) * 100,
            hp,
            max_hp: hp,
            slow: [0; TIERS],
        });
        id
    }
    /// A running battle whose scheduled wave has fully spawned, for hand-placed enemies.
    fn running(map: usize) -> Battle {
        let mut b = Battle::new(map, Difficulty::Normal);
        b.apply(Command::StartWave).unwrap();
        b.spawned = b.preview().spawns.len();
        b
    }

    #[test]
    fn placement_rejections_spend_nothing() {
        let mut b = Battle::new(0, Difficulty::Normal);
        let before = b.clone();
        // (0,2) is the spawn road; (10,0) is rock.
        assert_eq!(
            b.apply(Command::Build {
                x: 0,
                y: 2,
                kind: TowerKind::Cannon
            }),
            Err(Reject::Blocked)
        );
        assert_eq!(
            b.apply(Command::Build {
                x: 10,
                y: 0,
                kind: TowerKind::Cannon
            }),
            Err(Reject::Blocked)
        );
        assert_eq!(
            b.apply(Command::Build {
                x: -1,
                y: 0,
                kind: TowerKind::Cannon
            }),
            Err(Reject::Blocked)
        );
        assert_eq!(b, before);
        b.apply(Command::Build {
            x: 1,
            y: 1,
            kind: TowerKind::Cannon,
        })
        .unwrap();
        let placed = b.clone();
        assert_eq!(
            b.apply(Command::Build {
                x: 1,
                y: 1,
                kind: TowerKind::Flak
            }),
            Err(Reject::Occupied)
        );
        b.credits = 10;
        let poor = b.clone();
        assert_eq!(
            b.apply(Command::Build {
                x: 2,
                y: 1,
                kind: TowerKind::Mortar
            }),
            Err(Reject::Unaffordable { need: 80 })
        );
        assert_eq!(
            b.apply(Command::Upgrade { tower: 1 }),
            Err(Reject::Unaffordable { need: 35 })
        );
        assert_eq!(b, poor);
        assert_eq!(placed.credits, 150);
    }

    #[test]
    fn spending_upgrades_and_refunds_cannot_create_money() {
        let mut b = Battle::new(0, Difficulty::Normal);
        b.credits = 1000;
        b.apply(Command::Build {
            x: 1,
            y: 1,
            kind: TowerKind::Mortar,
        })
        .unwrap();
        b.apply(Command::Upgrade { tower: 1 }).unwrap();
        b.apply(Command::Upgrade { tower: 1 }).unwrap();
        assert_eq!(b.apply(Command::Upgrade { tower: 1 }), Err(Reject::MaxTier));
        assert_eq!(b.credits, 1000 - 300);
        b.apply(Command::Sell { tower: 1 }).unwrap();
        assert_eq!(b.credits, 700 + 210);
        assert!(b.towers.is_empty());
        assert_eq!(b.apply(Command::Sell { tower: 1 }), Err(Reject::NoTower));
    }

    #[test]
    fn upgrade_keeps_remaining_reload_and_inflight_snapshot() {
        let mut b = running(0);
        b.credits = 1000;
        b.apply(Command::Build {
            x: 5,
            y: 1,
            kind: TowerKind::Cannon,
        })
        .unwrap();
        let e = enemy(&mut b, EnemyKind::Hauler, 0, 5000);
        b.enemies[0].slow = [0; TIERS];
        b.step();
        assert_eq!(b.shots.len(), 1);
        let reload = b.towers[0].reload;
        assert_eq!(reload, tower(TowerKind::Cannon, 0).reload);
        b.apply(Command::Upgrade { tower: 1 }).unwrap();
        assert_eq!(b.towers[0].reload, reload);
        assert_eq!(b.shots[0].damage, tower(TowerKind::Cannon, 0).damage);
        // Selling leaves the snapshot in flight; it still strikes.
        let hp = b.enemies[0].hp;
        b.apply(Command::Sell { tower: 1 }).unwrap();
        for _ in 0..40 {
            b.step();
        }
        let after = b.enemies.iter().find(|x| x.id == e).unwrap().hp;
        assert_eq!(hp - after, 10 - 2);
    }

    #[test]
    fn target_order_prefers_least_remaining_then_spawn_id() {
        let mut b = running(0);
        b.apply(Command::Build {
            x: 5,
            y: 1,
            kind: TowerKind::Cannon,
        })
        .unwrap();
        let _behind = enemy(&mut b, EnemyKind::Crawler, 0, 4000);
        let ahead = enemy(&mut b, EnemyKind::Crawler, 0, 5500);
        let tie = enemy(&mut b, EnemyKind::Crawler, 0, 5500);
        let t = b.towers[0].clone();
        assert_eq!(b.target(&t), Some(ahead));
        assert!(ahead < tie);
    }

    #[test]
    fn armour_minimum_damage_and_slow_rules() {
        let mut b = running(0);
        let id = enemy(&mut b, EnemyKind::Plated, 0, 1000);
        let shot = |kind: TowerKind, tier: u8, damage: u32| Shot {
            id: 999,
            kind,
            tier,
            from: P::cell(0, 0),
            to: P::cell(0, 0),
            target: Some(id),
            damage,
            splash: 0,
            flight: 1,
            elapsed: 1,
        };
        let mut ev = vec![];
        b.resolve(&shot(TowerKind::Cannon, 0, 10), &mut ev);
        assert_eq!(b.enemies[0].hp, 70 - 6);
        b.resolve(&shot(TowerKind::Cryo, 0, 3), &mut ev);
        assert_eq!(b.enemies[0].hp, 64 - 1);
        assert_eq!(b.enemies[0].slow_percent(), 30);
        // A stronger slow takes over; a weaker hit refreshes only its own timer.
        b.resolve(&shot(TowerKind::Cryo, 2, 8), &mut ev);
        assert_eq!(b.enemies[0].slow_percent(), 50);
        b.enemies[0].slow[0] = 1;
        b.resolve(&shot(TowerKind::Cryo, 0, 3), &mut ev);
        assert_eq!(b.enemies[0].slow[0], 90);
        assert_eq!(b.enemies[0].slow_percent(), 50);
        assert!(b.enemies[0].step() > 0);
        b.enemies[0].slow[2] = 0;
        assert_eq!(b.enemies[0].slow_percent(), 30);
    }

    #[test]
    fn splash_reads_positions_at_impact_and_survives_target_death() {
        let mut b = running(0);
        let a = enemy(&mut b, EnemyKind::Crawler, 0, 3000);
        let c = enemy(&mut b, EnemyKind::Crawler, 0, 3400);
        let far = enemy(&mut b, EnemyKind::Crawler, 0, 9000);
        b.enemies.retain(|e| e.id != a);
        let to = b.data().routes[0].at(3000);
        let mut ev = vec![];
        b.resolve(
            &Shot {
                id: 99,
                kind: TowerKind::Mortar,
                tier: 0,
                from: P::cell(3, 5),
                to,
                target: None,
                damage: 16,
                splash: 900,
                flight: 1,
                elapsed: 1,
            },
            &mut ev,
        );
        assert_eq!(b.enemies.iter().find(|e| e.id == c).unwrap().hp, 24);
        assert_eq!(b.enemies.iter().find(|e| e.id == far).unwrap().hp, 40);
    }

    #[test]
    fn direct_shot_fizzles_when_target_is_gone_and_air_is_ineligible_for_ground() {
        let mut b = running(2);
        b.credits = 500;
        b.apply(Command::Build {
            x: 3,
            y: 2,
            kind: TowerKind::Cannon,
        })
        .unwrap();
        let g = enemy(&mut b, EnemyKind::Glider, 1, 3000);
        let t = b.towers[0].clone();
        assert!(b.enemy_pos(&b.enemies[0]).dist(t.pos()) < t.stats().range);
        assert_eq!(b.target(&t), None);
        b.apply(Command::Build {
            x: 3,
            y: 3,
            kind: TowerKind::Flak,
        })
        .unwrap();
        let f = b.towers[1].clone();
        assert_eq!(b.target(&f), Some(g));
        b.step();
        let flak_shots = b.shots.len();
        assert_eq!(flak_shots, 1);
        b.enemies.clear();
        b.spawned = 0; // keep the wave open
        let ev = (0..20).flat_map(|_| b.step()).collect::<Vec<_>>();
        assert!(ev.iter().any(|e| matches!(e, Event::Fizzle { .. })));
        assert!(!ev.iter().any(|e| matches!(e, Event::Kill { .. })));
    }

    #[test]
    fn simultaneous_hits_award_one_bounty() {
        let mut b = running(0);
        let id = enemy(&mut b, EnemyKind::Crawler, 0, 3000);
        b.enemies[0].hp = 5;
        let credits = b.credits;
        let s = |sid| Shot {
            id: sid,
            kind: TowerKind::Cannon,
            tier: 0,
            from: P::cell(0, 0),
            to: P::cell(0, 0),
            target: Some(id),
            damage: 10,
            splash: 0,
            flight: 1,
            elapsed: 0,
        };
        b.shots = vec![s(100), s(101), s(102)];
        b.next_id = 200;
        let ev = b.step();
        let kills = ev
            .iter()
            .filter(|e| matches!(e, Event::Kill { .. }))
            .count();
        assert_eq!(kills, 1);
        assert_eq!(
            ev.iter()
                .filter(|e| matches!(e, Event::Fizzle { .. }))
                .count(),
            2
        );
        // Wave also clears here (all spawned, none left) with its fixed award.
        assert_eq!(b.credits, credits + 4 + 25);
        assert_eq!(b.phase, Phase::Waiting);
        assert_eq!(b.kills, 1);
    }

    #[test]
    fn waiting_never_advances_and_start_wave_is_waiting_only() {
        let mut b = Battle::new(0, Difficulty::Normal);
        let before = b.clone();
        for _ in 0..100 {
            assert!(b.step().is_empty());
        }
        assert_eq!(b, before);
        b.apply(Command::StartWave).unwrap();
        assert_eq!(b.apply(Command::StartWave), Err(Reject::NotWaiting));
        b.step();
        assert_eq!(b.enemies.len(), 1);
        assert_eq!(b.tick, 1);
    }

    #[test]
    fn undefended_campaign_is_lost_and_exit_damage_is_exact() {
        for map in 0..maps().len() {
            let mut b = Battle::new(map, Difficulty::Normal);
            let mut leaks = 0;
            while !b.finished() {
                b.apply(Command::StartWave).unwrap();
                while b.phase == Phase::Running {
                    for e in b.step() {
                        if let Event::Leak { damage, .. } = e {
                            leaks += damage;
                        }
                    }
                }
            }
            assert_eq!(b.phase, Phase::Defeat, "map {map}");
            assert!(leaks >= BASE_HEALTH);
            assert_eq!(b.health, 0);
        }
    }

    #[test]
    fn final_wave_defeat_takes_precedence_over_clear() {
        let mut b = Battle::new(0, Difficulty::Normal);
        b.wave = WAVES - 1;
        b.apply(Command::StartWave).unwrap();
        b.spawned = b.preview().spawns.len();
        b.health = 1;
        let len = b.data().routes[0].length();
        enemy(&mut b, EnemyKind::Crawler, 0, len - 1);
        let ev = b.step();
        assert_eq!(b.phase, Phase::Defeat);
        assert!(ev.contains(&Event::Defeat));
        assert!(!ev.contains(&Event::Victory));
        assert!(b.valid());
        // With health left, the same exit is followed by victory.
        let mut b = Battle::new(0, Difficulty::Normal);
        b.wave = WAVES - 1;
        b.apply(Command::StartWave).unwrap();
        b.spawned = b.preview().spawns.len();
        enemy(&mut b, EnemyKind::Crawler, 0, len - 1);
        let ev = b.step();
        assert_eq!(b.phase, Phase::Victory);
        assert_eq!(b.health, BASE_HEALTH - 1);
        assert!(ev.contains(&Event::Victory));
        assert!(b.apply(Command::StartWave).is_err());
        let (x, y) = build_cell(&b);
        assert_eq!(
            b.apply(Command::Build {
                x,
                y,
                kind: TowerKind::Cannon
            }),
            Err(Reject::Finished)
        );
    }

    #[test]
    fn render_rate_and_speed_do_not_change_results() {
        let replay = Replay {
            map: 0,
            difficulty: Difficulty::Normal,
            rules: RULES,
            layout: LAYOUT,
            commands: vec![
                (
                    0,
                    Command::Build {
                        x: 5,
                        y: 1,
                        kind: TowerKind::Cannon,
                    },
                ),
                (
                    0,
                    Command::Build {
                        x: 7,
                        y: 7,
                        kind: TowerKind::Cryo,
                    },
                ),
                (0, Command::StartWave),
            ],
        };
        let reference = replay.run().unwrap();
        for (fps, speed) in [(30, 1), (60, 1), (144, 1), (75, 2), (240, 2)] {
            let mut b = Battle::new(0, Difficulty::Normal);
            for (_, c) in &replay.commands {
                b.apply(*c).unwrap();
            }
            let mut clock = Clock::default();
            while b.phase == Phase::Running {
                for _ in 0..clock.advance(1. / f64::from(fps), speed).unwrap() {
                    b.step();
                }
            }
            assert_eq!(b, reference, "{fps} fps at {speed}x");
        }
        let mut clock = Clock::default();
        // Hitches slow the defence (at most 15 ticks at 1×); long gaps pause.
        assert_eq!(clock.advance(0.5, 1), Ok(15));
        assert_eq!(clock.advance(0.9, 2), Ok(30));
        assert_eq!(
            clock.advance(1.5, 1),
            Err("Rendering stalled, so the defence paused. Resume when ready.")
        );
        let ticks: u32 = (0..60).map(|_| clock.advance(1. / 60., 2).unwrap()).sum();
        assert_eq!(ticks, 120);
    }
}
