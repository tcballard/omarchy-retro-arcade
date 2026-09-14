//! Incremental, cancellable shot search using the ordinary engine.
use crate::rules::{Game, Phase, Rejected, Weapon};

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Normal,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shot {
    pub angle: u16,
    pub power: u16,
    pub weapon: Weapon,
    pub movement: i8,
}
impl Shot {
    fn commit(self, game: &mut Game) -> Result<(), Rejected> {
        for _ in 0..self.movement.unsigned_abs() {
            game.move_one(self.movement > 0)?;
        }
        game.aim(self.angle, self.power)?;
        game.select(self.weapon)?;
        game.fire()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Search {
    source: Game,
    candidates: Vec<Shot>,
    cursor: usize,
    trial: Option<Game>,
    best: Option<(Shot, i32)>,
    ticks: u64,
    next_seed: u64,
    ammo_evaluated: bool,
}
impl Search {
    /// Start only after the AI's Ready handover has been acknowledged.
    pub fn new(game: &Game, difficulty: Difficulty, seed: u64) -> Option<Self> {
        if game.phase() != Phase::Aiming {
            return None;
        }
        let mut next_seed = seed;
        let mut candidates = Vec::new();
        // Both difficulties share a noisy coarse pass. Normal adds finer shots
        // and legal repositioning. Error is sampled before simulation: the bot
        // assesses the actual shot it will take rather than applying a surprise
        // perturbation that could turn a safe choice into self-destruction.
        for angle in (10..=170).step_by(20) {
            for power in (30..=100).step_by(14) {
                let error = (random(&mut next_seed) % 7) as i16 - 3;
                candidates.push(Shot {
                    angle: (angle + error).clamp(5, 175) as u16,
                    power,
                    weapon: Weapon::Shell,
                    movement: 0,
                });
            }
        }
        if difficulty == Difficulty::Normal {
            for angle in (10..=170).step_by(10) {
                for power in (35..=100).step_by(10) {
                    let error = (random(&mut next_seed) % 3) as i16 - 1;
                    candidates.push(Shot {
                        angle: (angle + error).clamp(5, 175) as u16,
                        power,
                        weapon: Weapon::Shell,
                        movement: 0,
                    });
                }
            }
            // A small movement search uses exactly the same slope/fuel rules.
            for movement in [-12, 12] {
                for angle in [30, 50, 70, 110, 130, 150] {
                    for power in [55, 75, 95] {
                        candidates.push(Shot {
                            angle,
                            power,
                            weapon: Weapon::Shell,
                            movement,
                        });
                    }
                }
            }
        }
        Some(Self {
            source: game.clone(),
            candidates,
            cursor: 0,
            trial: None,
            best: None,
            ticks: 0,
            next_seed,
            ammo_evaluated: false,
        })
    }
    /// Advance at most 4096 production ticks and 32 candidate starts per call.
    /// Pausing means not calling this method; cancellation means dropping Search.
    pub fn advance(&mut self, budget: usize) -> bool {
        let mut remaining = budget.min(4096);
        let mut starts = 0;
        while remaining > 0 && self.cursor < self.candidates.len() {
            if self.trial.is_none() {
                if starts == 32 {
                    break;
                }
                starts += 1;
                let mut trial = self.source.clone();
                if self.candidates[self.cursor].commit(&mut trial).is_err() {
                    self.cursor += 1;
                    continue;
                }
                self.trial = Some(trial);
            }
            let trial = self.trial.as_mut().expect("candidate owns a simulation");
            trial.tick();
            remaining -= 1;
            self.ticks += 1;
            if trial.phase() != Phase::Flying {
                let score = utility(&self.source, trial, self.candidates[self.cursor]);
                if self.best.is_none_or(|(_, best)| score > best) {
                    self.best = Some((self.candidates[self.cursor], score));
                }
                self.trial = None;
                self.cursor += 1;
                if self.cursor == self.candidates.len() && !self.ammo_evaluated {
                    self.ammo_evaluated = true;
                    // Once only: compare limited ammunition at the best aim.
                    if let Some((shot, _)) = self.best {
                        if shot.weapon == Weapon::Shell {
                            for weapon in [Weapon::Heavy, Weapon::Digger] {
                                if self.source.tanks()[self.source.active()].available(weapon) {
                                    self.candidates.push(Shot { weapon, ..shot });
                                }
                            }
                        }
                    }
                }
            }
        }
        self.is_done()
    }
    pub fn is_done(&self) -> bool {
        self.cursor == self.candidates.len()
    }
    pub fn ticks(&self) -> u64 {
        self.ticks
    }
    pub fn next_seed(&self) -> u64 {
        self.next_seed
    }
    pub fn best(&self) -> Option<Shot> {
        self.best.map(|(shot, _)| shot)
    }
    /// No partial movement or stale result is ever committed to the live game.
    pub fn apply(&self, game: &mut Game) -> Result<(), Rejected> {
        if !self.is_done() || *game != self.source {
            return Err(Rejected::WrongPhase);
        }
        let mut next = game.clone();
        self.best().ok_or(Rejected::WrongPhase)?.commit(&mut next)?;
        *game = next;
        Ok(())
    }
}
fn random(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state ^ (*state >> 29)
}
fn utility(before: &Game, after: &Game, shot: Shot) -> i32 {
    let me = before.active();
    let them = 1 - me;
    let own_loss = i32::from(before.tanks()[me].health) - i32::from(after.tanks()[me].health);
    let enemy_loss = i32::from(before.tanks()[them].health) - i32::from(after.tanks()[them].health);
    let death = if after.tanks()[me].health == 0 {
        100_000
    } else {
        0
    };
    let win = if after.tanks()[them].health == 0 {
        10_000
    } else {
        0
    };
    let ammo = match shot.weapon {
        Weapon::Shell => 0,
        Weapon::Heavy => 80,
        Weapon::Digger => 40,
    };
    enemy_loss * 100 - own_loss * 180 + win - death - ammo - i32::from(shot.movement.unsigned_abs())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn complete(search: &mut Search, budget: usize) {
        for _ in 0..20_000 {
            if search.advance(budget) {
                return;
            }
        }
        panic!("search failed to terminate");
    }
    #[test]
    fn computer_players_complete_a_match_with_valid_persistable_states() {
        let mut game = Game::new(42);
        let mut seed = 999;
        for turn in 0..300 {
            assert!(game.valid(), "invalid state at turn {turn}");
            match game.phase() {
                Phase::MatchOver { .. } => return,
                Phase::RoundOver { .. } => {
                    game.next_round().unwrap();
                }
                Phase::Ready => {
                    game.ready().unwrap();
                }
                Phase::Aiming => {
                    let mut search = Search::new(&game, Difficulty::Normal, seed).unwrap();
                    complete(&mut search, 4096);
                    seed = search.next_seed();
                    search.apply(&mut game).unwrap();
                }
                Phase::Flying => {
                    for _ in 0..2400 {
                        game.tick();
                    }
                }
            }
        }
        panic!("computer match did not finish");
    }
    #[test]
    fn budgets_pause_cloning_and_stale_results() {
        let mut game = Game::new(42);
        assert!(Search::new(&game, Difficulty::Easy, 123).is_none());
        game.ready().unwrap();
        let mut search = Search::new(&game, Difficulty::Easy, 123).unwrap();
        let before = search.clone();
        assert!(!search.advance(0));
        assert_eq!(search, before);
        search.advance(17);
        assert_eq!(search.ticks(), 17);
        let mut resumed = search.clone();
        complete(&mut search, 4096);
        complete(&mut resumed, 113);
        assert_eq!(search, resumed);
        let original = game.clone();
        search.apply(&mut game).unwrap();
        assert_eq!(game.phase(), Phase::Flying);
        assert_eq!(search.apply(&mut game), Err(Rejected::WrongPhase));
        let mut changed = original;
        changed.aim(90, 1).unwrap();
        let snapshot = changed.clone();
        assert_eq!(search.apply(&mut changed), Err(Rejected::WrongPhase));
        assert_eq!(changed, snapshot);
    }
    #[test]
    fn normal_search_improves_coarse_shots_and_respects_call_limits() {
        let mut improved = 0;
        for seed in 0..12 {
            let mut game = Game::new(seed);
            game.ready().unwrap();
            let mut easy = Search::new(&game, Difficulty::Easy, 456).unwrap();
            let mut normal = Search::new(&game, Difficulty::Normal, 456).unwrap();
            complete(&mut easy, 4096);
            for _ in 0..1000 {
                let ticks = normal.ticks();
                let done = normal.advance(usize::MAX);
                assert!(normal.ticks() - ticks <= 4096);
                if done {
                    break;
                }
            }
            assert!(normal.is_done());
            let easy_score = easy.best.unwrap().1;
            let normal_score = normal.best.unwrap().1;
            // Normal may pick a different coarse aim before ammo evaluation;
            // compare actual aggregate improvement rather than promise dominance.
            if normal_score > easy_score {
                improved += 1;
            }
            let mut live = game.clone();
            normal.apply(&mut live).unwrap();
            assert_eq!(live.phase(), Phase::Flying);
        }
        assert!(improved >= 6, "Normal improved only {improved}/12 fixtures");
    }
}
