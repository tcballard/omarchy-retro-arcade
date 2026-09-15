//! FreeSki menu components. Presentation only; callers own every action.
use eframe::egui::{self, Color32, RichText, Vec2};
use omarchy_chess::theme::Theme;

pub fn show(ctx: &egui::Context, id: &str, theme: &Theme, content: impl FnOnce(&mut egui::Ui)) {
    let width = 440_f32.min((ctx.screen_rect().width() - 96.).max(240.));
    egui::Modal::new(egui::Id::new(id))
        .backdrop_color(Color32::from_black_alpha(if theme.light() {
            110
        } else {
            165
        }))
        .frame(egui::Frame::popup(&ctx.style()).inner_margin(16))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.set_max_height((ctx.screen_rect().height() - 112.).max(220.));
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.spacing_mut().item_spacing = Vec2::new(10., 6.);
            egui::ScrollArea::vertical()
                .max_height((ctx.screen_rect().height() - 112.).max(220.))
                .auto_shrink([false, true])
                .show(ui, content);
        });
}
pub fn heading(ui: &mut egui::Ui, theme: &Theme, eyebrow: &str, title: &str, subtitle: &str) {
    ui.label(
        RichText::new(eyebrow)
            .monospace()
            .size(11.)
            .color(arcade_presentation::BRASS),
    );
    ui.add_space(2.);
    ui.label(RichText::new(title).size(22.).monospace().strong().color(
        theme.foreground.lerp_to_gamma(
            if theme.light() {
                Color32::BLACK
            } else {
                Color32::WHITE
            },
            0.65,
        ),
    ));
    if !subtitle.is_empty() {
        ui.label(RichText::new(subtitle).size(13.).color(muted(ui)));
    }
    ui.add_space(8.);
}
pub fn muted(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(163, 174, 182)
    } else {
        Color32::from_rgb(87, 101, 106)
    }
}
pub fn section(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.);
    ui.label(RichText::new(text).monospace().size(11.).color(muted(ui)));
}
pub fn button(label: &str, primary: bool) -> egui::Button<'_> {
    let button = egui::Button::new(RichText::new(label).size(13.));
    if primary {
        egui::Button::new(
            RichText::new(label)
                .size(13.)
                .color(arcade_presentation::INK),
        )
        .fill(arcade_presentation::BRASS)
    } else {
        button
    }
}
pub fn choice(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    ui.add(button(label, selected))
}
pub fn action(ui: &mut egui::Ui, _theme: &Theme, label: &str, primary: bool) -> egui::Response {
    ui.add_sized([ui.available_width(), 32.], button(label, primary))
}
pub fn pair(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui, &mut egui::Ui)) {
    ui.columns(2, |columns| {
        let (left, right) = columns.split_at_mut(1);
        content(&mut left[0], &mut right[0]);
    });
}
pub fn stats(ui: &mut egui::Ui, values: &[(&str, String)]) {
    ui.columns(values.len(), |columns| {
        for (column, (label, value)) in columns.iter_mut().zip(values) {
            egui::Frame::new()
                .fill(surface(column))
                .corner_radius(2)
                .inner_margin(8)
                .show(column, |ui| {
                    ui.set_min_width((ui.available_width() - 1.).max(0.));
                    ui.label(RichText::new(*label).monospace().size(11.).color(muted(ui)));
                    ui.label(RichText::new(value).size(18.).monospace().strong());
                });
        }
    });
    ui.add_space(8.);
}
pub fn setting(ui: &mut egui::Ui, value: &mut bool, title: &str, description: &str) -> bool {
    let changed = ui.checkbox(value, title).changed();
    ui.label(RichText::new(description).size(13.).color(muted(ui)));
    ui.separator();
    changed
}
pub fn control(ui: &mut egui::Ui, title: &str, keys: &str, detail: &str) {
    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = 22.;
        ui.spacing_mut().item_spacing.y = 4.;
        ui.horizontal(|ui| {
            ui.add_sized(
                [100., 22.],
                egui::Label::new(RichText::new(title).strong()).halign(egui::Align::Min),
            );
            ui.label(RichText::new(keys).monospace().size(14.));
        });
        ui.label(RichText::new(detail).size(14.).color(muted(ui)));
    });
    ui.add_space(8.);
}

fn surface(ui: &egui::Ui) -> Color32 {
    ui.visuals()
        .window_fill()
        .lerp_to_gamma(ui.visuals().text_color(), 0.055)
}
