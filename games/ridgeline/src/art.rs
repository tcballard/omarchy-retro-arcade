//! Original native geometry for Ridgeline. Read-only: nothing here changes play.
use crate::battle::{Battle, Enemy, Shot, Tower};
use crate::data::*;
use arcade_presentation::{BRASS, IVORY};
use eframe::egui::{self, epaint::Mesh, Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};
use std::f32::consts::{FRAC_PI_2, TAU};

/// One palette for the board and the interface, derived from the Omarchy theme.
#[derive(Clone, Copy)]
pub struct Palette {
    pub light: bool,
    /// Terrace levels from lowest to highest.
    pub ground: [Color32; 3],
    pub cliff: Color32,
    pub rock: Color32,
    pub rock_lit: Color32,
    pub road: Color32,
    pub curb: Color32,
    pub rut: Color32,
    pub ink: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub accent_text: Color32,
    pub surface: Color32,
    pub card: Color32,
    pub card_hi: Color32,
    pub line: Color32,
    pub metal: Color32,
    pub metal_lit: Color32,
    pub enemy: Color32,
    pub enemy_shade: Color32,
    pub air: Color32,
    pub cold: Color32,
    pub good: Color32,
    pub bad: Color32,
    pub gold: Color32,
}
impl Palette {
    pub fn new(theme: &arcade_platform::theme::Theme) -> Self {
        let light = theme.light();
        let bg = theme.background;
        let mix = |a: Color32, b: Color32, t: f32| a.lerp_to_gamma(b, t);
        let rgb = Color32::from_rgb;
        if light {
            Self {
                light,
                ground: [
                    mix(rgb(186, 134, 92), bg, 0.18),
                    mix(rgb(212, 164, 116), bg, 0.18),
                    mix(rgb(234, 198, 152), bg, 0.18),
                ],
                cliff: rgb(150, 104, 72),
                rock: rgb(128, 102, 86),
                rock_lit: rgb(172, 146, 126),
                road: rgb(238, 228, 208),
                curb: rgb(136, 98, 70),
                rut: rgb(206, 190, 166),
                ink: rgb(34, 30, 26),
                text: theme.foreground,
                muted: mix(theme.foreground, bg, 0.45),
                accent: theme.accent,
                accent_text: theme.accent_text(),
                surface: bg,
                card: mix(bg, rgb(255, 255, 255), 0.55),
                card_hi: mix(bg, rgb(255, 255, 255), 0.85),
                line: mix(bg, rgb(60, 50, 40), 0.22),
                metal: rgb(92, 88, 80),
                metal_lit: rgb(150, 144, 130),
                enemy: rgb(40, 34, 32),
                enemy_shade: rgb(96, 84, 76),
                air: rgb(34, 58, 104),
                cold: rgb(28, 112, 170),
                good: rgb(46, 122, 64),
                bad: rgb(184, 52, 40),
                gold: rgb(176, 128, 30),
            }
        } else {
            Self {
                light,
                ground: [
                    mix(rgb(78, 48, 34), bg, 0.10),
                    mix(rgb(114, 74, 48), bg, 0.10),
                    mix(rgb(152, 104, 66), bg, 0.10),
                ],
                cliff: rgb(40, 25, 19),
                rock: rgb(58, 48, 44),
                rock_lit: rgb(104, 88, 78),
                road: rgb(34, 28, 25),
                curb: rgb(150, 110, 74),
                rut: rgb(52, 42, 36),
                ink: rgb(14, 12, 10),
                text: IVORY,
                muted: rgb(166, 158, 140),
                accent: theme.accent,
                accent_text: theme.accent_text(),
                surface: bg,
                card: mix(bg, rgb(255, 255, 255), 0.05),
                card_hi: mix(bg, rgb(255, 255, 255), 0.10),
                line: mix(bg, BRASS, 0.28),
                metal: rgb(54, 56, 52),
                metal_lit: rgb(104, 104, 96),
                enemy: rgb(236, 228, 208),
                enemy_shade: rgb(150, 140, 122),
                air: rgb(178, 212, 246),
                cold: rgb(130, 206, 246),
                good: rgb(132, 204, 118),
                bad: rgb(240, 104, 84),
                gold: rgb(236, 190, 92),
            }
        }
    }
}

/// World-to-screen transform for a letterboxed 20 × 12 board.
#[derive(Clone, Copy)]
pub struct View {
    pub rect: Rect,
    pub cell: f32,
}
impl View {
    pub fn fit(avail: Rect) -> Self {
        let cell = (avail.width() / COLS as f32)
            .min(avail.height() / ROWS as f32)
            .max(4.);
        let size = Vec2::new(cell * COLS as f32, cell * ROWS as f32);
        Self {
            rect: Rect::from_center_size(avail.center(), size),
            cell,
        }
    }
    pub fn pt(&self, p: P) -> Pos2 {
        self.rect.min + Vec2::new(p.x as f32, p.y as f32) * (self.cell / CELL as f32)
    }
    pub fn cell_rect(&self, x: i32, y: i32) -> Rect {
        Rect::from_min_size(
            self.rect.min + Vec2::new(x as f32, y as f32) * self.cell,
            Vec2::splat(self.cell),
        )
    }
    pub fn cell_at(&self, pos: Pos2) -> Option<(i32, i32)> {
        if !self.rect.contains(pos) {
            return None;
        }
        let v = (pos - self.rect.min) / self.cell;
        Some((
            (v.x as i32).clamp(0, COLS - 1),
            (v.y as i32).clamp(0, ROWS - 1),
        ))
    }
    pub fn len(&self, units: i32) -> f32 {
        units as f32 * self.cell / CELL as f32
    }
}

