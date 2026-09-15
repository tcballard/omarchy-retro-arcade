use crate::{
    collision::circle_interval,
    world::{self, Kind, Obstacle},
};
use serde::{Deserialize, Serialize};
pub const RULES_VERSION: u32 = 3;
pub const HZ: u32 = 60;
pub const DT: f64 = 1. / HZ as f64;
/// Normal downhill speed cap. Kept as the compatibility-facing maximum name.
pub const MAX_SPEED: f64 = 60.;
pub const FAST_MAX_SPEED: f64 = 90.;
pub const ACCELERATION: f64 = 9.;
pub const OVERSPEED_DECELERATION: f64 = 12.;
pub const MAX_HEADING: f64 = std::f64::consts::FRAC_PI_2;
pub const TURN_RATE: f64 = 1.6;
pub const RADIUS: f64 = 0.65;
pub const JUMP_DURATION: f64 = 1.2;
pub const JUMP_HEIGHT: f64 = 2.5;
pub const PROTECTION_TICKS: u32 = 90;
pub const TUMBLE_TICKS: u32 = 42;
/// A corruption guard, far beyond the distance a one-year run can normally reach.
pub const MAX_ENDLESS_DISTANCE: f64 = 2_000_000_000.;
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Practice,
    FreeSki,
    Slalom,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
