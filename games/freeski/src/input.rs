use crate::engine::{Input, MAX_HEADING};
use eframe::egui::{self, Event, Key, Pos2, Rect};
#[derive(Default)]
pub struct Controls {
    mouse: bool,
    target: f64,
    pointer: Option<Pos2>,
    armed: bool,
    retain_heading: bool,
    braking: bool,
}
impl Controls {
    pub fn braking(&self) -> bool {
        self.braking
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    /// Re-evaluate a released keyboard heading for each simulation tick. A crash
    /// may reset heading between two ticks rendered in the same frame.
    pub fn for_tick(&self, mut input: Input, current_heading: f64) -> Input {
        if self.retain_heading {
            input.heading = current_heading;
        }
        input
    }
    pub fn sample(
        &mut self,
        i: &egui::InputState,
        field: Rect,
        skier: Pos2,
        brake_button: bool,
        current_heading: f64,
    ) -> (Input, bool) {
        let keys = [
            Key::A,
            Key::D,
            Key::ArrowLeft,
            Key::ArrowRight,
            Key::S,
            Key::ArrowDown,
        ];
        if !self.armed {
            self.retain_heading = true;
            self.pointer = i.pointer.latest_pos();
            if !keys.iter().any(|k| i.key_down(*k)) && !i.pointer.any_down() {
                self.armed = true;
            }
            return (
                Input {
                    heading: current_heading,
                    brake: false,
                },
                false,
            );
        }
        let mut intentional = false;
        for event in &i.events {
            match event {
                Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers,
                    ..
                } if !modifiers.ctrl
                    && !modifiers.alt
                    && !modifiers.command
                    && matches!(key, Key::A | Key::D | Key::ArrowLeft | Key::ArrowRight) =>
                {
                    self.mouse = false;
                    intentional = true;
                }
                Event::PointerMoved(p) => {
                    let moved = self.pointer.is_some_and(|old| old.distance(*p) > 1.);
                    self.pointer = Some(*p);
                    if moved && field.contains(*p) {
                        self.mouse = true;
                        let offset = (p.x - skier.x) as f64;
                        let dead = field.width() as f64 * 0.018;
                        self.target = if offset.abs() < dead {
                            0.
                        } else {
                            ((offset - offset.signum() * dead) / (field.width() as f64 * 0.32))
                                .clamp(-1., 1.)
                                * MAX_HEADING
                        };
                        // Ready requires the explicit start action; pointer motion only aims.
                    }
                }
                _ => {}
            }
        }
        let right = i.key_down(Key::D) || i.key_down(Key::ArrowRight);
        let left = i.key_down(Key::A) || i.key_down(Key::ArrowLeft);
        self.retain_heading = !self.mouse && right == left;
        let heading = if self.mouse {
            self.target
        } else if right == left {
            current_heading
        } else {
            (i32::from(right) - i32::from(left)) as f64 * MAX_HEADING
        };
        let brake = brake_button
            || i.key_down(Key::S)
            || i.key_down(Key::ArrowDown)
            || (i.pointer.secondary_down()
                && i.pointer.latest_pos().is_some_and(|p| field.contains(p)));
        self.braking = brake;
        (Input { heading, brake }, intentional)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Modifiers, RawInput, Vec2};
    #[test]
    fn brake_keys_exclude_space_and_space_does_not_block_rearming() {
        let ctx = egui::Context::default();
        let mut controls = Controls::default();
        let field = Rect::from_min_size(Pos2::ZERO, Vec2::splat(500.));
        let sample = |controls: &mut Controls, events| {
            let mut brake = false;
            let _ = ctx.run(
                RawInput {
                    events,
                    focused: true,
                    ..Default::default()
                },
                |ctx| {
                    brake =
                        ctx.input(|i| controls.sample(i, field, field.center(), false, 0.).0.brake);
                },
            );
            brake
        };
        let key = |key, pressed| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        assert!(!sample(&mut controls, vec![key(Key::Space, true)]));
        assert!(controls.armed);
        assert!(!sample(&mut controls, vec![]));
        for brake_key in [Key::S, Key::ArrowDown] {
            assert!(sample(&mut controls, vec![key(brake_key, true)]));
            assert!(!sample(&mut controls, vec![key(brake_key, false)]));
        }
        controls.clear();
        assert!(!sample(&mut controls, vec![]));
        assert!(controls.armed);
        assert!(sample(&mut controls, vec![key(Key::S, true)]));
    }

    #[test]
    fn input_owner_changes_only_on_intent_and_release_is_required_after_clear() {
        let ctx = egui::Context::default();
        let mut c = Controls::default();
        let mut time = 0.;
        let field = Rect::from_min_size(Pos2::ZERO, Vec2::splat(500.));
        let skier = field.center();
        let mut sample = |events, controls: &mut Controls| {
            time += 0.02;
            let mut result = (Input::default(), false);
            let _ = ctx.run(
                RawInput {
                    events,
                    time: Some(time),
                    focused: true,
                    ..Default::default()
                },
                |ctx| result = ctx.input(|i| controls.sample(i, field, skier, false, 0.4)),
            );
            result.0
        };
        sample(vec![Event::PointerMoved(Pos2::new(300., 250.))], &mut c);
        let mouse = sample(vec![Event::PointerMoved(Pos2::new(420., 250.))], &mut c);
        assert!(mouse.heading > 0.);
        let key = |pressed| Event::Key {
            key: Key::A,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        assert!(sample(vec![key(true)], &mut c).heading < 0.);
        assert!(sample(vec![], &mut c).heading < 0.);
        assert_eq!(sample(vec![key(false)], &mut c).heading, 0.4);
        sample(vec![Event::PointerMoved(Pos2::new(450., 250.))], &mut c);
        let outside = sample(vec![Event::PointerMoved(Pos2::new(550., 250.))], &mut c);
        assert!(outside.heading > 0.);
        sample(vec![key(true)], &mut c);
        c.clear();
        assert_eq!(sample(vec![], &mut c).heading, 0.4);
        sample(vec![key(false)], &mut c);
        assert!(sample(vec![key(true)], &mut c).heading < 0.);
    }
}
