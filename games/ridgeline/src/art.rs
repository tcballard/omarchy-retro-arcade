//! Original native geometry for Ridgeline. Read-only: nothing here changes play.
use crate::battle::{Battle, Enemy, Shot, Tower};
use crate::data::*;
use arcade_presentation::{BRASS, INK, IVORY};
use eframe::egui::{self, Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};

/// Colours derived from the active Omarchy theme so light themes stay legible.
#[derive(Clone, Copy)]
pub struct Palette {
    pub light: bool,
    pub ground: [Color32; 3],
    pub rock: Color32,
    pub road: Color32,
    pub road_edge: Color32,
    pub ink: Color32,
    pub text: Color32,
    pub accent: Color32,
    pub enemy: Color32,
    pub air: Color32,
    pub cold: Color32,
    pub good: Color32,
    pub bad: Color32,
}
impl Palette {
    pub fn new(theme: &arcade_platform::theme::Theme) -> Self {
        let light = theme.light();
        let sand = Color32::from_rgb(176, 124, 84);
        let base = theme.background;
        let mix = |a: Color32, b: Color32, t: f32| a.lerp_to_gamma(b, t);
        if light {
            Self {
                light,
                ground: [
                    mix(base, sand, 0.30),
                    mix(base, sand, 0.40),
                    mix(base, sand, 0.50),
                ],
                rock: mix(base, Color32::from_rgb(92, 74, 62), 0.75),
                road: mix(base, Color32::from_rgb(120, 96, 76), 0.18),
                road_edge: Color32::from_rgb(96, 70, 52),
                ink: Color32::from_rgb(34, 30, 26),
                text: theme.foreground,
                accent: theme.accent,
                enemy: Color32::from_rgb(40, 36, 34),
                air: Color32::from_rgb(38, 52, 84),
                cold: Color32::from_rgb(34, 104, 150),
                good: Color32::from_rgb(40, 110, 60),
                bad: Color32::from_rgb(170, 50, 40),
            }
        } else {
            Self {
                light,
                ground: [
                    mix(base, sand, 0.20),
                    mix(base, sand, 0.28),
                    mix(base, sand, 0.36),
                ],
                rock: mix(base, Color32::from_rgb(60, 50, 44), 0.8),
                road: mix(base, Color32::from_rgb(20, 16, 14), 0.55),
                road_edge: Color32::from_rgb(120, 88, 60),
                ink: INK,
                text: IVORY,
                accent: theme.accent,
                enemy: IVORY,
                air: Color32::from_rgb(200, 220, 240),
                cold: Color32::from_rgb(130, 200, 240),
                good: Color32::from_rgb(130, 200, 120),
                bad: Color32::from_rgb(235, 110, 90),
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

fn poly(points: &[Pos2], fill: Color32, stroke: Stroke) -> Shape {
    Shape::convex_polygon(points.to_vec(), fill, stroke)
}
fn around(c: Pos2, r: f32, n: usize, turn: f32) -> Vec<Pos2> {
    (0..n)
        .map(|i| {
            let a = turn + i as f32 * std::f32::consts::TAU / n as f32;
            c + Vec2::new(a.cos(), a.sin()) * r
        })
        .collect()
}

/// Stable per-cell terrace level: broad diagonal bands, varied per map.
fn terrace(map: usize, x: i32, y: i32) -> usize {
    ((x + 2 * y + map as i32 * 3).rem_euclid(15) / 5) as usize
}

pub fn terrain(p: &Painter, v: View, map: &Map, index: usize, pal: &Palette) {
    for y in 0..ROWS {
        for x in 0..COLS {
            let r = v.cell_rect(x, y);
            let level = terrace(index, x, y);
            p.rect_filled(r, 0., pal.ground[level]);
            // Terrace lips: a darker edge where the next cell steps down.
            if terrace(index, x + 1, y) < level {
                p.line_segment(
                    [r.right_top(), r.right_bottom()],
                    Stroke::new(v.cell * 0.06, pal.ground[0].gamma_multiply(0.7)),
                );
            }
            if terrace(index, x, y + 1) < level {
                p.line_segment(
                    [r.left_bottom(), r.right_bottom()],
                    Stroke::new(v.cell * 0.06, pal.ground[0].gamma_multiply(0.7)),
                );
            }
            match map.terrain(x, y) {
                Terrain::Rock => {
                    let c = r.center();
                    let s = v.cell * 0.42;
                    let pts = [
                        c + Vec2::new(-s, s * 0.6),
                        c + Vec2::new(-s * 0.7, -s * 0.4),
                        c + Vec2::new(-s * 0.1, -s * 0.9),
                        c + Vec2::new(s * 0.8, -s * 0.3),
                        c + Vec2::new(s, s * 0.6),
                    ];
                    p.add(poly(
                        &pts,
                        pal.rock,
                        Stroke::new(1.0_f32, pal.ink.gamma_multiply(0.6)),
                    ));
                    p.line_segment(
                        [pts[2], c + Vec2::new(0., s * 0.6)],
                        Stroke::new(1.0_f32, pal.ground[2].gamma_multiply(0.6)),
                    );
                }
                Terrain::Build => {
                    p.circle_filled(
                        r.center(),
                        (v.cell * 0.035).max(0.8),
                        pal.ink.gamma_multiply(0.25),
                    );
                }
                Terrain::Route => {}
            }
        }
    }
    for (_, route) in map.ground_routes() {
        let pts: Vec<Pos2> = route.points.iter().map(|q| v.pt(*q)).collect();
        p.add(Shape::line(
            pts.clone(),
            Stroke::new(v.cell * 0.92, pal.road_edge),
        ));
        p.add(Shape::line(
            pts.clone(),
            Stroke::new(v.cell * 0.78, pal.road),
        ));
        for w in pts.windows(2) {
            p.add(Shape::dashed_line(
                &[w[0], w[1]],
                Stroke::new(1.0_f32, pal.road_edge.gamma_multiply(0.6)),
                v.cell * 0.18,
                v.cell * 0.22,
            ));
        }
    }
    for (_, route) in map.air_routes() {
        let pts: Vec<Pos2> = route.points.iter().map(|q| v.pt(*q)).collect();
        for w in pts.windows(2) {
            p.add(Shape::dashed_line(
                &[w[0], w[1]],
                Stroke::new(1.2_f32, pal.air.gamma_multiply(0.55)),
                v.cell * 0.1,
                v.cell * 0.3,
            ));
        }
        // Chevrons show the flight direction along each lane.
        let len = route.length();
        let mut d = CELL as u32 * 2;
        while d + 200 < len {
            let a = v.pt(route.at(d));
            let b = v.pt(route.at(d + 200));
            let dir = (b - a).normalized();
            let side = dir.rot90() * v.cell * 0.14;
            p.line_segment(
                [a - dir * v.cell * 0.14 + side, a],
                Stroke::new(1.2_f32, pal.air.gamma_multiply(0.6)),
            );
            p.line_segment(
                [a - dir * v.cell * 0.14 - side, a],
                Stroke::new(1.2_f32, pal.air.gamma_multiply(0.6)),
            );
            d += CELL as u32 * 3;
        }
    }
    for route in &map.routes {
        let start = v.pt(route.points[0]);
        let end = v.pt(*route.points.last().unwrap());
        let s = v.cell * 0.28;
        // Spawn: open triangle. Exit: brass gate with a bar.
        p.add(Shape::closed_line(
            around(start, s, 3, -std::f32::consts::FRAC_PI_2).to_vec(),
            Stroke::new(1.5_f32, if route.air { pal.air } else { pal.bad }),
        ));
        let gate = Rect::from_center_size(end, Vec2::splat(v.cell * 0.7));
        p.rect_stroke(
            gate,
            1.,
            Stroke::new(2.0_f32, BRASS),
            egui::StrokeKind::Inside,
        );
        p.line_segment(
            [gate.left_center(), gate.right_center()],
            Stroke::new(2.0_f32, BRASS),
        );
    }
}

/// Distinct silhouettes: square cannon, round mortar, triangular flak, hex cryo.
pub fn tower_icon(
    p: &Painter,
    c: Pos2,
    size: f32,
    kind: TowerKind,
    tier: u8,
    pal: &Palette,
    aim: Vec2,
) {
    let s = size * 0.42;
    let body = if pal.light {
        Color32::from_rgb(86, 80, 70)
    } else {
        Color32::from_rgb(58, 60, 54)
    };
    let edge = Stroke::new((size * 0.05).max(1.), BRASS);
    p.circle_filled(
        c + Vec2::new(size * 0.05, size * 0.08),
        s,
        Color32::from_black_alpha(70),
    );
    let aim = if aim.length_sq() > 0. {
        aim.normalized()
    } else {
        Vec2::new(0., -1.)
    };
    match kind {
        TowerKind::Cannon => {
            p.rect_filled(Rect::from_center_size(c, Vec2::splat(s * 1.7)), 1., body);
            p.rect_stroke(
                Rect::from_center_size(c, Vec2::splat(s * 1.7)),
                1.,
                edge,
                egui::StrokeKind::Inside,
            );
            p.line_segment([c, c + aim * s * 1.25], Stroke::new(s * 0.42, IVORY));
            p.circle_filled(c, s * 0.45, BRASS);
        }
        TowerKind::Mortar => {
            p.add(poly(&around(c, s * 0.95, 8, 0.39), body, edge));
            p.circle_filled(c, s * 0.52, IVORY);
            p.circle_filled(c, s * 0.3, INK);
        }
        TowerKind::Flak => {
            p.add(poly(
                &around(c, s, 3, -std::f32::consts::FRAC_PI_2),
                body,
                edge,
            ));
            let side = aim.rot90() * s * 0.22;
            for off in [side, -side] {
                p.line_segment(
                    [c + off, c + off + aim * s * 1.1],
                    Stroke::new(s * 0.2, IVORY),
                );
            }
            p.circle_filled(c, s * 0.25, BRASS);
        }
        TowerKind::Cryo => {
            p.add(poly(&around(c, s * 0.95, 6, 0.), body, edge));
            let crystal = [
                c + Vec2::new(0., -s * 0.62),
                c + Vec2::new(s * 0.34, 0.),
                c + Vec2::new(0., s * 0.62),
                c + Vec2::new(-s * 0.34, 0.),
            ];
            p.add(poly(&crystal, pal.cold, Stroke::new(1.0_f32, IVORY)));
        }
    }
    // Tier pips below the base: one per upgrade level.
    for i in 0..=tier {
        let x = (f32::from(i) - f32::from(tier) / 2.) * s * 0.42;
        p.circle_filled(c + Vec2::new(x, s * 1.1), (s * 0.13).max(1.2), BRASS);
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

/// Shapes encode class: circle crawler, dart scout, plated square, winged
/// glider, hexagonal hauler. Armour adds a second outline; slow adds ice ticks.
pub fn enemy_icon(p: &Painter, c: Pos2, size: f32, kind: EnemyKind, pal: &Palette) {
    let s = size;
    let fill = if kind == EnemyKind::Glider {
        pal.air
    } else {
        pal.enemy
    };
    let edge = Stroke::new((s * 0.12).max(1.), pal.ink);
    match kind {
        EnemyKind::Crawler => {
            p.circle_filled(c, s * 0.55, fill);
            p.circle_stroke(c, s * 0.55, edge);
        }
        EnemyKind::Scout => {
            let pts = [
                c + Vec2::new(s * 0.75, 0.),
                c + Vec2::new(-s * 0.5, s * 0.45),
                c + Vec2::new(-s * 0.25, 0.),
                c + Vec2::new(-s * 0.5, -s * 0.45),
            ];
            p.add(Shape::convex_polygon(
                vec![pts[0], pts[1], pts[2]],
                fill,
                Stroke::NONE,
            ));
            p.add(Shape::convex_polygon(
                vec![pts[0], pts[2], pts[3]],
                fill,
                Stroke::NONE,
            ));
            p.add(Shape::closed_line(pts.to_vec(), edge));
        }
        EnemyKind::Plated => {
            let r = Rect::from_center_size(c, Vec2::splat(s * 1.15));
            p.rect_filled(r, 1., fill);
            p.rect_stroke(r, 1., edge, egui::StrokeKind::Middle);
            p.rect_stroke(
                r.shrink(s * 0.22),
                0.,
                Stroke::new((s * 0.1).max(1.), pal.ink),
                egui::StrokeKind::Middle,
            );
        }
        EnemyKind::Glider => {
            let pts = [
                c + Vec2::new(0., -s * 0.7),
                c + Vec2::new(s * 0.85, s * 0.45),
                c + Vec2::new(0., s * 0.15),
                c + Vec2::new(-s * 0.85, s * 0.45),
            ];
            p.add(Shape::convex_polygon(
                vec![pts[0], pts[1], pts[2]],
                fill,
                Stroke::NONE,
            ));
            p.add(Shape::convex_polygon(
                vec![pts[0], pts[2], pts[3]],
                fill,
                Stroke::NONE,
            ));
            p.add(Shape::closed_line(pts.to_vec(), edge));
        }
        EnemyKind::Hauler => {
            p.add(poly(&around(c, s * 0.85, 6, 0.), fill, edge));
            p.add(Shape::closed_line(
                around(c, s * 0.5, 6, 0.),
                Stroke::new((s * 0.1).max(1.), pal.ink),
            ));
        }
    }
}

pub fn enemy(p: &Painter, v: View, b: &Battle, e: &Enemy, pal: &Palette) {
    let mut c = v.pt(b.enemy_pos(e));
    let size = v.cell
        * if e.kind == EnemyKind::Hauler {
            0.42
        } else {
            0.3
        };
    let stats = e.kind.stats();
    if stats.air {
        // Aircraft float above their ground shadow.
        p.circle_filled(
            c + Vec2::new(0., v.cell * 0.12),
            size * 0.5,
            Color32::from_black_alpha(80),
        );
        c -= Vec2::new(0., v.cell * 0.18);
    }
    enemy_icon(p, c, size, e.kind, pal);
    if e.slow_percent() > 0 {
        for pt in around(c, size * 1.1, 6, 0.3) {
            p.line_segment([pt, pt + (pt - c) * 0.35], Stroke::new(1.5_f32, pal.cold));
        }
    }
    if e.hp == e.max_hp {
        // Undamaged units show no bar, so queued groups stay individually legible.
        return;
    }
    let w = v.cell * 0.6;
    let bar = Rect::from_min_size(
        c + Vec2::new(-w / 2., -size - v.cell * 0.2),
        Vec2::new(w, (v.cell * 0.08).max(2.)),
    );
    p.rect_filled(bar, 0., Color32::from_black_alpha(140));
    p.rect_filled(
        Rect::from_min_size(
            bar.min,
            Vec2::new(w * e.hp as f32 / e.max_hp.max(1) as f32, bar.height()),
        ),
        0.,
        pal.good,
    );
}

pub fn shot(p: &Painter, v: View, s: &Shot, pal: &Palette) {
    let a = v.pt(s.position());
    match s.kind {
        TowerKind::Mortar => {
            // Parabolic height: purely visual, the impact point is committed.
            let t = s.elapsed as f32 / s.flight.max(1) as f32;
            let lift = 4. * t * (1. - t) * v.cell * 0.9;
            p.circle_filled(
                a + Vec2::new(0., v.cell * 0.1),
                v.cell * 0.06,
                Color32::from_black_alpha(80),
            );
            p.circle_filled(a - Vec2::new(0., lift), v.cell * 0.1, pal.ink);
            p.circle_stroke(
                a - Vec2::new(0., lift),
                v.cell * 0.1,
                Stroke::new(1.0_f32, BRASS),
            );
            let target = v.pt(s.to);
            p.circle_stroke(
                target,
                v.len(s.splash),
                Stroke::new(1.0_f32, pal.bad.gamma_multiply(0.35)),
            );
        }
        TowerKind::Cryo => {
            p.circle_filled(a, v.cell * 0.07, pal.cold);
        }
        TowerKind::Flak => {
            p.circle_filled(a, v.cell * 0.05, pal.air.lerp_to_gamma(BRASS, 0.5));
        }
        TowerKind::Cannon => {
            p.circle_filled(a, v.cell * 0.07, if pal.light { pal.ink } else { IVORY });
        }
    }
}

/// Range overlay that never fills over enemies: outline plus faint tint.
pub fn range(p: &Painter, v: View, c: P, stats: &TowerStats, colour: Color32) {
    let at = v.pt(c);
    p.circle_filled(at, v.len(stats.range), colour.gamma_multiply(0.06));
    p.circle_stroke(
        at,
        v.len(stats.range),
        Stroke::new(1.5_f32, colour.gamma_multiply(0.8)),
    );
    if stats.min_range > 0 {
        p.add(Shape::dashed_line(
            &around(at, v.len(stats.min_range), 40, 0.)
                .into_iter()
                .chain(std::iter::once(at + Vec2::new(v.len(stats.min_range), 0.)))
                .collect::<Vec<_>>(),
            Stroke::new(1.2_f32, colour.gamma_multiply(0.8)),
            4.,
            4.,
        ));
    }
}
