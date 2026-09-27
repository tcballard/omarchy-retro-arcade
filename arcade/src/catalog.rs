//! Stable game identities, shelf order and presentation metadata.
use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Game {
    Chess,
    Solitaire,
    Scram,
    Invaders,
    Pinball,
    Stack,
    Snake,
    Bubble,
    Blast,
    TwentyFortyEight,
    FreeSki,
    Shatter,
    Tanks,
    Minesweeper,
    Ridgeline,
}
impl Game {
    pub(crate) const ALL: [Self; 15] = [
        Self::Pinball,
        Self::Solitaire,
        Self::Scram,
        Self::Invaders,
        Self::Chess,
        Self::Stack,
        Self::Snake,
        Self::Bubble,
        Self::Blast,
        Self::TwentyFortyEight,
        Self::Shatter,
        Self::Tanks,
        Self::Minesweeper,
        Self::FreeSki,
        Self::Ridgeline,
    ];
    pub(crate) fn id(self) -> &'static str {
        match self {
            Self::Stack => "stack",
            Self::Chess => "chess",
            Self::Solitaire => "solitaire",
            Self::Scram => "scram",
            Self::Invaders => "invaders",
            Self::Pinball => "pinball",
            Self::Snake => "snake",
            Self::Bubble => "bubble",
            Self::Blast => "blast",
            Self::TwentyFortyEight => "2048",
            Self::FreeSki => "freeski",
            Self::Tanks => "tanks",
            Self::Minesweeper => "minesweeper",
            Self::Shatter => "shatter",
            Self::Ridgeline => "ridgeline",
        }
    }
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Stack => "Stack",
            Self::Chess => "Chess",
            Self::Solitaire => "Solitaire",
            Self::Scram => "Scram",
            Self::Invaders => "Invaders",
            Self::Pinball => "Circuit Pinball",
            Self::Snake => "Snake",
            Self::Bubble => "Bubble",
            Self::Blast => "Blast",
            Self::TwentyFortyEight => "2048",
            Self::FreeSki => "FreeSki",
            Self::Tanks => "Tanks",
            Self::Minesweeper => "Minesweeper",
            Self::Shatter => "Shatter",
            Self::Ridgeline => "Ridgeline",
        }
    }
    pub(crate) fn line(self) -> &'static str {
        match self {
            Self::Stack => "Make room. Go again.",
            Self::Chess => "Take your time. Make your move.",
            Self::Solitaire => "A quiet hand of Klondike.",
            Self::Scram => "Keep moving. They are behind you.",
            Self::Invaders => "Hold the line. Clear the sky.",
            Self::Pinball => "One more ball. One more high score.",
            Self::Snake => "Eat. Grow. Leave yourself a way out.",
            Self::Bubble => "Make three. Clear your head.",
            Self::Blast => "Make room. Leave an exit.",
            Self::TwentyFortyEight => "Slide together. Make something bigger.",
            Self::FreeSki => "Find your edges. Leave fresh tracks.",
            Self::Tanks => "Read the wind. Change the landscape.",
            Self::Minesweeper => "Read the field. Trust your next move.",
            Self::Shatter => "Find your angle. Break through.",
            Self::Ridgeline => "Hold the pass. Spend every credit well.",
        }
    }
    pub(crate) fn image(self) -> egui::ImageSource<'static> {
        match self {
            Self::Stack => egui::include_image!("../../games/stack/docs/stack-game.png"),
            Self::Snake => egui::include_image!("../../games/snake/docs/shelf.svg"),
            Self::Bubble => egui::include_image!("../../games/bubble/docs/game.png"),
            Self::Blast => egui::include_image!("../../games/blast/docs/game.png"),
            Self::Minesweeper => egui::include_image!("../../games/minesweeper/docs/shelf.svg"),
            Self::Tanks => egui::include_image!("../../games/tanks/shelf.svg"),
            Self::Shatter => egui::include_image!("../../games/shatter/docs/shelf.svg"),
            Self::Ridgeline => egui::include_image!("../../games/ridgeline/docs/shelf.png"),
            Self::TwentyFortyEight => egui::include_image!("../../games/2048/docs/shelf.svg"),
            Self::FreeSki => egui::include_image!("../../games/freeski/assets/shelf.png"),
            Self::Chess => egui::include_image!("../../games/chess/docs/preview.png"),
            Self::Solitaire => {
                egui::include_image!("../../games/solitaire/docs/screenshots/table.png")
            }
            Self::Scram => egui::include_image!("../../games/scram/docs/screenshots/charcoal.png"),
            Self::Invaders => egui::include_image!("../../games/invaders/docs/orbit-opening.png"),
            Self::Pinball => egui::include_image!("../../games/pinball/docs/upstream-circuit.png"),
        }
    }
}
