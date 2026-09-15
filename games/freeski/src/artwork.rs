//! Original FreeSki artwork. World anchors and deterministic variation are
//! cosmetic only; the simulation owns every collision and outcome.
use crate::{
    engine::Sim,
    geometry::{ellipse, poly},
    world::{Kind, Obstacle},
};
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};

const INK: Color32 = Color32::from_rgb(38, 57, 66);
const SNOW: Color32 = Color32::from_rgb(249, 250, 235);
const ICE: Color32 = Color32::from_rgb(185, 210, 214);
const GREEN: Color32 = Color32::from_rgb(35, 80, 72);
const LEAF: Color32 = Color32::from_rgb(64, 109, 93);
const ORANGE: Color32 = Color32::from_rgb(209, 88, 43);
const GOLD: Color32 = Color32::from_rgb(238, 183, 83);

fn line(
    p: &egui::Painter,
    at: Pos2,
    s: f32,
    a: (f32, f32),
    b: (f32, f32),
    width: f32,
    color: Color32,
) {
    p.line_segment(
        [at + Vec2::new(a.0, a.1) * s, at + Vec2::new(b.0, b.1) * s],
        Stroke::new((width * s).max(0.6), color),
    );
}

pub fn obstacle(p: &egui::Painter, at: Pos2, s: f32, obstacle: &Obstacle) {
    if obstacle.kind == Kind::Pole {
        return;
    }
    ellipse(
        p,
        at + Vec2::new(0.9, 0.45) * s,
        s,
        2.4,
        0.68,
        Color32::from_black_alpha(32),
    );
    match obstacle.kind {
        Kind::Tree => tree(p, at, s, obstacle.id),
        Kind::Rock => rock(p, at, s, obstacle.id),
        Kind::Ramp => ramp(p, at, s),
        Kind::Pole => {}
    }
}

fn tree(p: &egui::Painter, at: Pos2, s: f32, id: usize) {
    let height = [1., 1.08, 0.94][id % 3];
    poly(
        p,
        at,
        s,
        &[(-0.43, 0.25), (-0.35, -2.7), (0.36, -2.7), (0.49, 0.25)],
        Color32::from_rgb(94, 78, 60),
        true,
    );
    line(
        p,
        at,
        s,
        (-0.17, -0.15),
        (-0.17, -1.8),
        0.16,
        GOLD.gamma_multiply(0.65),
    );
    ellipse(p, at + Vec2::new(-0.25, 0.25) * s, s, 0.8, 0.2, ICE);
    // Three stepped boughs retain a recognizable evergreen outline. Shape
    // variation changes foliage only, never the trunk or collision footprint.
    for (level, (y, w, h)) in [(1.1, 2.5, 3.4), (2.8, 1.95, 2.9), (4.4, 1.3, 2.6)]
        .into_iter()
        .enumerate()
    {
        let origin = at - Vec2::new(0., y * height) * s;
        let lean = if id.is_multiple_of(2) { 0.12 } else { -0.12 };
        let crown = &[
            (-w, 0.),
            (-w * 0.63, -0.85),
            (-w * 0.77, -0.72),
            (lean, -h * height),
            (w * 0.76, -0.73),
            (w * 0.63, -0.84),
            (w, 0.),
            (w * 0.44, -0.10),
            (w * 0.24, 0.18),
            (0., -0.03),
            (-w * 0.39, 0.16),
        ];
        poly(p, origin, s, crown, GREEN, true);
        poly(
            p,
            origin,
            s,
            &[
                (-w * 0.85, -0.15),
                (lean, -h * height),
                (0.03, -0.15),
                (-w * 0.28, -0.35),
                (-w * 0.4, 0.02),
            ],
            LEAF,
            false,
        );
        let cap = if level == 0 { 0.67 } else { 0.75 };
        poly(
            p,
            origin,
            s,
            &[
                (lean, -h * height),
                (-w * cap, -0.73),
                (-w * 0.39, -0.90),
                (-w * 0.24, -0.59),
                (0.04, -0.91),
                (w * 0.22, -0.58),
                (w * 0.42, -0.81),
                (w * cap, -0.62),
            ],
            SNOW,
            false,
        );
        line(
            p,
            origin,
            s,
            (-w * 0.62, -0.62),
            (-w * 0.32, -0.71),
            0.16,
            ICE,
        );
        line(
            p,
            origin,
            s,
            (w * 0.10, -0.4),
            (w * 0.42, -0.25),
            0.12,
            LEAF,
        );
    }
}

