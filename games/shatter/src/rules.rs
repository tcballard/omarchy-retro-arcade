//! Fixed 120 Hz simulation. Swept circle against faces and rounded corners;
//! moving paddle uses relative motion. Simultaneous normals are combined once.
use crate::levels::*;
use serde::{Deserialize, Serialize};
pub const DT: f64 = 1. / 120.;
pub const RADIUS: f64 = 6.;
pub const PADDLE_Y: f64 = 554.;
pub const RULES: u32 = 1;
const EPS: f64 = 1e-8;
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct V {
    pub x: f64,
    pub y: f64,
}
impl V {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    fn add(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y)
    }
    fn sub(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y)
    }
    fn mul(self, n: f64) -> Self {
        Self::new(self.x * n, self.y * n)
    }
    fn dot(self, b: Self) -> f64 {
        self.x * b.x + self.y * b.y
    }
    fn length(self) -> f64 {
        self.dot(self).sqrt()
    }
    fn unit(self) -> Self {
        self.mul(1. / self.length().max(EPS))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ball {
    pub p: V,
    pub v: V,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Brick {
    pub x: f64,
    pub y: f64,
    pub kind: u8,
    pub hp: u8,
}
impl Brick {
    pub fn alive(&self) -> bool {
        self.hp > 0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Power {
    Expand,
    Multiball,
    Laser,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Capsule {
    pub p: V,
    pub kind: Power,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Serve,
    Playing,
    Clear,
    Over,
    Complete,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub direction: i8,
    pub target: Option<f64>,
    pub launch: bool,
    pub fire: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    Paddle,
    Damage(V),
    Destroy(V),
    Pickup,
    Lost,
    Clear,
    Redirect,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Run {
    pub rules: u32,
    pub level: usize,
    pub practice: bool,
    pub phase: Phase,
    pub score: u64,
    pub lives: u8,
    pub paddle: f64,
    pub last_direction: i8,
    pub balls: Vec<Ball>,
    pub bricks: Vec<Brick>,
    pub shots: Vec<V>,
    pub capsules: Vec<Capsule>,
    pub expand: u32,
    pub laser: u32,
    pub cooldown: u32,
    pub rng: u64,
    pub no_damage: u32,
    pub tick: u64,
    pub recorded: bool,
}
impl Run {
    pub fn new(level: usize, practice: bool, seed: u64) -> Self {
        let mut run = Self {
            rules: RULES,
            level,
            practice,
            phase: Phase::Serve,
            score: 0,
            lives: 3,
            paddle: 400.,
            last_direction: 1,
            balls: vec![],
            bricks: vec![],
            shots: vec![],
            capsules: vec![],
            expand: 0,
            laser: 0,
            cooldown: 0,
            rng: seed.max(1),
            no_damage: 0,
            tick: 0,
            recorded: false,
        };
        run.load_level();
        run
    }
    fn load_level(&mut self) {
        self.bricks.clear();
        for (row, line) in LEVELS[self.level].rows.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let kind = match ch {
                    '#' => 1,
                    'A' => 2,
                    'X' => 3,
                    _ => continue,
                };
                self.bricks.push(Brick {
                    x: LEFT + col as f64 * PITCH_X,
                    y: TOP + row as f64 * PITCH_Y,
                    kind,
                    hp: if kind == 2 { 2 } else { 1 },
                });
            }
        }
        self.reset_serve();
    }
    fn reset_serve(&mut self) {
        self.phase = Phase::Serve;
        self.balls.clear();
        self.shots.clear();
        self.capsules.clear();
        self.expand = 0;
        self.laser = 0;
        self.cooldown = 0;
        self.no_damage = 0;
        self.paddle = self.paddle.clamp(50., 750.);
    }
    pub fn width(&self) -> f64 {
        if self.expand > 0 {
            150.
        } else {
            100.
        }
    }
    pub fn next(&mut self) {
        if self.phase == Phase::Clear && !self.practice && self.level < 19 {
            self.level += 1;
            self.load_level();
        }
    }
    fn random(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }
    fn damage(&mut self, index: usize, events: &mut Vec<Event>) -> bool {
        if !self.bricks[index].alive() || self.bricks[index].kind == 3 {
            return false;
        }
        self.no_damage = 0;
        self.bricks[index].hp -= 1;
        let b = self.bricks[index];
        let p = V::new(b.x + BRICK_W / 2., b.y + BRICK_H / 2.);
        if b.hp == 0 {
            self.score += if b.kind == 2 { 200 } else { 100 };
            events.push(Event::Destroy(p));
            if self.random() % 100 < 15 {
                let kind = match self.random() % 3 {
                    0 => Power::Expand,
                    1 => Power::Multiball,
                    _ => Power::Laser,
                };
                self.capsules.push(Capsule { p, kind });
            }
        } else {
            events.push(Event::Damage(p));
        }
        true
    }
    pub fn pickup(&mut self, kind: Power) {
        match kind {
            Power::Expand => {
                self.expand = 1800;
                self.paddle = self.paddle.clamp(75., 725.);
            }
            Power::Laser => self.laser = 1440,
            Power::Multiball => {
                if self.balls.len() >= 3 {
                    self.score += 250;
                    return;
                }
                if let Some(source) = self.balls.first().copied() {
                    for angle in [-0.42_f64, 0.42] {
                        if self.balls.len() >= 3 {
                            break;
                        }
                        let v = V::new(
                            source.v.x * angle.cos() - source.v.y * angle.sin(),
                            source.v.x * angle.sin() + source.v.y * angle.cos(),
                        );
                        // Balls do not collide with each other. Separated velocities, identical safe origin.
                        self.balls.push(Ball { p: source.p, v });
                    }
                }
            }
        }
    }
    pub fn step(&mut self, input: Input) -> Vec<Event> {
        let mut events = vec![];
        if !matches!(self.phase, Phase::Serve | Phase::Playing) {
            return events;
        }
        self.tick += 1;
        let old = self.paddle;
        let desired = input
            .target
            .map(|target| old + (target - old).clamp(-650. * DT, 650. * DT))
            .unwrap_or(old + f64::from(input.direction.clamp(-1, 1)) * 650. * DT);
        self.paddle = desired.clamp(self.width() / 2., 800. - self.width() / 2.);
        if (self.paddle - old).abs() > EPS {
            self.last_direction = if self.paddle > old { 1 } else { -1 };
        }
        if self.phase == Phase::Serve {
            if input.launch {
                let a = 15_f64.to_radians() * f64::from(self.last_direction);
                self.balls.push(Ball {
                    p: V::new(self.paddle, PADDLE_Y - RADIUS - 0.01),
                    v: V::new(300. * a.sin(), -300. * a.cos()),
                });
                self.phase = Phase::Playing;
            }
            return events;
        }
        self.no_damage += 1;
        self.expand = self.expand.saturating_sub(1);
        self.laser = self.laser.saturating_sub(1);
        self.cooldown = self.cooldown.saturating_sub(1);
        if input.fire && self.laser > 0 && self.cooldown == 0 {
            for dx in [-self.width() / 2. + 10., self.width() / 2. - 10.] {
                self.shots.push(V::new(self.paddle + dx, PADDLE_Y));
            }
            self.cooldown = 30;
        }
        let balls = std::mem::take(&mut self.balls);
        for mut ball in balls {
            self.fly(&mut ball, old, (self.paddle - old) / DT, &mut events);
            if ball.p.y - RADIUS <= 600. {
                self.balls.push(ball);
            }
        }
        let shots = std::mem::take(&mut self.shots);
        for mut shot in shots {
            let next = shot.y - 800. * DT;
            let hit = self
                .bricks
                .iter()
                .enumerate()
                .filter(|(_, b)| {
                    b.alive()
                        && shot.x >= b.x
                        && shot.x <= b.x + BRICK_W
                        && shot.y >= b.y
                        && next <= b.y + BRICK_H
                })
                .max_by(|a, b| a.1.y.total_cmp(&b.1.y))
                .map(|(i, _)| i);
            if let Some(i) = hit {
                self.damage(i, &mut events);
            } else {
                shot.y = next;
                if shot.y >= 0. {
                    self.shots.push(shot);
                }
            }
        }
        let capsules = std::mem::take(&mut self.capsules);
        for mut cap in capsules {
            let before = cap.p.y;
            cap.p.y += 110. * DT;
            if before <= PADDLE_Y + 14.
                && cap.p.y + 8. >= PADDLE_Y
                && (cap.p.x - self.paddle).abs() <= self.width() / 2. + 8.
            {
                self.pickup(cap.kind);
                events.push(Event::Pickup);
            } else if cap.p.y < 608. {
                self.capsules.push(cap);
            }
        }
        if self.bricks.iter().all(|b| b.kind == 3 || !b.alive()) {
            self.score += 1000;
            self.phase = if !self.practice && self.level == 19 {
                Phase::Complete
            } else {
                Phase::Clear
            };
            self.balls.clear();
            self.shots.clear();
            self.capsules.clear();
            events.push(Event::Clear);
        } else if self.balls.is_empty() {
            self.lives -= 1;
            self.reset_serve();
            if self.lives == 0 {
                self.phase = Phase::Over;
            }
            events.push(Event::Lost);
        } else if self.no_damage >= 1440 {
            self.redirect();
            events.push(Event::Redirect);
            self.no_damage = 0;
        }
        events
    }
    fn redirect(&mut self) {
        // Only target a brick with an unobstructed swept path. Stable distance/index order.
        for bi in 0..self.balls.len() {
            let ball = self.balls[bi];
            let mut candidates: Vec<_> = self
                .bricks
                .iter()
                .enumerate()
                .filter(|(_, b)| b.alive() && b.kind != 3)
                .map(|(i, b)| {
                    (
                        i,
                        V::new(b.x + BRICK_W / 2., b.y + BRICK_H / 2.).sub(ball.p),
                    )
                })
                .collect();
            candidates.sort_by(|a, b| a.1.length().total_cmp(&b.1.length()).then(a.0.cmp(&b.0)));
            for (target, delta) in candidates {
                let hit = self
                    .bricks
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| b.alive())
                    .filter_map(|(i, b)| {
                        sweep(ball.p, delta, b.x, b.y, BRICK_W, BRICK_H, RADIUS, 1.)
                            .map(|(t, _)| (i, t))
                    })
                    .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
                if hit.is_some_and(|(i, _)| i == target) {
                    self.balls[bi].v = delta.unit().mul(ball.v.length());
                    return;
                }
            }
        }
        // No direct target: rotate deterministically to leave the current corridor.
        let b = &mut self.balls[0];
        let speed = b.v.length();
        b.v = V::new(if b.p.x < 400. { 0.63 } else { -0.63 }, -0.78)
            .unit()
            .mul(speed);
    }
    fn fly(&mut self, ball: &mut Ball, old_paddle: f64, paddle_v: f64, events: &mut Vec<Event>) {
        let mut remaining = DT;
        let mut elapsed = 0.;
        let mut damaged = vec![];
        for _ in 0..16 {
            if remaining <= EPS {
                break;
            }
            let mut hits: Vec<(f64, V, Option<usize>, bool)> = vec![];
            for (t, n) in [
                (
                    if ball.v.x < 0. {
                        (RADIUS - ball.p.x) / ball.v.x
                    } else {
                        f64::INFINITY
                    },
                    V::new(1., 0.),
                ),
                (
                    if ball.v.x > 0. {
                        (800. - RADIUS - ball.p.x) / ball.v.x
                    } else {
                        f64::INFINITY
                    },
                    V::new(-1., 0.),
                ),
                (
                    if ball.v.y < 0. {
                        (RADIUS - ball.p.y) / ball.v.y
                    } else {
                        f64::INFINITY
                    },
                    V::new(0., 1.),
                ),
            ] {
                if t >= -EPS && t <= remaining {
                    hits.push((t.max(0.), n, None, false));
                }
            }
            for (i, b) in self.bricks.iter().enumerate().filter(|(_, b)| b.alive()) {
                if let Some((t, n)) = sweep(
                    ball.p, ball.v, b.x, b.y, BRICK_W, BRICK_H, RADIUS, remaining,
                ) {
                    hits.push((t, n, Some(i), false));
                }
            }
            if ball.v.y > 0. {
                let px = old_paddle + paddle_v * elapsed;
                if let Some((t, n)) = sweep(
                    ball.p,
                    V::new(ball.v.x - paddle_v, ball.v.y),
                    px - self.width() / 2.,
                    PADDLE_Y,
                    self.width(),
                    14.,
                    RADIUS,
                    remaining,
                ) {
                    if n.y < 0. {
                        hits.push((t, n, None, true));
                    }
                }
            }
            hits.sort_by(|a, b| a.0.total_cmp(&b.0));
            let Some(first) = hits.first() else {
                ball.p = ball.p.add(ball.v.mul(remaining));
                break;
            };
            let time = first.0;
            ball.p = ball.p.add(ball.v.mul(time));
            remaining -= time;
            elapsed += time;
            let mut normal = V::default();
            let mut paddle = false;
            let mut count = 0;
            for &(_, n, brick, is_paddle) in hits.iter().take_while(|h| (h.0 - time).abs() < EPS) {
                normal = normal.add(n);
                paddle |= is_paddle;
                if let Some(i) = brick {
                    if !damaged.contains(&i) && self.damage(i, events) {
                        damaged.push(i);
                        count += 1;
                    }
                }
            }
            let speed = (ball.v.length() + 5. * f64::from(count)).min(520.);
            if paddle {
                let center = old_paddle + paddle_v * elapsed;
                let offset = ((ball.p.x - center) / (self.width() / 2.)).clamp(-1., 1.);
                let sign = if offset.abs() < EPS {
                    if ball.v.x < 0. {
                        -1.
                    } else {
                        1.
                    }
                } else {
                    offset.signum()
                };
                let angle = sign * (offset.abs() * 65.).max(10.).to_radians();
                ball.v = V::new(angle.sin() * speed, -angle.cos() * speed);
                events.push(Event::Paddle);
            } else {
                let n = normal.unit();
                ball.v = ball.v.sub(n.mul(2. * ball.v.dot(n))).unit().mul(speed);
            }
            ball.p = ball.p.add(ball.v.unit().mul(0.00001));
        }
    }
    pub fn valid(&self) -> bool {
        if self.score > 10_000_000
            || self.tick > 1_000_000_000_000
            || self.rules != RULES
            || self.level >= 20
            || self.lives > 3
            || self.rng == 0
            || self.balls.len() > 3
            || self.shots.len() > 200
            || self.capsules.len() > 120
            || self.expand > 1800
            || self.laser > 1440
            || self.cooldown > 30
            || self.no_damage > 1440
            || ![-1, 1].contains(&self.last_direction)
            || !self.paddle.is_finite()
            || self.paddle < self.width() / 2.
            || self.paddle > 800. - self.width() / 2.
        {
            return false;
        }
        let original = Self::new(self.level, self.practice, 1);
        if self.bricks.len() != original.bricks.len()
            || self.bricks.iter().zip(original.bricks).any(|(b, o)| {
                b.x != o.x
                    || b.y != o.y
                    || b.kind != o.kind
                    || b.hp > o.hp
                    || (b.kind == 3 && b.hp != 1)
            })
        {
            return false;
        }
        let position = |p: V| {
            p.x.is_finite()
                && p.y.is_finite()
                && p.x >= -10.
                && p.x <= 810.
                && p.y >= -10.
                && p.y <= 610.
        };
        self.balls.iter().all(|b| {
            position(b.p) && b.v.length().is_finite() && (299.99..=520.01).contains(&b.v.length())
        }) && self.shots.iter().all(|p| position(*p))
            && self.capsules.iter().all(|c| position(c.p))
            && match self.phase {
                Phase::Playing => self.lives > 0 && !self.balls.is_empty(),
                Phase::Serve => self.lives > 0 && self.balls.is_empty(),
                Phase::Over => self.lives == 0 && self.balls.is_empty(),
                Phase::Clear | Phase::Complete => {
                    self.balls.is_empty()
                        && self.bricks.iter().all(|b| b.kind == 3 || b.hp == 0)
                        && (self.phase != Phase::Complete || self.level == 19)
                }
            }
    }
}
/// Earliest face/corner contact of a moving circle and an axis-aligned rectangle.
#[allow(clippy::too_many_arguments)]
fn sweep(p: V, v: V, x: f64, y: f64, w: f64, h: f64, r: f64, max: f64) -> Option<(f64, V)> {
    let mut best: Option<(f64, V)> = None;
    let mut add = |t: f64, n: V| {
        if t >= -EPS && t <= max + EPS && v.dot(n) < -EPS && best.is_none_or(|b| t < b.0) {
            best = Some((t.max(0.), n));
        }
    };
    if v.x.abs() > EPS {
        for (face, n) in [(x - r, V::new(-1., 0.)), (x + w + r, V::new(1., 0.))] {
            let t = (face - p.x) / v.x;
            let q = p.y + v.y * t;
            if q >= y - EPS && q <= y + h + EPS {
                add(t, n);
            }
        }
    }
    if v.y.abs() > EPS {
        for (face, n) in [(y - r, V::new(0., -1.)), (y + h + r, V::new(0., 1.))] {
            let t = (face - p.y) / v.y;
            let q = p.x + v.x * t;
            if q >= x - EPS && q <= x + w + EPS {
                add(t, n);
            }
        }
    }
    for (cx, cy, sx, sy) in [
        (x, y, -1., -1.),
        (x + w, y, 1., -1.),
        (x, y + h, -1., 1.),
        (x + w, y + h, 1., 1.),
    ] {
        let d = p.sub(V::new(cx, cy));
        let a = v.dot(v);
        let b = 2. * d.dot(v);
        let c = d.dot(d) - r * r;
        let disc = b * b - 4. * a * c;
        if a > EPS && disc >= 0. {
            let t = (-b - disc.sqrt()) / (2. * a);
            let q = d.add(v.mul(t));
            if q.x * sx >= -EPS && q.y * sy >= -EPS {
                add(t, q.unit());
            }
        }
    }
    best
}
/// Wall-clock adapter: at most 30 ticks (250 ms) per frame; stalls pause explicitly.
#[derive(Default)]
pub struct Clock {
    pub remainder: f64,
}
impl Clock {
    pub fn advance(&mut self, seconds: f64) -> Result<usize, &'static str> {
        if !seconds.is_finite() || !(0. ..=0.25).contains(&seconds) {
            self.remainder = 0.;
            return Err("Rendering stalled. Resume when ready.");
        }
        self.remainder += seconds;
        let ticks = ((self.remainder + 1e-10) / DT).floor() as usize;
        self.remainder -= ticks as f64 * DT;
        Ok(ticks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn active() -> Run {
        let mut r = Run::new(0, false, 12);
        r.step(Input {
            launch: true,
            ..Default::default()
        });
        r
    }
    #[test]
    fn swept_faces_corners_and_narrow_gaps() {
        let (t, n) = sweep(
            V::new(50., 150.),
            V::new(0., -520.),
            28.,
            100.,
            56.,
            20.,
            6.,
            0.1,
        )
        .unwrap();
        assert!((t - 24. / 520.).abs() < EPS);
        assert_eq!(n, V::new(0., 1.));
        assert!(sweep(
            V::new(20., 90.),
            V::new(300., 300.),
            28.,
            100.,
            56.,
            20.,
            6.,
            0.1
        )
        .is_some());
        assert!(sweep(
            V::new(21., 150.),
            V::new(0., -520.),
            28.,
            100.,
            56.,
            20.,
            6.,
            0.1
        )
        .is_none());
    }
    #[test]
    fn high_speed_armour_and_multiball_damage() {
        let mut r = active();
        r.bricks = vec![
            Brick {
                x: 300.,
                y: 200.,
                kind: 2,
                hp: 2,
            },
            Brick {
                x: 50.,
                y: 60.,
                kind: 1,
                hp: 1,
            },
        ];
        r.balls = vec![
            Ball {
                p: V::new(320., 228.),
                v: V::new(0., -520.)
            };
            2
        ];
        r.step(Input::default());
        assert_eq!(r.bricks[0].hp, 0);
        assert_eq!(r.score, 200);
        assert!(r.balls.iter().all(|b| b.v.y > 0.));
    }
    #[test]
    fn life_loss_once_and_clear_wins() {
        let mut r = active();
        r.balls = vec![
            Ball {
                p: V::new(100., 607.),
                v: V::new(0., 300.)
            };
            3
        ];
        r.step(Input::default());
        assert_eq!(r.lives, 2);
        assert_eq!(r.phase, Phase::Serve);
        r.step(Input::default());
        assert_eq!(r.lives, 2);
        r.phase = Phase::Playing;
        r.balls = vec![Ball {
            p: V::new(100., 607.),
            v: V::new(0., 300.),
        }];
        r.bricks = vec![Brick {
            x: 300.,
            y: 200.,
            kind: 1,
            hp: 1,
        }];
        r.shots = vec![V::new(320., 221.)];
        r.step(Input::default());
        assert_eq!(r.phase, Phase::Clear);
        assert_eq!(r.lives, 2);
        assert_eq!(r.score, 1100);
    }
    #[test]
    fn moving_paddle_sweep_and_steering() {
        let mut r = active();
        r.balls = vec![Ball {
            p: V::new(450., 547.),
            v: V::new(20., (300_f64.powi(2) - 400.).sqrt()),
        }];
        r.step(Input {
            target: Some(450.),
            ..Default::default()
        });
        assert!(r.balls[0].v.y < 0.);
        assert!(r.balls[0].v.x.abs() >= 300. * 10_f64.to_radians().sin() - 0.1);
    }
    #[test]
    fn powers_refresh_cap_and_reset() {
        let mut r = active();
        r.pickup(Power::Multiball);
        assert_eq!(r.balls.len(), 3);
        r.pickup(Power::Multiball);
        assert_eq!(r.score, 250);
        r.pickup(Power::Expand);
        r.pickup(Power::Laser);
        r.step(Input::default());
        r.pickup(Power::Expand);
        assert_eq!(r.expand, 1800);
        assert_eq!(r.laser, 1439);
        r.balls.clear();
        r.step(Input::default());
        assert_eq!((r.expand, r.laser), (0, 0));
    }
    #[test]
    fn laser_is_absorbed_by_first_brick() {
        let mut r = active();
        r.bricks = vec![
            Brick {
                x: 300.,
                y: 200.,
                kind: 1,
                hp: 1,
            },
            Brick {
                x: 300.,
                y: 230.,
                kind: 3,
                hp: 1,
            },
        ];
        r.shots = vec![V::new(320., 252.)];
        r.step(Input::default());
        assert_eq!(r.bricks[0].hp, 1);
        assert!(r.shots.is_empty());
    }
    #[test]
    fn seeded_resume_reproduces_drops() {
        let mut a = active();
        let mut b: Run = serde_json::from_slice(&serde_json::to_vec(&a).unwrap()).unwrap();
        for i in 0..a.bricks.len() {
            a.damage(i, &mut vec![]);
            b.damage(i, &mut vec![]);
        }
        assert_eq!(a, b);
        assert!(!a.capsules.is_empty());
    }
    #[test]
    fn frame_rates_are_identical_and_stalls_pause() {
        let mut results = vec![];
        for fps in [30, 60, 144] {
            let mut clock = Clock::default();
            let mut r = active();
            for _ in 0..fps * 10 {
                for _ in 0..clock.advance(1. / f64::from(fps)).unwrap() {
                    r.step(Input::default());
                }
            }
            results.push(r);
        }
        assert_eq!(results[0], results[1]);
        assert_eq!(results[1], results[2]);
        assert!(Clock::default().advance(0.251).is_err());
    }
    #[test]
    fn antistall_preserves_speed_and_points() {
        let mut r = active();
        r.no_damage = 1440;
        let speed = r.balls[0].v.length();
        r.redirect();
        assert!((r.balls[0].v.length() - speed).abs() < EPS);
        assert_eq!(r.score, 0);
    }
    #[test]
    fn layouts_have_no_sealed_pockets() {
        // Flood-fill ball centres on a 2-unit grid. All brick faces must be reachable
        // after destructibles are removed; steel is the only permanent obstruction.
        for (level, _) in LEVELS.iter().enumerate() {
            let r = Run::new(level, false, 1);
            assert!(r.valid());
            let blocked = |x: f64, y: f64| {
                r.bricks.iter().any(|b| {
                    b.kind == 3
                        && x > b.x - RADIUS
                        && x < b.x + BRICK_W + RADIUS
                        && y > b.y - RADIUS
                        && y < b.y + BRICK_H + RADIUS
                })
            };
            let mut seen = vec![false; 401 * 301];
            let mut queue = std::collections::VecDeque::from([(200_usize, 250_usize)]);
            seen[250 * 401 + 200] = true;
            while let Some((x, y)) = queue.pop_front() {
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if !(3..=397).contains(&nx) || !(3..=270).contains(&ny) {
                        continue;
                    }
                    let (nx, ny) = (nx as usize, ny as usize);
                    let i = ny * 401 + nx;
                    if !seen[i] && !blocked(nx as f64 * 2., ny as f64 * 2.) {
                        seen[i] = true;
                        queue.push_back((nx, ny));
                    }
                }
            }
            for b in r.bricks.iter().filter(|b| b.kind != 3) {
                assert!(
                    seen[((b.y + BRICK_H / 2.) / 2.) as usize * 401
                        + ((b.x + BRICK_W / 2.) / 2.) as usize],
                    "{} has sealed target",
                    LEVELS[level].name
                );
            }
        }
    }

    #[test]
    fn twenty_levels_clear_through_production_inputs() {
        for (level, _) in LEVELS.iter().enumerate() {
            let mut cleared = false;
            for seed in 1..=8 {
                let mut r = Run::new(level, true, seed);
                for _ in 0..120 * 900 {
                    // Deterministic paddle controller; never edits simulation state.
                    // Track the lowest descending ball, vary impact point slowly.
                    let ball = r
                        .balls
                        .iter()
                        .filter(|b| b.v.y > 0.)
                        .max_by(|a, b| a.p.y.total_cmp(&b.p.y))
                        .or_else(|| r.balls.first());
                    let target = ball
                        .map(|b| b.p.x + ((r.tick / 360) % 5) as f64 * 8. - 16.)
                        .unwrap_or(400.);
                    r.step(Input {
                        target: Some(target),
                        launch: true,
                        fire: true,
                        direction: 0,
                    });
                    if r.phase == Phase::Clear {
                        cleared = true;
                        break;
                    }
                    if r.phase == Phase::Over {
                        break;
                    }
                }
                if cleared {
                    eprintln!("level {} clear, seed {}", level + 1, seed);
                    break;
                }
            }
            assert!(
                cleared,
                "level {} failed deterministic paddle replay",
                level + 1
            );
        }
    }
    #[test]
    fn seam_contacts_damage_each_brick_once_without_double_rebound() {
        let mut r = active();
        r.bricks = vec![
            Brick {
                x: 300.,
                y: 200.,
                kind: 2,
                hp: 2,
            },
            Brick {
                x: 362.,
                y: 200.,
                kind: 2,
                hp: 2,
            },
        ];
        r.balls = vec![Ball {
            p: V::new(359., 228.),
            v: V::new(0., -520.),
        }];
        r.step(Input::default());
        assert_eq!(
            r.bricks.iter().map(|b| b.hp).collect::<Vec<_>>(),
            vec![1, 1]
        );
        assert!(r.balls[0].v.y > 0.);
        assert!(r.balls[0].v.x.abs() < EPS);
    }
    #[test]
    fn expansion_expiry_does_not_teleport_ball_or_false_bounce() {
        let mut r = active();
        r.pickup(Power::Expand);
        r.expand = 1;
        r.balls = vec![Ball {
            p: V::new(468., 547.),
            v: V::new(0., 300.),
        }];
        r.step(Input::default());
        assert_eq!(r.width(), 100.);
        assert_eq!(r.balls[0].p, V::new(468., 549.5));
        assert!(r.balls[0].v.y > 0.);
    }
}
