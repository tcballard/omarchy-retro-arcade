//! Original, versioned practice slope. World units are metres, +y is downhill.
use crate::engine::Point;
pub const COURSE_VERSION: u32 = 1;
pub const HALF_WIDTH: f64 = 40.;
pub const FINISH: f64 = 1200.;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Tree,
    Rock,
    Ramp,
    Pole,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Obstacle {
    pub id: usize,
    pub at: Point,
    pub kind: Kind,
}
impl Obstacle {
    pub fn radius(self) -> f64 {
        match self.kind {
            Kind::Tree => 1.4,
            Kind::Rock => 1.2,
            Kind::Ramp => 1.8,
            Kind::Pole => 0.3,
        }
    }
    pub fn height(self) -> f64 {
        match self.kind {
            Kind::Tree => 8.,
            Kind::Rock => 0.65,
            Kind::Ramp => 0.,
            Kind::Pole => 1.2,
        }
    }
}
pub fn practice() -> Vec<Obstacle> {
    let mut items = Vec::new();
    let mut add = |x, y, kind| {
        let id = items.len();
        items.push(Obstacle {
            id,
            at: Point { x, y },
            kind,
        });
    };
    // Alternating, generous turns. The first 90 metres are clear snow.
    for (x, y) in [
        (-18., 100.),
        (18., 135.),
        (-8., 180.),
        (23., 215.),
        (-24., 240.),
        (10., 285.),
        (-13., 310.),
        (26., 340.),
        (-27., 365.),
        (0., 360.),
        (7., 360.),
    ] {
        add(x, y, Kind::Tree);
    }
    for (x, y) in [(-12., 420.), (14., 750.), (0., 1000.)] {
        add(x, y, Kind::Ramp);
        add(x, y + 11., Kind::Rock);
    }
    for (x, y) in [
        (21., 465.),
        (-22., 500.),
        (9., 540.),
        (-10., 575.),
        (23., 610.),
        (-24., 650.),
        (-4., 695.),
        (-23., 800.),
        (24., 835.),
        (-9., 870.),
        (12., 900.),
        (-25., 940.),
        (23., 1050.),
        (-20., 1090.),
        (10., 1130.),
    ] {
        add(x, y, Kind::Tree);
    }
    for (x, y) in [
        (16., 320.),
        (-22., 560.),
        (26., 720.),
        (-15., 855.),
        (18., 960.),
    ] {
        add(x, y, Kind::Rock);
    }
    items
}
pub fn lesson(y: f64) -> (&'static str, &'static str) {
    match y as u32 {
        0..=90 => (
            "01 / FIND YOUR EDGES",
            "Carve left and right. Hold brake to slow down.",
        ),
        91..=275 => (
            "02 / WIDE TURNS",
            "Look downhill. Leave space around the trees.",
        ),
        276..=390 => (
            "03 / TAKE IT SLOW",
            "Brake before the turn, then let the skis run.",
        ),
        391..=470 => (
            "04 / A LITTLE AIR",
            "The striped ramp launches you. Aim for clear snow.",
        ),
        471..=950 => ("05 / FIND A RHYTHM", "Link your turns. Ramps are optional."),
        _ => (
            "06 / HOME STRETCH",
            "Finish at the flags. Your best distance is saved.",
        ),
    }
}
