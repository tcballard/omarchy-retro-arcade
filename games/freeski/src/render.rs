use crate::{
    engine::{Mode, Phase, Point, Sim},
    world::{self, Obstacle},
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Stroke, Vec2};
pub const VIEW_WIDTH: f64 = 96.;
pub const VIEW_HEIGHT: f64 = 96.;
pub fn field(area: Rect) -> Rect {
    let w = area.width().min(area.height());
    Rect::from_center_size(area.center(), Vec2::splat(w))
}
pub fn point(field: Rect, sim: &Sim, p: Point) -> Pos2 {
    Pos2::new(
        field.center().x + (p.x / VIEW_WIDTH) as f32 * field.width(),
        field.top()
            + field.height() * 0.12
            + ((p.y - sim.position.y) / VIEW_HEIGHT) as f32 * field.height(),
    )
}
pub struct Effects<'a> {
    pub tracks: &'a [(Point, Point)],
    pub braking: bool,
    pub landing: Option<(Point, u64)>,
}

pub fn draw(
    ui: &egui::Ui,
    field: Rect,
    state: &crate::storage::Save,
    obstacles: &[Obstacle],
    accent: Color32,
    effects: Effects<'_>,
) {
    let sim = &state.run;
    let reduced = state.reduced_effects;
    arcade_presentation::bezel(ui.painter(), field, accent);
    let p = ui.painter().with_clip_rect(field);
    let light = !ui.visuals().dark_mode;
    let snow = if light {
        Color32::from_rgb(239, 244, 242)
    } else {
        Color32::from_rgb(201, 220, 220)
    };
    let ink = Color32::from_rgb(40, 65, 70);
    let scale = field.width() / VIEW_WIDTH as f32;
    p.rect_filled(field, 0., snow);
    let at = |x, y| point(field, sim, Point { x, y });
    // Sparse shallow wind marks are world-anchored, faint, and bounded. They
    // add snow texture without resembling obstacles or covering the route.
    let drift_row = ((sim.position.y - 20.) / 16.).floor() as i64;
    for row in drift_row..drift_row + 9 {
        for column in 0..3i64 {
            let hash = row.wrapping_mul(37).wrapping_add(column * 71);
            let x = (hash.rem_euclid(78) - 39) as f64;
            let y = row as f64 * 16. + column as f64 * 3.;
            let pos = at(x, y);
            let shade = Color32::from_rgba_unmultiplied(95, 139, 149, 18);
            p.line_segment(
                [pos, pos + Vec2::new(2.4, -0.22) * scale],
                Stroke::new((scale * 0.10).max(0.6), shade),
            );
            p.line_segment(
                [
                    pos + Vec2::new(0.8, 0.25) * scale,
                    pos + Vec2::new(3.3, 0.02) * scale,
                ],
                Stroke::new((scale * 0.08).max(0.5), Color32::from_white_alpha(30)),
            );
        }
    }
    // A fixed world view reveals the same hazards at every window size.
    let start = ((sim.position.y - 20.) / 10.).floor() as i32;
    for row in start..start + 12 {
        let y = row as f64 * 10.;
        for side in [-1., 1.] {
            let a = at(side * world::HALF_WIDTH, y);
            p.line_segment(
                [a, a + Vec2::new(0., 5. * scale)],
                Stroke::new(scale * 0.22, Color32::from_rgb(136, 166, 173)),
            );
            p.line_segment(
                [
                    a + Vec2::new(-1.5 * scale, 0.),
                    a + Vec2::new(1.5 * scale, 0.),
                ],
                Stroke::new(scale * 0.13, ink),
            );
        }
    }
    if !reduced {
        for (a, b) in effects.tracks {
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let length = dx.hypot(dy).max(0.001);
            for offset in [-0.35, 0.35] {
                p.line_segment(
                    [
                        at(a.x + offset * dy / length, a.y - offset * dx / length),
                        at(b.x + offset * dy / length, b.y - offset * dx / length),
                    ],
                    Stroke::new((scale * 0.12).max(0.7), Color32::from_black_alpha(20)),
                );
            }
        }
    }
    for o in obstacles {
        let pos = point(field, sim, o.at);
        if !field.expand(10. * scale).contains(pos) {
            continue;
        }
        crate::artwork::obstacle(&p, pos, scale, o);
    }
    if state.mode == Mode::Slalom {
        if let Some(course) = crate::course::course(state.course_index) {
            for (index, gate) in course.gates.iter().enumerate() {
                let centre = at(gate.x, gate.y);
                if !field.expand(8. * scale).contains(centre) {
                    continue;
                }
                let color = if index < state.slalom.next_gate {
                    Color32::from_rgb(125, 145, 145)
                } else if index == state.slalom.next_gate {
                    Color32::from_rgb(25, 95, 180)
                } else {
                    Color32::from_rgb(180, 55, 50)
                };
                for side in [-1., 1.] {
                    let base = at(gate.x + side * gate.half_width, gate.y);
                    crate::artwork::gate(
                        &p,
                        base,
                        scale,
                        side as f32,
                        color,
                        index == state.slalom.next_gate,
                    );
                }
                if index >= state.slalom.next_gate {
                    p.text(
                        centre - Vec2::new(0., 2.5) * scale,
                        Align2::CENTER_CENTER,
                        format!(
                            "{}{}",
                            index + 1,
                            if index == state.slalom.next_gate {
                                "  NEXT"
                            } else {
                                ""
                            }
                        ),
                        FontId::monospace((1.5 * scale).max(10.)),
                        color,
                    );
                }
            }
        }
    }
    let finish_distance = if state.mode == Mode::Slalom {
        crate::course::course(state.course_index).map_or(world::FINISH, |c| c.length)
    } else {
        world::FINISH
    };
    let finish = at(0., finish_distance);
    if state.mode != Mode::FreeSki && field.expand(8. * scale).contains(finish) {
        for side in [-1., 1.] {
            crate::artwork::finish_post(
                &p,
                at(side * world::HALF_WIDTH, finish_distance),
                scale,
                side as f32,
            );
        }
        for row in 0..2 {
            for n in -16..16 {
                let x = n as f32 * 2.5 * scale;
                let r = Rect::from_min_size(
                    Pos2::new(finish.x + x, finish.y + row as f32 * 1.2 * scale),
                    Vec2::new(2.5, 1.2) * scale,
                );
                p.rect_filled(
                    r,
                    0.,
                    if (n + row) % 2 == 0 {
                        ink
                    } else {
                        Color32::WHITE
                    },
                );
            }
        }
        p.text(
            finish - Vec2::new(0., 4. * scale),
            Align2::CENTER_CENTER,
            if state.mode == Mode::Slalom {
                "SLALOM FINISH"
            } else {
                "PRACTICE FINISH"
            },
            FontId::monospace(1.7 * scale),
            ink,
        );
    }
    if state.chase_enabled && state.chase.phase == crate::chase::ChasePhase::Active {
        let creature = point(field, sim, state.chase.position);
        let separation = (state.chase.position.x - sim.position.x)
            .hypot(state.chase.position.y - sim.position.y);
        if !field.shrink(4. * scale).contains(creature) {
            let marker = Pos2::new(
                creature.x.clamp(field.left() + 65., field.right() - 65.),
                creature.y.clamp(field.top() + 18., field.bottom() - 18.),
            );
            p.rect_filled(
                Rect::from_center_size(marker, Vec2::new(122., 25.)),
                4.,
                Color32::from_rgb(38, 57, 66),
            );
            p.text(
                marker,
                Align2::CENTER_CENTER,
                format!("YETI {:.0} m", separation),
                FontId::monospace(12.),
                Color32::WHITE,
            );
        } else {
            crate::yeti::draw(
                &p,
                creature,
                scale,
                sim.ticks,
                state.chase.speed,
                state.chase.heading,
                reduced,
            );
        }
    }
    let ground = point(field, sim, sim.position);
    if !reduced {
        if let Some((pos, tick)) = effects.landing {
            crate::artwork::landing(
                &p,
                point(field, sim, pos),
                scale,
                sim.ticks.saturating_sub(tick),
            );
        }
    }
    crate::artwork::skier(&p, ground, scale, sim, effects.braking, reduced);
    if sim.phase == Phase::Ready {
        p.text(
            ground + Vec2::new(0., 5. * scale),
            Align2::CENTER_TOP,
            "YOUR FIRST TRACKS",
            FontId::monospace(2. * scale),
            ink,
        );
    }
    p.text(
        field.left_bottom() + Vec2::new(12., -10.),
        Align2::LEFT_BOTTOM,
        if state.mode == Mode::Practice {
            "PRACTICE / 1,200 m"
        } else if state.mode == Mode::Slalom {
            "SLALOM / FIND YOUR LINE"
        } else {
            "FREE SKI / KEEP GOING"
        },
        FontId::monospace(12.),
        ink,
    );
}
