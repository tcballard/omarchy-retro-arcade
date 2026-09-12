//! Rust adaptation of avibarit/2048 game.js, revision 3f10becf.
//! Original implementation: Avi Barit (MIT). Original 2048: Gabriele Cirulli.
//! See ../THIRD_PARTY.md for source and licence notices.
use rand::Rng;
use serde::{Deserialize, Serialize};

pub type Board = [[u64; 4]; 4];
pub type Cell = [usize; 2];
pub const UNDO_LIMIT: usize = 20;
// Larger than the largest attainable tile on a classic 4x4 board.
pub const MAX_TILE: u64 = 1 << 32;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
impl Direction {
    fn cell(self, line: usize, offset: usize) -> Cell {
        match self {
            Self::Left => [line, offset],
            Self::Right => [line, 3 - offset],
            Self::Up => [offset, line],
            Self::Down => [3 - offset, line],
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Motion {
    pub from: Cell,
    pub to: Cell,
    pub value: u64,
}
#[derive(Clone, Debug, Serialize)]
pub struct Turn {
    pub board: Board,
    pub score_delta: u64,
    pub moved: bool,
    pub merges: Vec<Cell>,
    pub moves: Vec<Motion>,
    pub spawned: Option<Cell>,
}

/// Pack each line in movement order, consuming equal source tiles once.
/// Metadata describes source-to-destination motion, never a second game state.
pub fn slide(board: &Board, direction: Direction) -> Turn {
    let mut turn = Turn {
        board: [[0; 4]; 4],
        score_delta: 0,
        moved: false,
        merges: Vec::new(),
        moves: Vec::new(),
        spawned: None,
    };
    for line in 0..4 {
        let packed: Vec<_> = (0..4)
            .map(|i| direction.cell(line, i))
            .filter(|[r, c]| board[*r][*c] != 0)
            .collect();
        let (mut i, mut out) = (0, 0);
        while i < packed.len() {
            let from = packed[i];
            let value = board[from[0]][from[1]];
            let to = direction.cell(line, out);
            let merge = i + 1 < packed.len()
                && value < MAX_TILE
                && value == board[packed[i + 1][0]][packed[i + 1][1]];
            turn.moves.push(Motion { from, to, value });
            turn.board[to[0]][to[1]] = if merge {
                turn.moves.push(Motion {
                    from: packed[i + 1],
                    to,
                    value,
                });
                turn.merges.push(to);
                turn.score_delta += value * 2;
                i += 1;
                value * 2
            } else {
                value
            };
            i += 1;
            out += 1;
        }
    }
    turn.moved = turn.board != *board;
    turn
}
pub fn spawn(board: &mut Board, rng: &mut impl Rng) -> Option<Cell> {
    let empty: Vec<_> = (0..4)
        .flat_map(|r| (0..4).map(move |c| [r, c]))
        .filter(|[r, c]| board[*r][*c] == 0)
        .collect();
    if empty.is_empty() {
        return None;
    }
    let cell = empty[rng.gen_range(0..empty.len())];
    board[cell[0]][cell[1]] = if rng.gen_bool(0.9) { 2 } else { 4 };
    Some(cell)
}
pub fn won(board: &Board) -> bool {
    board.iter().flatten().any(|v| *v >= 2048)
}
pub fn over(board: &Board) -> bool {
    !board.iter().flatten().any(|v| *v == 0)
        && [
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ]
        .iter()
        .all(|d| !slide(board, *d).moved)
}
pub fn valid(board: &Board) -> bool {
    board
        .iter()
        .flatten()
        .all(|v| *v == 0 || (*v >= 2 && *v <= MAX_TILE && v.is_power_of_two()))
        && board.iter().flatten().filter(|v| **v != 0).count() >= 2
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snapshot {
    pub board: Board,
    pub score: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Game {
    pub board: Board,
    pub score: u64,
    pub best: u64,
    pub continued: bool,
    pub history: Vec<Snapshot>,
}
impl Game {
    pub fn new(rng: &mut impl Rng, best: u64) -> Self {
        let mut board = [[0; 4]; 4];
        spawn(&mut board, rng);
        spawn(&mut board, rng);
        Self {
            board,
            score: 0,
            best,
            continued: false,
            history: Vec::new(),
        }
    }
    pub fn win_prompt(&self) -> bool {
        !self.continued && won(&self.board)
    }
    pub fn play(&mut self, direction: Direction, rng: &mut impl Rng) -> Option<Turn> {
        if self.win_prompt() {
            return None;
        }
        let mut turn = slide(&self.board, direction);
        if !turn.moved {
            return None;
        }
        self.history.push(Snapshot {
            board: self.board,
            score: self.score,
        });
        if self.history.len() > UNDO_LIMIT {
            self.history.remove(0);
        }
        turn.spawned = spawn(&mut turn.board, rng);
        self.board = turn.board;
        self.score = self.score.saturating_add(turn.score_delta);
        self.best = self.best.max(self.score);
        Some(turn)
    }
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.history.pop() else {
            return false;
        };
        self.board = previous.board;
        self.score = previous.score;
        self.continued &= won(&self.board);
        true
    }
    pub fn keep_going(&mut self) {
        if won(&self.board) {
            self.continued = true;
        }
    }
    pub fn valid(&self) -> bool {
        valid(&self.board)
            && self.best >= self.score
            && self.history.len() <= UNDO_LIMIT
            && (!self.continued || won(&self.board))
            && self
                .history
                .iter()
                .all(|s| valid(&s.board) && s.score <= self.best)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};
    fn rng() -> StdRng {
        StdRng::seed_from_u64(2048)
    }
    #[test]
    fn classic_merge_once_cases() {
        for (row, expected, score) in [
            ([2, 2, 2, 2], [4, 4, 0, 0], 8),
            ([4, 2, 2, 2], [4, 4, 2, 0], 4),
            ([2, 2, 4, 4], [4, 8, 0, 0], 12),
            ([2, 2, 2, 0], [4, 2, 0, 0], 4),
            ([4, 8, 16, 32], [4, 8, 16, 32], 0),
            ([0, 4, 0, 4], [8, 0, 0, 0], 8),
        ] {
            let mut b = [[0; 4]; 4];
            b[0] = row;
            let t = slide(&b, Direction::Left);
            assert_eq!(t.board[0], expected);
            assert_eq!(t.score_delta, score);
        }
    }
    #[test]
    fn upstream_stuck_128_regression() {
        let b = [
            [2, 128, 2, 32],
            [16, 128, 32, 16],
            [4, 256, 16, 2],
            [2, 2, 0, 4],
        ];
        for d in [Direction::Up, Direction::Down] {
            assert_eq!(slide(&b, d).score_delta, 256);
        }
        let t = slide(&b, Direction::Up);
        assert_eq!([t.board[0][1], t.board[1][1], t.board[2][1]], [256, 256, 2]);
    }
    #[test]
    fn motion_coordinates_follow_each_direction() {
        let b = [[2, 0, 0, 2], [0; 4], [0, 2, 0, 0], [0, 0, 2, 0]];
        for d in [
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ] {
            let t = slide(&b, d);
            let mut sums = [[0; 4]; 4];
            for m in &t.moves {
                assert_eq!(m.value, b[m.from[0]][m.from[1]]);
                sums[m.to[0]][m.to[1]] += m.value;
            }
            assert_eq!(sums, t.board);
        }
    }
    #[test]
    fn invalid_moves_do_not_spawn_score_or_consume_rng() {
        let mut r = rng();
        let mut g = Game::new(&mut r, 0);
        g.board = [
            [2, 4, 8, 16],
            [32, 64, 128, 256],
            [512, 1024, 2, 4],
            [8, 16, 32, 64],
        ];
        let before = g.clone();
        let mut control = r.clone();
        assert!(g.play(Direction::Left, &mut r).is_none());
        assert_eq!(g, before);
        assert_eq!(r.gen::<u64>(), control.gen::<u64>());
    }
    #[test]
    fn starts_with_two_and_spawns_only_in_empty_cells() {
        let mut r = rng();
        let g = Game::new(&mut r, 77);
        assert_eq!(g.board.iter().flatten().filter(|v| **v != 0).count(), 2);
        assert_eq!(g.best, 77);
        let mut b = [[8; 4]; 4];
        b[2][3] = 0;
        assert_eq!(spawn(&mut b, &mut r), Some([2, 3]));
        assert!([2, 4].contains(&b[2][3]));
        assert_eq!(b[1], [8; 4]);
        assert_eq!(spawn(&mut b, &mut r), None);
    }
    #[test]
    fn win_continue_and_undo_preserve_best() {
        let mut r = rng();
        let mut g = Game::new(&mut r, 0);
        g.board = [[1024, 1024, 2, 0], [0; 4], [0; 4], [0; 4]];
        let before = g.board;
        g.play(Direction::Left, &mut r).unwrap();
        assert!(g.win_prompt());
        assert!(g.play(Direction::Right, &mut r).is_none());
        g.keep_going();
        assert!(!g.win_prompt());
        assert!(g.undo());
        assert_eq!(g.board, before);
        assert_eq!(g.score, 0);
        assert_eq!(g.best, 2048);
        assert!(!g.continued);
    }
    #[test]
    fn full_board_needs_adjacent_equals() {
        let mut b = [
            [2, 4, 8, 16],
            [4, 8, 16, 32],
            [8, 16, 32, 64],
            [16, 32, 64, 128],
        ];
        assert!(over(&b));
        b[3][3] = 64;
        assert!(!over(&b));
        b[3][3] = 0;
        assert!(!over(&b));
    }
    #[test]
    fn rapid_moves_remain_authoritative_and_history_bounded() {
        let mut r = rng();
        let mut g = Game::new(&mut r, 0);
        for _ in 0..100 {
            for d in [
                Direction::Left,
                Direction::Down,
                Direction::Right,
                Direction::Up,
            ] {
                if let Some(t) = g.play(d, &mut r) {
                    assert_eq!(t.board, g.board);
                }
                assert!(g.valid());
                assert!(g.history.len() <= 20);
            }
        }
        let n = g.history.len();
        for _ in 0..n {
            assert!(g.undo());
        }
        assert!(!g.undo());
    }
    #[test]
    fn bad_tiles_are_rejected() {
        let mut g = Game::new(&mut rng(), 0);
        g.board[0][0] = 3;
        assert!(!g.valid());
        g.board[0][0] = u64::MAX;
        assert!(!g.valid());
    }
}
