//! A transparent reference strategy used only for balance evidence.
//!
//! It issues ordinary player commands between waves and never reads future
//! randomness (there is none) or engine internals the player cannot see: map
//! geometry, tower data and the published wave previews. The live game never
//! calls this module.
use crate::battle::{Battle, Command, Phase, Replay, RULES};
use crate::data::*;

/// How far ahead the planner reads wave previews, with decaying weights.
const LOOKAHEAD: [f64; 3] = [1.0, 0.6, 0.3];
const SAMPLE: u32 = 200;

struct Demand {
    /// Weighted enemy health by route.
    route: Vec<f64>,
    /// Weighted health per class: [ground, air].
    need: [f64; 2],
    /// Weighted mean armour for ground units.
    armour: f64,
}

fn demand(b: &Battle) -> Demand {
    let map = b.data();
    let mut route = vec![0.; map.routes.len()];
    let mut need = [0.; 2];
    let mut armour = 0.;
    for (i, w) in LOOKAHEAD.iter().enumerate() {
        let n = b.wave + i;
        if n >= WAVES {
            break;
        }
        let wave = map.wave(b.difficulty, n);
        for s in &wave.spawns {
            let stats = s.kind.stats();
            let hp = f64::from(stats.hp * wave.health / 100) * w;
            route[s.route] += hp;
            need[usize::from(stats.air)] += hp;
            if !stats.air {
                armour += f64::from(stats.armour) * hp;
            }
        }
    }
    Demand {
        route,
        armour: if need[0] > 0. { armour / need[0] } else { 0. },
        need,
    }
}

fn class(kind: TowerKind) -> usize {
    usize::from(kind.targets() == Targets::Air)
}

/// Route samples a tower at (x, y) could engage, weighted by each route's share.
fn coverage(map: &Map, kind: TowerKind, tier: u8, x: i32, y: i32, d: &Demand) -> f64 {
    let s = tower(kind, tier);
    let pos = P::cell(x, y);
    let (max, min) = (i64::from(s.range).pow(2), i64::from(s.min_range).pow(2));
    let c = class(kind);
    if d.need[c] <= 0. {
        return 0.;
    }
    map.routes
        .iter()
        .enumerate()
        .filter(|(_, r)| usize::from(r.air) == c)
        .map(|(i, r)| {
            let hits = (0..r.length() / SAMPLE)
                .map(|k| pos.dist2(r.at(k * SAMPLE)))
                .filter(|d2| *d2 <= max && *d2 >= min)
                .count();
            hits as f64 * d.route[i] / d.need[c]
        })
        .sum()
}

/// Damage potential: effective damage per second times weighted route exposure.
fn potential(map: &Map, kind: TowerKind, tier: u8, x: i32, y: i32, d: &Demand) -> f64 {
    let s = tower(kind, tier);
    let armour = if kind.targets() == Targets::Air {
        0.
    } else {
        d.armour
    };
    let per_shot = (f64::from(s.damage) - armour).max(1.);
    let mut v = per_shot * 60. / f64::from(s.reload);
    if s.splash > 0 {
        v *= 2.5;
    }
    if s.slow > 0 {
        v += f64::from(s.slow) * 0.12;
    }
    v * coverage(map, kind, tier, x, y, d)
}

/// Concave utility: each class saturates relative to its own threat, so the
/// planner funds air and ground defence in proportion rather than one alone.
fn utility(p: [f64; 2], d: &Demand, alpha: f64) -> f64 {
    (0..2)
        .filter(|c| d.need[*c] > 0.)
        .map(|c| d.need[c] * (1. - (-p[c] / (alpha * d.need[c])).exp()))
        .sum()
}

/// Candidate spending styles: saturation strength and highest tier to buy.
const STYLES: [(f64, u8); 7] = [
    (0.5, 2),
    (0.25, 2),
    (1.0, 2),
    (2.0, 2),
    (0.5, 1),
    (1.0, 1),
    (0.5, 0),
];
/// Waves simulated when comparing styles.
const TRIAL_WAVES: usize = 3;

