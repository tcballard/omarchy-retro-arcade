//! Ridgeline's interface kit: painted, focusable widgets sharing one palette.
use crate::art::Palette;
use crate::storage::Medal;
use arcade_presentation::{BRASS, IVORY};
use eframe::egui::{
    self, Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Vec2,
};

pub const RADIUS: f32 = 4.;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}
pub fn text(size: f32) -> FontId {
    FontId::proportional(size)
}

/// A card surface; `lit` draws the selected brass treatment.
pub fn card(p: &Painter, r: Rect, pal: &Palette, hovered: bool, lit: bool) {
    p.rect_filled(
        r.translate(Vec2::new(0., 2.)),
        RADIUS,
        Color32::from_black_alpha(if pal.light { 18 } else { 60 }),
    );
    p.rect_filled(
        r,
        RADIUS,
        if hovered || lit {
            pal.card_hi
        } else {
            pal.card
        },
    );
    let stroke = if lit {
        Stroke::new(2_f32, BRASS)
    } else if hovered {
        Stroke::new(1_f32, BRASS.gamma_multiply(0.75))
    } else {
        Stroke::new(1_f32, pal.line)
    };
    p.rect_stroke(r, RADIUS, stroke, egui::StrokeKind::Inside);
    if lit {
        p.rect_filled(
            Rect::from_min_size(r.min + Vec2::new(0., 6.), Vec2::new(3., r.height() - 12.)),
            1.,
            pal.accent,
        );
    }
}

pub fn focus_ring(p: &Painter, r: Rect, pal: &Palette) {
    p.rect_stroke(
        r.expand(2.5),
        RADIUS + 2.,
        Stroke::new(2_f32, pal.accent),
        egui::StrokeKind::Outside,
    );
}

/// Small uppercase tag; returns its width so callers can lay out rows.
pub fn chip(p: &Painter, left_center: Pos2, label: &str, colour: Color32, pal: &Palette) -> f32 {
    let galley = p.layout_no_wrap(label.to_owned(), mono(9.5), colour);
    let size = galley.size() + Vec2::new(10., 4.);
    let r = Rect::from_min_size(left_center - Vec2::new(0., size.y / 2.), size);
    p.rect_filled(
        r,
        3.,
        colour.gamma_multiply(if pal.light { 0.12 } else { 0.16 }),
    );
    p.rect_stroke(
        r,
        3.,
        Stroke::new(1_f32, colour.gamma_multiply(0.55)),
        egui::StrokeKind::Inside,
    );
    p.galley(r.min + Vec2::new(5., 2.), galley, colour);
    size.x
}

/// Keyboard key cap, e.g. for hotkey hints.
pub fn keycap(p: &Painter, center: Pos2, key: &str, pal: &Palette) -> f32 {
    let galley = p.layout_no_wrap(key.to_owned(), mono(10.), pal.muted);
    let size = Vec2::new(galley.size().x.max(8.) + 8., 16.);
    let r = Rect::from_center_size(center, size);
    p.rect_filled(r.translate(Vec2::new(0., 1.5)), 3., pal.line);
    p.rect_filled(r, 3., pal.card_hi);
    p.rect_stroke(
        r,
        3.,
        Stroke::new(1_f32, pal.line),
        egui::StrokeKind::Inside,
    );
    p.galley(r.center() - galley.size() / 2., galley, pal.muted);
    size.x
}

/// Section heading in spaced small caps with a hairline rule.
pub fn section(ui: &mut egui::Ui, title: &str, hint: Option<&str>, pal: &Palette) {
    ui.add_space(6.);
    let (r, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 18.), Sense::hover());
    let p = ui.painter();
    let g = p.layout_no_wrap(title.to_owned(), mono(10.5), BRASS);
    let w = g.size().x;
    p.galley(
        Pos2::new(r.left(), r.center().y - g.size().y / 2.),
        g,
        BRASS,
    );
    let mut right = r.right();
    if let Some(h) = hint {
        let g = p.layout_no_wrap(h.to_owned(), mono(9.5), pal.muted);
        right -= g.size().x;
        p.galley(
            Pos2::new(right, r.center().y - g.size().y / 2.),
            g,
            pal.muted,
        );
        right -= 8.;
    }
    p.line_segment(
        [
            Pos2::new(r.left() + w + 8., r.center().y),
            Pos2::new(right, r.center().y),
        ],
        Stroke::new(1_f32, pal.line),
    );
    ui.add_space(2.);
}