pub fn around(c: Pos2, r: f32, n: usize, turn: f32) -> Vec<Pos2> {
    (0..n)
        .map(|i| {
            let a = turn + i as f32 * TAU / n as f32;
            c + Vec2::new(a.cos(), a.sin()) * r
        })
        .collect()
}
fn poly(p: &Painter, points: Vec<Pos2>, fill: Color32, stroke: Stroke) {
    p.add(Shape::convex_polygon(points, fill, stroke));
}
/// Stable small hash for cosmetic variation (never used by the simulation).
pub fn hash(a: i32, b: i32, c: i32) -> u32 {
    let mut h = (a as u32).wrapping_mul(0x9E37_79B1)
        ^ (b as u32).wrapping_mul(0x85EB_CA77)
        ^ (c as u32).wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 12)
}
fn unit(h: u32) -> f32 {
    (h % 1000) as f32 / 1000.
}
fn rotate(v: Vec2, dir: Vec2) -> Vec2 {
    // Rotate a local vector (x forward, y right) into the direction `dir`.
    Vec2::new(v.x * dir.x - v.y * dir.y, v.x * dir.y + v.y * dir.x)
}

/// Mesa plateaus from smooth deterministic waves, varied per map.
pub fn terrace(map: usize, x: i32, y: i32) -> usize {
    let s = map as f32 * 1.7;
    let (fx, fy) = (x as f32, y as f32);
    let f = (fx * 0.42 + s).sin()
        + (fy * 0.63 + s * 1.3).sin()
        + 0.7 * ((fx + fy) * 0.27 + s * 0.4).sin();
    if f < -0.55 {
        0
    } else if f < 0.75 {
        1
    } else {
        2
    }
}

/// Soft edge darkening built from a vertex-coloured mesh.
pub fn vignette(p: &Painter, r: Rect, depth: f32, colour: Color32) {
    let inner = r.shrink(depth);
    let clear = Color32::TRANSPARENT;
    let mut mesh = Mesh::default();
    let outer = [
        r.left_top(),
        r.right_top(),
        r.right_bottom(),
        r.left_bottom(),
    ];
    let inn = [
        inner.left_top(),
        inner.right_top(),
        inner.right_bottom(),
        inner.left_bottom(),
    ];
    for i in 0..4 {
        let j = (i + 1) % 4;
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(outer[i], colour);
        mesh.colored_vertex(outer[j], colour);
        mesh.colored_vertex(inn[j], clear);
        mesh.colored_vertex(inn[i], clear);
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
    }
    p.add(Shape::mesh(mesh));
}