fn rock(p: &egui::Painter, at: Pos2, s: f32, id: usize) {
    let flip = if id.is_multiple_of(2) { 1. } else { -1. };
    let shape = |pts: &[(f32, f32)]| pts.iter().map(|&(x, y)| (x * flip, y)).collect::<Vec<_>>();
    poly(
        p,
        at,
        s,
        &shape(&[
            (-1.48, 0.15),
            (-1.2, -0.92),
            (-0.48, -1.45),
            (0.76, -1.25),
            (1.48, -0.32),
            (1.17, 0.55),
            (-0.20, 0.72),
        ]),
        Color32::from_rgb(100, 123, 138),
        true,
    );
    poly(
        p,
        at,
        s,
        &shape(&[(-1.43, 0.1), (-0.59, -0.47), (0.20, -0.08), (-0.20, 0.66)]),
        Color32::from_rgb(153, 177, 185),
        false,
    );
    poly(
        p,
        at,
        s,
        &shape(&[
            (0.20, -0.08),
            (0.76, -1.18),
            (1.42, -0.31),
            (1.14, 0.49),
            (-0.20, 0.66),
        ]),
        Color32::from_rgb(70, 94, 108),
        false,
    );
    poly(
        p,
        at,
        s,
        &shape(&[
            (-1.22, -0.90),
            (-0.49, -1.47),
            (0.76, -1.27),
            (1.05, -0.85),
            (0.60, -0.70),
            (0.23, -0.85),
            (-0.13, -0.52),
            (-0.50, -0.63),
            (-0.78, -0.42),
        ]),
        SNOW,
        false,
    );
    line(
        p,
        at,
        s,
        (flip * 0.20, -0.03),
        (flip * 0.51, 0.29),
        0.10,
        INK,
    );
}

fn ramp(p: &egui::Painter, at: Pos2, s: f32) {
    // Uphill approach rises to the downhill takeoff lip. Warm wood and bright
    // chevrons distinguish a jump from both snow-covered stone and gate cloth.
    poly(
        p,
        at,
        s,
        &[(-2., -1.55), (2., -1.55), (2.25, 0.60), (-2.25, 0.60)],
        Color32::from_rgb(43, 96, 111),
        true,
    );
    poly(
        p,
        at,
        s,
        &[(-2.25, 0.60), (2.25, 0.60), (2.25, 1.4), (-2.25, 1.4)],
        Color32::from_rgb(102, 72, 48),
        true,
    );
    for x in [-1.5, -0.5, 0.5, 1.5] {
        line(p, at, s, (x, 0.72), (x, 1.30), 0.11, GOLD);
    }
    for y in [-1.1, -0.42] {
        line(p, at, s, (-1.30, y - 0.17), (0., y + 0.19), 0.22, SNOW);
        line(p, at, s, (0., y + 0.19), (1.30, y - 0.17), 0.22, SNOW);
    }
    line(p, at, s, (-2.25, 0.57), (2.25, 0.57), 0.35, GOLD);
    line(p, at, s, (-2.05, -1.55), (-2.25, 0.5), 0.22, ICE);
    line(p, at, s, (2.05, -1.55), (2.25, 0.5), 0.22, SNOW);
}

pub fn gate(p: &egui::Painter, base: Pos2, s: f32, side: f32, color: Color32, next: bool) {
    ellipse(
        p,
        base + Vec2::new(0.6, 0.25) * s,
        s,
        0.9,
        0.28,
        Color32::from_black_alpha(35),
    );
    ellipse(p, base, s, 0.55, 0.2, ICE);
    line(p, base, s, (0., 0.), (0., -4.25), 0.26, INK);
    line(p, base, s, (-0.06, -0.5), (-0.06, -4.1), 0.12, SNOW);
    let pts = [
        (0., -4.1),
        (-side * 1.45, -3.95),
        (-side * 2.25, -4.05),
        (-side * 2.25, -2.55),
        (-side * 1.25, -2.45),
        (0., -2.6),
    ];
    poly(p, base, s, &pts, color, true);
    line(
        p,
        base,
        s,
        (-side * 0.3, -3.77),
        (-side * 1.9, -3.65),
        0.16,
        SNOW,
    );
    if next {
        poly(
            p,
            base,
            s,
            &[
                (-side * 0.7, -3.45),
                (-side * 1.3, -3.1),
                (-side * 0.7, -2.77),
            ],
            SNOW,
            false,
        );
    }
    p.circle_filled(base - Vec2::new(0., 4.32) * s, 0.23 * s, GOLD);
}