/// Spending decisions for one planning moment. Only affordable, legal commands.
///
/// Each candidate style is tried on a copy through the next three waves using the
/// production engine; the plan losing the least base health is chosen, with
/// ties going to the earlier style. Players see the same previews.
pub fn plan(b: &Battle) -> Vec<Command> {
    let mut best: Option<(u32, Vec<Command>)> = None;
    for (i, &(alpha, max_tier)) in STYLES.iter().enumerate() {
        let commands = plan_with(b, alpha, max_tier);
        if i > 0 && best.as_ref().is_some_and(|(lost, _)| *lost == 0) {
            break;
        }
        let mut trial = b.clone();
        for c in &commands {
            trial.apply(*c).unwrap();
        }
        for n in 0..TRIAL_WAVES {
            if trial.phase != Phase::Waiting {
                break;
            }
            if n > 0 {
                for c in plan_with(&trial, STYLES[0].0, STYLES[0].1) {
                    trial.apply(c).unwrap();
                }
            }
            trial.apply(Command::StartWave).unwrap();
            while trial.phase == Phase::Running {
                trial.step();
            }
        }
        let lost = b.health - trial.health;
        if best.as_ref().is_none_or(|(l, _)| lost < *l) {
            best = Some((lost, commands));
        }
    }
    best.map(|(_, c)| c).unwrap_or_default()
}

fn plan_with(b: &Battle, alpha: f64, max_tier: u8) -> Vec<Command> {
    let mut b = b.clone();
    let mut commands = vec![];
    let d = demand(&b);
    let map = b.data();
    loop {
        let mut p = [0.; 2];
        for t in &b.towers {
            p[class(t.kind)] += potential(map, t.kind, t.tier, t.x, t.y, &d);
        }
        let base = utility(p, &d, alpha);
        let gain = |kind: TowerKind, delta: f64| {
            let mut q = p;
            q[class(kind)] += delta;
            utility(q, &d, alpha) - base
        };
        let mut best: Option<(f64, Command)> = None;
        let mut consider = |ratio: f64, c: Command| {
            if ratio > 1e-9 && best.is_none_or(|(r, _)| ratio > r) {
                best = Some((ratio, c));
            }
        };
        for t in b.towers.iter().filter(|t| t.tier < max_tier) {
            if let Ok(cost) = b.check_upgrade(t.id) {
                let delta = potential(map, t.kind, t.tier + 1, t.x, t.y, &d)
                    - potential(map, t.kind, t.tier, t.x, t.y, &d);
                consider(
                    gain(t.kind, delta) / f64::from(cost),
                    Command::Upgrade { tower: t.id },
                );
            }
        }
        for kind in TowerKind::ALL {
            // Keep cryo as a supporting role: at most one per four towers.
            let cryo = b
                .towers
                .iter()
                .filter(|t| t.kind == TowerKind::Cryo)
                .count();
            if kind == TowerKind::Cryo && cryo * 4 >= b.towers.len().max(1) {
                continue;
            }
            for y in 0..ROWS {
                for x in 0..COLS {
                    if let Ok(cost) = b.check_build(x, y, kind) {
                        let v = potential(map, kind, 0, x, y, &d);
                        consider(
                            gain(kind, v) / f64::from(cost),
                            Command::Build { x, y, kind },
                        );
                    }
                }
            }
        }
        match best {
            Some((_, c)) => {
                b.apply(c).expect("planned commands are legal");
                commands.push(c);
            }
            None => return commands,
        }
    }
}

/// Play a whole battle with the reference strategy, returning its replay.
pub fn play(map: usize, difficulty: Difficulty) -> (Replay, Battle) {
    let mut b = Battle::new(map, difficulty);
    let mut commands = vec![];
    while b.phase == Phase::Waiting {
        for c in plan(&b) {
            b.apply(c).unwrap();
            commands.push((b.tick, c));
        }
        b.apply(Command::StartWave).unwrap();
        commands.push((b.tick, Command::StartWave));
        while b.phase == Phase::Running {
            b.step();
        }
    }
    (
        Replay {
            map,
            difficulty,
            rules: RULES,
            layout: LAYOUT,
            commands,
        },
        b,
    )
}
