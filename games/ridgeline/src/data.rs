//! Authored campaign data: towers, enemies, maps and wave tables.
//!
//! All quantities are integers. One cell is 1,000 world units and the simulation
//! runs at 60 ticks per second. Maps are 20 × 12 cells so the whole battlefield
//! fits in the window without camera movement.
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub const COLS: i32 = 20;
pub const ROWS: i32 = 12;
pub const CELL: i32 = 1000;
pub const TICKS_PER_SECOND: u32 = 60;
pub const WAVES: usize = 20;
/// Bumped whenever authored geometry or wave tables change meaning.
pub const LAYOUT: u32 = 1;
/// Late-wave pressure: wave index squared divided by this adds health percentage.
pub const LATE: u32 = 3;
/// Pause between successive groups in one wave.
pub const GROUP_GAP: u32 = 75;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TowerKind {
    Cannon,
    Mortar,
    Flak,
    Cryo,
}
impl TowerKind {
    pub const ALL: [Self; 4] = [Self::Cannon, Self::Mortar, Self::Flak, Self::Cryo];
    pub fn name(self) -> &'static str {
        match self {
            Self::Cannon => "Cannon",
            Self::Mortar => "Mortar",
            Self::Flak => "Flak",
            Self::Cryo => "Cryo",
        }
    }
    pub fn role(self) -> &'static str {
        match self {
            Self::Cannon => "Affordable direct damage",
            Self::Mortar => "Slow splash; cannot fire close",
            Self::Flak => "Fast anti-aircraft bursts",
            Self::Cryo => "Light damage and slowdown",
        }
    }
    pub fn targets(self) -> Targets {
        if self == Self::Flak {
            Targets::Air
        } else {
            Targets::Ground
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Targets {
    Ground,
    Air,
}
impl Targets {
    pub fn name(self) -> &'static str {
        match self {
            Self::Ground => "Ground",
            Self::Air => "Air",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TowerStats {
    /// Build price for tier 0; upgrade price into this tier otherwise.
    pub cost: u32,
    pub damage: u32,
    pub range: i32,
    pub min_range: i32,
    pub reload: u32,
    /// World units per tick.
    pub shot_speed: i32,
    pub splash: i32,
    /// Percentage speed reduction; 0 for no slow.
    pub slow: u32,
    pub slow_ticks: u32,
}
/// Compact table-row constructor; argument order matches `TowerStats`.
#[allow(clippy::too_many_arguments)]
const fn t(
    cost: u32,
    damage: u32,
    range: i32,
    min_range: i32,
    reload: u32,
    shot_speed: i32,
    splash: i32,
    slow: u32,
    slow_ticks: u32,
) -> TowerStats {
    TowerStats {
        cost,
        damage,
        range,
        min_range,
        reload,
        shot_speed,
        splash,
        slow,
        slow_ticks,
    }
}
pub const TIERS: usize = 3;
const TOWERS: [[TowerStats; TIERS]; 4] = [
    // Cannon
    [
        t(50, 10, 2600, 0, 36, 320, 0, 0, 0),
        t(45, 17, 2800, 0, 32, 340, 0, 0, 0),
        t(80, 30, 3100, 0, 28, 360, 0, 0, 0),
    ],
    // Mortar
    [
        t(90, 16, 4200, 1500, 110, 110, 900, 0, 0),
        t(80, 27, 4500, 1500, 100, 115, 1000, 0, 0),
        t(130, 44, 4900, 1500, 90, 120, 1150, 0, 0),
    ],
    // Flak
    [
        t(70, 7, 3200, 0, 14, 450, 0, 0, 0),
        t(60, 12, 3500, 0, 12, 470, 0, 0, 0),
        t(100, 20, 3800, 0, 10, 490, 0, 0, 0),
    ],
    // Cryo
    [
        t(60, 3, 2300, 0, 45, 260, 0, 30, 90),
        t(55, 5, 2500, 0, 42, 270, 0, 40, 110),
        t(85, 8, 2700, 0, 38, 280, 0, 50, 130),
    ],
];
pub fn tower(kind: TowerKind, tier: u8) -> TowerStats {
    TOWERS[kind.index()][usize::from(tier).min(TIERS - 1)]
}
/// Total credits invested to reach `tier`.
pub fn invested(kind: TowerKind, tier: u8) -> u32 {
    (0..=usize::from(tier).min(TIERS - 1))
        .map(|i| TOWERS[kind.index()][i].cost)
        .sum()
}
/// Sell refund: floor(70% of the credits spent on the tower).
pub fn refund(spent: u32) -> u32 {
    spent * 7 / 10
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnemyKind {
    Crawler,
    Scout,
    Plated,
    Glider,
    Hauler,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnemyStats {
    pub hp: u32,
    /// World units per tick.
    pub speed: u32,
    pub armour: u32,
    pub bounty: u32,
    pub damage: u32,
    pub air: bool,
}
impl EnemyKind {
    pub const ALL: [Self; 5] = [
        Self::Crawler,
        Self::Scout,
        Self::Plated,
        Self::Glider,
        Self::Hauler,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Crawler => "Crawler",
            Self::Scout => "Scout",
            Self::Plated => "Plated",
            Self::Glider => "Glider",
            Self::Hauler => "Hauler",
        }
    }
    pub fn role(self) -> &'static str {
        match self {
            Self::Crawler => "Basic ground unit",
            Self::Scout => "Fast, fragile ground unit",
            Self::Plated => "Armoured ground unit",
            Self::Glider => "Aircraft; only Flak can hit it",
            Self::Hauler => "Slow, heavy ground unit",
        }
    }
    pub fn stats(self) -> EnemyStats {
        let (hp, speed, armour, bounty, damage, air) = match self {
            Self::Crawler => (40, 18, 0, 4, 1, false),
            Self::Scout => (24, 34, 0, 3, 1, false),
            Self::Plated => (70, 14, 4, 7, 2, false),
            Self::Glider => (36, 22, 0, 5, 1, true),
            Self::Hauler => (260, 9, 2, 20, 5, false),
        };
        EnemyStats {
            hp,
            speed,
            armour,
            bounty,
            damage,
            air,
        }
    }
    /// Frames between successive spawns of this kind within one group.
    pub fn spacing(self) -> u32 {
        match self {
            Self::Crawler => 36,
            Self::Scout => 22,
            Self::Plated => 48,
            Self::Glider => 40,
            Self::Hauler => 110,
        }
    }
    fn code(c: char) -> Option<Self> {
        Some(match c {
            'c' => Self::Crawler,
            's' => Self::Scout,
            'p' => Self::Plated,
            'g' => Self::Glider,
            'h' => Self::Hauler,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Difficulty {
    Normal,
    Hard,
}
impl Difficulty {
    pub const ALL: [Self; 2] = [Self::Normal, Self::Hard];
    pub fn name(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Hard => "Hard",
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
    /// Hard's documented table: 30% more units per group (rounded up), enemy
    /// health +15 percentage points on wave 1 rising by one per wave, 90%
    /// starting credits and a smaller wave-clear award. Tower accuracy and damage never change.
    pub fn count(self, n: u32) -> u32 {
        match self {
            Self::Normal => n,
            Self::Hard => n + (n * 3).div_ceil(10),
        }
    }
    /// Extra percentage points of enemy health for wave index `wave`.
    pub fn health_bonus(self, wave: usize) -> u32 {
        match self {
            Self::Normal => 0,
            Self::Hard => 15 + wave as u32,
        }
    }
    pub fn credits(self, normal: u32) -> u32 {
        match self {
            Self::Normal => normal,
            Self::Hard => normal * 9 / 10,
        }
    }
    pub fn wave_award(self) -> u32 {
        match self {
            Self::Normal => 25,
            Self::Hard => 20,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct P {
    pub x: i32,
    pub y: i32,
}
impl P {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub fn cell(x: i32, y: i32) -> Self {
        Self::new(x * CELL + CELL / 2, y * CELL + CELL / 2)
    }
    pub fn dist2(self, o: Self) -> i64 {
        let dx = i64::from(self.x - o.x);
        let dy = i64::from(self.y - o.y);
        dx * dx + dy * dy
    }
    pub fn dist(self, o: Self) -> i32 {
        isqrt(self.dist2(o)) as i32
    }
}
/// Exact integer square root (floor).
pub fn isqrt(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut x = (n as f64).sqrt() as i64;
    while x * x > n {
        x -= 1;
    }
    while (x + 1) * (x + 1) <= n {
        x += 1;
    }
    x
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    pub air: bool,
    pub points: Vec<P>,
    /// Cumulative length at each waypoint.
    pub marks: Vec<u32>,
}
impl Route {
    fn new(air: bool, cells: &[(i8, i8)]) -> Self {
        let points: Vec<P> = cells
            .iter()
            .map(|&(x, y)| P::cell(i32::from(x), i32::from(y)))
            .collect();
        let mut marks = vec![0];
        for w in points.windows(2) {
            marks.push(marks.last().unwrap() + w[0].dist(w[1]) as u32);
        }
        Self { air, points, marks }
    }
    pub fn length(&self) -> u32 {
        *self.marks.last().unwrap_or(&0)
    }
    /// Deterministic integer interpolation along the polyline.
    pub fn at(&self, d: u32) -> P {
        let d = d.min(self.length());
        let i = self
            .marks
            .partition_point(|m| *m <= d)
            .clamp(1, self.points.len() - 1);
        let (a, b) = (self.points[i - 1], self.points[i]);
        let span = i64::from((self.marks[i] - self.marks[i - 1]).max(1));
        let into = i64::from(d - self.marks[i - 1]);
        P::new(
            a.x + (i64::from(b.x - a.x) * into / span) as i32,
            a.y + (i64::from(b.y - a.y) * into / span) as i32,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Group {
    pub kind: EnemyKind,
    pub count: u32,
    /// Index into `Map::routes` (ground or air, as appropriate for the kind).
    pub route: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spawn {
    pub tick: u32,
    pub kind: EnemyKind,
    pub route: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wave {
    pub groups: Vec<Group>,
    pub spawns: Vec<Spawn>,
    /// Percentage applied to base enemy health.
    pub health: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    Build,
    Rock,
    Route,
}

pub struct Map {
    pub name: &'static str,
    pub lesson: &'static str,
    pub terrain: Vec<Terrain>,
    pub routes: Vec<Route>,
    pub credits: [u32; 2],
    pub waves: [Vec<Wave>; 2],
}
impl Map {
    pub fn terrain(&self, x: i32, y: i32) -> Terrain {
        if !(0..COLS).contains(&x) || !(0..ROWS).contains(&y) {
            return Terrain::Rock;
        }
        self.terrain[(y * COLS + x) as usize]
    }
    pub fn wave(&self, difficulty: Difficulty, wave: usize) -> &Wave {
        &self.waves[difficulty.index()][wave.min(WAVES - 1)]
    }
    pub fn ground_routes(&self) -> impl Iterator<Item = (usize, &Route)> {
        self.routes.iter().enumerate().filter(|(_, r)| !r.air)
    }
    pub fn air_routes(&self) -> impl Iterator<Item = (usize, &Route)> {
        self.routes.iter().enumerate().filter(|(_, r)| r.air)
    }
}

struct MapDef {
    name: &'static str,
    lesson: &'static str,
    terrain: [&'static str; 12],
    ground: &'static [&'static [(i8, i8)]],
    air: &'static [&'static [(i8, i8)]],
    credits: u32,
    /// Base health percentage for wave 1 and the per-wave increase.
    health: (u32, u32),
    waves: [&'static str; WAVES],
}

const DEFS: [MapDef; 10] = [
    MapDef {
        name: "First Terrace",
        lesson: "One winding road. Cannons near the bends cover the most ground.",
        terrain: [
            "..........#.........",
            "....................",
            "..............##....",
            "..#.................",
            "....................",
            ".........#..........",
            "..................#.",
            "....................",
            "...#................",
            "...............#....",
            "........##..........",
            "#...................",
        ],
        ground: &[&[(0, 2), (6, 2), (6, 8), (13, 8), (13, 3), (19, 3)]],
        air: &[],
        credits: 200,
        health: (100, 10),
        waves: [
            "c6", "c8", "c10", "c12", "c8 c8", "c14", "c10 s4", "c16", "c12 s6", "c20", "c14 s8",
            "c18 s6", "c20 s8", "c16 c16", "s14 c14", "c24 s10", "c20 s16", "c28 s12", "c24 s20",
            "c30 s20",
        ],
    },
    MapDef {
        name: "Dry Wash",
        lesson: "Scouts are quick but fragile. Cover the long straights early.",
        terrain: [
            "....................",
            "....................",
            "..#..............#..",
            "........##..........",
            "..................#.",
            "....................",
            "...........#........",
            "#...................",
            "....................",
            "....................",
            "......#.......##....",
            "..................#.",
        ],
        ground: &[&[(0, 1), (16, 1), (16, 5), (3, 5), (3, 9), (19, 9)]],
        air: &[],
        credits: 200,
        health: (100, 11),
        waves: [
            "c6", "s6", "c8 s4", "s10", "c10 s6", "s14", "c12 s8", "s16", "c14 s10", "s20",
            "c16 s12", "s24", "c18 s14", "s28", "c20 s16", "s30 c10", "c22 s20", "s34", "c24 s24",
            "s40 c20",
        ],
    },
    MapDef {
        name: "Hawk Ledge",
        lesson: "Gliders ignore the road and only Flak can reach them.",
        terrain: [
            "......#.............",
            "....................",
            "..#.................",
            "...........#........",
            "....................",
            ".................#..",
            "....................",
            "....#...............",
            "..........#.........",
            "....................",
            "..#.............#...",
            "....................",
        ],
        ground: &[&[(0, 6), (8, 6), (8, 2), (14, 2), (14, 9), (19, 9)]],
        air: &[&[(0, 1), (19, 10)]],
        credits: 220,
        health: (105, 12),
        waves: [
            "c8",
            "g4",
            "c8 g4",
            "g8",
            "c10 g6",
            "s8 g8",
            "c12 g8",
            "g12",
            "c14 s6 g8",
            "g16",
            "c16 g10",
            "s12 g12",
            "c18 g12",
            "g20",
            "c16 s10 g12",
            "g22 c10",
            "c20 g16",
            "s16 g18",
            "c22 g20",
            "g26 c20 s12",
        ],
    },
    MapDef {
        name: "Iron Gate",
        lesson: "Armour subtracts from every hit. Heavier shots matter here.",
        terrain: [
            "#...................",
            "............#.......",
            "....................",
            "...................#",
            "........#...........",
            ".................#..",
            "..#.................",
            "....................",
            "............#.......",
            "....................",
            "......#.............",
            "..........##......#.",
        ],
        ground: &[&[
            (0, 9),
            (4, 9),
            (4, 2),
            (10, 2),
            (10, 9),
            (15, 9),
            (15, 4),
            (19, 4),
        ]],
        air: &[],
        credits: 230,
        health: (105, 12),
        waves: [
            "c8",
            "p3",
            "c8 p3",
            "p6",
            "c10 p5",
            "s8 p6",
            "c12 p7",
            "p10",
            "c12 p8 s6",
            "p14",
            "c14 p10",
            "p12 s10",
            "c16 p12",
            "p18",
            "c16 p14 s8",
            "p20 c10",
            "c18 p16",
            "p22 s12",
            "c20 p20",
            "p26 c20 s10",
        ],
    },
    MapDef {
        name: "Twin Draws",
        lesson: "Two roads meet at the ford. Mortars punish crowds.",
        terrain: [
            "....................",
            "...#............#...",
            "....................",
            "..............#.....",
            "#...................",
            "....................",
            "....................",
            "....#...............",
            "...............#....",
            "....................",
            "....................",
            "..........#.........",
        ],
        ground: &[
            &[(0, 2), (9, 2), (9, 6), (19, 6)],
            &[(0, 10), (9, 10), (9, 6), (19, 6)],
        ],
        air: &[],
        credits: 240,
        health: (110, 13),
        waves: [
            "c10",
            "c12",
            "c14 s4",
            "c16",
            "h1 c10",
            "c20",
            "c16 p4",
            "h2 c12",
            "c24 s8",
            "h3 c16",
            "c28",
            "p8 c20",
            "h4 c20",
            "c32 s10",
            "h4 p8 c16",
            "c36",
            "h5 c24",
            "c30 p12",
            "h6 c28 s10",
            "h8 c36",
        ],
    },
    MapDef {
        name: "Mesa Crossing",
        lesson: "Ground and air arrive together. Balance the budget.",
        terrain: [
            "...#................",
            "..............#.....",
            "....................",
            "..................#.",
            "....................",
            ".......#............",
            "................#...",
            "#...................",
            "....................",
            "..........#.........",
            "....................",
            "...#......#.........",
        ],
        ground: &[&[(10, 0), (10, 4), (3, 4), (3, 8), (16, 8), (16, 11)]],
        air: &[&[(0, 5), (19, 5)]],
        credits: 240,
        health: (110, 14),
        waves: [
            "c10",
            "g6",
            "c10 s6",
            "p4 g6",
            "c14 g8",
            "h1 c12",
            "s12 g8",
            "p8 c12",
            "g14 c10",
            "h2 s10 g8",
            "c20 p6",
            "g16 s10",
            "h3 c16",
            "p12 g12",
            "c24 s14",
            "h4 g14",
            "p14 c20",
            "g20 s16",
            "h5 p12 c16",
            "h6 g20 c24 s12",
        ],
    },
    MapDef {
        name: "Split Rock",
        lesson: "The road forks. Every group takes the next branch in turn.",
        terrain: [
            "........#...........",
            "....................",
            "..#.................",
            "...........#........",
            ".................#..",
            "....................",
            "..........##........",
            "....................",
            "..#.............#...",
            "....................",
            "....................",
            "........#...........",
        ],
        ground: &[
            &[(0, 5), (6, 5), (6, 1), (19, 1)],
            &[(0, 5), (6, 5), (6, 10), (19, 10)],
        ],
        air: &[],
        credits: 260,
        health: (115, 14),
        waves: [
            "c10",
            "s8",
            "c14",
            "p4 c8",
            "s14",
            "c20",
            "p8 s8",
            "h2 c10",
            "c24 s10",
            "p12 c12",
            "h3 s12",
            "c28",
            "p14 c14",
            "h4 s16",
            "c30 p10",
            "s24 c16",
            "h5 p12",
            "c34 s16",
            "h6 p16 c16",
            "h8 c30 s20 p12",
        ],
    },
    MapDef {
        name: "Windward",
        lesson: "Two flight lanes. Flak placed where they cross does double duty.",
        terrain: [
            "....................",
            "......#.............",
            "..#.................",
            "....................",
            "...............#....",
            "......#.............",
            "....................",
            ".................#..",
            "....#...............",
            "....................",
            "..........#.........",
            "....................",
        ],
        ground: &[&[(0, 3), (12, 3), (12, 8), (19, 8)]],
        air: &[&[(0, 0), (19, 6)], &[(0, 11), (19, 2)]],
        credits: 260,
        health: (115, 15),
        waves: [
            "g6", "c10", "g10", "c12 g6", "g14", "s10 g10", "g18", "p6 g12", "g22", "c16 g16",
            "g26", "h2 g18", "g30", "s16 g20", "g34", "p10 g24", "g38", "h3 g28", "g42 c16",
            "g48 s20",
        ],
    },
    MapDef {
        name: "Long Drop",
        lesson: "A long switchback gives time to wear down Haulers.",
        terrain: [
            "....................",
            "....................",
            "..........#.........",
            "...................#",
            "....................",
            "#.........#.........",
            "...................#",
            "....................",
            "#.........#.........",
            "....................",
            "....................",
            "...#...........#....",
        ],
        ground: &[&[
            (0, 1),
            (17, 1),
            (17, 4),
            (2, 4),
            (2, 7),
            (17, 7),
            (17, 10),
            (0, 10),
        ]],
        air: &[&[(19, 0), (0, 11)]],
        credits: 260,
        health: (120, 15),
        waves: [
            "c12",
            "h1",
            "c14 s6",
            "h2",
            "p6 c12",
            "h2 c14",
            "s16",
            "h3 p4",
            "c24 g8",
            "h4 c12",
            "p12 s10",
            "h5",
            "c28 g12",
            "h5 p8",
            "s24 c16",
            "h6 g12",
            "p16 c20",
            "h8",
            "h6 p14 s16",
            "h10 c30 g16",
        ],
    },
    MapDef {
        name: "Last Pass",
        lesson: "Everything at once: two roads, two flight lanes, every enemy.",
        terrain: [
            "..........#.........",
            "...................#",
            "..#.................",
            "...............#....",
            "....................",
            "#...................",
            "....................",
            "....................",
            "..#..........#......",
            "..................#.",
            "....................",
            "....................",
        ],
        ground: &[
            &[(0, 1), (7, 1), (7, 6), (19, 6)],
            &[(0, 10), (12, 10), (12, 6), (19, 6)],
        ],
        air: &[&[(0, 0), (19, 6)], &[(10, 11), (19, 6)]],
        credits: 300,
        health: (100, 17),
        waves: [
            "c8",
            "s8 g4",
            "p3 c8",
            "h1 g6",
            "c16 s8",
            "p8 g8",
            "h2 c14",
            "g16 s12",
            "p14 c16",
            "h4 g14",
            "c28 s14",
            "p16 g14",
            "h5 c20",
            "g22 s18",
            "p18 c20",
            "h6 g18",
            "c32 s20 p10",
            "h7 g22",
            "p20 g20 c24",
            "h10 p16 g24 s20",
        ],
    },
];

fn build(def: &MapDef) -> Result<Map, String> {
    let mut terrain = Vec::with_capacity((COLS * ROWS) as usize);
    if def.terrain.len() != ROWS as usize {
        return Err(format!("{}: wrong row count", def.name));
    }
    for row in def.terrain {
        if row.len() != COLS as usize {
            return Err(format!("{}: row width {}", def.name, row.len()));
        }
        for c in row.chars() {
            terrain.push(match c {
                '.' => Terrain::Build,
                '#' => Terrain::Rock,
                _ => return Err(format!("{}: unknown terrain {c}", def.name)),
            });
        }
    }
    let mut routes = vec![];
    for cells in def.ground {
        if cells.len() < 2 {
            return Err(format!("{}: short route", def.name));
        }
        for w in cells.windows(2) {
            let ((ax, ay), (bx, by)) = (w[0], w[1]);
            if (ax != bx) == (ay != by) {
                return Err(format!("{}: ground segments must be orthogonal", def.name));
            }
            let (dx, dy) = ((bx - ax).signum(), (by - ay).signum());
            let (mut x, mut y) = (ax, ay);
            loop {
                let (cx, cy) = (i32::from(x), i32::from(y));
                if !(0..COLS).contains(&cx) || !(0..ROWS).contains(&cy) {
                    return Err(format!("{}: route leaves the map", def.name));
                }
                let cell = &mut terrain[(cy * COLS + cx) as usize];
                if *cell == Terrain::Rock {
                    return Err(format!("{}: route crosses rock at {x},{y}", def.name));
                }
                *cell = Terrain::Route;
                if (x, y) == (bx, by) {
                    break;
                }
                x += dx;
                y += dy;
            }
        }
        routes.push(Route::new(false, cells));
    }
    for cells in def.air {
        routes.push(Route::new(true, cells));
    }
    for r in &routes {
        for end in [r.points[0], *r.points.last().unwrap()] {
            let (x, y) = (end.x / CELL, end.y / CELL);
            if !(x == 0 || y == 0 || x == COLS - 1 || y == ROWS - 1) {
                return Err(format!("{}: route ends must touch the border", def.name));
            }
        }
    }
    let ground = routes.iter().filter(|r| !r.air).count();
    let air = routes.len() - ground;
    let mut waves = [vec![], vec![]];
    for difficulty in Difficulty::ALL {
        let (mut next_ground, mut next_air) = (0, 0);
        for (i, text) in def.waves.iter().enumerate() {
            let mut groups = vec![];
            for token in text.split_whitespace() {
                let mut chars = token.chars();
                let kind = chars
                    .next()
                    .and_then(EnemyKind::code)
                    .ok_or_else(|| format!("{}: bad group {token}", def.name))?;
                let count: u32 = chars
                    .as_str()
                    .parse()
                    .map_err(|_| format!("{}: bad count {token}", def.name))?;
                if count == 0 || count > 60 {
                    return Err(format!("{}: group size {count}", def.name));
                }
                // Groups alternate across the map's routes of the matching class.
                let route = if kind.stats().air {
                    if air == 0 {
                        return Err(format!("{}: aircraft without an air route", def.name));
                    }
                    next_air += 1;
                    ground + (next_air - 1) % air
                } else {
                    next_ground += 1;
                    (next_ground - 1) % ground
                };
                groups.push(Group {
                    kind,
                    count: difficulty.count(count),
                    route,
                });
            }
            if groups.is_empty() {
                return Err(format!("{}: empty wave {}", def.name, i + 1));
            }
            let mut spawns = vec![];
            let mut tick = 0;
            for g in &groups {
                for _ in 0..g.count {
                    spawns.push(Spawn {
                        tick,
                        kind: g.kind,
                        route: g.route,
                    });
                    tick += g.kind.spacing();
                }
                tick += GROUP_GAP;
            }
            waves[difficulty.index()].push(Wave {
                groups,
                spawns,
                health: def.health.0
                    + def.health.1 * i as u32
                    + (i * i) as u32 / LATE
                    + difficulty.health_bonus(i),
            });
        }
    }
    let build_cells = terrain.iter().filter(|t| **t == Terrain::Build).count();
    if build_cells < 60 {
        return Err(format!("{}: too little buildable terrain", def.name));
    }
    Ok(Map {
        name: def.name,
        lesson: def.lesson,
        terrain,
        routes,
        credits: [
            Difficulty::Normal.credits(def.credits),
            Difficulty::Hard.credits(def.credits),
        ],
        waves,
    })
}

/// Parse and validate every authored map. Errors describe the first invalid field.
pub fn validate() -> Result<Vec<Map>, String> {
    DEFS.iter().map(build).collect()
}

pub fn maps() -> &'static [Map] {
    static MAPS: OnceLock<Vec<Map>> = OnceLock::new();
    MAPS.get_or_init(|| validate().expect("authored Ridgeline maps are valid"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_data_is_valid_and_complete() {
        let maps = validate().unwrap();
        assert_eq!(maps.len(), 10);
        for map in &maps {
            for d in Difficulty::ALL {
                assert_eq!(map.waves[d.index()].len(), WAVES);
                for w in &map.waves[d.index()] {
                    assert!(w.spawns.windows(2).all(|p| p[0].tick < p[1].tick));
                    for s in &w.spawns {
                        assert_eq!(map.routes[s.route].air, s.kind.stats().air);
                    }
                }
            }
            for r in &map.routes {
                assert!(r.length() > 0);
                assert_eq!(r.at(0), r.points[0]);
                assert_eq!(r.at(r.length()), *r.points.last().unwrap());
            }
        }
    }
    #[test]
    fn hard_table_is_larger_but_towers_are_identical() {
        let map = &maps()[0];
        let normal: u32 = map.waves[0].iter().map(|w| w.spawns.len() as u32).sum();
        let hard: u32 = map.waves[1].iter().map(|w| w.spawns.len() as u32).sum();
        assert!(hard > normal);
        assert_eq!(Difficulty::Hard.count(10), 13);
        assert_eq!(Difficulty::Hard.count(1), 2);
        assert!(map.credits[1] < map.credits[0]);
    }
    #[test]
    fn integer_geometry_and_prices() {
        assert_eq!(isqrt(999_999), 999);
        assert_eq!(isqrt(1_000_000), 1000);
        assert_eq!(refund(invested(TowerKind::Cannon, 0)), 35);
        assert_eq!(refund(invested(TowerKind::Cannon, 2)), 122);
        let r = Route::new(true, &[(0, 0), (3, 4)]);
        assert_eq!(r.length(), 5000);
        assert_eq!(r.at(2500), P::new(2000, 2500));
    }
}