/// A painted, focusable button; the label is its accessible name.
pub fn button(
    ui: &mut egui::Ui,
    label: &str,
    hint: Option<&str>,
    tone: Tone,
    size: Vec2,
    enabled: bool,
    pal: &Palette,
) -> Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (r, response) = ui.allocate_exact_size(size, sense);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, label));
    let p = ui.painter();
    let hot = enabled && (response.hovered() || response.has_focus());
    let down = response.is_pointer_button_down_on();
    let (fill, stroke, ink) = match tone {
        Tone::Primary => (
            if hot {
                pal.accent.lerp_to_gamma(IVORY, 0.15)
            } else {
                pal.accent
            },
            Stroke::new(1_f32, pal.accent.lerp_to_gamma(pal.ink, 0.3)),
            pal.accent_text,
        ),
        Tone::Danger => (
            if hot {
                pal.bad.lerp_to_gamma(IVORY, 0.12)
            } else {
                pal.bad
            },
            Stroke::new(1_f32, pal.bad.lerp_to_gamma(pal.ink, 0.3)),
            Color32::WHITE,
        ),
        Tone::Secondary => (
            if hot { pal.card_hi } else { pal.card },
            Stroke::new(
                1_f32,
                if hot {
                    BRASS
                } else {
                    BRASS.gamma_multiply(0.55)
                },
            ),
            pal.text,
        ),
        Tone::Ghost => (
            if hot {
                pal.card_hi
            } else {
                Color32::TRANSPARENT
            },
            Stroke::new(1_f32, if hot { pal.line } else { Color32::TRANSPARENT }),
            if hot { pal.text } else { pal.muted },
        ),
    };
    let (fill, ink) = if enabled {
        (fill, ink)
    } else {
        (pal.card.gamma_multiply(0.6), pal.muted.gamma_multiply(0.6))
    };
    let face = r.translate(Vec2::new(0., if down { 1. } else { 0. }));
    if tone != Tone::Ghost && enabled {
        p.rect_filled(
            r.translate(Vec2::new(0., 2.)),
            RADIUS,
            Color32::from_black_alpha(if pal.light { 25 } else { 70 }),
        );
    }
    p.rect_filled(face, RADIUS, fill);
    p.rect_stroke(
        face,
        RADIUS,
        if enabled {
            stroke
        } else {
            Stroke::new(1_f32, pal.line)
        },
        egui::StrokeKind::Inside,
    );
    let font = if matches!(tone, Tone::Primary | Tone::Danger) {
        text(14.)
    } else {
        text(13.)
    };
    let g = p.layout_no_wrap(label.to_owned(), font, ink);
    let hint_g = hint.map(|h| p.layout_no_wrap(h.to_owned(), mono(10.), ink.gamma_multiply(0.7)));
    let total = g.size().x + hint_g.as_ref().map_or(0., |h| h.size().x + 10.);
    let mut x = face.center().x - total / 2.;
    p.galley(
        Pos2::new(x, face.center().y - g.size().y / 2.),
        g.clone(),
        ink,
    );
    x += g.size().x + 10.;
    if let Some(h) = hint_g {
        p.galley(Pos2::new(x, face.center().y - h.size().y / 2.), h, ink);
    }
    if response.has_focus() {
        focus_ring(p, r, pal);
    }
    response
}

/// Segmented choice (e.g. 1× / 2×). Returns the newly clicked index.
pub fn segmented(
    ui: &mut egui::Ui,
    labels: &[&str],
    selected: usize,
    width: f32,
    pal: &Palette,
) -> Option<usize> {
    let h = 30.;
    let (r, _) = ui.allocate_exact_size(Vec2::new(width, h), Sense::hover());
    let p = ui.painter().clone();
    p.rect_filled(r, RADIUS, pal.card);
    p.rect_stroke(
        r,
        RADIUS,
        Stroke::new(1_f32, pal.line),
        egui::StrokeKind::Inside,
    );
    let w = width / labels.len() as f32;
    let mut clicked = None;
    for (i, label) in labels.iter().enumerate() {
        let cell = Rect::from_min_size(r.min + Vec2::new(w * i as f32, 0.), Vec2::new(w, h));
        let response = ui.interact(cell, ui.id().with(("segment", label)), Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                true,
                i == selected,
                *label,
            )
        });
        if i == selected {
            p.rect_filled(cell.shrink(3.), RADIUS - 1., BRASS);
        } else if response.hovered() {
            p.rect_filled(cell.shrink(3.), RADIUS - 1., pal.card_hi);
        }
        let ink = if i == selected { pal.ink } else { pal.text };
        p.text(
            cell.center(),
            Align2::CENTER_CENTER,
            *label,
            mono(12.5),
            ink,
        );
        if response.has_focus() {
            focus_ring(&p, cell.shrink(2.), pal);
        }
        if response.clicked() {
            clicked = Some(i);
        }
    }
    clicked
}