pub fn terrain(p: &Painter, v: View, map: &Map, index: usize, pal: &Palette) {
    let c = v.cell;
    for y in 0..ROWS {
        for x in 0..COLS {
            let r = v.cell_rect(x, y);
            let level = terrace(index, x, y);
            let h = hash(x, y, index as i32);
            // Gentle per-cell tonal variation keeps plateaus from looking flat.
            let tone = pal.ground[level].gamma_multiply(0.94 + 0.08 * unit(h));
            p.rect_filled(r.expand(0.4), 0., tone);
            let kind = h % 7;
            if kind == 0 && map.terrain(x, y) == Terrain::Build {
                // Hairline crack.
                let a =
                    r.min + Vec2::new(unit(h >> 3), unit(h >> 9)) * c * 0.8 + Vec2::splat(c * 0.1);
                let b = a + Vec2::new(c * 0.22, c * (unit(h >> 14) - 0.5) * 0.3);
                let d = b + Vec2::new(c * 0.16, c * (unit(h >> 19) - 0.5) * 0.3);
                p.add(Shape::line(
                    vec![a, b, d],
                    Stroke::new(c * 0.025, pal.cliff.gamma_multiply(0.45)),
                ));
            } else if kind == 1 && map.terrain(x, y) == Terrain::Build {
                // Dry scrub.
                let at =
                    r.min + Vec2::new(0.25 + 0.5 * unit(h >> 5), 0.25 + 0.5 * unit(h >> 11)) * c;
                let scrub = if pal.light {
                    Color32::from_rgb(118, 118, 72)
                } else {
                    Color32::from_rgb(92, 96, 58)
                };
                for (dx, dy, rr) in [(-0.07, 0.02, 0.07), (0.05, -0.02, 0.08), (0.0, 0.05, 0.06)] {
                    p.circle_filled(at + Vec2::new(dx, dy) * c, rr * c, scrub);
                }
                p.circle_filled(
                    at + Vec2::new(0.03, -0.04) * c,
                    0.03 * c,
                    scrub.lerp_to_gamma(IVORY, 0.25),
                );
            }
            for k in 0..2 {
                let hk = hash(x, y, k + 17 * index as i32);
                let at = r.min + Vec2::new(unit(hk), unit(hk >> 10)) * c;
                let col = if hk & 1 == 0 {
                    pal.cliff.gamma_multiply(0.3)
                } else {
                    pal.ground[2].lerp_to_gamma(IVORY, 0.15).gamma_multiply(0.7)
                };
                p.circle_filled(at, c * (0.015 + 0.015 * unit(hk >> 4)), col);
            }
        }
    }
    // Cliff faces: where a plateau steps down to the south or east.
    for y in 0..ROWS {
        for x in 0..COLS {
            let r = v.cell_rect(x, y);
            let level = terrace(index, x, y);
            if y + 1 < ROWS && terrace(index, x, y + 1) < level {
                let face = Rect::from_min_max(
                    Pos2::new(r.left(), r.bottom() - c * 0.15),
                    r.right_bottom(),
                );
                p.rect_filled(face, 0., pal.cliff);
                p.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(face.left(), face.center().y),
                        face.right_bottom(),
                    ),
                    0.,
                    pal.cliff.gamma_multiply(0.8),
                );
                p.line_segment(
                    [face.left_top(), face.right_top()],
                    Stroke::new(c * 0.03, pal.ground[2].gamma_multiply(0.8)),
                );
            }
            if x + 1 < COLS && terrace(index, x + 1, y) < level {
                p.rect_filled(
                    Rect::from_min_max(Pos2::new(r.right() - c * 0.08, r.top()), r.right_bottom()),
                    0.,
                    pal.cliff.gamma_multiply(0.7),
                );
            }
            if y > 0 && terrace(index, x, y - 1) < level {
                p.line_segment(
                    [r.left_top(), r.right_top()],
                    Stroke::new(c * 0.03, pal.ground[2].lerp_to_gamma(IVORY, 0.25)),
                );
            }
        }
    }
    roads(p, v, map, pal);
    for y in 0..ROWS {
        for x in 0..COLS {
            if map.terrain(x, y) == Terrain::Rock {
                boulders(p, v.cell_rect(x, y), hash(x, y, 91 + index as i32), pal);
            }
        }
    }
    sunlight(p, v.rect, pal);
    lanes(p, v, map, pal);
    for route in &map.routes {
        let start = v.pt(route.points[0]);
        let dir = (v.pt(route.points[1]) - start).normalized();
        if route.air {
            beacon(p, start, c, pal);
        } else {
            cave(p, start + dir * c * 0.05, dir, c, pal);
        }
    }
    let mut exits: Vec<Pos2> = map
        .routes
        .iter()
        .map(|r| v.pt(*r.points.last().unwrap()))
        .collect();
    exits.sort_by(|a, b| (a.x, a.y).partial_cmp(&(b.x, b.y)).unwrap());
    exits.dedup_by(|a, b| a.distance(*b) < 1.);
    for at in exits {
        gate(p, at, c, pal);
    }
}

fn roads(p: &Painter, v: View, map: &Map, pal: &Palette) {
    let c = v.cell;
    let ground: Vec<Vec<Pos2>> = map
        .ground_routes()
        .map(|(_, r)| r.points.iter().map(|q| v.pt(*q)).collect())
        .collect();
    // Cast shadow, then layered strokes with round joins: curb, bed, worn ruts.
    let shadow = Vec2::new(c * 0.05, c * 0.09);
    for pts in &ground {
        for w in pts.windows(2) {
            p.line_segment(
                [w[0] + shadow, w[1] + shadow],
                Stroke::new(c, Color32::from_black_alpha(60)),
            );
        }
    }
    for (width, colour) in [(0.96, pal.curb), (0.8, pal.road)] {
        for pts in &ground {
            for w in pts.windows(2) {
                p.line_segment([w[0], w[1]], Stroke::new(c * width, colour));
            }
            for q in pts {
                p.circle_filled(*q, c * width / 2., colour);
            }
        }
    }
    for pts in &ground {
        for w in pts.windows(2) {
            let d = (w[1] - w[0]).normalized();
            let side = d.rot90() * c * 0.17;
            for s in [side, -side] {
                p.add(Shape::dashed_line(
                    &[w[0] + s, w[1] + s],
                    Stroke::new(c * 0.045, pal.rut),
                    c * 0.5,
                    c * 0.16,
                ));
            }
        }
    }
}

fn lanes(p: &Painter, v: View, map: &Map, pal: &Palette) {
    let c = v.cell;
    for (_, route) in map.air_routes() {
        let pts: Vec<Pos2> = route.points.iter().map(|q| v.pt(*q)).collect();
        for w in pts.windows(2) {
            p.line_segment(
                [w[0], w[1]],
                Stroke::new(c * 0.28, pal.air.gamma_multiply(0.06)),
            );
            p.add(Shape::dashed_line(
                &[w[0], w[1]],
                Stroke::new(c * 0.035, pal.air.gamma_multiply(0.5)),
                c * 0.08,
                c * 0.22,
            ));
        }
        let len = route.length();
        let mut d = CELL as u32 * 2;
        while d + 200 < len {
            let a = v.pt(route.at(d));
            let dir = (v.pt(route.at(d + 200)) - a).normalized();
            let side = dir.rot90() * c * 0.16;
            let back = a - dir * c * 0.18;
            p.add(Shape::line(
                vec![back + side, a, back - side],
                Stroke::new(c * 0.05, pal.air.gamma_multiply(0.7)),
            ));
            d += CELL as u32 * 3;
        }
    }
}

