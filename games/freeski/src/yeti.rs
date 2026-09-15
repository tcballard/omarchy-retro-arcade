//! Original layered vector yeti. Cosmetic geometry only; `ground` is the actor's
//! physical position. Animation uses saved simulation ticks and current speed.
use crate::geometry::{ellipse, poly};
use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};

const OUTLINE: Color32 = Color32::from_rgb(38, 57, 66);
const SHADE: Color32 = Color32::from_rgb(126, 158, 169);
const FUR: Color32 = Color32::from_rgb(215, 232, 230);
const SNOW: Color32 = Color32::from_rgb(249, 250, 235);
const SKIN: Color32 = Color32::from_rgb(48, 65, 72);

pub fn draw(
    p: &egui::Painter,
    ground: Pos2,
    scale: f32,
    ticks: u64,
    speed: f64,
    heading: f64,
    reduced: bool,
) {
    let movement = if reduced {
        0.
    } else {
        (speed as f32 / 18.).clamp(0., 1.)
    };
    let stride = (ticks as f32 * 0.22).sin() * movement;
    let bob = stride.abs() * 0.17;
    let body = ground + Vec2::new(0., -bob) * scale;
    let facing = heading.sin() as f32;
    ellipse(
        p,
        ground + Vec2::new(0.2, 0.25) * scale,
        scale,
        2.25,
        0.68,
        Color32::from_black_alpha(62),
    );

    // Heavy, staggered feet and bowed legs anchor the running body to the snow.
    for side in [-1., 1.] {
        let foot = ground + Vec2::new(side * 0.83, side * stride * 0.43) * scale;
        poly(
            p,
            foot,
            scale,
            &[
                (-0.48, 0.25),
                (-0.61, -0.22),
                (-0.36, -1.6),
                (0.40, -1.6),
                (0.53, -0.25),
                (0.65, 0.25),
            ],
            SHADE,
            true,
        );
        ellipse(p, foot, scale, 0.65, 0.30, SKIN);
        for toe in [-0.32, 0., 0.32] {
            poly(
                p,
                foot,
                scale,
                &[(toe - 0.10, 0.02), (toe + 0.10, 0.02), (toe + 0.08, 0.43)],
                SNOW,
                false,
            );
        }
        poly(
            p,
            foot,
            scale,
            &[
                (-0.47, -0.80),
                (-0.37, -1.7),
                (0.38, -1.7),
                (0.52, -0.75),
                (0.21, -0.97),
                (0.10, -0.57),
                (-0.15, -0.90),
                (-0.38, -0.55),
            ],
            FUR,
            false,
        );
    }

    // Long arms swing out of phase, ending in charcoal hands and ivory talons.
    for side in [-1., 1.] {
        let swing = side * stride * 0.43;
        let arm = body + Vec2::new(side * 1.50, -3.95) * scale;
        let shape = [
            (-0.35, -0.35),
            (0.38, -0.27),
            (0.97, 0.30),
            (0.81, 0.58),
            (1.28, 1.05 + swing),
            (1.07, 1.13 + swing),
            (1.40, 2.21 + swing),
            (0.61, 2.44 + swing),
            (0.38, 1.31),
            (0.00, 0.72),
        ];
        let arm_points: Vec<_> = shape.iter().map(|&(x, y)| (side * x, y)).collect();
        poly(p, arm, scale, &arm_points, SHADE, true);
        let fore = [
            (0.38, 0.20),
            (0.82, 0.57),
            (0.71, 0.86),
            (1.12, 1.25 + swing),
            (0.90, 1.40 + swing),
            (1.18, 2.00 + swing),
            (0.72, 2.12 + swing),
            (0.42, 1.03),
        ];
        poly(
            p,
            arm,
            scale,
            &fore.iter().map(|&(x, y)| (side * x, y)).collect::<Vec<_>>(),
            FUR,
            false,
        );
        let hand = arm + Vec2::new(side * 1.02, 2.32 + swing) * scale;
        ellipse(p, hand, scale, 0.53, 0.55, OUTLINE);
        for finger in [-0.31, 0., 0.31] {
            poly(
                p,
                hand,
                scale,
                &[
                    (finger - 0.11, 0.16),
                    (finger + 0.12, 0.16),
                    (finger + side * 0.16, 0.88),
                    (finger - 0.06, 0.62),
                ],
                SNOW,
                false,
            );
        }
    }

    // Saw-toothed fur, a high shoulder ridge and narrow hips give an unmistakable
    // hunched silhouette. Triangulation preserves the concave fur tips.
    poly(
        p,
        body,
        scale,
        &[
            (-1.08, -0.87),
            (-1.45, -1.40),
            (-1.32, -2.03),
            (-1.79, -2.05),
            (-1.64, -2.62),
            (-2.20, -2.60),
            (-2.04, -3.11),
            (-2.36, -3.37),
            (-2.11, -4.24),
            (-1.59, -4.87),
            (-1.02, -5.10),
            (-0.75, -5.44),
            (-0.28, -5.18),
            (0.10, -5.53),
            (0.46, -5.20),
            (1.11, -5.22),
            (1.58, -4.82),
            (2.12, -4.32),
            (2.35, -3.45),
            (2.05, -3.12),
            (2.24, -2.70),
            (1.73, -2.72),
            (1.83, -2.12),
            (1.33, -2.14),
            (1.45, -1.45),
            (1.03, -0.85),
            (0.56, -1.15),
            (0.25, -0.72),
            (-0.12, -1.03),
            (-0.57, -0.72),
        ],
        FUR,
        true,
    );
    poly(
        p,
        body,
        scale,
        &[
            (-1.88, -3.61),
            (-1.50, -4.45),
            (-0.80, -4.80),
            (0.0, -4.98),
            (0.91, -4.73),
            (1.66, -4.11),
            (1.87, -3.39),
            (1.27, -3.68),
            (1.02, -3.10),
            (0.61, -3.37),
            (0.17, -2.93),
            (-0.21, -3.35),
            (-0.80, -3.08),
            (-1.08, -3.64),
            (-1.51, -3.35),
        ],
        SNOW,
        false,
    );
    poly(
        p,
        body,
        scale,
        &[
            (-1.02, -2.89),
            (-0.65, -2.46),
            (-0.77, -1.49),
            (-0.32, -1.72),
            (0., -1.32),
            (0.47, -1.57),
            (0.82, -2.36),
            (1.02, -2.95),
            (0.56, -2.70),
            (0.02, -3.00),
            (-0.49, -2.64),
        ],
        SHADE,
        false,
    );
    for side in [-1., 1.] {
        p.line_segment(
            [
                body + Vec2::new(side * 1.26, -3.00) * scale,
                body + Vec2::new(side * 1.05, -2.14) * scale,
            ],
            Stroke::new(0.13 * scale, OUTLINE),
        );
    }

    let head = body + Vec2::new(facing * 0.30, -4.02) * scale;
    if heading.cos() < -0.4 {
        // A retreat/escape turn shows the shaggy back of the head, not a face
        // staring at the camera while its body travels the other way.
        poly(
            p,
            head,
            scale,
            &[
                (-0.95, -0.63),
                (-0.55, -1.07),
                (0.12, -1.19),
                (0.68, -0.94),
                (1.03, -0.31),
                (0.75, 0.78),
                (0.31, 0.52),
                (0., 0.94),
                (-0.34, 0.52),
                (-0.85, 0.78),
            ],
            FUR,
            true,
        );
        return;
    }
    // Face sunk beneath the shoulders: deep sockets, an angled brow and a
    // bared-teeth snarl remain readable in a roughly 30–45 px native silhouette.
    ellipse(p, head, scale, 1.04, 1.00, OUTLINE);
    poly(
        p,
        head,
        scale,
        &[
            (-0.90, -0.41),
            (-0.60, -0.82),
            (0., -0.96),
            (0.62, -0.81),
            (0.91, -0.34),
            (0.79, 0.57),
            (0.36, 0.85),
            (-0.38, 0.85),
            (-0.82, 0.52),
        ],
        SKIN,
        false,
    );
    for side in [-1., 1.] {
        let eye = head + Vec2::new(side * 0.45, -0.25) * scale;
        p.line_segment(
            [
                eye + Vec2::new(-0.19, -side * 0.065) * scale,
                eye + Vec2::new(0.19, side * 0.065) * scale,
            ],
            Stroke::new((0.19 * scale).max(1.), Color32::from_rgb(238, 91, 45)),
        );
        poly(
            p,
            head,
            scale,
            &[
                (side * 0.08, -0.47),
                (side * 0.88, -0.83),
                (side * 1.00, -0.58),
                (side * 0.12, -0.23),
            ],
            SNOW,
            false,
        );
    }
    poly(
        p,
        head,
        scale,
        &[
            (-0.23, -0.08),
            (0.23, -0.08),
            (0.31, 0.18),
            (0., 0.29),
            (-0.31, 0.18),
        ],
        OUTLINE,
        false,
    );
    ellipse(
        p,
        head + Vec2::new(0., 0.55) * scale,
        scale,
        0.65,
        0.37,
        Color32::from_rgb(40, 24, 29),
    );
    for side in [-1., 1.] {
        poly(
            p,
            head,
            scale,
            &[
                (side * 0.25, 0.30),
                (side * 0.50, 0.27),
                (side * 0.38, 0.79),
            ],
            SNOW,
            false,
        );
    }
    p.line_segment(
        [
            head + Vec2::new(-0.19, 0.74) * scale,
            head + Vec2::new(0.19, 0.74) * scale,
        ],
        Stroke::new(0.12 * scale, FUR),
    );
    poly(
        p,
        head,
        scale,
        &[
            (-0.98, 0.19),
            (-0.70, 0.73),
            (-0.45, 0.96),
            (-0.14, 0.79),
            (0., 1.14),
            (0.22, 0.83),
            (0.49, 0.99),
            (0.80, 0.61),
            (0.97, 0.14),
            (1.14, 0.68),
            (0.84, 1.05),
            (0.59, 1.00),
            (0.30, 1.33),
            (-0.01, 1.15),
            (-0.39, 1.35),
            (-0.58, 0.97),
            (-1.04, 0.94),
        ],
        SNOW,
        false,
    );
}