/// Toggle switch with label; returns true when flipped.
pub fn toggle(ui: &mut egui::Ui, label: &str, on: bool, pal: &Palette) -> bool {
    let (r, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.), Sense::click());
    response
        .widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, on, label));
    let p = ui.painter();
    let track = Rect::from_min_size(
        Pos2::new(r.right() - 42., r.center().y - 10.),
        Vec2::new(42., 20.),
    );
    p.rect_filled(track, 10., if on { pal.accent } else { pal.line });
    let knob = if on {
        track.right_center() - Vec2::new(10., 0.)
    } else {
        track.left_center() + Vec2::new(10., 0.)
    };
    p.circle_filled(knob, 7.5, if on { pal.accent_text } else { pal.card_hi });
    p.text(
        Pos2::new(r.left(), r.center().y),
        Align2::LEFT_CENTER,
        label,
        text(13.5),
        pal.text,
    );
    p.text(
        track.left_center() - Vec2::new(8., 0.),
        Align2::RIGHT_CENTER,
        if on { "On" } else { "Off" },
        mono(10.),
        pal.muted,
    );
    if response.has_focus() {
        focus_ring(p, r, pal);
    }
    response.clicked()
}

/// Labelled stat with a bar; `next` shows an upgrade's gain in the accent colour.
pub fn stat(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    frac: f32,
    next: Option<(f32, &str)>,
    pal: &Palette,
) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.), Sense::hover());
    let p = ui.painter();
    p.text(
        r.left_top() + Vec2::new(0., 1.),
        Align2::LEFT_TOP,
        label,
        text(11.5),
        pal.muted,
    );
    let mut right = r.right();
    if let Some((_, delta)) = next {
        let g = p.layout_no_wrap(delta.to_owned(), mono(11.), pal.accent);
        right -= g.size().x;
        p.galley(Pos2::new(right, r.top()), g, pal.accent);
        right -= 6.;
    }
    p.text(
        Pos2::new(right, r.top()),
        Align2::RIGHT_TOP,
        value,
        mono(11.5),
        pal.text,
    );
    let bar = Rect::from_min_size(
        Pos2::new(r.left(), r.bottom() - 6.),
        Vec2::new(r.width(), 4.),
    );
    p.rect_filled(bar, 2., pal.line);
    if let Some((to, _)) = next {
        p.rect_filled(
            Rect::from_min_size(bar.min, Vec2::new(bar.width() * to.clamp(0., 1.), 4.)),
            2.,
            pal.accent.gamma_multiply(0.55),
        );
    }
    p.rect_filled(
        Rect::from_min_size(bar.min, Vec2::new(bar.width() * frac.clamp(0., 1.), 4.)),
        2.,
        BRASS,
    );
}

/// Segmented base-health meter.
pub fn pips(p: &Painter, r: Rect, filled: u32, total: u32, pal: &Palette) {
    let gap = 2.;
    let w = (r.width() - gap * (total - 1) as f32) / total as f32;
    let colour = if filled * 4 > total * 3 {
        pal.good
    } else if filled * 3 > total {
        pal.gold
    } else {
        pal.bad
    };
    for i in 0..total {
        let cell = Rect::from_min_size(
            r.min + Vec2::new((w + gap) * i as f32, 0.),
            Vec2::new(w, r.height()),
        );
        p.rect_filled(cell, 1.5, if i < filled { colour } else { pal.line });
    }
}

pub fn coin(p: &Painter, c: Pos2, r: f32, pal: &Palette) {
    p.circle_filled(c, r, pal.gold);
    p.circle_stroke(
        c,
        r,
        Stroke::new(1_f32, pal.gold.lerp_to_gamma(pal.ink, 0.5)),
    );
    p.circle_stroke(
        c,
        r * 0.58,
        Stroke::new(1_f32, pal.gold.lerp_to_gamma(pal.ink, 0.35)),
    );
}
pub fn shield(p: &Painter, c: Pos2, r: f32, colour: Color32, pal: &Palette) {
    let pts = vec![
        c + Vec2::new(-r * 0.8, -r * 0.8),
        c + Vec2::new(r * 0.8, -r * 0.8),
        c + Vec2::new(r * 0.8, r * 0.1),
        c + Vec2::new(0., r),
        c + Vec2::new(-r * 0.8, r * 0.1),
    ];
    p.add(Shape::convex_polygon(
        pts,
        colour,
        Stroke::new(1_f32, pal.ink.gamma_multiply(0.6)),
    ));
}
pub fn lock(p: &Painter, c: Pos2, r: f32, colour: Color32) {
    p.circle_stroke(
        c - Vec2::new(0., r * 0.35),
        r * 0.5,
        Stroke::new(r * 0.18, colour),
    );
    p.rect_filled(
        Rect::from_center_size(c + Vec2::new(0., r * 0.3), Vec2::new(r * 1.4, r * 1.)),
        2.,
        colour,
    );
}

