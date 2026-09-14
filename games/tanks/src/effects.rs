//! Saved presentation timing over an already-resolved engine impact.
use crate::rules::{Game, Impact, Phase, FLOOR, WIDTH};
pub const IMPACT_TICKS: u16 = 108;
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Resolution {
    pub impact: Impact,
    pub ticks: u16,
}
impl Resolution {
    pub fn new(impact: Impact) -> Self {
        Self { impact, ticks: 0 }
    }
    pub fn advance(&mut self) -> bool {
        self.ticks = self.ticks.saturating_add(1);
        self.ticks >= IMPACT_TICKS
    }
    pub fn progress(&self) -> f32 {
        f32::from(self.ticks) / f32::from(IMPACT_TICKS)
    }
    pub fn ground_progress(&self, reduced: bool) -> f64 {
        if reduced {
            1.0
        } else {
            smooth((f64::from(self.progress()) - 0.08) / 0.42)
        }
    }
    pub fn tank_progress(&self, reduced: bool) -> f64 {
        if reduced {
            1.0
        } else {
            smooth((f64::from(self.progress()) - 0.3) / 0.45)
        }
    }
    pub fn valid(&self, game: &Game) -> bool {
        let i = &self.impact;
        i.shooter < 2
            && self.ticks < IMPACT_TICKS
            && !matches!(game.phase(), Phase::Flying | Phase::Aiming)
            && i.terrain_before.len() == WIDTH + 1
            && i.terrain_before
                .iter()
                .zip(game.terrain().heights())
                .all(|(a, b)| a.is_finite() && (280.0..=FLOOR).contains(a) && a <= b)
            && i.at.x.is_finite()
            && (0.0..=1000.0).contains(&i.at.x)
            && i.at.y.is_finite()
            && (0.0..=FLOOR).contains(&i.at.y)
            && (0..2).all(|n| {
                let old = &i.tanks_before[n];
                let new = &game.tanks()[n];
                old.position.x == new.position.x
                    && old.position.y.is_finite()
                    && (280.0..=new.position.y).contains(&old.position.y)
                    && old.health <= 100
                    && i.blast[n] <= old.health
                    && i.fall[n] <= old.health - i.blast[n]
                    && old.health - i.blast[n] - i.fall[n] == new.health
                    && old.angle == new.angle
                    && old.power == new.power
                    && old.weapon == new.weapon
                    && old.fuel == new.fuel
                    && old.heavy == new.heavy
                    && old.diggers == new.diggers
            })
    }
}
fn smooth(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presentation_never_mutates_rules_and_roundtrips_mid_settle() {
        let mut g = Game::new(42);
        g.ready().unwrap();
        g.aim(90, 1).unwrap();
        g.fire().unwrap();
        let impact = (0..2400).find_map(|_| g.tick_event()).expect("impact");
        let snapshot = g.clone();
        let mut r = Resolution::new(impact);
        assert!(r.valid(&g));
        for _ in 0..43 {
            assert!(!r.advance());
        }
        let bytes = serde_json::to_vec(&r).unwrap();
        let mut resumed: Resolution = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(r, resumed);
        assert_eq!(r.ground_progress(true), 1.0);
        assert_eq!(r.tank_progress(true), 1.0);
        for _ in 43..IMPACT_TICKS {
            assert_eq!(r.advance(), resumed.advance());
        }
        assert_eq!(g, snapshot);
        assert_eq!(r.ticks, IMPACT_TICKS);
    }
}
