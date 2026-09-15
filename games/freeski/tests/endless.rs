use std::collections::{BTreeMap, BTreeSet};

use omarchy_freeski::{
    endless::{self, CHUNK_LENGTH},
    engine::{Event, Input, Mode, Point, Sim},
    world::{Kind, Obstacle, HALF_WIDTH},
};

fn signature(obstacle: &Obstacle) -> (usize, u64, u64, u8) {
    (
        obstacle.id,
        obstacle.at.x.to_bits(),
        obstacle.at.y.to_bits(),
        match obstacle.kind {
            Kind::Tree => 0,
            Kind::Rock => 1,
            Kind::Ramp => 2,
            Kind::Pole => 3,
        },
    )
}

#[test]
fn generator_one_preserves_exact_saved_mountain_geometry() {
    assert_eq!(
        endless::chunk_versioned(17, 3, 1)
            .iter()
            .map(signature)
            .collect::<Vec<_>>(),
        vec![
            (192, 13845559848042601621, 4646489275168701679, 2),
            (193, 13845645894051310767, 4646770750145412335, 1),
            (194, 4623958276977946684, 4646764348637839073, 0),
            (195, 4626554050233680788, 4646942267122577452, 0),
            (196, 4612237854219767536, 4647283341495942603, 0),
            (197, 4622048295485826388, 4647441317581374463, 0),
            (198, 4623077280200258032, 4645910863699675365, 0),
            (199, 13850277120807198758, 4647362299407416528, 0),
            (200, 4622369900347231740, 4646180305422926082, 0),
            (201, 13850490208752917678, 4647100387426936591, 0),
            (202, 13850439494551000650, 4646679125092628488, 0),
        ]
    );
}

#[test]
fn windows_are_deterministic_bounded_and_cover_recovery_overlap() {
    for seed in 0..128 {
        for chunk in [0_u64, 1, 2, 9, 40, 400, 4_000] {
            let middle = chunk as f64 * CHUNK_LENGTH + CHUNK_LENGTH * 0.5;
            let a = endless::obstacles(seed, middle);
            let b = endless::obstacles(seed, middle);
            assert_eq!(
                a.iter().map(signature).collect::<Vec<_>>(),
                b.iter().map(signature).collect::<Vec<_>>()
            );
            assert!(a.len() <= 88, "seed {seed}, chunk {chunk}: {}", a.len());
            assert_eq!(
                a.len(),
                a.iter()
                    .map(|obstacle| obstacle.id)
                    .collect::<BTreeSet<_>>()
                    .len()
            );

            let first = chunk.saturating_sub(1);
            let last = chunk + 2;
            assert!(a.iter().all(|obstacle| {
                let obstacle_chunk = obstacle.id / 64;
                obstacle_chunk >= first as usize && obstacle_chunk <= last as usize
            }));
            if chunk > 0 {
                assert!(a.iter().any(|obstacle| obstacle.id / 64 == first as usize));
            }
            assert!(a.iter().any(|obstacle| obstacle.id / 64 == last as usize));
        }
    }
    let far = endless::obstacles(7, f64::MAX);
    assert!(far.len() <= 88);
    assert_eq!(
        far.len(),
        far.iter()
            .map(|obstacle| obstacle.id)
            .collect::<BTreeSet<_>>()
            .len()
    );
}

#[test]
fn shared_chunks_are_identical_across_window_boundaries() {
    for seed in 0..128 {
        for chunk in 1..24_u64 {
            let before = endless::obstacles(seed, chunk as f64 * CHUNK_LENGTH - 0.001);
            let after = endless::obstacles(seed, chunk as f64 * CHUNK_LENGTH + 0.001);
            let before = before
                .iter()
                .map(|obstacle| (obstacle.id, signature(obstacle)))
                .collect::<BTreeMap<_, _>>();
            let after = after
                .iter()
                .map(|obstacle| (obstacle.id, signature(obstacle)))
                .collect::<BTreeMap<_, _>>();
            let shared = before
                .keys()
                .filter(|id| after.contains_key(id))
                .collect::<Vec<_>>();
            assert!(!shared.is_empty());
            for id in shared {
                assert_eq!(before[id], after[id]);
            }
        }
    }
}

#[test]
fn generator_one_edge_recovery_corridors_stay_clear_for_saved_runs() {
    const SKIER_RADIUS: f64 = 0.65;
    const RECOVERY_MARGIN: f64 = 2.;
    for seed in 0..128 {
        for chunk in 0..80_u64 {
            let obstacles =
                endless::obstacles_versioned(seed, chunk as f64 * CHUNK_LENGTH + 64., 1);
            for obstacle in obstacles {
                for x in [-36., 36.] {
                    assert!(
                        (obstacle.at.x - x).abs()
                            > obstacle.radius() + SKIER_RADIUS + RECOVERY_MARGIN,
                        "seed {seed}, obstacle {} blocks recovery at {x}",
                        obstacle.id
                    );
                }
                assert!(obstacle.at.x.abs() + obstacle.radius() < HALF_WIDTH);
            }
        }
    }
}