/// Medal emblem: laurel-ringed disc. Perfect is gold with a star, Clear is
/// brass with a chevron, no medal is an empty outline.
pub fn medal(p: &Painter, c: Pos2, r: f32, medal: Medal, pal: &Palette) {
    let (fill, rim) = match medal {
        Medal::Perfect => (pal.gold, pal.gold.lerp_to_gamma(IVORY, 0.4)),
        Medal::Clear => (BRASS, BRASS.lerp_to_gamma(IVORY, 0.3)),
        Medal::None => (Color32::TRANSPARENT, pal.line),
    };
    if medal == Medal::None {
        p.circle_stroke(c, r, Stroke::new((r * 0.1).max(1.), pal.line));
        return;
    }
    // Laurel leaves around the lower rim.
    for side in [-1., 1.] {
        for k in 0..5 {
            let a = std::f32::consts::FRAC_PI_2 + side * (0.35 + 0.42 * k as f32);
            let at = c + Vec2::new(a.cos(), a.sin()) * r * 1.18;
            let tangent = Vec2::new(-a.sin(), a.cos()) * side;
            p.add(Shape::convex_polygon(
                vec![
                    at - tangent * r * 0.22,
                    at + Vec2::new(a.cos(), a.sin()) * r * 0.1,
                    at + tangent * r * 0.22,
                ],
                rim.gamma_multiply(0.8),
                Stroke::NONE,
            ));
        }
    }
    p.circle_filled(
        c + Vec2::new(0., r * 0.08),
        r,
        Color32::from_black_alpha(60),
    );
    p.circle_filled(c, r, fill);
    p.circle_stroke(c, r, Stroke::new((r * 0.1).max(1.), rim));
    p.circle_stroke(
        c,
        r * 0.78,
        Stroke::new((r * 0.04).max(0.8), pal.ink.gamma_multiply(0.35)),
    );
    if medal == Medal::Perfect {
        let star: Vec<Pos2> = (0..10)
            .map(|i| {
                let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::TAU / 10.;
                let rr = if i % 2 == 0 { r * 0.55 } else { r * 0.23 };
                c + Vec2::new(a.cos(), a.sin()) * rr
            })
            .collect();
        for i in 0..10 {
            p.add(Shape::convex_polygon(
                vec![c, star[i], star[(i + 1) % 10]],
                pal.ink.gamma_multiply(0.75),
                Stroke::NONE,
            ));
        }
    } else {
        p.add(Shape::line(
            vec![
                c + Vec2::new(-r * 0.4, r * 0.1),
                c + Vec2::new(0., -r * 0.3),
                c + Vec2::new(r * 0.4, r * 0.1),
            ],
            Stroke::new(r * 0.16, pal.ink.gamma_multiply(0.75)),
        ));
    }
}

/// Shared modal frame for windows.
pub fn modal_frame(pal: &Palette) -> egui::Frame {
    egui::Frame::NONE
        .fill(pal.surface.lerp_to_gamma(pal.card_hi, 0.6))
        .stroke(Stroke::new(1_f32, BRASS.gamma_multiply(0.8)))
        .corner_radius(6_u8)
        .inner_margin(egui::Margin::same(22))
        .shadow(egui::epaint::Shadow {
            offset: [0, 10],
            blur: 36,
            spread: 0,
            color: Color32::from_black_alpha(if pal.light { 50 } else { 140 }),
        })
}

pub fn title(ui: &mut egui::Ui, eyebrow: &str, heading: &str, pal: &Palette) {
    ui.label(egui::RichText::new(eyebrow).font(mono(10.5)).color(BRASS));
    ui.label(
        egui::RichText::new(heading)
            .font(text(24.))
            .color(pal.text)
            .strong(),
    );
    ui.add_space(6.);
}