fn boulders(p: &Painter, r: Rect, h: u32, pal: &Palette) {
    let c = r.width();
    let count = 1 + (h % 3) as usize;
    for k in 0..count {
        let hk = hash(h as i32, k as i32, 3);
        let s = c * (0.2 + 0.13 * unit(hk));
        let at = r.center() + Vec2::new(unit(hk >> 8) - 0.5, unit(hk >> 16) - 0.5) * c * 0.35;
        p.circle_filled(
            at + Vec2::new(s * 0.3, s * 0.45),
            s * 0.95,
            Color32::from_black_alpha(55),
        );
        let body: Vec<Pos2> = (0..9)
            .map(|i| {
                let a = i as f32 * TAU / 9. + unit(hk >> 3);
                let rr = s * (0.82 + 0.22 * unit(hash(hk as i32, i, 1)));
                at + Vec2::new(a.cos() * rr, a.sin() * rr * 0.86)
            })
            .collect();
        poly(
            p,
            body,
            pal.rock,
            Stroke::new(c * 0.02, pal.ink.gamma_multiply(0.55)),
        );
        let lit: Vec<Pos2> = (0..7)
            .map(|i| {
                let a = i as f32 * TAU / 7.;
                at + Vec2::new(-s * 0.22, -s * 0.24)
                    + Vec2::new(a.cos() * s * 0.5, a.sin() * s * 0.36)
            })
            .collect();
        poly(p, lit, pal.rock_lit, Stroke::NONE);
        p.circle_filled(
            at + Vec2::new(-s * 0.35, -s * 0.38),
            s * 0.1,
            pal.rock_lit.lerp_to_gamma(IVORY, 0.4),
        );
    }
}

fn cave(p: &Painter, at: Pos2, dir: Vec2, c: f32, pal: &Palette) {
    let r = c * 0.52;
    let side = dir.rot90();
    let arch: Vec<Pos2> = (0..=12)
        .map(|i| {
            let a = -FRAC_PI_2 + i as f32 * std::f32::consts::PI / 12.;
            at + side * a.sin() * r - dir * a.cos() * r * 0.8
        })
        .collect();
    let mut fill = arch.clone();
    fill.push(at);
    p.add(Shape::convex_polygon(fill, pal.ink, Stroke::NONE));
    p.add(Shape::line(arch, Stroke::new(c * 0.08, pal.cliff)));
    p.circle_stroke(
        at + dir * c * 0.1,
        c * 0.2,
        Stroke::new(c * 0.04, pal.bad.gamma_multiply(0.8)),
    );
}

fn beacon(p: &Painter, at: Pos2, c: f32, pal: &Palette) {
    p.circle_filled(at, c * 0.3, pal.air.gamma_multiply(0.12));
    p.circle_stroke(
        at,
        c * 0.3,
        Stroke::new(c * 0.03, pal.air.gamma_multiply(0.6)),
    );
    poly(
        p,
        around(at, c * 0.14, 3, -FRAC_PI_2),
        pal.air,
        Stroke::NONE,
    );
}

fn gate(p: &Painter, at: Pos2, c: f32, pal: &Palette) {
    let s = c * 0.46;
    let keep = Rect::from_center_size(at, Vec2::splat(s * 1.7));
    p.rect_filled(
        keep.translate(Vec2::new(c * 0.05, c * 0.08)),
        2.,
        Color32::from_black_alpha(90),
    );
    p.rect_filled(keep, 2., pal.metal);
    p.rect_stroke(
        keep,
        2.,
        Stroke::new(c * 0.05, BRASS),
        egui::StrokeKind::Inside,
    );
    // Crenellations along the top edge.
    let n = 4;
    for i in 0..n {
        let w = keep.width() / (2 * n - 1) as f32;
        let x = keep.left() + w * (2 * i) as f32;
        p.rect_filled(
            Rect::from_min_size(Pos2::new(x, keep.top() - s * 0.28), Vec2::new(w, s * 0.3)),
            0.,
            BRASS,
        );
    }
    let door = Rect::from_center_size(at + Vec2::new(0., s * 0.25), Vec2::new(s * 0.7, s * 0.95));
    p.rect_filled(door, 1., pal.ink);
    p.line_segment(
        [door.center_top(), door.center_bottom()],
        Stroke::new(c * 0.03, BRASS.gamma_multiply(0.6)),
    );
}