#[test]
fn generator_two_pressures_both_fixed_edge_lines_without_blocking_recovery() {
    const SKIER_RADIUS: f64 = 0.65;
    const RECOVERY_MARGIN: f64 = 2.;
    for seed in 0..128 {
        for chunk in 0..80_u64 {
            let obstacles = endless::chunk_versioned(seed, chunk, 2);
            for x in [-38.8, -36., 36., 38.8] {
                assert!(
                    obstacles.iter().any(|obstacle| {
                        (obstacle.at.x - x).abs() <= obstacle.radius() + SKIER_RADIUS
                    }),
                    "seed {seed}, chunk {chunk} leaves fixed edge {x} clear"
                );
            }

            // At every hazard's downhill level, at least one of the two engine
            // fallback points remains genuinely clear by production clearance.
            for hazard in &obstacles {
                assert!(
                    [-36., 36.].into_iter().any(|x| {
                        obstacles.iter().all(|other| {
                            (other.at.x - x).hypot(other.at.y - hazard.at.y)
                                > other.radius() + SKIER_RADIUS + RECOVERY_MARGIN
                        })
                    }),
                    "seed {seed}, chunk {chunk} blocks both recovery edges at {:.2}",
                    hazard.at.y
                );
            }
        }
    }
}

#[test]
fn density_caps_and_every_seed_offers_useful_jumps() {
    for seed in 0..128 {
        let mut ramps = 0;
        for chunk in 1..64_u64 {
            let at = chunk as f64 * CHUNK_LENGTH + 64.;
            let window = endless::obstacles(seed, at);
            let in_chunk = window
                .iter()
                .filter(|obstacle| obstacle.id / 64 == chunk as usize)
                .collect::<Vec<_>>();
            assert!(in_chunk.len() <= 22);
            for ramp in in_chunk
                .iter()
                .filter(|obstacle| obstacle.kind == Kind::Ramp)
            {
                ramps += 1;
                assert!(endless::is_ramp(seed, ramp.id));
                assert!(in_chunk.iter().any(|obstacle| {
                    obstacle.id == ramp.id + 1
                        && obstacle.kind == Kind::Rock
                        && obstacle.at.y > ramp.at.y
                        && obstacle.at.y - ramp.at.y <= 16.001
                }));
            }
        }
        assert!(ramps >= 10, "seed {seed} generated only {ramps} ramps");
    }
}

#[test]
fn reference_heading_is_finite_and_within_production_limits() {
    for seed in 0..128 {
        let mut sim = Sim::default();
        for chunk in 0..100_u64 {
            sim.position = Point {
                x: ((chunk as i64 % 9) - 4) as f64 * 4.,
                y: chunk as f64 * CHUNK_LENGTH + 31.,
            };
            let heading = endless::reference_heading(seed, &sim);
            assert!(heading.is_finite());
            assert!(heading.abs() <= std::f64::consts::FRAC_PI_2);
        }
    }
}

#[test]
fn production_engine_completes_a_long_seed_corpus_through_the_reserved_route() {
    let mut total_jumps = 0;
    for seed in 0..128 {
        let mut sim = Sim::default();
        sim.start();
        let mut active_chunk = u64::MAX;
        let mut obstacles = vec![];
        for _ in 0..12_000 {
            let chunk = (sim.position.y / CHUNK_LENGTH).floor() as u64;
            if chunk != active_chunk {
                active_chunk = chunk;
                obstacles = endless::obstacles(seed, sim.position.y);
            }
            let events = sim.step_mode(
                Input {
                    heading: endless::reference_heading(seed, &sim),
                    brake: false,
                },
                &obstacles,
                Mode::FreeSki,
            );
            total_jumps += events.iter().filter(|event| **event == Event::Jump).count();
            assert_eq!(
                sim.crashes, 0,
                "seed {seed} left the reserved route at ({:.2}, {:.2})",
                sim.position.x, sim.position.y
            );
            if sim.distance >= 5_000. {
                break;
            }
            assert!(!sim.ended(), "seed {seed} ended at {:.1} m", sim.distance);
        }
        assert!(
            sim.distance >= 5_000.,
            "seed {seed} reached only {:.1} m",
            sim.distance
        );
        assert!(sim.valid_mode(&obstacles, Mode::FreeSki));
    }
    assert!(
        total_jumps >= 1_000,
        "corpus produced only {total_jumps} jumps"
    );
}

#[test]
fn fast_mode_can_follow_the_connected_route_at_ninety_metres_per_second() {
    for seed in 0..64 {
        let mut sim = Sim::default();
        sim.start();
        assert!(sim.toggle_fast_mode());
        let mut active_chunk = u64::MAX;
        let mut obstacles = vec![];
        let mut maximum_speed = 0_f64;
        for _ in 0..8_000 {
            let chunk = (sim.position.y / CHUNK_LENGTH).floor() as u64;
            if chunk != active_chunk {
                active_chunk = chunk;
                obstacles = endless::obstacles(seed, sim.position.y);
            }
            sim.step_mode(
                Input {
                    heading: endless::reference_heading(seed, &sim),
                    brake: false,
                },
                &obstacles,
                Mode::FreeSki,
            );
            maximum_speed = maximum_speed.max(sim.speed);
            assert_eq!(
                sim.crashes, 0,
                "fast seed {seed} left the route at ({:.2}, {:.2})",
                sim.position.x, sim.position.y
            );
            if sim.distance >= 5_000. {
                break;
            }
        }
        assert!(
            sim.distance >= 5_000.,
            "fast seed {seed} reached only {:.1} m",
            sim.distance
        );
        assert!(sim.fast_mode);
        assert!(
            maximum_speed >= 89.9,
            "fast seed {seed} peaked at only {maximum_speed:.2} m/s"
        );
        assert!(sim.valid_mode(&obstacles, Mode::FreeSki));
    }
}