impl Point {
    pub fn lerp(self, other: Self, t: f64) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Phase {
    Ready,
    Running,
    Paused,
    Finished,
    Crashed,
    Caught,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Input {
    /// Desired heading in radians, not a position.
    pub heading: f64,
    pub brake: bool,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Event {
    Jump,
    Crash,
    Finish,
    Warning,
    Spawn,
    Caught,
    Gate,
    Missed,
}

/// The skier's physical path for one fixed tick. `end` is the contact point
/// when a crash or finish stops movement; recovery relocation is excluded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TickSegment {
    pub start: Point,
    /// Uninterrupted skier endpoint for ordering actor contacts in this tick.
    pub proposed_end: Point,
    pub end: Point,
    pub stop_fraction: f64,
    pub crash_fraction: Option<f64>,
    pub finish_fraction: Option<f64>,
    pub ramp_fraction: Option<f64>,
    pub proposed_speed: f64,
    pub proposed_heading: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StepOutcome {
    pub events: Vec<Event>,
    pub segment: TickSegment,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Sim {
    pub phase: Phase,
    pub position: Point,
    pub speed: f64,
    /// The player-selected fast tuck. This is simulation state so saves and
    /// deterministic continuation retain the exact speed mode.
    #[serde(default)]
    pub fast_mode: bool,
    pub heading: f64,
    pub ticks: u64,
    pub crashes: u8,
    pub tumble: u32,
    pub protection: u32,
    /// Elapsed flight time. None means grounded.
    pub jump: Option<f64>,
    pub last_ramp: Option<usize>,
    pub distance: f64,
}
impl Default for Sim {
    fn default() -> Self {
        Self {
            phase: Phase::Ready,
            position: Point::default(),
            speed: 0.,
            fast_mode: false,
            heading: 0.,
            ticks: 0,
            crashes: 0,
            tumble: 0,
            protection: 0,
            jump: None,
            last_ramp: None,
            distance: 0.,
        }
    }
}
impl Sim {
    pub fn start(&mut self) {
        if matches!(self.phase, Phase::Ready | Phase::Paused) {
            self.phase = Phase::Running;
        }
    }
    pub fn pause(&mut self) {
        if self.phase == Phase::Running {
            self.phase = Phase::Paused;
        }
    }
    pub fn toggle_fast_mode(&mut self) -> bool {
        if self.phase != Phase::Running {
            return false;
        }
        self.fast_mode = !self.fast_mode;
        true
    }
    pub fn ended(&self) -> bool {
        matches!(self.phase, Phase::Finished | Phase::Crashed | Phase::Caught)
    }
    pub fn height(&self) -> f64 {
        self.jump.map(height).unwrap_or(0.)
    }
    pub fn valid(&self, obstacles: &[Obstacle]) -> bool {
        self.valid_mode(obstacles, Mode::Practice)
    }
    pub fn valid_mode(&self, obstacles: &[Obstacle], mode: Mode) -> bool {
        let maximum_y = match mode {
            Mode::Practice => world::FINISH,
            Mode::FreeSki | Mode::Slalom => MAX_ENDLESS_DISTANCE,
        };
        [
            self.position.x,
            self.position.y,
            self.speed,
            self.heading,
            self.distance,
        ]
        .iter()
        .all(|n| n.is_finite())
            && self.position.x.abs() <= world::HALF_WIDTH - RADIUS + 1e-8
            && (0. ..=maximum_y).contains(&self.position.y)
            && (self.position.y..=maximum_y).contains(&self.distance)
            && (0. ..=FAST_MAX_SPEED).contains(&self.speed)
            && self.heading.abs() <= MAX_HEADING
            && self.crashes <= 3
            && self.tumble <= TUMBLE_TICKS
            && self.protection <= PROTECTION_TICKS
            && self
                .jump
                .is_none_or(|t| t.is_finite() && (0. ..JUMP_DURATION).contains(&t))
            && self.last_ramp.is_none_or(|id| {
                mode != Mode::Practice
                    || obstacles.iter().any(|o| o.id == id && o.kind == Kind::Ramp)
            })
            && (self.phase != Phase::Crashed || self.crashes == 3)
            && (self.crashes != 3 || self.phase == Phase::Crashed)
            && (match mode {
                Mode::Practice => self.phase != Phase::Finished || self.position.y == world::FINISH,
                Mode::FreeSki => self.phase != Phase::Finished,
                Mode::Slalom => true,
            })
            && (self.phase != Phase::Ready || *self == Self::default())
            && (self.tumble == 0 || (self.jump.is_none() && self.speed == 0.))
            && self.ticks < HZ as u64 * 60 * 60 * 24 * 365
    }
    pub fn step(&mut self, input: Input, obstacles: &[Obstacle]) -> Vec<Event> {
        self.step_mode(input, obstacles, Mode::Practice)
    }
    pub fn step_mode(&mut self, input: Input, obstacles: &[Obstacle], mode: Mode) -> Vec<Event> {
        self.step_outcome_mode(input, obstacles, mode, None).events
    }
    /// Step through one fixed tick and retain its physical segment. A custom
    /// finish line lets Slalom share this exact movement/contact implementation.
    pub fn step_outcome_mode(
        &mut self,
        input: Input,
        obstacles: &[Obstacle],
        mode: Mode,
        custom_finish: Option<f64>,
    ) -> StepOutcome {
        let initial = self.position;
        if self.phase != Phase::Running {
            return stationary_outcome(initial);
        }
        self.ticks += 1;
        if self.tumble > 0 {
            self.tumble -= 1;
            return stationary_outcome(initial);
        }
        let protected = self.protection > 0;
        self.protection = self.protection.saturating_sub(1);
        let desired = if input.heading.is_finite() {
            input.heading.clamp(-MAX_HEADING, MAX_HEADING)
        } else {
            0.
        };
        let turn = TURN_RATE * DT * if self.jump.is_some() { 0.4 } else { 1. };
        self.heading += (desired - self.heading).clamp(-turn, turn);
        let accel = ACCELERATION * self.heading.cos()
            - 0.05 * self.speed
            - 4.5 * self.heading.sin().abs()
            - if input.brake { 18. } else { 0. };
        let cap = if self.fast_mode {
            FAST_MAX_SPEED
        } else {
            MAX_SPEED
        };
        self.speed = if self.speed > cap {
            // Leaving fast mode should feel physical. Bleed excess speed over
            // time while retaining stronger braking/turning deceleration.
            (self.speed + (accel.min(0.) - OVERSPEED_DECELERATION) * DT).clamp(cap, FAST_MAX_SPEED)
        } else {
            (self.speed + accel * DT).clamp(0., cap)
        };
        let a = self.position;
        let b = Point {
            x: (a.x + self.heading.sin() * self.speed * DT)
                .clamp(-world::HALF_WIDTH + RADIUS, world::HALF_WIDTH - RADIUS),
            y: a.y + self.heading.cos() * self.speed * DT,
        };
        let proposed_speed = self.speed;
        let proposed_heading = self.heading;
        let finish_line = custom_finish.or((mode == Mode::Practice).then_some(world::FINISH));
        let finish = finish_line
            .filter(|y| y.is_finite() && b.y >= *y && b.y > a.y)
            .map(|y| (y - a.y) / (b.y - a.y));
        // Find ramps first, so flight height can be evaluated at every contact time.
        let ramp = if self.jump.is_none() {
            obstacles
                .iter()
                .filter(|o| o.kind == Kind::Ramp && Some(o.id) != self.last_ramp)
                .filter_map(|o| {
                    circle_interval(a, b, o.at, o.radius() + RADIUS).map(|(t, _)| (t, o.id))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
        } else {
            None
        };
        let flight_at = |t: f64| {
            self.jump
                .map(|elapsed| elapsed + t * DT)
                .or_else(|| ramp.filter(|(at, _)| t >= *at).map(|(at, _)| (t - at) * DT))
        };
        let crash = if protected {
            None
        } else {
            obstacles
                .iter()
                .filter(|o| o.kind != Kind::Ramp)
                .filter_map(|o| {
                    let (lo, hi) = circle_interval(a, b, o.at, o.radius() + RADIUS)?;
                    if flight_at(lo).map(height).unwrap_or(0.) <= o.height() {
                        return Some(lo);
                    }
                    // On the descending half of the parabola, solve exact clearance loss.
                    let landing_time =
                        JUMP_DURATION * 0.5 * (1. + (1. - o.height() / JUMP_HEIGHT).max(0.).sqrt());
                    let elapsed = flight_at(lo)?;
                    let contact = lo + (landing_time - elapsed) / DT;
                    (contact >= lo && contact <= hi).then_some(contact)
                })
                .min_by(f64::total_cmp)
        };
        let mut events = vec![];
        let stop = crash
            .filter(|t| finish.is_none_or(|f| *t <= f))
            .or(finish)
            .unwrap_or(1.);
        if let Some((at, id)) = ramp.filter(|(t, _)| *t < stop) {
            self.last_ramp = Some(id);
            self.jump = Some(-at * DT);
            events.push(Event::Jump);
        }
        self.position = a.lerp(b, stop);
        self.position.y = self.position.y.min(match mode {
            Mode::Practice => world::FINISH,
            Mode::FreeSki | Mode::Slalom => MAX_ENDLESS_DISTANCE,
        });
        self.distance = self.distance.max(self.position.y);
        if crash.is_some_and(|t| t == stop) {
            self.crashes += 1;
            self.speed = 0.;
            self.jump = None;
            self.heading = 0.;
            if self.crashes == 3 {
                self.phase = Phase::Crashed;
            } else {
                self.position = recovery(self.position, obstacles, mode);
                self.distance = self.distance.max(self.position.y);
                self.tumble = TUMBLE_TICKS;
                self.protection = PROTECTION_TICKS;
            }
            events.push(Event::Crash);
        } else if finish.is_some() {
            let y = finish_line.unwrap_or(world::FINISH);
            self.position.y = y;
            self.distance = self.distance.max(y);
            self.phase = Phase::Finished;
            self.jump = None;
            events.push(Event::Finish);
        } else if let Some(t) = self.jump {
            let t = t + DT;
            self.jump = (t < JUMP_DURATION).then_some(t);
        }
        StepOutcome {
            events,
            segment: TickSegment {
                start: a,
                proposed_end: b,
                end: a.lerp(b, stop),
                stop_fraction: stop,
                crash_fraction: crash.filter(|t| *t == stop),
                finish_fraction: finish.filter(|t| *t == stop),
                ramp_fraction: ramp.map(|(t, _)| t),
                proposed_speed,
                proposed_heading,
            },
        }
    }
}

fn stationary_outcome(at: Point) -> StepOutcome {
    StepOutcome {
        events: vec![],
        segment: TickSegment {
            start: at,
            proposed_end: at,
            end: at,
            stop_fraction: 0.,
            crash_fraction: None,
            finish_fraction: None,
            ramp_fraction: None,
            proposed_speed: 0.,
            proposed_heading: 0.,
        },
    }
}
pub fn height(elapsed: f64) -> f64 {
    let t = (elapsed / JUMP_DURATION).clamp(0., 1.);
    4. * JUMP_HEIGHT * t * (1. - t)
}
pub fn clear(p: Point, obstacles: &[Obstacle]) -> bool {
    clear_mode(p, obstacles, Mode::Practice)
}
pub fn clear_mode(p: Point, obstacles: &[Obstacle], mode: Mode) -> bool {
    p.x.abs() <= world::HALF_WIDTH - RADIUS
        && (match mode {
            Mode::Practice => (0. ..world::FINISH).contains(&p.y),
            Mode::FreeSki | Mode::Slalom => (0. ..MAX_ENDLESS_DISTANCE).contains(&p.y),
        })
        && obstacles
            .iter()
            .all(|o| (p.x - o.at.x).hypot(p.y - o.at.y) > o.radius() + RADIUS + 2.)
}
fn recovery(p: Point, obstacles: &[Obstacle], mode: Mode) -> Point {
    // Search sideways and uphill only: recovery never manufactures distance.
    for dy in [0., -3., -6., -9.] {
        for dx in [0., -4., 4., -8., 8., -12., 12., -20., 20., -30., 30.] {
            let q = Point {
                x: (p.x + dx).clamp(-world::HALF_WIDTH + RADIUS, world::HALF_WIDTH - RADIUS),
                y: (p.y + dy).max(0.),
            };
            if clear_mode(q, obstacles, mode) {
                return q;
            }
        }
    }
    // Authored terrain reserves both edges; endless terrain keeps at least one
    // fallback clear at each hazard level. Always validate before relocating.
    for x in [-36., 36.] {
        let q = Point { x, y: p.y };
        if clear_mode(q, obstacles, mode) {
            return q;
        }
    }
    // Only reachable for an invalid, non-production course: stay stopped and protected.
    p
}