/// Tower silhouettes: plate plus a kind-specific turret aimed along `aim`.
pub fn tower_icon(
    p: &Painter,
    c: Pos2,
    size: f32,
    kind: TowerKind,
    tier: u8,
    pal: &Palette,
    aim: Vec2,
) {
    let s = size * 0.5;
    let aim = if aim.length_sq() > 1e-6 {
        aim.normalized()
    } else {
        Vec2::new(0., -1.)
    };
    let plate = Rect::from_center_size(c, Vec2::splat(s * 1.7));
    let hue = kind_colour(kind, pal);
    p.circle_filled(c, s * 1.18, hue.gamma_multiply(0.14));
    p.rect_filled(
        plate.translate(Vec2::new(s * 0.1, s * 0.16)),
        s * 0.22,
        Color32::from_black_alpha(90),
    );
    p.rect_filled(plate, s * 0.22, pal.metal);
    // Kind colour on the rim and a lit top edge: each role reads at a glance.
    p.rect_stroke(
        plate,
        s * 0.22,
        Stroke::new((s * 0.09).max(1.2), hue),
        egui::StrokeKind::Inside,
    );
    p.line_segment(
        [
            plate.left_top() + Vec2::new(s * 0.3, s * 0.14),
            plate.right_top() + Vec2::new(-s * 0.3, s * 0.14),
        ],
        Stroke::new((s * 0.05).max(0.8), Color32::from_white_alpha(50)),
    );
    for corner in [
        plate.left_top(),
        plate.right_top(),
        plate.left_bottom(),
        plate.right_bottom(),
    ] {
        p.circle_filled(
            corner + (c - corner) * 0.16,
            (s * 0.05).max(0.8),
            BRASS.gamma_multiply(0.7),
        );
    }
    let lit = pal.metal_lit;
    let barrel = |from: f32, to: f32, off: f32, w: f32| {
        let side = aim.rot90() * off;
        p.line_segment(
            [c + side + aim * from, c + side + aim * to],
            Stroke::new(w, IVORY.gamma_multiply(0.92)),
        );
        p.line_segment(
            [c + side + aim * (to - w * 0.3), c + side + aim * to],
            Stroke::new(w * 1.25, BRASS),
        );
    };
    match kind {
        TowerKind::Cannon => {
            match tier {
                0 => barrel(0., s * 1.05, 0., s * 0.24),
                1 => barrel(0., s * 1.15, 0., s * 0.32),
                _ => {
                    barrel(0., s * 1.15, s * 0.16, s * 0.22);
                    barrel(0., s * 1.15, -s * 0.16, s * 0.22);
                }
            }
            p.circle_filled(c, s * 0.5, lit);
            p.circle_stroke(c, s * 0.5, Stroke::new((s * 0.06).max(1.), pal.ink));
            p.circle_filled(c - aim * s * 0.12, s * 0.2, BRASS);
        }
        TowerKind::Mortar => {
            poly(
                p,
                around(c, s * 0.68, 8, 0.39),
                lit,
                Stroke::new((s * 0.06).max(1.), pal.ink),
            );
            let bore = s * (0.36 + 0.05 * f32::from(tier));
            p.circle_filled(c, bore + s * 0.1, IVORY.gamma_multiply(0.9));
            for band in 0..=tier {
                p.circle_stroke(
                    c,
                    bore + s * (0.02 + 0.04 * f32::from(band)),
                    Stroke::new(s * 0.03, BRASS),
                );
            }
            p.circle_filled(c, bore * 0.72, pal.ink);
        }
        TowerKind::Flak => {
            let n = if tier >= 2 { 4 } else { 2 };
            for i in 0..n {
                let off = (i as f32 - (n - 1) as f32 / 2.) * s * 0.2;
                barrel(s * 0.1, s * (0.95 + 0.08 * f32::from(tier)), off, s * 0.13);
            }
            let tri: Vec<Pos2> = [
                Vec2::new(-s * 0.35, -s * 0.55),
                Vec2::new(-s * 0.35, s * 0.55),
                Vec2::new(s * 0.45, 0.),
            ]
            .iter()
            .map(|v| c + rotate(*v, aim))
            .collect();
            poly(p, tri, lit, Stroke::new((s * 0.06).max(1.), pal.ink));
            p.circle_filled(c, s * 0.17, pal.air.lerp_to_gamma(BRASS, 0.5));
        }
        TowerKind::Cryo => {
            poly(
                p,
                around(c, s * 0.66, 6, 0.),
                lit,
                Stroke::new((s * 0.06).max(1.), pal.ink),
            );
            p.circle_filled(c, s * 0.5, pal.cold.gamma_multiply(0.18));
            let shards = 1 + usize::from(tier);
            for i in 0..shards {
                let a = -FRAC_PI_2 + (i as f32 - (shards - 1) as f32 / 2.) * 0.7;
                let d = Vec2::new(a.cos(), a.sin());
                let tip = c + d * s * (0.7 + 0.08 * f32::from(tier));
                let side = d.rot90() * s * 0.16;
                poly(
                    p,
                    vec![
                        c + d * s * 0.05 + side,
                        tip,
                        c + d * s * 0.05 - side,
                        c - d * s * 0.12,
                    ],
                    pal.cold,
                    Stroke::new(1.0_f32, IVORY.gamma_multiply(0.8)),
                );
            }
        }
    }
    // Tier chevrons along the plate's lower edge.
    for i in 0..=tier {
        let x = (f32::from(i) - f32::from(tier) / 2.) * s * 0.34;
        let b = plate.center_bottom() + Vec2::new(x, -s * 0.12);
        p.add(Shape::line(
            vec![
                b + Vec2::new(-s * 0.1, -s * 0.07),
                b,
                b + Vec2::new(s * 0.1, -s * 0.07),
            ],
            Stroke::new((s * 0.06).max(1.), BRASS),
        ));
    }
}

pub fn tower(p: &Painter, v: View, b: &Battle, t: &Tower, pal: &Palette) {
    let c = v.pt(t.pos());
    let aim = b
        .shots
        .iter()
        .rev()
        .find(|s| s.from == t.pos())
        .map(|s| v.pt(s.to) - c)
        .unwrap_or(Vec2::new(0., -1.));
    tower_icon(p, c, v.cell, t.kind, t.tier, pal, aim);
}

