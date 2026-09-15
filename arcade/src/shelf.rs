use super::{Arcade, Game};
use arcade_presentation::{BRASS, IVORY};
use eframe::egui::{self, Align2, Color32, FontId, Key, Pos2, Rect, RichText, Sense, Stroke, Vec2};

impl Game {
    fn genre(self) -> &'static str {
        match self {
            Self::Pinball => "PINBALL / CIRCUIT",
            Self::Solitaire => "CARDS / KLONDIKE",
            Self::Scram => "MAZE / CHASE",
            Self::Invaders => "SHOOTER / ORBIT",
            Self::Chess => "STRATEGY / CLASSIC",
            Self::Stack => "PUZZLE / FALLING BLOCKS",
            Self::Snake => "ARCADE / CLASSIC",
            Self::Bubble => "PUZZLE / 20 LEVELS",
            Self::Blast => "ARENA / SOLO + LOCAL",
            Self::TwentyFortyEight => "PUZZLE / 2048",
            Self::FreeSki => "SPORT / DOWNHILL SKIING",
            Self::Tanks => "ARTILLERY / SOLO + LOCAL",
            Self::Shatter => "ARCADE / BRICK BREAKER",
        }
    }
    fn crop(self) -> Rect {
        let (a, b) = match self {
            Self::Pinball => ([0.0, 0.03], [1.0, 1.0]),
            Self::Solitaire => ([0.0, 0.16], [1.0, 0.86]),
            Self::Scram => ([0.22, 0.21], [0.78, 0.90]),
            Self::Invaders => ([0.10, 0.16], [0.90, 0.94]),
            Self::Chess => ([0.08, 0.12], [0.77, 0.90]),
            Self::Stack => ([0.265, 0.145], [0.585, 0.975]),
            Self::Snake => ([0., 0.], [1., 1.]),
            Self::Bubble => ([0.30, 0.10], [0.73, 0.93]),
            Self::Blast => ([0.07, 0.13], [0.93, 0.96]),
            Self::TwentyFortyEight => ([0., 0.], [1., 1.]),
            // Native 1280×900 capture: frame the chase and the approaching terrain.
            Self::FreeSki => ([316. / 1280., 206. / 900.], [964. / 1280., 656. / 900.]),
            Self::Tanks => ([0., 0.], [1., 1.]),
            Self::Shatter => ([0., 0.], [1., 1.]),
        };
        Rect::from_min_max(Pos2::from(a), Pos2::from(b))
    }
}
impl Arcade {
    pub(super) fn shelf(&mut self, ctx: &egui::Context) {
        if self.themed.elapsed() > std::time::Duration::from_secs(2) {
            self.theme = omarchy_chess::theme::Theme::load();
            self.themed = std::time::Instant::now();
        }
        let light = self.theme.background.r() as u32
            + self.theme.background.g() as u32
            + self.theme.background.b() as u32
            > 400;
        let mut visuals = if light {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        visuals.panel_fill = self.theme.background;
        visuals.window_fill = self.theme.background;
        visuals.override_text_color = Some(self.theme.foreground);
        visuals.selection.bg_fill = self.theme.accent;
        ctx.set_visuals(visuals);
        arcade_presentation::apply(ctx);
        let ink = if light { self.theme.foreground } else { IVORY };
        let muted = if light {
            Color32::from_rgb(88, 92, 77)
        } else {
            Color32::from_rgb(162, 163, 142)
        };
        let accent = self.theme.accent;
        let mut play = false;
        // Arrow navigation selects, Enter launches. Tab remains standard widget navigation.
        let widget_focused = ctx.wants_keyboard_input();
        let previous_selection = self.selected;
        {
            ctx.input_mut(|i| {
                if i.consume_key(egui::Modifiers::NONE, Key::ArrowDown)
                    || i.consume_key(egui::Modifiers::NONE, Key::ArrowRight)
                {
                    self.selected = (self.selected + 1) % Game::ALL.len();
                }
                if i.consume_key(egui::Modifiers::NONE, Key::ArrowUp)
                    || i.consume_key(egui::Modifiers::NONE, Key::ArrowLeft)
                {
                    self.selected = (self.selected + Game::ALL.len() - 1) % Game::ALL.len();
                }
                play = !widget_focused && i.consume_key(egui::Modifiers::NONE, Key::Enter);
            });
        }
        if self.selected != previous_selection {
            ctx.memory_mut(|m| {
                if let Some(id) = m.focused() {
                    m.surrender_focus(id);
                }
            });
        }
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(self.theme.background)
                    .inner_margin(24.),
            )
            .show(ctx, |ui| {
                arcade_presentation::backdrop(ui);
                let all = ui.max_rect();
                let pad = if all.width() < 1000. { 20. } else { 30. };
                let r = all.shrink2(Vec2::new(pad, 8.));
                let p = ui.painter();
                p.text(
                    r.left_top(),
                    Align2::LEFT_TOP,
                    "O M A R C H Y",
                    FontId::monospace(12.),
                    accent,
                );
                p.text(
                    r.left_top() + Vec2::new(-2., 20.),
                    Align2::LEFT_TOP,
                    "ARCADE",
                    FontId::proportional(48.),
                    ink,
                );
                p.text(
                    r.right_top() + Vec2::new(0., 8.),
                    Align2::RIGHT_TOP,
                    "THE COLLECTION  /  01",
                    FontId::monospace(11.),
                    BRASS,
                );
                p.text(
                    r.right_top() + Vec2::new(0., 30.),
                    Align2::RIGHT_TOP,
                    format!("{} games. One more go.", Game::ALL.len()),
                    FontId::proportional(16.),
                    ink,
                );
                p.line_segment(
                    [
                        r.left_top() + Vec2::new(0., 87.),
                        r.right_top() + Vec2::new(0., 87.),
                    ],
                    Stroke::new(1_f32, BRASS.gamma_multiply(0.5)),
                );
                let bottom = r.bottom() - 46.;
                let top = r.top() + 112.;
                let gap = 30.;
                let rail_w = (r.width() * 0.32).clamp(240., 350.);
                let left = Rect::from_min_max(
                    Pos2::new(r.left(), top),
                    Pos2::new(r.right() - rail_w - gap, bottom),
                );
                let rail = Rect::from_min_max(
                    Pos2::new(r.right() - rail_w, top),
                    Pos2::new(r.right(), bottom),
                );
                let game = Game::ALL[self.selected];
                let preview =
                    Rect::from_min_max(left.min, left.right_bottom() - Vec2::new(0., 123.));
                arcade_presentation::bezel(p, preview, accent);
                p.rect_filled(preview, 0., Color32::from_rgb(8, 12, 11));
                // A contained image preserves aspect ratio; source UV removes old app controls.
                let source = game.image();
                let image = egui::Image::new(source)
                    .uv(game.crop())
                    .maintain_aspect_ratio(true)
                    .fit_to_exact_size(preview.size());
                if let Ok(egui::load::TexturePoll::Ready { texture }) =
                    image.load_for_size(ctx, preview.size())
                {
                    let size = texture.size * game.crop().size();
                    let fit = (preview.width() / size.x).min(preview.height() / size.y);
                    image.paint_at(ui, Rect::from_center_size(preview.center(), size * fit));
                } else {
                    image.paint_at(ui, preview);
                }
                // Embossed nameplate and launch control, outside the artwork.
                let title = left.left_bottom() - Vec2::new(0., 96.);
                ui.painter().text(
                    title,
                    Align2::LEFT_TOP,
                    game.genre(),
                    FontId::monospace(10.),
                    BRASS,
                );
                ui.painter().text(
                    title + Vec2::new(0., 18.),
                    Align2::LEFT_TOP,
                    game.name(),
                    FontId::proportional(31.),
                    ink,
                );
                ui.painter().text(
                    title + Vec2::new(0., 60.),
                    Align2::LEFT_TOP,
                    game.line(),
                    FontId::proportional(13.),
                    muted,
                );
                let button = Rect::from_min_size(
                    Pos2::new(left.right() - 126., left.bottom() - 75.),
                    Vec2::new(126., 43.),
                );
                let response = ui.put(
                    button,
                    egui::Button::new(
                        RichText::new("PLAY   →")
                            .monospace()
                            .size(15.)
                            .color(self.theme.background),
                    )
                    .fill(accent),
                );
                if response.clicked() {
                    play = true;
                }
                let row_h = rail.height() / Game::ALL.len() as f32;
                for (i, g) in Game::ALL.iter().enumerate() {
                    let row = Rect::from_min_size(
                        rail.min + Vec2::new(0., i as f32 * row_h),
                        Vec2::new(rail.width(), row_h - 6.),
                    );
                    let response = ui
                        .interact(row, ui.id().with(("cartridge", i)), Sense::click())
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::WidgetType::SelectableLabel,
                            true,
                            i == self.selected,
                            g.name(),
                        )
                    });
                    let selected = i == self.selected;
                    let hover = response.hovered() || response.has_focus();
                    let pressed = response.is_pointer_button_down_on();
                    let fill = if light {
                        Color32::from_white_alpha(if selected { 200 } else { 115 })
                    } else {
                        Color32::from_black_alpha(if selected { 150 } else { 90 })
                    };
                    let p = ui.painter();
                    p.rect_filled(row, 1., fill);
                    p.rect_stroke(
                        row,
                        1.,
                        Stroke::new(
                            1_f32,
                            if selected || pressed {
                                accent
                            } else if hover {
                                BRASS
                            } else {
                                BRASS.gamma_multiply(0.27)
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );
                    if selected {
                        p.rect_filled(
                            Rect::from_min_size(row.min, Vec2::new(3., row.height())),
                            0.,
                            accent,
                        );
                    }
                    let cy = row.center().y;
                    p.text(
                        Pos2::new(row.left() + 15., cy),
                        Align2::LEFT_CENTER,
                        format!("{:02}", i + 1),
                        FontId::monospace(12.),
                        if selected { accent } else { muted },
                    );
                    p.text(
                        Pos2::new(row.left() + 47., cy - 8.),
                        Align2::LEFT_CENTER,
                        g.name(),
                        FontId::proportional(17.),
                        ink,
                    );
                    p.text(
                        Pos2::new(row.left() + 47., cy + 11.),
                        Align2::LEFT_CENTER,
                        g.genre(),
                        FontId::monospace(8.),
                        muted,
                    );
                    if selected {
                        p.circle_filled(Pos2::new(row.right() - 18., cy), 2.5, accent);
                    }
                    if response.clicked()
                        || (response.has_focus() && ui.input(|i| i.key_pressed(Key::Space)))
                    {
                        self.selected = i;
                    }
                    if response.double_clicked()
                        || (response.has_focus() && ui.input(|i| i.key_pressed(Key::Enter)))
                    {
                        self.selected = i;
                        play = true;
                    }
                }
                ui.painter().line_segment(
                    [
                        Pos2::new(r.left(), bottom + 17.),
                        Pos2::new(r.right(), bottom + 17.),
                    ],
                    Stroke::new(1_f32, BRASS.gamma_multiply(0.4)),
                );
                ui.painter().text(
                    Pos2::new(r.left(), bottom + 29.),
                    Align2::LEFT_CENTER,
                    "CLICK A GAME, THEN PLAY  /  DOUBLE-CLICK TO LAUNCH",
                    FontId::monospace(10.),
                    muted,
                );
                ui.painter().text(
                    Pos2::new(r.left(), bottom + 44.),
                    Align2::LEFT_CENTER,
                    "↑ ↓  SELECT     ENTER  PLAY     F11  FULL SCREEN",
                    FontId::monospace(9.),
                    muted,
                );
                if ui
                    .put(
                        Rect::from_min_size(
                            Pos2::new(r.right() - 190., bottom + 23.),
                            Vec2::new(116., 30.),
                        ),
                        egui::Button::new("Full screen").frame(false),
                    )
                    .on_hover_text("Toggle fullscreen · F11")
                    .clicked()
                {
                    super::toggle_fullscreen(ctx);
                }
                if ui
                    .put(
                        Rect::from_min_size(
                            Pos2::new(r.right() - 64., bottom + 23.),
                            Vec2::new(64., 30.),
                        ),
                        egui::Button::new("About").frame(false),
                    )
                    .clicked()
                {
                    self.about = true;
                }
            });
        if play {
            self.open(Game::ALL[self.selected], ctx);
        }
    }
}
