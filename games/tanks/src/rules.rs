//! Fixed-step artillery; screen coordinates have positive y downwards.

pub const RULES_VERSION: &str = "tanks-v1-preview";
pub const WIDTH: usize = 1000;
pub const HEIGHT: f64 = 600.0;
pub const FLOOR: f64 = 580.0;
pub const DT: f64 = 1.0 / 120.0;
pub const MAX_FLIGHT_TICKS: u32 = 2400;
const HALF_WIDTH: f64 = 12.0;
const BODY_HEIGHT: f64 = 14.0;
const GRAVITY: f64 = 100.0;

/// Authoritative impact snapshot. Rendering may interpolate it but never applies damage.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Impact {
    pub shooter: usize,
    pub at: Point,
    pub weapon: Weapon,
    pub terrain_before: Vec<f64>,
    pub tanks_before: [Tank; 2],
    pub blast: [u16; 2],
    pub fall: [u16; 2],
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
impl Point {
    fn lerp(self, end: Self, t: f64) -> Self {
        Self {
            x: self.x + (end.x - self.x) * t,
            y: self.y + (end.y - self.y) * t,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weapon {
    Shell,
    Heavy,
    Digger,
}
impl Weapon {
    pub fn radius(self) -> f64 {
        match self {
            Self::Shell => 45.0,
            Self::Heavy => 65.0,
            Self::Digger => 85.0,
        }
    }
    fn damage(self) -> f64 {
        match self {
            Self::Shell => 50.0,
            Self::Heavy => 70.0,
            Self::Digger => 20.0,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Tank {
    pub position: Point,
    pub health: u16,
    pub fuel: f64,
    pub angle: u16,
    pub power: u16,
    pub heavy: u8,
    pub diggers: u8,
    pub weapon: Weapon,
}
impl Tank {
    fn new(x: f64, terrain: &Terrain, angle: u16) -> Self {
        Self {
            position: Point {
                x,
                y: terrain.support(x),
            },
            health: 100,
            fuel: 60.0,
            angle,
            power: 65,
            heavy: 3,
            diggers: 2,
            weapon: Weapon::Shell,
        }
    }
    pub fn available(&self, weapon: Weapon) -> bool {
        match weapon {
            Weapon::Shell => true,
            Weapon::Heavy => self.heavy > 0,
            Weapon::Digger => self.diggers > 0,
        }
    }
    pub fn centre(&self) -> Point {
        Point {
            x: self.position.x,
            y: self.position.y - BODY_HEIGHT / 2.0,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Terrain {
    heights: Vec<f64>,
}
impl Terrain {
    pub fn heights(&self) -> &[f64] {
        &self.heights
    }
    pub fn surface(&self, x: f64) -> f64 {
        let x = x.clamp(0.0, WIDTH as f64);
        let i = x.floor() as usize;
        if i == WIDTH {
            return self.heights[i];
        }
        self.heights[i] + (self.heights[i + 1] - self.heights[i]) * (x - i as f64)
    }
    // Horizontal track rests on the highest point of its entire footprint.
    pub fn support(&self, x: f64) -> f64 {
        let left = (x - HALF_WIDTH).max(0.0);
        let right = (x + HALF_WIDTH).min(WIDTH as f64);
        let mut y = self.surface(left).min(self.surface(right));
        for i in left.ceil() as usize..=right.floor() as usize {
            y = y.min(self.heights[i]);
        }
        y
    }
    fn valid(&self) -> bool {
        self.heights.len() == WIDTH + 1
            && self
                .heights
                .iter()
                .all(|y| y.is_finite() && (280.0..=480.0).contains(y))
            && [140.0, 860.0]
                .iter()
                .all(|&x| (self.surface(x - HALF_WIDTH) - self.surface(x + HALF_WIDTH)).abs() < 1.0)
            && self.heights.windows(2).all(|p| (p[1] - p[0]).abs() <= 1.0)
    }
    fn generate(rng: &mut Rng) -> Self {
        for _ in 0..8 {
            let anchors: Vec<f64> = (0..=10)
                .map(|_| 330.0 + (rng.next() % 100) as f64)
                .collect();
            let heights = (0..=WIDTH)
                .map(|x| {
                    let i = (x / 100).min(9);
                    let t = (x - i * 100) as f64 / 100.0;
                    anchors[i] + (anchors[i + 1] - anchors[i]) * t
                })
                .collect();
            let mut terrain = Self { heights };
            for x in [140, 860] {
                // Wide flat pads, with bounded-slope shoulders.
                let y = terrain.heights[x];
                for i in x - 40..=x + 40 {
                    let weight = ((40.0 - (i as f64 - x as f64).abs()) / 20.0).clamp(0.0, 1.0);
                    terrain.heights[i] += (y - terrain.heights[i]) * weight;
                }
            }
            if terrain.valid() {
                return terrain;
            }
        }
        Self {
            heights: vec![400.0; WIDTH + 1],
        }
    }
    fn crater(&mut self, at: Point, radius: f64) {
        for (x, y) in self.heights.iter_mut().enumerate() {
            let dx = x as f64 - at.x;
            if dx.abs() > radius {
                continue;
            }
            let reach = (radius * radius - dx * dx).sqrt();
            if at.y + reach >= *y {
                *y = y.max(at.y + reach).min(FLOOR);
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Ready,
    Aiming,
    Flying,
    RoundOver { winner: Option<usize> },
    MatchOver { winner: usize },
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejected {
    WrongPhase,
    OutOfRange,
    NoAmmo,
    NoFuel,
    Edge,
    Steep,
    Occupied,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Projectile {
    pub position: Point,
    pub velocity: Point,
    pub ticks: u32,
    pub weapon: Weapon,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Game {
    terrain: Terrain,
    tanks: [Tank; 2],
    phase: Phase,
    active: usize,
    starter: usize,
    wins: [u8; 2],
    round: u64,
    wind: f64,
    rng: Rng,
    seed: u64,
    projectile: Option<Projectile>,
    traces: [Vec<Point>; 2],
}
impl Game {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng(seed);
        let starter = (rng.next() % 2) as usize;
        let terrain = Terrain::generate(&mut rng);
        let tanks = [
            Tank::new(140.0, &terrain, 45),
            Tank::new(860.0, &terrain, 135),
        ];
        let wind = (rng.next() % 41) as f64 - 20.0;
        Self {
            terrain,
            tanks,
            phase: Phase::Ready,
            active: starter,
            starter,
            wins: [0; 2],
            round: 1,
            wind,
            rng,
            seed,
            projectile: None,
            traces: [Vec::new(), Vec::new()],
        }
    }
    /// Validate untrusted saved state before allowing it into the simulation.
    pub fn valid(&self) -> bool {
        if self.terrain.heights.len() != WIDTH + 1
            || !self
                .terrain
                .heights
                .iter()
                .all(|y| y.is_finite() && (280.0..=FLOOR).contains(y))
            || self.active > 1
            || self.starter > 1
            || self.round == 0
            || !self.wind.is_finite()
            || !(-20.0..=20.0).contains(&self.wind)
            || self.wins.iter().any(|&v| v > 2)
            || self.wins.iter().filter(|&&v| v == 2).count() > 1
        {
            return false;
        }
        for tank in &self.tanks {
            if !tank.position.x.is_finite()
                || !(HALF_WIDTH..=WIDTH as f64 - HALF_WIDTH).contains(&tank.position.x)
                || !tank.position.y.is_finite()
                || tank.position.y != self.terrain.support(tank.position.x)
                || tank.health > 100
                || !tank.fuel.is_finite()
                || !(0.0..=60.0).contains(&tank.fuel)
                || !(5..=175).contains(&tank.angle)
                || !(1..=100).contains(&tank.power)
                || tank.heavy > 3
                || tank.diggers > 2
            {
                return false;
            }
        }
        if (self.tanks[0].position.x - self.tanks[1].position.x).abs() <= 2.0 * HALF_WIDTH {
            return false;
        }
        let point_ok = |p: &Point| {
            p.x.is_finite()
                && p.y.is_finite()
                && (0.0..=WIDTH as f64).contains(&p.x)
                && (-10_000.0..=HEIGHT).contains(&p.y)
        };
        if !self
            .traces
            .iter()
            .all(|t| t.len() <= MAX_FLIGHT_TICKS as usize + 1 && t.iter().all(point_ok))
        {
            return false;
        }
        let alive = [self.tanks[0].health > 0, self.tanks[1].health > 0];
        match self.phase {
            Phase::Flying => {
                if alive != [true, true] || self.wins.contains(&2) {
                    return false;
                }
                self.projectile.is_some_and(|p| {
                    point_ok(&p.position)
                        && p.velocity.x.is_finite()
                        && p.velocity.y.is_finite()
                        && p.velocity.x.abs() <= 1000.0
                        && p.velocity.y.abs() <= 2500.0
                        && p.ticks < MAX_FLIGHT_TICKS
                })
            }
            Phase::Ready | Phase::Aiming => {
                self.projectile.is_none() && alive == [true, true] && !self.wins.contains(&2)
            }
            Phase::RoundOver { winner } => {
                self.projectile.is_none()
                    && !self.wins.contains(&2)
                    && match winner {
                        None => alive == [false, false],
                        Some(i) => i < 2 && alive[i] && !alive[1 - i] && self.wins[i] > 0,
                    }
            }
            Phase::MatchOver { winner } => {
                self.projectile.is_none()
                    && winner < 2
                    && self.wins[winner] == 2
                    && alive[winner]
                    && !alive[1 - winner]
            }
        }
    }
    pub fn terrain(&self) -> &Terrain {
        &self.terrain
    }
    pub fn tanks(&self) -> &[Tank; 2] {
        &self.tanks
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn active(&self) -> usize {
        self.active
    }
    pub fn wins(&self) -> [u8; 2] {
        self.wins
    }
    pub fn round(&self) -> u64 {
        self.round
    }
    pub fn wind(&self) -> f64 {
        self.wind
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn projectile(&self) -> Option<Projectile> {
        self.projectile
    }
    pub fn traces(&self) -> &[Vec<Point>; 2] {
        &self.traces
    }
    fn require(&self, phase: Phase) -> Result<(), Rejected> {
        if self.phase == phase {
            Ok(())
        } else {
            Err(Rejected::WrongPhase)
        }
    }
    pub fn ready(&mut self) -> Result<(), Rejected> {
        self.require(Phase::Ready)?;
        self.phase = Phase::Aiming;
        Ok(())
    }
    pub fn aim(&mut self, angle: u16, power: u16) -> Result<(), Rejected> {
        self.require(Phase::Aiming)?;
        if !(5..=175).contains(&angle) || !(1..=100).contains(&power) {
            return Err(Rejected::OutOfRange);
        }
        self.tanks[self.active].angle = angle;
        self.tanks[self.active].power = power;
        Ok(())
    }
    pub fn select(&mut self, weapon: Weapon) -> Result<(), Rejected> {
        self.require(Phase::Aiming)?;
        if !self.tanks[self.active].available(weapon) {
            return Err(Rejected::NoAmmo);
        }
        self.tanks[self.active].weapon = weapon;
        Ok(())
    }
    /// Move one horizontal logical unit; callers repeat on simulation ticks.
    pub fn move_one(&mut self, right: bool) -> Result<(), Rejected> {
        self.require(Phase::Aiming)?;
        let tank = &self.tanks[self.active];
        let x = tank.position.x + if right { 1.0 } else { -1.0 };
        if !(HALF_WIDTH..=WIDTH as f64 - HALF_WIDTH).contains(&x) {
            return Err(Rejected::Edge);
        }
        if (x - self.tanks[1 - self.active].position.x).abs() <= 2.0 * HALF_WIDTH {
            return Err(Rejected::Occupied);
        }
        let y = self.terrain.support(x);
        let dy = y - tank.position.y;
        if dy.abs() > 1.0
            || (self.terrain.surface(x - HALF_WIDTH) - self.terrain.surface(x + HALF_WIDTH)).abs()
                > 12.0
        {
            return Err(Rejected::Steep);
        }
        let distance = 1.0_f64.hypot(dy);
        if tank.fuel < distance {
            return Err(Rejected::NoFuel);
        }
        let tank = &mut self.tanks[self.active];
        tank.fuel -= distance;
        tank.position = Point { x, y };
        Ok(())
    }
    pub fn fire(&mut self) -> Result<(), Rejected> {
        self.require(Phase::Aiming)?;
        let tank = &mut self.tanks[self.active];
        if !tank.available(tank.weapon) {
            return Err(Rejected::NoAmmo);
        }
        match tank.weapon {
            Weapon::Heavy => tank.heavy -= 1,
            Weapon::Digger => tank.diggers -= 1,
            Weapon::Shell => {}
        }
        let radians = (tank.angle as f64).to_radians();
        let direction = Point {
            x: radians.cos(),
            y: -radians.sin(),
        };
        let centre = tank.centre();
        let position = Point {
            x: centre.x + 20.0 * direction.x,
            y: centre.y + 20.0 * direction.y,
        };
        let speed = 40.0 + 3.6 * tank.power as f64;
        self.projectile = Some(Projectile {
            position,
            velocity: Point {
                x: direction.x * speed,
                y: direction.y * speed,
            },
            ticks: 0,
            weapon: tank.weapon,
        });
        self.traces[self.active] = vec![position];
        self.phase = Phase::Flying;
        Ok(())
    }
    /// Exactly one 120 Hz tick. Outside flight this is a no-op.
    pub fn tick(&mut self) {
        let _ = self.tick_event();
    }
    pub fn tick_event(&mut self) -> Option<Impact> {
        if self.phase != Phase::Flying {
            return None;
        }
        let mut shot = self.projectile.expect("flight owns a projectile");
        let start = shot.position;
        shot.velocity.x += self.wind * DT;
        shot.velocity.y += GRAVITY * DT;
        let end = Point {
            x: start.x + shot.velocity.x * DT,
            y: start.y + shot.velocity.y * DT,
        };
        shot.ticks += 1;
        // Sweep every crossed heightfield segment and both tank rectangles.
        let collision = self.collision(start, end);
        let exit = exit_time(start, end);
        if let Some(t) = collision.filter(|&t| exit.is_none_or(|e| t <= e)) {
            let at = start.lerp(end, t);
            self.traces[self.active].push(at);
            let impact = self.explode(at, shot.weapon);
            self.finish_shot();
            return Some(impact);
        }
        if let Some(t) = exit {
            self.traces[self.active].push(start.lerp(end, t));
            self.finish_shot();
            return None;
        }
        self.traces[self.active].push(end);
        if shot.ticks >= MAX_FLIGHT_TICKS {
            self.finish_shot();
            return None;
        }
        shot.position = end;
        self.projectile = Some(shot);
        None
    }
    fn collision(&self, start: Point, end: Point) -> Option<f64> {
        let mut first: Option<f64> = None;
        let mut keep = |t: f64| {
            first = Some(first.map_or(t, |old| old.min(t)));
        };
        for tank in &self.tanks {
            if let Some(t) = box_hit(
                start,
                end,
                Point {
                    x: tank.position.x - HALF_WIDTH,
                    y: tank.position.y - BODY_HEIGHT,
                },
                Point {
                    x: tank.position.x + HALF_WIDTH,
                    y: tank.position.y,
                },
            ) {
                keep(t);
            }
        }
        let low = start.x.min(end.x).floor().max(0.0) as usize;
        let high = (start.x.max(end.x).floor().max(0.0) as usize).min(WIDTH - 1);
        for i in low..=high {
            let dx = end.x - start.x;
            let (a, b) = if dx.abs() < 1e-12 {
                (0.0, 1.0)
            } else {
                let u = (i as f64 - start.x) / dx;
                let v = (i as f64 + 1.0 - start.x) / dx;
                (u.min(v).max(0.0), u.max(v).min(1.0))
            };
            if a > b {
                continue;
            }
            let p = start.lerp(end, a);
            let q = start.lerp(end, b);
            let gap_a = p.y - self.terrain.surface(p.x);
            let gap_b = q.y - self.terrain.surface(q.x);
            if gap_a >= 0.0 {
                keep(a);
            } else if gap_b >= 0.0 {
                keep(a + (b - a) * -gap_a / (gap_b - gap_a));
            }
        }
        first
    }
    fn explode(&mut self, at: Point, weapon: Weapon) -> Impact {
        let terrain_before = self.terrain.heights.clone();
        let mut blast = [0; 2];
        let mut fall_damage = [0; 2];
        let snapshot = self.tanks.clone();
        let damage: [u16; 2] = std::array::from_fn(|i| {
            let p = snapshot[i].centre();
            (weapon.damage()
                * (1.0 - (p.x - at.x).hypot(p.y - at.y) / weapon.radius()).clamp(0.0, 1.0))
            .floor() as u16
        });
        self.terrain.crater(at, weapon.radius());
        for i in 0..2 {
            let y = self.terrain.support(snapshot[i].position.x);
            let fall = ((y - snapshot[i].position.y - 20.0).max(0.0) / 2.0).floor() as u16;
            blast[i] = damage[i].min(snapshot[i].health);
            fall_damage[i] = fall.min(snapshot[i].health - blast[i]);
            self.tanks[i].health = snapshot[i].health.saturating_sub(damage[i] + fall);
            self.tanks[i].position.y = y;
        }
        Impact {
            shooter: self.active,
            at,
            weapon,
            terrain_before,
            tanks_before: snapshot,
            blast,
            fall: fall_damage,
        }
    }
    fn finish_shot(&mut self) {
        self.projectile = None;
        let alive = [self.tanks[0].health > 0, self.tanks[1].health > 0];
        if alive == [true, true] {
            self.active = 1 - self.active;
            self.phase = Phase::Ready;
            return;
        }
        let winner = match alive {
            [true, false] => Some(0),
            [false, true] => Some(1),
            _ => None,
        };
        if let Some(i) = winner {
            self.wins[i] += 1;
            if self.wins[i] == 2 {
                self.phase = Phase::MatchOver { winner: i };
                return;
            }
        }
        self.phase = Phase::RoundOver { winner };
    }
    pub fn next_round(&mut self) -> Result<(), Rejected> {
        if !matches!(self.phase, Phase::RoundOver { .. }) {
            return Err(Rejected::WrongPhase);
        }
        self.terrain = Terrain::generate(&mut self.rng);
        self.tanks = [
            Tank::new(140.0, &self.terrain, 45),
            Tank::new(860.0, &self.terrain, 135),
        ];
        self.wind = (self.rng.next() % 41) as f64 - 20.0;
        self.starter = 1 - self.starter;
        self.active = self.starter;
        self.round += 1;
        self.traces = [Vec::new(), Vec::new()];
        self.phase = Phase::Ready;
        Ok(())
    }
}

fn box_hit(start: Point, end: Point, min: Point, max: Point) -> Option<f64> {
    let mut near: f64 = 0.0;
    let mut far: f64 = 1.0;
    for (s, e, lo, hi) in [
        (start.x, end.x, min.x, max.x),
        (start.y, end.y, min.y, max.y),
    ] {
        let d = e - s;
        if d.abs() < 1e-12 {
            if s < lo || s > hi {
                return None;
            }
        } else {
            let a = (lo - s) / d;
            let b = (hi - s) / d;
            near = near.max(a.min(b));
            far = far.min(a.max(b));
        }
        if near > far {
            return None;
        }
    }
    Some(near)
}
fn exit_time(start: Point, end: Point) -> Option<f64> {
    let mut exit: Option<f64> = None;
    for (s, e, boundary, crossed) in [
        (start.x, end.x, 0.0, end.x < 0.0),
        (start.x, end.x, WIDTH as f64, end.x > WIDTH as f64),
        (start.y, end.y, HEIGHT, end.y > HEIGHT),
    ] {
        if crossed {
            let t = ((boundary - s) / (e - s)).clamp(0.0, 1.0);
            exit = Some(exit.map_or(t, |v| v.min(t)));
        }
    }
    exit
}

#[cfg(test)]
mod tests;
