//! Game adapters and session ownership; no navigation or shelf rendering.
use crate::{catalog::Game, pinball};
use eframe::egui;

/// Host lifecycle adapter. Rules and save schemas remain in each game crate.
/// `suspend` pauses without implicitly resuming; `set_input_enabled` gates modal
/// input, including its closing frame. `ready` is for capture, not construction.
/// `finished` requests return to the shelf. Active owns the single on_exit call.
pub(crate) trait ArcadeGame: eframe::App {
    fn begin_shutdown(&mut self) {}
    fn poll_shutdown(&mut self) -> bool {
        true
    }

    fn prepare_style(&mut self, _: &egui::Context) {}
    fn ready(&self) -> bool {
        true
    }
    fn suspend(&mut self) {}
    fn set_input_enabled(&mut self, _: bool) {}
    fn finished(&mut self) -> bool {
        false
    }
}
impl ArcadeGame for omarchy_stack::app::StackApp {
    fn suspend(&mut self) {
        self.suspend();
    }
    fn finished(&mut self) -> bool {
        omarchy_stack::app::StackApp::finished(self)
    }
}
impl ArcadeGame for omarchy_bubble::app::BubbleApp {
    fn suspend(&mut self) {
        self.suspend();
    }
    fn finished(&mut self) -> bool {
        omarchy_bubble::app::BubbleApp::finished(self)
    }
}
impl ArcadeGame for omarchy_blast::app::App {
    fn suspend(&mut self) {
        self.suspend();
    }
    fn finished(&mut self) -> bool {
        omarchy_blast::app::App::finished(self)
    }
}
impl ArcadeGame for omarchy_freeski::app::App {
    fn suspend(&mut self) {
        self.suspend();
    }
    fn finished(&mut self) -> bool {
        self.finished()
    }
}
impl ArcadeGame for omarchy_shatter::app::App {
    fn suspend(&mut self) {
        self.suspend();
    }
    fn set_input_enabled(&mut self, enabled: bool) {
        self.set_input_enabled(enabled);
    }
    fn finished(&mut self) -> bool {
        omarchy_shatter::app::App::finished(self)
    }
}
impl ArcadeGame for omarchy_tanks::app::App {
    fn suspend(&mut self) {
        self.suspend();
    }
    fn set_input_enabled(&mut self, enabled: bool) {
        self.set_input_enabled(enabled);
    }
    fn finished(&mut self) -> bool {
        omarchy_tanks::app::App::finished(self)
    }
}
impl ArcadeGame for omarchy_minesweeper::app::App {
    fn prepare_style(&mut self, ctx: &egui::Context) {
        self.prepare_style(ctx);
    }
    fn suspend(&mut self) {
        self.suspend();
    }
    fn set_input_enabled(&mut self, enabled: bool) {
        self.set_input_enabled(enabled);
    }
}
impl ArcadeGame for omarchy_ridgeline::app::App {
    fn suspend(&mut self) {
        self.suspend();
    }
    fn set_input_enabled(&mut self, enabled: bool) {
        self.set_input_enabled(enabled);
    }
    fn finished(&mut self) -> bool {
        omarchy_ridgeline::app::App::finished(self)
    }
}
impl ArcadeGame for omarchy_2048::app::App {}
impl ArcadeGame for omarchy_chess::ui::ChessApp {}
impl ArcadeGame for omarchy_solitaire::app::SolitaireApp {}
impl ArcadeGame for omarchy_scram::app::ScramApp {}
impl ArcadeGame for omarchy_invaders::App {}
impl ArcadeGame for pinball::Pinball {
    fn begin_shutdown(&mut self) {
        self.begin_shutdown();
    }
    fn poll_shutdown(&mut self) -> bool {
        self.poll_shutdown()
    }

    fn set_input_enabled(&mut self, enabled: bool) {
        self.set_input_enabled(enabled);
    }
    fn ready(&self) -> bool {
        self.ready()
    }
    fn finished(&mut self) -> bool {
        self.finished()
    }
    fn suspend(&mut self) {
        self.pause();
    }
}
impl ArcadeGame for omarchy_snake::app::SnakeApp {
    fn finished(&mut self) -> bool {
        omarchy_snake::app::SnakeApp::finished(self)
    }
    fn suspend(&mut self) {
        omarchy_snake::app::SnakeApp::suspend(self);
    }
}
pub(crate) struct Active {
    pub(crate) game: Game,
    pub(crate) app: Box<dyn ArcadeGame>,
    // Field order matters: destroy the game before releasing its save lock.
    _lock: Option<Box<dyn std::any::Any>>,
}
impl Drop for Active {
    fn drop(&mut self) {
        self.app.on_exit(None);
    }
}