pub fn finish_post(p: &egui::Painter, base: Pos2, s: f32, side: f32) {
    line(p, base, s, (0., 0.), (0., -5.), 0.36, INK);
    poly(
        p,
        base,
        s,
        &[
            (0., -5.),
            (-side * 3., -4.85),
            (-side * 3., -2.8),
            (0., -3.),
        ],
        SNOW,
        true,
    );
    for row in 0..2 {
        for col in 0..3 {
            if (row + col) % 2 == 0 {
                let x = -side * col as f32;
                let y = -4.85 + row as f32;
                poly(
                    p,
                    base,
                    s,
                    &[(x, y), (x - side, y), (x - side, y + 0.9), (x, y + 0.9)],
                    INK,
                    false,
                );
            }
        }
    }
    ellipse(p, base, s, 0.9, 0.25, ICE);
}

pub fn skier(p: &egui::Painter, ground: Pos2, s: f32, sim: &Sim, braking: bool, reduced: bool) {
    let flight = sim.height() as f32;
    ellipse(
        p,
        ground + Vec2::new(0.3, 0.45) * s,
        s,
        1.6 + flight * 0.10,
        0.55,
        Color32::from_black_alpha(65),
    );
    if sim.protection > 0 || sim.tumble > 0 {
        p.circle_stroke(ground, 2.6 * s, Stroke::new(0.20 * s, GOLD));
    }
    let origin = ground - Vec2::new(0., flight * 1.3) * s;
    let crashed = sim.tumble > 0;
    let tucked = sim.fast_mode && !braking && !crashed && flight == 0.;
    let yaw = -(sim.heading as f32);
    let facing = sim.heading.sin() as f32;
    let crouch = if crashed || tucked {
        0.
    } else if braking {
        0.32
    } else {
        (sim.speed as f32 / 50.) * 0.22 + if flight > 0. { 0.18 } else { 0. }
    };
    let roll = if crashed { 1.32 } else { 0. };
    let rotate =
        |v: Vec2, a: f32| Vec2::new(v.x * a.cos() - v.y * a.sin(), v.x * a.sin() + v.y * a.cos());
    // Fast mode folds the silhouette at the knees: a low, broad jacket,
    // lowered helmet and hands together. It remains legible with effects off.
    let body = |x: f32, y: f32| {
        let pose = if tucked {
            Vec2::new(x * 1.10, y * 0.60 + 0.10)
        } else {
            Vec2::new(x, y + crouch)
        };
        rotate(pose, roll)
    };
    let draw_body = |pts: &[(f32, f32)], color, outline| {
        let pts: Vec<_> = pts
            .iter()
            .map(|&(x, y)| {
                let v = body(x, y);
                (v.x, v.y)
            })
            .collect();
        poly(p, origin, s, &pts, color, outline);
    };
    // Skis yaw through the full turn; the torso stays upright in the oblique
    // camera. Braking opens a small snowplough without changing actual heading.
    for side in [-1., 1.] {
        let angle = if crashed {
            roll + side * 0.28
        } else {
            yaw + if braking { side * 0.20 } else { 0. }
        };
        let stance = if tucked { 0.43 } else { 0.59 };
        let ski = |x: f32, y: f32| rotate(Vec2::new(x + side * stance, y), angle);
        let pts: Vec<_> = [
            (-0.16, -1.42),
            (-0.16, 1.53),
            (-0.08, 1.83),
            (0.09, 1.83),
            (0.17, 1.53),
            (0.17, -1.42),
        ]
        .into_iter()
        .map(|(x, y)| {
            let v = ski(x, y);
            (v.x, v.y)
        })
        .collect();
        poly(p, origin, s, &pts, INK, false);
        let a = ski(0., -1.28);
        let b = ski(0., 1.42);
        p.line_segment(
            [origin + a * s, origin + b * s],
            Stroke::new(0.12 * s, GOLD),
        );
        let boot = ski(0., 0.0);
        let knee = body(side * if tucked { 0.64 } else { 0.46 }, -0.71);
        p.line_segment(
            [origin + body(side * 0.24, -1.19) * s, origin + knee * s],
            Stroke::new(0.50 * s, INK),
        );
        p.line_segment(
            [origin + knee * s, origin + boot * s],
            Stroke::new(0.40 * s, Color32::from_rgb(64, 87, 105)),
        );
        ellipse(p, origin + boot * s, s, 0.28, 0.38, INK);
        line(
            p,
            origin + boot * s,
            s,
            (-0.15, -0.05),
            (0.15, -0.05),
            0.12,
            SNOW,
        );
    }
    // Shaped insulated jacket with shadow panels, bright yoke and zipper.
    draw_body(
        &[
            (-0.61, -1.05),
            (-0.72, -2.08),
            (-0.41, -2.42),
            (0.42, -2.42),
            (0.72, -2.08),
            (0.60, -1.05),
            (0., -0.92),
        ],
        ORANGE,
        true,
    );
    draw_body(
        &[(0.19, -2.33), (0.64, -2.09), (0.57, -1.11), (0.19, -1.02)],
        Color32::from_rgb(153, 58, 39),
        false,
    );
    draw_body(
        &[
            (-0.59, -2.12),
            (-0.39, -2.35),
            (0.40, -2.35),
            (0.58, -2.12),
            (0.30, -1.94),
            (-0.26, -1.94),
        ],
        GOLD,
        false,
    );
    let zipper = body(facing * 0.27, -1.9);
    let zip_end = body(facing * 0.27, -1.17);
    p.line_segment(
        [origin + zipper * s, origin + zip_end * s],
        Stroke::new(0.10 * s, INK),
    );
    for side in [-1., 1.] {
        let shoulder = body(side * 0.57, -2.14);
        let elbow = body(
            side * if tucked { 0.77 } else { 0.95 },
            if tucked { -1.05 } else { -1.72 },
        );
        let hand = if tucked {
            body(side * 0.35, -1.12)
        } else {
            body(
                side * (if flight > 0. { 1.25 } else { 0.93 }),
                if braking { -1.35 } else { -1.55 },
            )
        };
        p.line_segment(
            [origin + shoulder * s, origin + elbow * s],
            Stroke::new(0.45 * s, INK),
        );
        p.line_segment(
            [origin + shoulder * s, origin + elbow * s],
            Stroke::new(0.32 * s, ORANGE),
        );
        p.line_segment(
            [origin + elbow * s, origin + hand * s],
            Stroke::new(0.30 * s, ORANGE),
        );
        let tip = hand
            + rotate(
                Vec2::new(
                    side * if tucked { 0.48 } else { 0.20 },
                    if tucked { -2.9 } else { -1.70 },
                ),
                if crashed { roll } else { yaw },
            );
        p.line_segment(
            [origin + hand * s, origin + tip * s],
            Stroke::new((0.10 * s).max(0.6), INK),
        );
        ellipse(p, origin + tip * s, s, 0.20, 0.11, INK);
        ellipse(p, origin + hand * s, s, 0.24, 0.24, INK);
    }
    let head = origin + body(facing * 0.15, -2.78) * s;
    ellipse(p, head, s, 0.68, 0.65, INK);
    ellipse(p, head - Vec2::new(0.04, 0.08) * s, s, 0.58, 0.54, GOLD);
    poly(
        p,
        head,
        s,
        &[
            (-0.49, -0.22),
            (-0.25, -0.48),
            (0.18, -0.5),
            (0.38, -0.27),
            (0.02, -0.31),
        ],
        SNOW,
        false,
    );
    let goggles = head + Vec2::new(facing * 0.24, 0.11) * s;
    ellipse(p, goggles, s, 0.49 - facing.abs() * 0.1, 0.24, INK);
    line(p, goggles, s, (-0.27, -0.05), (0.19, -0.05), 0.13, ICE);
    line(
        p,
        head,
        s,
        (-0.20, 0.43),
        (0.24, 0.43),
        0.17,
        Color32::from_rgb(236, 192, 143),
    );
    if !reduced && sim.speed > 6. && flight == 0. && !crashed {
        let strength = if braking {
            1.
        } else {
            (facing.abs() * 0.7 + 0.12).min(1.)
        };
        // Twelve recycled points follow simulation time. No particle allocation,
        // wall-clock movement, or growing queue; powder stays behind the boots.
        for i in 0..12u64 {
            let age = ((sim.ticks + i * 7) % 24) as f32 / 24.;
            let side = if i % 2 == 0 { -1. } else { 1. };
            let offset = rotate(
                Vec2::new(
                    side * (0.8 + age * 1.9 * strength),
                    -age * (1.4 + strength * 1.5),
                ),
                yaw,
            );
            p.circle_filled(
                ground + offset * s,
                (0.08 + age * 0.16) * s,
                Color32::from_white_alpha(((1. - age) * 100. * strength) as u8),
            );
        }
    }
}

pub fn landing(p: &egui::Painter, at: Pos2, s: f32, age: u64) {
    if age >= 24 {
        return;
    }
    let t = age as f32 / 24.;
    for i in 0..10 {
        let angle = i as f32 * std::f32::consts::TAU / 10.;
        let offset = Vec2::new(angle.cos() * (0.8 + t * 2.0), angle.sin() * (0.3 + t * 0.8));
        ellipse(
            p,
            at + offset * s,
            s,
            0.18 + t * 0.18,
            0.10 + t * 0.10,
            Color32::from_white_alpha(((1. - t) * 140.) as u8),
        );
    }
}