/// Enemy silhouettes, oriented along travel: beetle crawler, dart scout,
/// plated tank, delta-winged glider and hexagonal hauler.
pub fn enemy_icon(
    p: &Painter,
    c: Pos2,
    size: f32,
    kind: EnemyKind,
    pal: &Palette,
    dir: Vec2,
    phase: f32,
) {
    let s = size;
    let dir = if dir.length_sq() > 1e-6 {
        dir.normalized()
    } else {
        Vec2::new(1., 0.)
    };
    let at = |x: f32, y: f32| c + rotate(Vec2::new(x, y) * s, dir);
    let fill = if kind == EnemyKind::Glider {
        pal.air
    } else {
        pal.enemy
    };
    let edge = Stroke::new((s * 0.12).max(1.), pal.ink);
    match kind {
        EnemyKind::Crawler => {
            let step = phase.sin() * 0.18;
            for (x, sgn) in [(-0.3, 1.), (0.05, -1.), (0.35, 1.)] {
                for side in [-1., 1.] {
                    let off = step * sgn * side;
                    p.line_segment(
                        [at(x, 0.3 * side), at(x + off, 0.78 * side)],
                        Stroke::new((s * 0.1).max(1.), pal.enemy_shade),
                    );
                }
            }
            let body: Vec<Pos2> = (0..10)
                .map(|i| {
                    let a = i as f32 * TAU / 10.;
                    at(a.cos() * 0.62, a.sin() * 0.44)
                })
                .collect();
            poly(p, body, fill, edge);
            p.line_segment(
                [at(-0.5, 0.), at(0.3, 0.)],
                Stroke::new((s * 0.06).max(0.8), pal.enemy_shade),
            );
            p.circle_filled(at(0.48, 0.), s * 0.16, pal.enemy_shade);
        }
        EnemyKind::Scout => {
            for k in 0..3 {
                let y = (k as f32 - 1.) * 0.28;
                p.line_segment(
                    [
                        at(-0.7, y),
                        at(-1.25 - 0.2 * (phase + k as f32).sin().abs(), y),
                    ],
                    Stroke::new((s * 0.06).max(0.8), fill.gamma_multiply(0.45)),
                );
            }
            let pts = vec![at(0.85, 0.), at(-0.55, 0.5), at(-0.3, 0.), at(-0.55, -0.5)];
            poly(p, vec![pts[0], pts[1], pts[2]], fill, Stroke::NONE);
            poly(p, vec![pts[0], pts[2], pts[3]], fill, Stroke::NONE);
            p.add(Shape::closed_line(pts, edge));
        }
        EnemyKind::Plated => {
            for side in [-1., 1.] {
                let tread = [at(-0.62, 0.52 * side), at(0.62, 0.52 * side)];
                p.line_segment(tread, Stroke::new(s * 0.22, pal.enemy_shade));
            }
            let hull = vec![
                at(0.6, -0.42),
                at(0.6, 0.42),
                at(-0.6, 0.42),
                at(-0.6, -0.42),
            ];
            poly(p, hull.clone(), fill, edge);
            let inner = vec![
                at(0.36, -0.22),
                at(0.36, 0.22),
                at(-0.36, 0.22),
                at(-0.36, -0.22),
            ];
            poly(
                p,
                inner,
                BRASS.gamma_multiply(0.85),
                Stroke::new((s * 0.07).max(0.8), pal.ink),
            );
        }
        EnemyKind::Glider => {
            let flap = phase.sin() * 0.08;
            let pts = vec![
                at(0.8, 0.),
                at(-0.45, 0.85 + flap),
                at(-0.2, 0.),
                at(-0.45, -0.85 - flap),
            ];
            poly(p, vec![pts[0], pts[1], pts[2]], fill, Stroke::NONE);
            poly(p, vec![pts[0], pts[2], pts[3]], fill, Stroke::NONE);
            p.add(Shape::closed_line(pts, edge));
            p.line_segment(
                [at(0.55, 0.), at(-0.15, 0.)],
                Stroke::new((s * 0.08).max(0.8), pal.ink.gamma_multiply(0.6)),
            );
        }
        EnemyKind::Hauler => {
            let bob = phase.sin() * 0.04;
            for x in [-0.55, 0., 0.55] {
                for side in [-1., 1.] {
                    p.circle_filled(at(x, (0.72 + bob) * side), s * 0.16, pal.enemy_shade);
                }
            }
            let hull: Vec<Pos2> = (0..6)
                .map(|i| {
                    let a = i as f32 * TAU / 6.;
                    at(a.cos() * 0.92, a.sin() * 0.7)
                })
                .collect();
            poly(p, hull, fill, edge);
            let deck: Vec<Pos2> = (0..6)
                .map(|i| {
                    let a = i as f32 * TAU / 6.;
                    at(a.cos() * 0.52, a.sin() * 0.38)
                })
                .collect();
            poly(
                p,
                deck,
                BRASS.gamma_multiply(0.8),
                Stroke::new((s * 0.08).max(0.8), pal.ink),
            );
            p.circle_filled(at(0.1, 0.), s * 0.12, pal.ink);
        }
    }
}

