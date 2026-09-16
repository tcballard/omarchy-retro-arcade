//! Desktop-derived palette and fontconfig monospace selection.
use eframe::egui::{self, Color32, FontId, Stroke, Vec2};
use omarchy_chess::theme::Theme;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

pub fn paths() -> Vec<PathBuf> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let state = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/state"));
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    vec![
        state.join("omarchy/current/theme/colors.toml"),
        home.join(".local/state/omarchy/current/theme/colors.toml"),
        config.join("omarchy/current/theme/colors.toml"),
    ]
}
pub fn reload(theme: &mut Theme, paths: &[PathBuf]) {
    if let Some(next) = Theme::load_from(paths) {
        *theme = next;
    }
}
fn luminance(c: Color32) -> f32 {
    let v = |n: u8| {
        let x = n as f32 / 255.;
        if x <= 0.04045 {
            x / 12.92
        } else {
            ((x + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * v(c.r()) + 0.7152 * v(c.g()) + 0.0722 * v(c.b())
}
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let a = luminance(a);
    let b = luminance(b);
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
pub fn ink(bg: Color32, preferred: Color32) -> Color32 {
    if contrast(bg, preferred) >= 4.5 {
        preferred
    } else if contrast(bg, Color32::BLACK) > contrast(bg, Color32::WHITE) {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}
#[derive(Clone)]
pub struct Palette {
    pub bg: Color32,
    pub ink: Color32,
    pub muted: Color32,
    pub surface: Color32,
    pub raised: Color32,
    pub hover: Color32,
    pub line: Color32,
    pub accent: Color32,
    pub accent_ink: Color32,
    pub focus: Color32,
}
impl Palette {
    pub fn new(t: &Theme) -> Self {
        let foreground = ink(t.background, t.foreground);
        let surface = t.background.lerp_to_gamma(foreground, 0.04);
        Self {
            bg: t.background,
            ink: foreground,
            muted: ink(t.background, foreground.lerp_to_gamma(t.background, 0.24)),
            surface,
            raised: t.background.lerp_to_gamma(foreground, 0.12),
            hover: t.background.lerp_to_gamma(t.accent, 0.25),
            line: t.background.lerp_to_gamma(foreground, 0.25),
            accent: t.accent,
            accent_ink: ink(t.accent, foreground),
            focus: if contrast(surface, t.accent) >= 3. {
                t.accent
            } else {
                foreground
            },
        }
    }
    pub fn apply(&self, ctx: &egui::Context) {
        let mut v = if luminance(self.bg) > 0.4 {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        v.panel_fill = self.bg;
        v.window_fill = self.bg;
        v.extreme_bg_color = self.surface;
        v.override_text_color = None;
        v.window_stroke = Stroke::new(1_f32, self.line);
        v.selection.bg_fill = self.accent;
        v.selection.stroke = Stroke::new(1_f32, self.accent_ink);
        for (widget, bg, stroke) in [
            (&mut v.widgets.noninteractive, self.surface, self.line),
            (&mut v.widgets.inactive, self.raised, self.line),
            (&mut v.widgets.hovered, self.hover, self.focus),
            (&mut v.widgets.active, self.accent, self.focus),
            (&mut v.widgets.open, self.raised, self.focus),
        ] {
            widget.bg_fill = bg;
            widget.weak_bg_fill = bg;
            widget.bg_stroke = Stroke::new(1_f32, stroke);
            widget.fg_stroke = Stroke::new(1_f32, ink(bg, self.ink));
            widget.corner_radius = 0.into();
        }
        v.window_corner_radius = 0.into();
        v.menu_corner_radius = 0.into();
        ctx.set_visuals(v);
        ctx.style_mut(|s| {
            s.spacing.button_padding = Vec2::new(12., 8.);
            s.spacing.item_spacing = Vec2::new(10., 8.);
            s.text_styles
                .insert(egui::TextStyle::Button, FontId::monospace(13.));
            s.text_styles
                .insert(egui::TextStyle::Body, FontId::monospace(13.));
        });
    }
}

type FontResult = (String, Option<(Vec<u8>, u32)>);
pub struct DesktopFont {
    receiver: Option<mpsc::Receiver<FontResult>>,
    checked: Instant,
    identity: String,
}
impl Default for DesktopFont {
    fn default() -> Self {
        Self {
            receiver: None,
            checked: Instant::now() - Duration::from_secs(5),
            identity: String::new(),
        }
    }
}
impl DesktopFont {
    pub fn poll(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.receiver {
            match rx.try_recv() {
                Ok((id, font)) => {
                    self.identity = id;
                    self.receiver = None;
                    if let Some((bytes, index)) = font {
                        let mut defs = egui::FontDefinitions::default();
                        let mut data = egui::FontData::from_owned(bytes);
                        data.index = index;
                        defs.font_data
                            .insert("desktop-monospace".into(), std::sync::Arc::new(data));
                        for family in [egui::FontFamily::Monospace, egui::FontFamily::Proportional]
                        {
                            defs.families
                                .entry(family)
                                .or_default()
                                .insert(0, "desktop-monospace".into());
                        }
                        ctx.set_fonts(defs);
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => self.receiver = None,
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }
        if self.receiver.is_none() && self.checked.elapsed() >= Duration::from_secs(2) {
            self.checked = Instant::now();
            let previous = self.identity.clone();
            let (tx, rx) = mpsc::channel();
            self.receiver = Some(rx);
            std::thread::spawn(move || {
                if let Some(result) = resolve_font(&previous) {
                    let _ = tx.send(result);
                }
            });
        }
    }
}
fn resolve_font(previous: &str) -> Option<FontResult> {
    let mut child = Command::new("fc-match")
        .args(["-f", "%{file}\n%{index}", "monospace"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                break;
            }
            Ok(None) => {}
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
        if start.elapsed() > Duration::from_millis(500) {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().ok()?;
    let text = String::from_utf8(output.stdout).ok()?;
    let mut lines = text.lines();
    let path = PathBuf::from(lines.next()?);
    let index = lines.next()?.parse::<u32>().ok()?;
    let metadata = std::fs::metadata(&path).ok()?;
    if metadata.len() > 16 * 1024 * 1024 {
        return None;
    }
    let id = format!(
        "{}:{index}:{:?}:{}",
        path.display(),
        metadata.modified().ok()?,
        metadata.len()
    );
    if id == previous {
        return Some((id, None));
    }
    let bytes = std::fs::read(path).ok()?;
    ttf_parser::Face::parse(&bytes, index).ok()?;
    Some((id, Some((bytes, index))))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reload_preserves_last_good_theme_and_obeys_precedence() {
        let d = tempfile::tempdir().unwrap();
        let current = d.path().join("current");
        let legacy = d.path().join("legacy");
        std::fs::write(
            &legacy,
            "background='#ffffff'\nforeground='#111111'\naccent='#116677'",
        )
        .unwrap();
        let mut t = Theme::default();
        reload(&mut t, &[current.clone(), legacy]);
        assert_eq!(t.background, Color32::WHITE);
        std::fs::write(
            &current,
            "background='#151821'\nforeground='#eef0f5'\naccent='#ffab70'",
        )
        .unwrap();
        reload(&mut t, std::slice::from_ref(&current));
        assert_eq!(t.accent, Color32::from_rgb(255, 171, 112));
        let old = t.clone();
        std::fs::write(&current, "incomplete").unwrap();
        reload(&mut t, &[current]);
        assert_eq!(t, old);
    }
    #[test]
    fn text_contrast_and_controls_follow_palette() {
        for t in [
            Theme::default(),
            Theme {
                background: Color32::WHITE,
                foreground: Color32::from_gray(30),
                accent: Color32::from_rgb(32, 95, 120),
            },
            Theme {
                background: Color32::GRAY,
                foreground: Color32::GRAY,
                accent: Color32::GRAY,
            },
        ] {
            let p = Palette::new(&t);
            assert!(contrast(p.bg, p.ink) >= 4.5);
            assert!(contrast(p.accent, p.accent_ink) >= 4.5);
            assert!(contrast(p.bg, p.muted) >= 4.5);
            let ctx = egui::Context::default();
            p.apply(&ctx);
            assert_eq!(ctx.style().visuals.selection.bg_fill, t.accent);
            assert_eq!(ctx.style().visuals.window_stroke.color, p.line);
        }
    }
}
