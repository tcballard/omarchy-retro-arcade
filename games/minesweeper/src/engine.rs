use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    #[default]
    Beginner,
    Intermediate,
    Expert,
}
impl Difficulty {
    pub const ALL: [Self; 3] = [Self::Beginner, Self::Intermediate, Self::Expert];
    pub fn dimensions(self) -> (usize, usize, usize) {
        match self {
            Self::Beginner => (9, 9, 10),
            Self::Intermediate => (16, 16, 40),
            Self::Expert => (30, 16, 99),
        }
    }
    pub fn index(self) -> usize {
        match self {
            Self::Beginner => 0,
            Self::Intermediate => 1,
            Self::Expert => 2,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Beginner => "Beginner",
            Self::Intermediate => "Intermediate",
            Self::Expert => "Expert",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub mine: bool,
    pub revealed: bool,
    pub flagged: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Ready,
    Playing,
    Won,
    Lost,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub difficulty: Difficulty,
    pub cells: Vec<Cell>,
    pub status: Status,
    pub elapsed_ms: u64,
    pub opening: Option<usize>,
    pub exploded: Option<usize>,
}
impl Game {
    pub fn new(difficulty: Difficulty) -> Self {
        let (w, h, _) = difficulty.dimensions();
        Self {
            difficulty,
            cells: vec![Cell::default(); w * h],
            status: Status::Ready,
            elapsed_ms: 0,
            opening: None,
            exploded: None,
        }
    }
    pub fn neighbours(&self, i: usize) -> Vec<usize> {
        let (w, h, _) = self.difficulty.dimensions();
        if i >= w * h {
            return vec![];
        }
        let (x, y) = (i % w, i / w);
        let mut out = Vec::with_capacity(8);
        for yy in y.saturating_sub(1)..=(y + 1).min(h - 1) {
            for xx in x.saturating_sub(1)..=(x + 1).min(w - 1) {
                if yy * w + xx != i {
                    out.push(yy * w + xx);
                }
            }
        }
        out
    }
    pub fn adjacent(&self, i: usize) -> usize {
        self.neighbours(i)
            .iter()
            .filter(|&&n| self.cells[n].mine)
            .count()
    }
    pub fn remaining(&self) -> isize {
        self.difficulty.dimensions().2 as isize
            - self.cells.iter().filter(|c| c.flagged).count() as isize
    }
    pub fn active(&self) -> bool {
        matches!(self.status, Status::Ready | Status::Playing)
    }
    pub fn flag(&mut self, i: usize) {
        if self.active() {
            if let Some(c) = self.cells.get_mut(i) {
                if !c.revealed {
                    c.flagged = !c.flagged;
                }
            }
        }
    }
    pub fn reveal(&mut self, i: usize, rng: &mut impl Rng) {
        if !self.active() || self.cells.get(i).is_none_or(|c| c.flagged || c.revealed) {
            return;
        }
        if self.status == Status::Ready {
            let mut excluded = self.neighbours(i);
            excluded.push(i);
            let mut candidates: Vec<_> = (0..self.cells.len())
                .filter(|n| !excluded.contains(n))
                .collect();
            candidates.shuffle(rng);
            for &n in candidates.iter().take(self.difficulty.dimensions().2) {
                self.cells[n].mine = true;
            }
            self.opening = Some(i);
            self.status = Status::Playing;
        }
        self.uncover(i);
        self.check_win();
    }
    fn uncover(&mut self, i: usize) {
        let mut pending = vec![i];
        while let Some(n) = pending.pop() {
            if self.cells[n].flagged || self.cells[n].revealed {
                continue;
            }
            self.cells[n].revealed = true;
            if self.cells[n].mine {
                self.exploded = Some(n);
                self.status = Status::Lost;
                return;
            }
            if self.adjacent(n) == 0 {
                pending.extend(self.neighbours(n));
            }
        }
    }
    pub fn chord(&mut self, i: usize) {
        if self.status != Status::Playing || self.cells.get(i).is_none_or(|c| !c.revealed || c.mine)
        {
            return;
        }
        let ns = self.neighbours(i);
        if ns.iter().filter(|&&n| self.cells[n].flagged).count() != self.adjacent(i) {
            return;
        }
        for n in ns {
            self.uncover(n);
            if self.status == Status::Lost {
                break;
            }
        }
        self.check_win();
    }
    fn check_win(&mut self) {
        if self.status == Status::Playing && self.cells.iter().all(|c| c.mine || c.revealed) {
            self.status = Status::Won;
        }
    }
    pub fn valid(&self) -> bool {
        let (w, h, mines) = self.difficulty.dimensions();
        if self.cells.len() != w * h || self.cells.iter().any(|c| c.flagged && c.revealed) {
            return false;
        }
        if self.status == Status::Ready {
            return self.opening.is_none()
                && self.exploded.is_none()
                && self.elapsed_ms == 0
                && self.cells.iter().all(|c| !c.mine && !c.revealed);
        }
        let Some(opening) = self.opening.filter(|&i| i < w * h) else {
            return false;
        };
        if !self.cells[opening].revealed
            || self.cells[opening].mine
            || self.adjacent(opening) != 0
            || self.cells.iter().filter(|c| c.mine).count() != mines
        {
            return false;
        }
        let exposed: Vec<_> = self
            .cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.mine && c.revealed)
            .map(|(i, _)| i)
            .collect();
        let won = self.cells.iter().all(|c| c.mine || c.revealed);
        match self.status {
            Status::Playing => self.exploded.is_none() && exposed.is_empty() && !won,
            Status::Won => self.exploded.is_none() && exposed.is_empty() && won,
            Status::Lost => self.exploded.is_some_and(|i| exposed == vec![i]) && !won,
            Status::Ready => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};
    #[test]
    fn safe_openings_all_presets_and_positions() {
        for d in Difficulty::ALL {
            let (w, h, mines) = d.dimensions();
            for i in [0, w - 1, w * (h - 1), w * h - 1, w * (h / 2) + w / 2] {
                for seed in 0..32 {
                    let mut g = Game::new(d);
                    g.reveal(i, &mut StdRng::seed_from_u64(seed));
                    assert!(g.valid());
                    assert_eq!(g.adjacent(i), 0);
                    assert_eq!(g.cells.iter().filter(|c| c.mine).count(), mines);
                }
            }
        }
    }
    #[test]
    fn flags_block_reveal_and_flood_and_allow_negative_counter() {
        let mut g = Game::new(Difficulty::Beginner);
        g.flag(1);
        g.reveal(1, &mut StdRng::seed_from_u64(0));
        assert_eq!(g.status, Status::Ready);
        g.reveal(0, &mut StdRng::seed_from_u64(0));
        assert!(!g.cells[1].revealed);
        assert_eq!(g.neighbours(0), vec![1, 9, 10]);
        for i in 0..g.cells.len() {
            if !g.cells[i].revealed && !g.cells[i].flagged {
                g.flag(i);
            }
        }
        assert!(g.remaining() < 0);
        assert!(g.valid());
    }
    #[test]
    fn correct_and_incorrect_chords() {
        let mut rng = StdRng::seed_from_u64(24);
        let mut g = Game::new(Difficulty::Beginner);
        g.reveal(0, &mut rng);
        let i = (0..g.cells.len())
            .find(|&i| {
                g.cells[i].revealed
                    && g.adjacent(i) > 0
                    && g.neighbours(i)
                        .iter()
                        .any(|&n| !g.cells[n].mine && !g.cells[n].revealed)
            })
            .unwrap();
        let mut bad = g.clone();
        let ns = g.neighbours(i);
        for &n in &ns {
            if g.cells[n].mine {
                g.flag(n);
            }
        }
        g.chord(i);
        assert_ne!(g.status, Status::Lost);
        assert!(g.valid());
        let safe = *ns
            .iter()
            .find(|&&n| !bad.cells[n].mine && !bad.cells[n].revealed)
            .unwrap();
        bad.flag(safe);
        let mines: Vec<_> = ns.iter().copied().filter(|&n| bad.cells[n].mine).collect();
        for &n in mines.iter().skip(1) {
            bad.flag(n);
        }
        bad.chord(i);
        assert_eq!(bad.status, Status::Lost);
        assert!(bad.valid());
        let frozen = bad.clone();
        bad.flag(safe);
        bad.reveal(2, &mut rng);
        bad.chord(i);
        assert_eq!(bad, frozen);
    }
    #[test]
    fn wins_without_flags_and_finished_board_is_frozen() {
        let mut g = Game::new(Difficulty::Expert);
        let mut rng = StdRng::seed_from_u64(3);
        g.reveal(0, &mut rng);
        for i in 0..g.cells.len() {
            if !g.cells[i].mine {
                g.reveal(i, &mut rng);
            }
        }
        assert_eq!(g.status, Status::Won);
        assert!(g.valid());
        let frozen = g.clone();
        g.reveal(10, &mut rng);
        g.flag(0);
        g.chord(0);
        assert_eq!(g, frozen);
    }
    #[test]
    fn arbitrary_actions_preserve_invariants() {
        for d in Difficulty::ALL {
            for seed in 0..50 {
                let mut rng = StdRng::seed_from_u64(seed);
                let mut g = Game::new(d);
                for _ in 0..300 {
                    let i = rng.gen_range(0..g.cells.len());
                    match rng.gen_range(0..3) {
                        0 => g.flag(i),
                        1 => g.reveal(i, &mut rng),
                        _ => g.chord(i),
                    }
                    assert!(g.valid());
                }
            }
        }
    }
    #[test]
    fn invalid_state_is_rejected() {
        let mut g = Game::new(Difficulty::Beginner);
        g.cells.pop();
        assert!(!g.valid());
        let mut g = Game::new(Difficulty::Beginner);
        g.cells[0].mine = true;
        assert!(!g.valid());
        let mut g = Game::new(Difficulty::Beginner);
        g.reveal(0, &mut StdRng::seed_from_u64(1));
        g.opening = Some(9999);
        assert!(!g.valid());
    }
}
