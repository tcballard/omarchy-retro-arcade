//! One deterministic owner for a complete FreeSki run.

use crate::{
    chase::{ChaseEvent, ChasePhase, TickInput},
    course::{self, GateOutcome},
    endless,
    engine::{Event, Input, Mode, Phase},
    storage::Save,
    world::{self, Obstacle},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CacheKey {
    mode: Mode,
    seed: u64,
    generator: u32,
    course: u8,
    chunk: i64,
    creature_chunk: Option<i64>,
}

/// Authoritative fixed-tick gameplay. UI, evidence and replay callers all use
/// this path; `obstacles` is a bounded, reproducible cache derived from `state`.
pub struct Session {
    pub state: Save,
    pub obstacles: Vec<Obstacle>,
    cache_key: Option<CacheKey>,
}

impl Session {
    pub fn new(state: Save) -> Self {
        let mut session = Self {
            state,
            obstacles: Vec::new(),
            cache_key: None,
        };
        session.refresh();
        session
    }

    pub fn refresh(&mut self) {
        let chunk = match self.state.mode {
            Mode::FreeSki => (self.state.run.position.y / endless::CHUNK_LENGTH).floor() as i64,
            Mode::Practice | Mode::Slalom => 0,
        };
        let key = CacheKey {
            mode: self.state.mode,
            seed: self.state.seed,
            generator: self.state.generator_version,
            course: self.state.course_index,
            chunk,
            creature_chunk: (self.state.mode == Mode::FreeSki
                && self.state.chase_enabled
                && self.state.chase.phase == ChasePhase::Active)
                .then_some((self.state.chase.position.y / endless::CHUNK_LENGTH).floor() as i64),
        };
        if self.cache_key == Some(key) {
            return;
        }
        self.obstacles = match self.state.mode {
            Mode::Practice => world::practice(),
            Mode::FreeSki => {
                let mut obstacles = endless::obstacles_versioned(
                    self.state.seed,
                    self.state.run.position.y,
                    self.state.generator_version,
                );
                if key.creature_chunk.is_some() && key.creature_chunk != Some(key.chunk) {
                    obstacles.extend(endless::obstacles_versioned(
                        self.state.seed,
                        self.state.chase.position.y,
                        self.state.generator_version,
                    ));
                    obstacles.sort_by_key(|obstacle| obstacle.id);
                    obstacles.dedup_by_key(|obstacle| obstacle.id);
                }
                debug_assert!(obstacles.len() <= 176);
                obstacles
            }
            Mode::Slalom => course::course(self.state.course_index)
                .map(|course| course.obstacles)
                .unwrap_or_default(),
        };
        self.cache_key = Some(key);
    }

    pub fn step(&mut self, input: Input) -> Vec<Event> {
        if self.state.run.phase != Phase::Running {
            return Vec::new();
        }
        self.refresh();
        let before = self.state.run.clone();
        let was_protected = self.state.run.tumble > 0 || self.state.run.protection > 0;
        let custom_finish = (self.state.mode == Mode::Slalom)
            .then(|| course::course(self.state.course_index))
            .flatten()
            .filter(|course| self.state.slalom.all_resolved(course))
            .map(|course| course.length);
        let outcome = self.state.run.step_outcome_mode(
            input,
            &self.obstacles,
            self.state.mode,
            custom_finish,
        );
        let mut events = outcome.events;

        if self.state.mode == Mode::Slalom {
            if let Some(course) = course::course(self.state.course_index) {
                for resolution in self.state.slalom.cross_segment(
                    &course,
                    outcome.segment.start,
                    outcome.segment.end,
                ) {
                    events.push(match resolution.outcome {
                        GateOutcome::Passed => Event::Gate,
                        GateOutcome::Missed => Event::Missed,
                    });
                }
            }
        }

        if self.state.mode == Mode::FreeSki && self.state.chase_enabled {
            let plan = self.state.chase.plan_tick(TickInput {
                skier_start: outcome.segment.start,
                skier_end: outcome.segment.proposed_end,
                skier_distance: self.state.run.distance,
                skier_speed: outcome.segment.proposed_speed,
                protected: was_protected,
                obstacles: &self.obstacles,
            });
            let contact_stop = outcome
                .segment
                .crash_fraction
                .or(outcome.segment.finish_fraction)
                .unwrap_or(1.);
            if let Some(catch) = plan.catch_fraction.filter(|catch| *catch <= contact_stop) {
                self.state.chase = plan.commit(catch);
                let turned_heading = outcome.segment.proposed_heading;
                let last_ramp = outcome
                    .segment
                    .ramp_fraction
                    .filter(|ramp| *ramp < catch)
                    .and(self.state.run.last_ramp);
                self.state.run = before;
                self.state.run.ticks += 1;
                self.state.run.position = outcome
                    .segment
                    .start
                    .lerp(outcome.segment.proposed_end, catch);
                self.state.run.distance = self.state.run.distance.max(self.state.run.position.y);
                self.state.run.phase = Phase::Caught;
                self.state.run.speed = 0.;
                self.state.run.heading = turned_heading;
                self.state.run.jump = None;
                self.state.run.tumble = 0;
                self.state.run.protection = 0;
                if last_ramp.is_some() {
                    self.state.run.last_ramp = last_ramp;
                }
                events.retain(|event| {
                    *event == Event::Jump
                        && outcome
                            .segment
                            .ramp_fraction
                            .is_some_and(|ramp| ramp < catch)
                });
                events.push(Event::Caught);
            } else {
                for event in &plan.events {
                    events.push(match event {
                        ChaseEvent::Warning => Event::Warning,
                        ChaseEvent::Spawned => Event::Spawn,
                    });
                }
                self.state.chase = plan.commit(contact_stop);
            }
        } else if self.state.chase.phase != ChasePhase::Dormant {
            // This is unreachable for validated state, but keeps Session-created
            // snapshots causal before a persistence boundary rejects them.
            self.state.chase = Default::default();
        }

        self.state.record_result();
        self.refresh();
        events
    }
}