pub fn enemy(p: &Painter, v: View, b: &Battle, e: &Enemy, pal: &Palette) {
    let route = &b.data().routes[e.route];
    let d = e.distance();
    let mut c = v.pt(route.at(d));
    let dir = v.pt(route.at(d + 120)) - v.pt(route.at(d.saturating_sub(120)));
    let size = v.cell
        * if e.kind == EnemyKind::Hauler {
            0.42
        } else {
            0.3
        };
    let stats = e.kind.stats();
    let phase = b.tick as f32 * 0.35 + e.id as f32;
    if stats.air {
        p.circle_filled(
            c + Vec2::new(v.cell * 0.14, v.cell * 0.2),
            size * 0.55,
            Color32::from_black_alpha(70),
        );
        c -= Vec2::new(0., v.cell * 0.12);
    } else {
        p.circle_filled(
            c + Vec2::new(v.cell * 0.04, v.cell * 0.07),
            size * 0.8,
            Color32::from_black_alpha(60),
        );
    }
    enemy_icon(p, c, size, e.kind, pal, dir, phase);
    if e.slow_percent() > 0 {
        p.circle_filled(c, size * 1.15, pal.cold.gamma_multiply(0.16));
        for (i, pt) in around(c, size * 1.15, 6, phase * 0.2)
            .into_iter()
            .enumerate()
        {
            let out = (pt - c).normalized();
            p.line_segment([pt, pt + out * size * 0.35], Stroke::new(1.5_f32, pal.cold));
            if i % 2 == 0 {
                p.circle_filled(pt + out * size * 0.4, 1.4, pal.cold);
            }
        }
    }
    if e.hp < e.max_hp {
        let w = v.cell * 0.62;
        let h = (v.cell * 0.075).max(2.5);
        let bar = Rect::from_min_size(
            c + Vec2::new(-w / 2., -size - v.cell * 0.22),
            Vec2::new(w, h),
        );
        p.rect_filled(bar.expand(1.), h, Color32::from_black_alpha(170));
        let frac = e.hp as f32 / e.max_hp.max(1) as f32;
        let colour = if frac > 0.5 {
            pal.good
        } else if frac > 0.25 {
            pal.gold
        } else {
            pal.bad
        };
        p.rect_filled(
            Rect::from_min_size(bar.min, Vec2::new(w * frac, h)),
            h,
            colour,
        );
    }
}

pub fn shot(p: &Painter, v: View, s: &Shot, pal: &Palette) {
    let a = v.pt(s.position());
    let dir = (v.pt(s.to) - v.pt(s.from)).normalized();
    match s.kind {
        TowerKind::Mortar => {
            let t = s.elapsed as f32 / s.flight.max(1) as f32;
            let lift = 4. * t * (1. - t) * v.cell * 0.9;
            let target = v.pt(s.to);
            p.circle_stroke(
                target,
                v.len(s.splash) * (0.35 + 0.65 * t),
                Stroke::new(1.2_f32, pal.bad.gamma_multiply(0.25 + 0.35 * t)),
            );
            p.circle_filled(
                a + Vec2::new(0., v.cell * 0.08),
                v.cell * (0.05 + 0.04 * (1. - lift / v.cell)),
                Color32::from_black_alpha(90),
            );
            let shell = a - Vec2::new(0., lift);
            p.circle_filled(shell, v.cell * 0.11, pal.ink);
            p.circle_stroke(shell, v.cell * 0.11, Stroke::new(1.2_f32, BRASS));
            p.circle_filled(
                shell - Vec2::new(v.cell * 0.03, v.cell * 0.03),
                v.cell * 0.03,
                IVORY.gamma_multiply(0.7),
            );
        }
        TowerKind::Cryo => {
            p.line_segment(
                [a - dir * v.cell * 0.35, a],
                Stroke::new(v.cell * 0.05, pal.cold.gamma_multiply(0.4)),
            );
            poly(
                p,
                vec![
                    a + dir * v.cell * 0.12,
                    a + dir.rot90() * v.cell * 0.06,
                    a - dir * v.cell * 0.08,
                    a - dir.rot90() * v.cell * 0.06,
                ],
                pal.cold,
                Stroke::NONE,
            );
        }
        TowerKind::Flak => {
            p.line_segment(
                [a - dir * v.cell * 0.25, a],
                Stroke::new(v.cell * 0.04, pal.gold.gamma_multiply(0.5)),
            );
            p.circle_filled(a, v.cell * 0.05, pal.gold);
        }
        TowerKind::Cannon => {
            p.line_segment(
                [a - dir * v.cell * 0.4, a],
                Stroke::new(v.cell * 0.06, IVORY.gamma_multiply(0.35)),
            );
            p.circle_filled(a, v.cell * 0.07, if pal.light { pal.ink } else { IVORY });
        }
    }
}

/// Range overlay: faint fill, a crisp ring, and a dashed minimum-range ring.
pub fn range(p: &Painter, v: View, c: P, stats: &TowerStats, colour: Color32) {
    let at = v.pt(c);
    let r = v.len(stats.range);
    p.circle_filled(at, r, colour.gamma_multiply(0.07));
    p.circle_stroke(at, r, Stroke::new(1.6_f32, colour.gamma_multiply(0.9)));
    p.circle_stroke(
        at,
        r - 3.,
        Stroke::new(1.0_f32, colour.gamma_multiply(0.25)),
    );
    if stats.min_range > 0 {
        let m = v.len(stats.min_range);
        let mut ring = around(at, m, 48, 0.);
        ring.push(ring[0]);
        p.add(Shape::dashed_line(
            &ring,
            Stroke::new(1.3_f32, colour.gamma_multiply(0.85)),
            5.,
            4.,
        ));
        p.circle_filled(at, m, Color32::from_black_alpha(22));
    }
}

