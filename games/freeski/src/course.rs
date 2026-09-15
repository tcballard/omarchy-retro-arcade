//! Authored Slalom courses and objective accounting.
//!
//! Gates are objective geometry. Their poles are also ordinary world obstacles,
//! so the physics engine remains the sole owner of impacts. Call
//! [`CourseProgress::cross_segment`] with the skier's physical movement segment;
//! recovery relocation is deliberately not a movement segment.

use crate::{
    engine::{Input, Point, Sim, HZ, MAX_HEADING},
    world::{Kind, Obstacle},
};
use serde::{Deserialize, Serialize};

pub const COURSE_COUNT: u8 = 5;
pub const MISSED_GATE_PENALTY_TICKS: u64 = HZ as u64 * 5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gate {
    pub x: f64,
    pub y: f64,
    pub half_width: f64,
}

#[derive(Clone, Debug)]
pub struct Course {
    pub index: u8,
    pub name: &'static str,
    pub length: f64,
    pub gates: Vec<Gate>,
    pub obstacles: Vec<Obstacle>,
    pub gold_ticks: u64,
    pub silver_ticks: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateOutcome {
    Passed,
    Missed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GateResolution {
    pub gate_index: usize,
    pub outcome: GateOutcome,
    pub crossing_x: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct CourseProgress {
    pub next_gate: usize,
    pub missed: u32,
    pub penalty_ticks: u64,
}

impl CourseProgress {
    pub fn valid(&self, course: &Course) -> bool {
        self.next_gate <= course.gates.len()
            && self.missed as usize <= self.next_gate
            && self.penalty_ticks
                == u64::from(self.missed).saturating_mul(MISSED_GATE_PENALTY_TICKS)
    }

    /// Resolve every ordered gate line crossed by one downhill physical segment.
    /// Sideways and uphill segments cannot count or miss a gate.
    pub fn cross_segment(
        &mut self,
        course: &Course,
        from: Point,
        to: Point,
    ) -> Vec<GateResolution> {
        if !self.valid(course)
            || ![from.x, from.y, to.x, to.y].iter().all(|n| n.is_finite())
            || to.y <= from.y
        {
            return Vec::new();
        }

        let mut resolved = Vec::new();
        while let Some(gate) = course.gates.get(self.next_gate).copied() {
            if gate.y <= from.y || gate.y > to.y {
                break;
            }
            let at = (gate.y - from.y) / (to.y - from.y);
            let crossing_x = from.x + (to.x - from.x) * at;
            let outcome = if (crossing_x - gate.x).abs() < gate.half_width {
                GateOutcome::Passed
            } else {
                self.missed += 1;
                self.penalty_ticks += MISSED_GATE_PENALTY_TICKS;
                GateOutcome::Missed
            };
            resolved.push(GateResolution {
                gate_index: self.next_gate,
                outcome,
                crossing_x,
            });
            self.next_gate += 1;
        }
        resolved
    }

    pub fn all_resolved(&self, course: &Course) -> bool {
        self.valid(course) && self.next_gate == course.gates.len()
    }

    pub fn final_ticks(&self, raw_ticks: u64) -> u64 {
        raw_ticks.saturating_add(self.penalty_ticks)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Medal {
    Bronze,
    Silver,
    Gold,
}

pub fn medal(course: &Course, raw_ticks: u64, progress: &CourseProgress) -> Option<Medal> {
    if !progress.all_resolved(course) {
        return None;
    }
    let time = progress.final_ticks(raw_ticks);
    Some(if time <= course.gold_ticks {
        Medal::Gold
    } else if time <= course.silver_ticks {
        Medal::Silver
    } else {
        Medal::Bronze
    })
}

/// True only for a downhill finish crossing after every gate was resolved.
pub fn crosses_finish(course: &Course, progress: &CourseProgress, from: Point, to: Point) -> bool {
    progress.all_resolved(course)
        && [from.y, to.y].iter().all(|n| n.is_finite())
        && from.y < course.length
        && to.y >= course.length
        && to.y > from.y
}

/// Production-input reference steering for repeatable course feasibility runs.
/// It anticipates the next authored gate and never alters simulation state.
pub fn reference_heading(index: u8, sim: &Sim) -> f64 {
    let Some(course) = course(index) else {
        return 0.;
    };
    let next_gate = course
        .gates
        .iter()
        .enumerate()
        .find(|(_, gate)| gate.y > sim.position.y);
    let target_gate_index = next_gate.map(|(gate_index, gate)| {
        let remaining = gate.y - sim.position.y;
        let crossing_x = sim.position.x + sim.heading.tan() * remaining;
        let committed =
            remaining <= sim.speed * 0.18 && (crossing_x - gate.x).abs() <= gate.half_width * 0.75;
        if committed && gate_index + 1 < course.gates.len() {
            gate_index + 1
        } else {
            gate_index
        }
    });
    let (target, half_width, approach_x) = target_gate_index
        .map(|gate_index| (gate_index, &course.gates[gate_index]))
        .map(|(gate_index, gate)| {
            (
                Point {
                    x: gate.x,
                    y: gate.y,
                },
                gate.half_width,
                gate_index
                    .checked_sub(1)
                    .map_or(0., |previous| course.gates[previous].x),
            )
        })
        .unwrap_or((
            Point {
                x: 0.,
                y: course.length,
            },
            0.,
            course.gates.last().map_or(0., |gate| gate.x),
        ));
    // Compensate for the finite turn rate by steering from the skier's projected
    // position. This leaves real clearance from the arrival-side pole at both
    // normal and fast speed without moving the authored centre line.
    let horizon = 0.25 + sim.speed / 300.;
    let projected_x = sim.position.x + sim.heading.sin() * sim.speed * horizon;
    let projected_y = sim.position.y + sim.heading.cos() * sim.speed * horizon;
    let arrival_side = (target.x - approach_x).signum();
    let target_x = target.x + arrival_side * half_width * 0.8;
    (target_x - projected_x)
        // Keep correcting toward the safe gate window until the line is
        // actually crossed. A large fixed denominator lets a faster skier
        // coast into a pole during the final few metres.
        .atan2((target.y - projected_y).max(2.))
        .clamp(-MAX_HEADING, MAX_HEADING)
}

/// Production controls for a clean authored-course reference run. Fast mode
/// remains useful on open sections but brakes for the tightest direction
/// changes, demonstrating that the higher cap is a skill choice rather than a
/// free time reduction.
pub fn reference_input(index: u8, sim: &Sim) -> Input {
    let heading = reference_heading(index, sim);
    Input {
        heading,
        brake: sim.fast_mode && sim.speed > 68. && (heading - sim.heading).abs() > 0.12,
    }
}

pub fn course(index: u8) -> Option<Course> {
    let spec = SPECS.get(index as usize)?;
    let gates = spec
        .gates
        .iter()
        .map(|&(x, y, half_width)| Gate { x, y, half_width })
        .collect::<Vec<_>>();
    let mut obstacles = Vec::with_capacity(gates.len() * 2 + spec.obstacles.len());
    for gate in &gates {
        for x in [gate.x - gate.half_width, gate.x + gate.half_width] {
            obstacles.push(Obstacle {
                id: obstacles.len(),
                at: Point { x, y: gate.y },
                kind: Kind::Pole,
            });
        }
    }
    for &(x, y, kind) in spec.obstacles {
        obstacles.push(Obstacle {
            id: obstacles.len(),
            at: Point { x, y },
            kind,
        });
    }
    Some(Course {
        index,
        name: spec.name,
        length: spec.length,
        gates,
        obstacles,
        gold_ticks: spec.gold_ticks,
        silver_ticks: spec.silver_ticks,
    })
}

struct CourseSpec {
    name: &'static str,
    length: f64,
    gates: &'static [(f64, f64, f64)],
    obstacles: &'static [(f64, f64, Kind)],
    gold_ticks: u64,
    silver_ticks: u64,
}

// Medal targets are calibrated from the fast-capable production-input reference
// runs in tests/slalom.rs. Gold allows roughly 10% and Silver roughly 25% over
// those repeatable no-penalty runs; Bronze requires only a valid finish.
static SPECS: [CourseSpec; 5] = [
    CourseSpec {
        name: "Pinecone Path",
        length: 760.,
        gates: &[
            (-6., 85., 10.),
            (8., 140., 10.),
            (-10., 198., 10.),
            (12., 258., 10.),
            (-12., 320., 9.5),
            (8., 382., 9.5),
            (-6., 446., 9.),
            (14., 510., 9.),
            (-12., 574., 9.),
            (5., 638., 8.5),
            (-5., 695., 8.5),
        ],
        obstacles: &[
            (-29., 175., Kind::Tree),
            (28., 225., Kind::Tree),
            (-30., 365., Kind::Rock),
            (29., 470., Kind::Tree),
            (-28., 610., Kind::Tree),
        ],
        gold_ticks: 1_150,
        silver_ticks: 1_320,
    },
    CourseSpec {
        name: "Long Turns",
        length: 900.,
        gates: &[
            (-14., 90., 10.),
            (15., 150., 10.),
            (-16., 212., 9.5),
            (17., 276., 9.5),
            (-17., 342., 9.),
            (13., 408., 9.),
            (-10., 474., 9.),
            (16., 540., 8.5),
            (-16., 606., 8.5),
            (12., 672., 8.5),
            (-13., 738., 8.),
            (8., 802., 8.),
        ],
        obstacles: &[
            (29., 180., Kind::Rock),
            (-32., 300., Kind::Tree),
            (31., 446., Kind::Tree),
            (-29., 570., Kind::Rock),
            (-31., 705., Kind::Tree),
            (29., 825., Kind::Rock),
        ],
        gold_ticks: 1_380,
        silver_ticks: 1_560,
    },
    CourseSpec {
        name: "Split Pines",
        length: 1_020.,
        gates: &[
            (8., 82., 9.),
            (-8., 138., 9.),
            (13., 194., 8.5),
            (-14., 250., 8.5),
            (10., 308., 8.),
            (-5., 368., 8.),
            (-16., 430., 8.),
            (15., 492., 8.),
            (5., 554., 7.5),
            (-14., 616., 7.5),
            (16., 678., 7.5),
            (-8., 742., 7.5),
            (13., 806., 7.),
            (-15., 870., 7.),
            (6., 934., 7.),
        ],
        obstacles: &[
            (-27., 110., Kind::Tree),
            (27., 166., Kind::Tree),
            (29., 280., Kind::Rock),
            (28., 400., Kind::Tree),
            (-28., 525., Kind::Tree),
            (-29., 650., Kind::Rock),
            (-29., 775., Kind::Tree),
            (29., 900., Kind::Tree),
        ],
        gold_ticks: 1_440,
        silver_ticks: 1_620,
    },
    CourseSpec {
        name: "Needle Run",
        length: 1_140.,
        gates: &[
            (-10., 78., 8.),
            (12., 132., 8.),
            (-14., 188., 7.5),
            (14., 244., 7.5),
            (-9., 300., 7.5),
            (4., 358., 7.),
            (15., 416., 7.),
            (-15., 474., 7.),
            (6., 532., 7.),
            (-12., 590., 6.75),
            (14., 648., 6.75),
            (-4., 706., 6.75),
            (-16., 764., 6.5),
            (16., 822., 6.5),
            (-7., 880., 6.5),
            (12., 938., 6.5),
            (-10., 996., 6.5),
            (4., 1054., 6.5),
        ],
        obstacles: &[
            (29., 104., Kind::Rock),
            (-30., 220., Kind::Tree),
            (30., 330., Kind::Tree),
            (-29., 445., Kind::Rock),
            (-29., 560., Kind::Tree),
            (29., 676., Kind::Tree),
            (29., 792., Kind::Rock),
            (-30., 908., Kind::Tree),
            (30., 1024., Kind::Tree),
        ],
        gold_ticks: 1_560,
        silver_ticks: 1_800,
    },
    CourseSpec {
        name: "Summit Cup",
        length: 1_280.,
        gates: &[
            (6., 74., 8.),
            (-12., 126., 7.5),
            (15., 178., 7.5),
            (-15., 230., 7.25),
            (9., 282., 7.25),
            (-4., 336., 7.),
            (-16., 390., 7.),
            (16., 444., 7.),
            (-8., 498., 6.75),
            (12., 552., 6.75),
            (-14., 606., 6.75),
            (3., 660., 6.5),
            (16., 714., 6.5),
            (-16., 768., 6.5),
            (7., 822., 6.5),
            (-11., 876., 6.25),
            (15., 930., 6.25),
            (-15., 984., 6.25),
            (5., 1038., 6.25),
            (13., 1092., 6.),
            (-10., 1146., 6.),
            (3., 1200., 6.),
        ],
        obstacles: &[
            (-30., 100., Kind::Tree),
            (29., 204., Kind::Tree),
            (29., 310., Kind::Rock),
            (-29., 418., Kind::Tree),
            (29., 525., Kind::Tree),
            (29., 633., Kind::Rock),
            (-30., 741., Kind::Tree),
            (30., 849., Kind::Tree),
            (-29., 957., Kind::Rock),
            (-29., 1065., Kind::Tree),
            (29., 1173., Kind::Tree),
        ],
        gold_ticks: 1_710,
        silver_ticks: 1_950,
    },
];
