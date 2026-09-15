//! Deterministic pursuit policy for optional Free Ski chase runs.
//!
//! The actor is planned independently from rendering and wall time. `plan_tick`
//! is deliberately two phase: the session can order a catch against skier
//! contacts, then commit the actor at the same fractional point in the tick.

use crate::{
    collision::circle_interval,
    engine::{Point, DT, MAX_ENDLESS_DISTANCE},
    world::{Kind, Obstacle, HALF_WIDTH},
};
use serde::{Deserialize, Serialize};

pub const TRIGGER_DISTANCE: f64 = 1_000.;
pub const WARNING_TICKS: u32 = 180;
pub const SPAWN_RETRY_TICKS: u32 = 60;
pub const MAX_FAILED_RETRIES: u8 = 8;
pub const RADIUS: f64 = 0.95;
pub const CATCH_RADIUS: f64 = RADIUS + crate::engine::RADIUS;
pub const RECOVERY_SAFE_GAP: f64 = 14.;
pub const MAX_SPEED_PURSUER: f64 = 82.;
pub const ACCELERATION: f64 = 16.;
pub const BRAKING: f64 = 72.;
pub const TURN_RATE: f64 = 4.;
pub const TURN_DRAG: f64 = 7.;

const OBSTACLE_MARGIN: f64 = 0.35;
const MIN_LOOK_AHEAD: f64 = 24.;
const MAX_LOOK_AHEAD: f64 = 56.;
const LOOK_AHEAD_SECONDS: f64 = 0.75;
const DETOUR_TICKS: u16 = 120;
const DETOUR_MIN_TICKS: u16 = 30;
const MIN_PROBE_TRAVEL: f64 = 10.;
const SPAWN_GAPS: [f64; 4] = [48., 54., 60., 42.];
const SPAWN_OFFSETS: [f64; 5] = [0., -14., 14., -25., 25.];

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChasePhase {
    #[default]
    Dormant,
    Warning,
    Active,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Chase {
    pub phase: ChasePhase,
    pub position: Point,
    pub speed: f64,
    pub heading: f64,
    pub warning_ticks: u32,
    pub retry_ticks: u32,
    pub failed_retries: u8,
    /// A bounded physical waypoint selected when the direct route is blocked.
    /// Persisting it prevents left/right probe oscillation across save/restore.
    #[serde(default)]
    pub detour: Option<Point>,
    #[serde(default)]
    pub detour_ticks: u16,
}

impl Default for Chase {
    fn default() -> Self {
        Self {
            phase: ChasePhase::Dormant,
            position: Point::default(),
            speed: 0.,
            heading: 0.,
            warning_ticks: 0,
            retry_ticks: 0,
            failed_retries: 0,
            detour: None,
            detour_ticks: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChaseEvent {
    Warning,
    Spawned,
}

#[derive(Clone, Copy)]
pub struct TickInput<'a> {
    pub skier_start: Point,
    /// The skier's physical segment endpoint, before any recovery relocation.
    pub skier_end: Point,
    pub skier_distance: f64,
    pub skier_speed: f64,
    pub protected: bool,
    pub obstacles: &'a [Obstacle],
}

#[derive(Clone, Debug)]
pub struct TickPlan {
    next: Chase,
    pub events: Vec<ChaseEvent>,
    pub creature_start: Option<Point>,
    pub creature_end: Option<Point>,
    pub catch_fraction: Option<f64>,
}

impl TickPlan {
    /// Commit this plan at the session's earliest physical contact fraction.
    /// Timers and phase transitions belong to the tick; only actor movement is
    /// clipped when another contact stops the tick early.
    pub fn commit(mut self, fraction: f64) -> Chase {
        if let (Some(start), Some(end)) = (self.creature_start, self.creature_end) {
            self.next.position = start.lerp(end, fraction.clamp(0., 1.));
        }
        self.next
    }

    pub fn finish(self) -> Chase {
        self.next
    }
}

impl Chase {
    pub fn valid(&self) -> bool {
        let finite = [self.position.x, self.position.y, self.speed, self.heading]
            .iter()
            .all(|value| value.is_finite());
        if !finite
            || !(0. ..=MAX_SPEED_PURSUER).contains(&self.speed)
            || self.heading.abs() > std::f64::consts::PI
            || self.warning_ticks > WARNING_TICKS
            || self.retry_ticks > SPAWN_RETRY_TICKS
            || self.failed_retries > MAX_FAILED_RETRIES
            || self.detour_ticks > DETOUR_TICKS
            || self.detour.is_some_and(|point| {
                !point.x.is_finite()
                    || !point.y.is_finite()
                    || point.x.abs() > HALF_WIDTH - RADIUS + 1e-8
                    || !(0. ..=MAX_ENDLESS_DISTANCE).contains(&point.y)
            })
            || self.detour.is_none() && self.detour_ticks != 0
        {
            return false;
        }
        match self.phase {
            ChasePhase::Dormant => {
                self.position == Point::default()
                    && self.speed == 0.
                    && self.heading == 0.
                    && self.warning_ticks == 0
                    && self.retry_ticks == 0
                    && self.failed_retries == 0
                    && self.detour.is_none()
                    && self.detour_ticks == 0
            }
            ChasePhase::Warning => {
                self.position == Point::default()
                    && self.speed == 0.
                    && self.heading == 0.
                    && self.detour.is_none()
                    && self.detour_ticks == 0
            }
            ChasePhase::Active => {
                self.position.x.abs() <= HALF_WIDTH - RADIUS + 1e-8
                    && (0. ..=MAX_ENDLESS_DISTANCE).contains(&self.position.y)
                    && self.warning_ticks == 0
                    && self.retry_ticks == 0
            }
        }
    }

    pub fn plan_tick(&self, input: TickInput<'_>) -> TickPlan {
        let mut next = self.clone();
        let mut events = Vec::new();
        let mut creature_start = None;
        let mut creature_end = None;
        let mut catch_fraction = None;

        match self.phase {
            ChasePhase::Dormant => {
                if input.skier_distance >= TRIGGER_DISTANCE {
                    next.phase = ChasePhase::Warning;
                    next.warning_ticks = WARNING_TICKS;
                    events.push(ChaseEvent::Warning);
                }
            }
            ChasePhase::Warning => {
                if next.warning_ticks > 0 {
                    next.warning_ticks -= 1;
                } else if next.retry_ticks > 0 {
                    next.retry_ticks -= 1;
                } else if let Some(position) = spawn_position(input.skier_end, input.obstacles) {
                    next.phase = ChasePhase::Active;
                    next.position = position;
                    next.speed = 26.;
                    next.heading = heading_to(position, input.skier_end);
                    next.failed_retries = 0;
                    next.detour = None;
                    next.detour_ticks = 0;
                    events.push(ChaseEvent::Spawned);
                } else {
                    next.failed_retries = next
                        .failed_retries
                        .saturating_add(1)
                        .min(MAX_FAILED_RETRIES);
                    next.retry_ticks = SPAWN_RETRY_TICKS;
                }
            }
            ChasePhase::Active => {
                let start = self.position;
                let movement = move_actor(self, input);
                next.position = movement.end;
                next.speed = movement.speed;
                next.heading = movement.heading;
                next.detour = movement.detour;
                next.detour_ticks = movement.detour_ticks;
                creature_start = Some(start);
                creature_end = Some(movement.end);
                if !input.protected {
                    catch_fraction =
                        relative_catch(start, movement.end, input.skier_start, input.skier_end);
                }
            }
        }

        TickPlan {
            next,
            events,
            creature_start,
            creature_end,
            catch_fraction,
        }
    }
}

fn spawn_position(skier: Point, obstacles: &[Obstacle]) -> Option<Point> {
    for gap in SPAWN_GAPS {
        let y = (skier.y - gap).max(0.);
        for offset in SPAWN_OFFSETS {
            let candidate = Point {
                x: (skier.x + offset).clamp(-HALF_WIDTH + RADIUS, HALF_WIDTH - RADIUS),
                y,
            };
            if candidate.y < skier.y - CATCH_RADIUS
                && obstacles.iter().all(|obstacle| {
                    (candidate.x - obstacle.at.x).hypot(candidate.y - obstacle.at.y)
                        > obstacle.radius() + RADIUS + OBSTACLE_MARGIN
                })
            {
                return Some(candidate);
            }
        }
    }
    None
}

struct Movement {
    end: Point,
    speed: f64,
    heading: f64,
    detour: Option<Point>,
    detour_ticks: u16,
}

fn move_actor(chase: &Chase, input: TickInput<'_>) -> Movement {
    let target = input.skier_end;
    // Lead the physical skier segment by a short, bounded reaction horizon.
    // Chasing their old location makes a crossing approach turn behind them.
    let lead = (distance(chase.position, target) / MAX_SPEED_PURSUER).min(0.5) / DT;
    let pursuit_target = Point {
        x: (target.x + (input.skier_end.x - input.skier_start.x) * lead)
            .clamp(-HALF_WIDTH + RADIUS, HALF_WIDTH - RADIUS),
        y: (target.y + (input.skier_end.y - input.skier_start.y) * lead)
            .clamp(0., MAX_ENDLESS_DISTANCE),
    };
    let separation = distance(chase.position, target);
    let protected_close = input.protected && separation <= RECOVERY_SAFE_GAP + 8.;
    let (navigation_target, mut detour, mut detour_ticks) =
        navigation_target(chase, pursuit_target, input.obstacles);
    let desired = heading_to(chase.position, navigation_target);
    let turn_error = wrap_angle(desired - chase.heading).abs();
    let pursuit_speed = (input.skier_speed + 18.).clamp(48., MAX_SPEED_PURSUER);
    // Bound the turning circle to the remaining approach. A fixed minimum
    // corner speed made a close, stationary target physically unreachable.
    let corner_speed = if turn_error > std::f64::consts::FRAC_PI_2 {
        0.
    } else {
        TURN_RATE * distance(chase.position, navigation_target) / (2. * turn_error.sin().max(0.05))
    };
    let desired_speed = if protected_close {
        0.
    } else {
        pursuit_speed.min(corner_speed)
    };
    let speed = (approach(
        chase.speed,
        desired_speed,
        if desired_speed < chase.speed {
            BRAKING
        } else {
            ACCELERATION
        } * DT,
    ) - TURN_DRAG * turn_error.sin().abs() * DT)
        .clamp(0., MAX_SPEED_PURSUER);
    let heading = turn_toward(chase.heading, desired, TURN_RATE * DT);
    let mut end = Point {
        x: (chase.position.x + heading.sin() * speed * DT)
            .clamp(-HALF_WIDTH + RADIUS, HALF_WIDTH - RADIUS),
        y: (chase.position.y + heading.cos() * speed * DT).clamp(0., MAX_ENDLESS_DISTANCE),
    };

    let obstacle_contact = first_obstacle_hit(chase.position, end, input.obstacles);
    let recovery_contact = input
        .protected
        .then(|| circle_interval(chase.position, end, target, RECOVERY_SAFE_GAP))
        .flatten()
        .map(|(entry, _)| entry);

    // Recovery protection wins an equal or earlier contact, so later terrain
    // cannot leave the actor inside the promised physical gap.
    if recovery_contact
        .is_some_and(|safe| obstacle_contact.is_none_or(|(terrain, _)| safe <= terrain))
    {
        let entry = recovery_contact.expect("checked above");
        end = chase.position.lerp(end, (entry - 1e-6).max(0.));
        return Movement {
            end,
            speed: 0.,
            heading,
            detour,
            detour_ticks,
        };
    }

    // The steering policy normally avoids geometry. This exact sweep is the
    // safety boundary: contacts stop the actor instead of allowing phasing.
    if let Some((contact, _)) = obstacle_contact {
        end = chase.position.lerp(end, (contact - 1e-6).max(0.));
        if let Some(waypoint) = escape_waypoint(end, input.obstacles, pursuit_target, speed) {
            detour = Some(waypoint);
            detour_ticks = DETOUR_TICKS;
        }
        // Keep enough momentum to resume immediately once the physical turn
        // clears the contact. The actor still cannot cross the obstacle.
        return Movement {
            end,
            speed: (speed * 0.65).max(18.),
            heading,
            detour,
            detour_ticks,
        };
    }
    if detour.is_some_and(|waypoint| distance(end, waypoint) <= (speed * DT + 1.5).max(2.)) {
        detour = None;
        detour_ticks = 0;
    }
    Movement {
        end,
        speed,
        heading,
        detour,
        detour_ticks,
    }
}

fn navigation_target(
    chase: &Chase,
    target: Point,
    obstacles: &[Obstacle],
) -> (Point, Option<Point>, u16) {
    // Intercept a nearby skier on a clear segment. Probing a full navigation
    // horizon beyond them could invent an edge/terrain detour and run past
    // the catch, even though the path to the skier was unobstructed.
    if distance(chase.position, target) <= look_ahead(chase.speed)
        && first_obstacle_contact(chase.position, target, obstacles).is_none()
    {
        return (target, None, 0);
    }
    if let Some(waypoint) = chase.detour.filter(|_| chase.detour_ticks > 0) {
        let old_enough = chase.detour_ticks <= DETOUR_TICKS - DETOUR_MIN_TICKS;
        let direct_clear = probe_point(
            chase.position,
            heading_to(chase.position, target),
            look_ahead(chase.speed),
        )
        .is_some_and(|probe| first_obstacle_contact(chase.position, probe, obstacles).is_none());
        if !old_enough || !direct_clear {
            return (waypoint, Some(waypoint), chase.detour_ticks - 1);
        }
    }

    let from = chase.position;
    let desired = heading_to(from, target);
    let probe_length = look_ahead(chase.speed);
    // The wider candidates matter after an exact contact. Every heading within
    // 90 degrees of an obstacle can still point into its collision circle, so
    // the old five probes could all fail and repeatedly choose `desired` while
    // the creature stood against the same obstacle. The escape probes let it
    // turn physically away from the contact before resuming pursuit.
    let mut best_blocked = None;
    for offset in [
        0., -0.35, 0.35, -0.7, 0.7, -1.05, 1.05, -1.4, 1.4, -1.75, 1.75, -2.35, 2.35,
    ] {
        let heading = wrap_angle(desired + offset);
        let Some(probe) = probe_point(from, heading, probe_length) else {
            continue;
        };
        let contact = first_obstacle_contact(from, probe, obstacles);
        if contact.is_none() {
            if offset == 0. {
                return (target, None, 0);
            }
            return (probe, Some(probe), DETOUR_TICKS);
        }
        let clear_distance = distance(from, probe) * contact.unwrap_or_default();
        if best_blocked.is_none_or(|(best, _)| clear_distance > best) {
            best_blocked = Some((clear_distance, probe));
        }
    }
    // Dense clusters can block every complete probe. Commit to the sampled
    // direction with the most clear travel rather than repeatedly selecting
    // the known-blocked direct route.
    let waypoint = best_blocked.map_or(target, |(_, point)| point);
    (waypoint, Some(waypoint), DETOUR_TICKS)
}

fn look_ahead(speed: f64) -> f64 {
    (speed * LOOK_AHEAD_SECONDS).clamp(MIN_LOOK_AHEAD, MAX_LOOK_AHEAD)
}

fn probe_point(from: Point, heading: f64, distance: f64) -> Option<Point> {
    let side = heading.sin();
    let downhill = heading.cos();
    let edge = HALF_WIDTH - RADIUS;
    let mut travel = distance;
    if side > 0. {
        travel = travel.min((edge - from.x) / side);
    } else if side < 0. {
        travel = travel.min((-edge - from.x) / side);
    }
    if downhill > 0. {
        travel = travel.min((MAX_ENDLESS_DISTANCE - from.y) / downhill);
    } else if downhill < 0. {
        travel = travel.min((0. - from.y) / downhill);
    }
    if !travel.is_finite() || travel < MIN_PROBE_TRAVEL {
        return None;
    }
    Some(Point {
        x: from.x + side * travel,
        y: from.y + downhill * travel,
    })
}

fn escape_waypoint(
    from: Point,
    obstacles: &[Obstacle],
    target: Point,
    speed: f64,
) -> Option<Point> {
    let look_ahead = look_ahead(speed);
    let mut best = None;
    let touching: Vec<_> = obstacles
        .iter()
        .filter(|obstacle| obstacle.kind != Kind::Ramp)
        .filter(|obstacle| distance(from, obstacle.at) <= obstacle.radius() + RADIUS + 1e-4)
        .collect();
    for index in 0..64 {
        let heading = -std::f64::consts::PI + index as f64 * std::f64::consts::TAU / 64.;
        let Some(probe) = probe_point(from, heading, look_ahead) else {
            continue;
        };
        let movement = Point {
            x: probe.x - from.x,
            y: probe.y - from.y,
        };
        if !touching.iter().all(|obstacle| {
            movement.x * (from.x - obstacle.at.x) + movement.y * (from.y - obstacle.at.y) > 1e-6
        }) {
            continue;
        }
        let travel = distance(from, probe);
        let target_progress = distance(from, target) - distance(probe, target);
        let score = travel + target_progress * 0.25;
        if best.is_none_or(|(best_score, _)| score > best_score) {
            best = Some((score, probe));
        }
    }
    best.map(|(_, probe)| probe)
}

fn first_obstacle_hit(a: Point, b: Point, obstacles: &[Obstacle]) -> Option<(f64, &Obstacle)> {
    obstacles
        .iter()
        .filter(|obstacle| obstacle.kind != Kind::Ramp)
        .filter_map(|obstacle| {
            let radius = obstacle.radius() + RADIUS;
            circle_interval(a, b, obstacle.at, radius).and_then(|(entry, _)| {
                let start_x = a.x - obstacle.at.x;
                let start_y = a.y - obstacle.at.y;
                let movement_x = b.x - a.x;
                let movement_y = b.y - a.y;
                let at_boundary = start_x * start_x + start_y * start_y >= radius * radius - 1e-6;
                let moving_outward = start_x * movement_x + start_y * movement_y >= 0.;
                (!(entry <= 1e-8 && at_boundary && moving_outward)).then_some((entry, obstacle))
            })
        })
        .min_by(|(a, _), (b, _)| a.total_cmp(b))
}

fn first_obstacle_contact(a: Point, b: Point, obstacles: &[Obstacle]) -> Option<f64> {
    first_obstacle_hit(a, b, obstacles).map(|(entry, _)| entry)
}

fn relative_catch(
    creature_start: Point,
    creature_end: Point,
    skier_start: Point,
    skier_end: Point,
) -> Option<f64> {
    let relative_start = Point {
        x: creature_start.x - skier_start.x,
        y: creature_start.y - skier_start.y,
    };
    let relative_end = Point {
        x: creature_end.x - skier_end.x,
        y: creature_end.y - skier_end.y,
    };
    circle_interval(relative_start, relative_end, Point::default(), CATCH_RADIUS)
        .map(|(entry, _)| entry)
}

fn heading_to(from: Point, to: Point) -> f64 {
    (to.x - from.x).atan2(to.y - from.y)
}

fn distance(a: Point, b: Point) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

fn approach(current: f64, target: f64, amount: f64) -> f64 {
    current + (target - current).clamp(-amount, amount)
}

fn turn_toward(current: f64, target: f64, amount: f64) -> f64 {
    wrap_angle(current + wrap_angle(target - current).clamp(-amount, amount))
}

fn wrap_angle(angle: f64) -> f64 {
    let two_pi = std::f64::consts::TAU;
    (angle + std::f64::consts::PI).rem_euclid(two_pi) - std::f64::consts::PI
}