/// Cosmetic effects. Ages are 0..1; callers drop finished effects.
pub fn burst(p: &Painter, at: Pos2, cell: f32, age: f32, seed: u32, colour: Color32) {
    for k in 0..8 {
        let hk = hash(seed as i32, k, 5);
        let a = unit(hk) * TAU;
        let speed = 0.5 + unit(hk >> 9) * 0.6;
        let q = at + Vec2::new(a.cos(), a.sin()) * cell * speed * age;
        p.circle_filled(q, cell * 0.05 * (1. - age), colour.gamma_multiply(1. - age));
    }
    p.circle_stroke(
        at,
        cell * (0.15 + 0.45 * age),
        Stroke::new(1.5_f32, colour.gamma_multiply(0.6 * (1. - age))),
    );
}
pub fn flash(p: &Painter, at: Pos2, dir: Vec2, cell: f32, age: f32, colour: Color32) {
    let tip = at + dir * cell * 0.62;
    p.circle_filled(
        tip,
        cell * 0.16 * (1. - age),
        colour.gamma_multiply(0.9 * (1. - age)),
    );
    p.circle_filled(
        tip,
        cell * 0.28 * (1. - age),
        colour.gamma_multiply(0.25 * (1. - age)),
    );
}
pub fn ring(p: &Painter, at: Pos2, radius: f32, age: f32, colour: Color32) {
    p.circle_filled(
        at,
        radius * (0.3 + 0.7 * age),
        colour.gamma_multiply(0.12 * (1. - age)),
    );
    p.circle_stroke(
        at,
        radius * (0.3 + 0.7 * age),
        Stroke::new(2.0_f32, colour.gamma_multiply(1. - age)),
    );
}

/// Lightweight map preview for the campaign screen: plateaus, rock, roads,
/// flight lanes and gates in a few hundred shapes.
pub fn thumbnail(p: &Painter, v: View, map: &Map, index: usize, pal: &Palette) {
    let c = v.cell;
    for y in 0..ROWS {
        for x in 0..COLS {
            let r = v.cell_rect(x, y);
            let level = terrace(index, x, y);
            p.rect_filled(r.expand(0.5), 0., pal.ground[level]);
            if y + 1 < ROWS && terrace(index, x, y + 1) < level {
                p.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(r.left(), r.bottom() - c * 0.22),
                        r.right_bottom(),
                    ),
                    0.,
                    pal.cliff,
                );
            }
            if map.terrain(x, y) == Terrain::Rock {
                poly(
                    p,
                    around(r.center(), c * 0.34, 5, 0.3),
                    pal.rock,
                    Stroke::NONE,
                );
            }
        }
    }
    for (_, route) in map.ground_routes() {
        let pts: Vec<Pos2> = route.points.iter().map(|q| v.pt(*q)).collect();
        for (width, colour) in [(0.9, pal.curb), (0.7, pal.road)] {
            for w in pts.windows(2) {
                p.line_segment([w[0], w[1]], Stroke::new(c * width, colour));
            }
            for q in &pts {
                p.circle_filled(*q, c * width / 2., colour);
            }
        }
    }
    for (_, route) in map.air_routes() {
        let pts: Vec<Pos2> = route.points.iter().map(|q| v.pt(*q)).collect();
        for w in pts.windows(2) {
            p.add(Shape::dashed_line(
                &[w[0], w[1]],
                Stroke::new(c * 0.12, pal.air.gamma_multiply(0.8)),
                c * 0.35,
                c * 0.3,
            ));
        }
    }
    for route in &map.routes {
        let end = v.pt(*route.points.last().unwrap());
        p.rect_filled(Rect::from_center_size(end, Vec2::splat(c * 0.8)), 1., BRASS);
        let start = v.pt(route.points[0]);
        p.circle_filled(start, c * 0.3, if route.air { pal.air } else { pal.bad });
    }
    vignette(
        p,
        v.rect,
        c * 1.5,
        Color32::from_black_alpha(if pal.light { 30 } else { 90 }),
    );
}

/// Signature colour per tower role, used on rims, glows and cards.
pub fn kind_colour(kind: TowerKind, pal: &Palette) -> Color32 {
    let c = match kind {
        TowerKind::Cannon => Color32::from_rgb(232, 188, 104),
        TowerKind::Mortar => Color32::from_rgb(228, 114, 76),
        TowerKind::Flak => Color32::from_rgb(118, 176, 236),
        TowerKind::Cryo => Color32::from_rgb(116, 214, 236),
    };
    if pal.light {
        c.lerp_to_gamma(Color32::BLACK, 0.3)
    } else {
        c
    }
}

/// Low sun from the upper left: warm light fading to shadow at lower right.
pub fn sunlight(p: &Painter, r: Rect, pal: &Palette) {
    let warm = if pal.light {
        Color32::from_white_alpha(40)
    } else {
        Color32::from_rgba_unmultiplied(255, 220, 170, 22)
    };
    let shade = Color32::from_black_alpha(if pal.light { 30 } else { 70 });
    let clear = Color32::TRANSPARENT;
    let mut mesh = Mesh::default();
    mesh.colored_vertex(r.left_top(), warm);
    mesh.colored_vertex(r.right_top(), clear);
    mesh.colored_vertex(r.right_bottom(), shade);
    mesh.colored_vertex(r.left_bottom(), clear);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(Shape::mesh(mesh));
}