impl Active {
    /// Construct the game while retaining its legacy save lock until teardown.
    pub(crate) fn open(game: Game, ctx: &egui::Context) -> Result<Self, String> {
        let mut lock: Option<Box<dyn std::any::Any>> = None;
        let app: Box<dyn ArcadeGame> = match game {
            Game::Stack => Box::new(omarchy_stack::app::StackApp::new()),
            Game::Snake => Box::new(omarchy_snake::app::SnakeApp::new()),
            Game::Bubble => Box::new(omarchy_bubble::app::BubbleApp::new()),
            Game::Blast => Box::new(omarchy_blast::app::App::new()),
            Game::Minesweeper => Box::new(omarchy_minesweeper::app::App::new()?),
            Game::Tanks => Box::new(omarchy_tanks::app::App::new()),
            Game::Shatter => Box::new(omarchy_shatter::app::App::new()),
            Game::Ridgeline => Box::new(omarchy_ridgeline::app::App::new()),
            Game::TwentyFortyEight => Box::new(omarchy_2048::app::App::new()?),
            Game::FreeSki => Box::new(omarchy_freeski::app::App::new()?),
            Game::Chess => {
                let dir = omarchy_chess::storage::state_dir();
                lock = Some(Box::new(omarchy_chess::storage::SessionLock::acquire(
                    &dir,
                )?));
                Box::new(omarchy_chess::ui::ChessApp::new(dir))
            }
            Game::Solitaire => {
                let dir = omarchy_solitaire::storage::state_dir();
                lock = Some(Box::new(omarchy_solitaire::storage::SessionLock::acquire(
                    &dir,
                )?));
                Box::new(omarchy_solitaire::app::SolitaireApp::new(ctx, dir, None))
            }
            Game::Scram => {
                let dir = omarchy_scram::storage::state_dir();
                lock = Some(Box::new(
                    omarchy_scram::storage::SessionLock::acquire(&dir)
                        .map_err(|e| e.to_string())?,
                ));
                Box::new(omarchy_scram::app::ScramApp::new(ctx, dir, None))
            }
            Game::Invaders => Box::new(omarchy_invaders::App::new(
                omarchy_invaders::storage::Store::open().map_err(|e| e.to_string())?,
            )),
            Game::Pinball => Box::new(pinball::Pinball::new(ctx).map_err(|e| e.to_string())?),
        };
        Ok(Active {
            game,
            app,
            _lock: lock,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    type Events = Rc<RefCell<Vec<&'static str>>>;
    struct GameProbe(Events);
    impl eframe::App for GameProbe {
        fn update(&mut self, _: &egui::Context, _: &mut eframe::Frame) {}
        fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
            self.0.borrow_mut().push("save");
        }
    }
    impl ArcadeGame for GameProbe {}
    impl Drop for GameProbe {
        fn drop(&mut self) {
            self.0.borrow_mut().push("game dropped");
        }
    }
    struct LockProbe(Events);
    impl Drop for LockProbe {
        fn drop(&mut self) {
            self.0.borrow_mut().push("lock released");
        }
    }
    fn probe(events: &Events) -> Active {
        Active {
            game: Game::Chess,
            app: Box::new(GameProbe(events.clone())),
            _lock: Some(Box::new(LockProbe(events.clone()))),
        }
    }

    #[test]
    fn leaving_a_session_saves_once_before_game_and_lock_teardown() {
        let events = Events::default();
        let mut active = Some(probe(&events));
        drop(active.take());
        drop(active);
        assert_eq!(
            &*events.borrow(),
            &["save", "game dropped", "lock released"]
        );
    }

    #[test]
    fn replacing_a_session_finishes_the_old_session_only() {
        let old = Events::default();
        let new = Events::default();
        let mut active = Some(probe(&old));
        drop(active.replace(probe(&new)));
        assert_eq!(&*old.borrow(), &["save", "game dropped", "lock released"]);
        assert!(new.borrow().is_empty());
        drop(active);
        assert_eq!(&*new.borrow(), &["save", "game dropped", "lock released"]);
    }
    #[test]
    fn pending_shutdown_retains_game_and_lock_until_completion() {
        use crate::{Arcade, Closing};
        use std::cell::Cell;
        struct Pending {
            events: Events,
            done: Rc<Cell<bool>>,
        }
        impl eframe::App for Pending {
            fn update(&mut self, _: &egui::Context, _: &mut eframe::Frame) {}
            fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
                self.events.borrow_mut().push("save");
            }
        }
        impl ArcadeGame for Pending {
            fn begin_shutdown(&mut self) {
                self.events.borrow_mut().push("begin");
            }
            fn poll_shutdown(&mut self) -> bool {
                self.done.get()
            }
        }
        impl Drop for Pending {
            fn drop(&mut self) {
                self.events.borrow_mut().push("game dropped");
            }
        }
        let events = Events::default();
        let done = Rc::new(Cell::new(false));
        let mut arcade = Arcade {
            closing: None,
            input: Default::default(),
            active: Some(Active {
                game: Game::Pinball,
                app: Box::new(Pending {
                    events: events.clone(),
                    done: done.clone(),
                }),
                _lock: Some(Box::new(LockProbe(events.clone()))),
            }),
            selected: 0,
            error: None,
            about: false,
            confirm_home: true,
            theme: arcade_platform::theme::Theme::load(),
            themed: std::time::Instant::now(),
            capture: None,
            frames: 0,
            initial: None,
            _lock: tempfile::tempfile().unwrap(),
        };
        let ctx = egui::Context::default();
        arcade.begin_close(Closing::Home);
        let output = ctx.run(Default::default(), |ctx| {
            assert!(arcade.poll_close(ctx));
        });
        assert!(!output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .iter()
            .any(|c| matches!(c, egui::ViewportCommand::Close)));
        assert!(arcade.active.is_some());
        assert_eq!(&*events.borrow(), &["begin"]);
        arcade.begin_close(Closing::Quit);
        arcade.begin_close(Closing::Home);
        assert!(arcade.closing == Some(Closing::Quit));
        done.set(true);
        let output = ctx.run(Default::default(), |ctx| {
            assert!(arcade.poll_close(ctx));
        });
        assert!(output.viewport_output[&egui::ViewportId::ROOT]
            .commands
            .iter()
            .any(|c| matches!(c, egui::ViewportCommand::Close)));
        assert!(arcade.active.is_none());
        assert!(arcade.closing.is_none());
        assert_eq!(
            &*events.borrow(),
            &["begin", "save", "game dropped", "lock released"]
        );
    }
}
