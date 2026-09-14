use super::*;
use eframe::egui::{Color32, FontId, Mesh, Shape, Stroke};
fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgb(
        (f32::from(a.r()) * (1.0 - t) + f32::from(b.r()) * t) as u8,
        (f32::from(a.g()) * (1.0 - t) + f32::from(b.g()) * t) as u8,
        (f32::from(a.b()) * (1.0 - t) + f32::from(b.b()) * t) as u8,
    )
}
fn surface(p: &egui::Painter, rect: Rect, scale: f32, heights: &[f64], color: Color32) {
    let mut mesh = Mesh::default();
    for (x, &y) in heights.iter().enumerate() {
        mesh.colored_vertex(
            Pos2::new(
                rect.left() + x as f32 * scale,
                rect.top() + y as f32 * scale,
            ),
            color,
        );
        mesh.colored_vertex(
            Pos2::new(rect.left() + x as f32 * scale, rect.bottom()),
            color,
        );
    }
    for x in 0..heights.len() - 1 {
        let i = x as u32 * 2;
        mesh.add_triangle(i, i + 1, i + 2);
        mesh.add_triangle(i + 1, i + 3, i + 2);
    }
    p.add(Shape::mesh(mesh));
}
pub(super) fn field(app: &App, ui: &mut egui::Ui) -> (Rect, f32, egui::Response) {
    let available = ui.available_size();
    let scale = (available.x / 1000.0).min(available.y / 600.0).max(0.01);
    let (outer, _) = ui.allocate_exact_size(available, Sense::hover());
    let rect = Rect::from_center_size(outer.center(), Vec2::new(1000.0, 600.0) * scale);
    let response = ui.interact(rect, ui.id().with("battlefield"), Sense::drag());
    let p = ui.painter_at(rect);
    let pt = |x: f64, y: f64| {
        Pos2::new(
            rect.left() + x as f32 * scale,
            rect.top() + y as f32 * scale,
        )
    };
    let bg = app.theme.background;
    let fg = app.theme.foreground;
    let accent = app.theme.accent;
    p.rect_filled(rect, 0.0, bg);
    // Angular ridgelines, drawn from a stable seed rather than gameplay randomness.
    for layer in 0..3 {
        let heights: Vec<f64> = (0..=1000)
            .map(|x| {
                let k = x / 100;
                let t = (x % 100) as f64 / 100.0;
                let h = |n: usize| {
                    let v = (app
                        .save
                        .game
                        .seed()
                        .wrapping_add((n + layer * 29) as u64)
                        .wrapping_mul(6364136223846793005)
                        >> 32)
                        % 70;
                    220.0 + layer as f64 * 45.0 + v as f64
                };
                h(k) + (h(k + 1) - h(k)) * t
            })
            .collect();
        surface(
            &p,
            rect,
            scale,
            &heights,
            mix(bg, accent, 0.04 + layer as f32 * 0.035),
        );
    }
    for x in (0..=1000).step_by(100) {
        p.line_segment(
            [pt(x as f64, 90.0), pt(x as f64, 600.0)],
            Stroke::new(0.5_f32, mix(bg, fg, 0.07)),
        );
    }
    let resolution = app.save.resolution.as_ref();
    let reduced = app.save.reduced_effects;
    let heights: Vec<f64> = app
        .save
        .game
        .terrain()
        .heights()
        .iter()
        .enumerate()
        .map(|(x, &y)| {
            resolution.map_or(y, |r| {
                r.impact.terrain_before[x]
                    + (y - r.impact.terrain_before[x]) * r.ground_progress(reduced)
            })
        })
        .collect();
    surface(
        &p,
        rect,
        scale,
        &heights,
        mix(bg, accent, if app.theme.light() { 0.3 } else { 0.24 }),
    );
    // Subtle geological bands remain clipped by the real terrain surface.
    for depth in [24.0, 55.0, 96.0] {
        let points = (0..=1000)
            .step_by(4)
            .map(|x| pt(x as f64, (heights[x] + depth).min(600.0)))
            .collect();
        p.add(Shape::line(
            points,
            Stroke::new(0.7_f32, mix(bg, accent, 0.35)),
        ));
    }
    p.add(Shape::line(
        heights
            .iter()
            .enumerate()
            .map(|(x, &y)| pt(x as f64, y))
            .collect(),
        Stroke::new(2.0_f32, accent),
    ));
    for (i, trace) in app.save.game.traces().iter().enumerate() {
        let c = mix(bg, if i == 0 { accent } else { fg }, 0.38);
        for pair in trace.windows(2).step_by(3) {
            p.line_segment(
                [pt(pair[0].x, pair[0].y), pt(pair[1].x, pair[1].y)],
                Stroke::new(1.0_f32, c),
            );
        }
    }
    for (i, tank) in app.save.game.tanks().iter().enumerate() {
        let x = tank.position.x;
        let y = resolution.map_or(tank.position.y, |r| {
            let old = r.impact.tanks_before[i].position.y;
            old + (tank.position.y - old) * r.tank_progress(reduced)
        });
        let ink = if i == 0 { accent } else { fg };
        let track = mix(bg, ink, 0.7);
        let a = f64::from(tank.angle).to_radians();
        let recoil = if !reduced && i == app.save.game.active() {
            app.save
                .game
                .projectile()
                .map_or(0.0, |s| 3.0 * (1.0 - f64::from(s.ticks) / 18.0).max(0.0))
        } else {
            0.0
        };
        p.add(Shape::convex_polygon(
            vec![
                pt(x - 15.0, y - 4.0),
                pt(x - 11.0, y - 10.0),
                pt(x + 11.0, y - 10.0),
                pt(x + 15.0, y - 4.0),
                pt(x + 11.0, y + 1.0),
                pt(x - 11.0, y + 1.0),
            ],
            track,
            Stroke::NONE,
        ));
        for dx in [-9.0, -3.0, 3.0, 9.0] {
            p.circle_filled(pt(x + dx, y - 4.0), 2.0 * scale, bg);
        }
        p.rect_filled(
            Rect::from_min_max(pt(x - 10.0, y - 14.0), pt(x + 10.0, y - 8.0)),
            0.0,
            ink,
        );
        p.rect_filled(
            Rect::from_min_max(
                pt(x - 6.0 - recoil * a.cos(), y - 18.0),
                pt(x + 6.0 - recoil * a.cos(), y - 12.0),
            ),
            0.0,
            ink,
        );
        if i == 1 {
            p.line_segment(
                [pt(x - 4.0, y - 16.0), pt(x + 4.0, y - 16.0)],
                Stroke::new(1.0_f32, bg),
            );
        }
        p.line_segment(
            [
                pt(x, y - 7.0),
                pt(
                    x + (20.0 - recoil) * a.cos(),
                    y - 7.0 - (20.0 - recoil) * a.sin(),
                ),
            ],
            Stroke::new(3.2 * scale, ink),
        );
        if tank.health == 0 {
            p.line_segment(
                [pt(x - 13.0, y - 21.0), pt(x + 13.0, y + 2.0)],
                Stroke::new(2.0_f32, fg),
            );
        }
        p.text(
            pt(x, y - 34.0),
            Align2::CENTER_CENTER,
            if i == 0 {
                "P1"
            } else if app.save.mode == Mode::Solo {
                "CPU"
            } else {
                "P2"
            },
            FontId::monospace((14.0 * scale).max(10.0)),
            ink,
        );
        let active = resolution.map_or(app.save.game.active(), |r| r.impact.shooter);
        if i == active {
            p.add(Shape::convex_polygon(
                vec![
                    pt(x - 5.0, y - 58.0),
                    pt(x + 5.0, y - 58.0),
                    pt(x, y - 51.0),
                ],
                ink,
                Stroke::NONE,
            ));
        }
        if i == active && app.save.game.phase() == Phase::Aiming && resolution.is_none() {
            let points = (5..=175)
                .step_by(5)
                .map(|d| {
                    let a = (d as f64).to_radians();
                    pt(x + 44.0 * a.cos(), y - 7.0 - 44.0 * a.sin())
                })
                .collect();
            p.add(Shape::line(points, Stroke::new(0.7_f32, mix(bg, ink, 0.5))));
            for d in [15.0_f64, 45.0, 90.0, 135.0, 165.0] {
                let a = d.to_radians();
                p.line_segment(
                    [
                        pt(x + 41.0 * a.cos(), y - 7.0 - 41.0 * a.sin()),
                        pt(x + 47.0 * a.cos(), y - 7.0 - 47.0 * a.sin()),
                    ],
                    Stroke::new(0.8_f32, ink),
                );
            }
            p.circle_filled(
                pt(x + 44.0 * a.cos(), y - 7.0 - 44.0 * a.sin()),
                2.8 * scale,
                ink,
            );
        }
    }
    if let Some(shot) = app.save.game.projectile() {
        if !reduced && shot.ticks < 14 {
            let tank = &app.save.game.tanks()[app.save.game.active()];
            let a = f64::from(tank.angle).to_radians();
            let at = pt(
                tank.position.x + 20.0 * a.cos(),
                tank.position.y - 7.0 - 20.0 * a.sin(),
            );
            p.circle_filled(
                at,
                (7.0 - shot.ticks as f32 * 0.4) * scale,
                arcade_presentation::AMBER,
            );
        }
        let at = shot.position;
        if at.y < 0.0 {
            p.text(
                pt(at.x, 12.0),
                Align2::CENTER_TOP,
                format!("↑ {:.0}", -at.y),
                FontId::monospace(13.0),
                fg,
            );
        } else {
            if !reduced {
                let len = shot.velocity.x.hypot(shot.velocity.y).max(1.0);
                p.line_segment(
                    [
                        pt(
                            at.x - shot.velocity.x / len * 14.0,
                            at.y - shot.velocity.y / len * 14.0,
                        ),
                        pt(at.x, at.y),
                    ],
                    Stroke::new(2.0_f32, mix(bg, fg, 0.65)),
                );
            }
            p.circle_filled(pt(at.x, at.y), (3.0 * scale).max(2.0), fg);
        }
    }
    if let Some(r) = resolution {
        let t = f64::from(r.progress());
        let at = r.impact.at;
        let radius = r.impact.weapon.radius();
        let color = if r.impact.weapon == Weapon::Digger {
            accent
        } else {
            arcade_presentation::AMBER
        };
        if reduced {
            p.circle_stroke(
                pt(at.x, at.y),
                radius as f32 * scale,
                Stroke::new(1.5_f32, color),
            );
        } else if t < 0.6 {
            p.circle_stroke(
                pt(at.x, at.y),
                (radius * (t / 0.6).sqrt()) as f32 * scale,
                Stroke::new((3.0 * (1.0 - t / 0.6)) as f32, color),
            );
            if t < 0.16 {
                p.circle_filled(
                    pt(at.x, at.y),
                    ((1.0 - t / 0.16) * 14.0) as f32 * scale,
                    color,
                );
            }
            for n in 0..14 {
                let a = (n as f64 / 14.0) * std::f64::consts::TAU;
                let speed = radius * (0.7 + (n % 3) as f64 * 0.2);
                let x = at.x + a.cos() * speed * t * 2.0;
                let y = at.y - a.sin().abs() * speed * t * 2.0 + 90.0 * t * t;
                p.rect_filled(
                    Rect::from_center_size(pt(x, y), Vec2::splat((3.0 * (1.0 - t)) as f32 * scale)),
                    0.0,
                    color,
                );
            }
        }
        if t >= 0.65 || reduced {
            for n in 0..2 {
                let damage = r.impact.blast[n] + r.impact.fall[n];
                if damage > 0 {
                    let tank = &app.save.game.tanks()[n];
                    let label = if r.impact.fall[n] > 0 {
                        format!("−{damage}  ({} fall)", r.impact.fall[n])
                    } else {
                        format!("−{damage} blast")
                    };
                    p.text(
                        pt(
                            tank.position.x,
                            tank.position.y - 82.0 - if reduced { 0.0 } else { (t - 0.65) * 24.0 },
                        ),
                        Align2::CENTER_CENTER,
                        label,
                        FontId::monospace((16.0 * scale).max(12.0)),
                        fg,
                    );
                }
            }
        }
    }
    let wind = app.save.game.wind();
    let origin = pt(940.0, 38.0);
    p.line_segment(
        [origin, origin + Vec2::new(0.0, 45.0 * scale)],
        Stroke::new(1.5_f32, fg),
    );
    let flutter = if reduced {
        0.0
    } else {
        (app.visual_clock * (3.0 + wind.abs() / 8.0)).sin() * 3.0
    };
    let end = origin + Vec2::new((wind * 1.5) as f32 * scale, (8.0 + flutter) as f32 * scale);
    p.add(Shape::convex_polygon(
        vec![
            origin,
            origin + Vec2::new(0.0, 13.0 * scale),
            end + Vec2::new(0.0, 10.0 * scale),
            end,
        ],
        accent,
        Stroke::NONE,
    ));
    p.text(
        pt(900.0, 24.0),
        Align2::RIGHT_CENTER,
        format!(
            "WIND {} {:.0}",
            if wind < 0.0 {
                "←"
            } else if wind > 0.0 {
                "→"
            } else {
                "·"
            },
            wind.abs()
        ),
        FontId::monospace((15.0 * scale).max(11.0)),
        fg,
    );
    p.text(
        pt(24.0, 27.0),
        Align2::LEFT_CENTER,
        "READ THE WIND. CHANGE THE LANDSCAPE.",
        FontId::monospace((12.0 * scale).max(9.0)),
        mix(bg, fg, 0.6),
    );
    (rect, scale, response)
}
