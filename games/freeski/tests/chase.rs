use omarchy_freeski::{
    chase::{
        Chase, ChaseEvent, ChasePhase, TickInput, CATCH_RADIUS, MAX_FAILED_RETRIES,
        MAX_SPEED_PURSUER, RECOVERY_SAFE_GAP, SPAWN_RETRY_TICKS, TRIGGER_DISTANCE, WARNING_TICKS,
    },
    endless,
    engine::{Event, Input, Phase, Point, Sim, DT, MAX_SPEED},
    session::Session,
    storage::Save,
    world::{Kind, Obstacle, HALF_WIDTH},
};

fn tick<'a>(
    start: Point,
    end: Point,
    distance: f64,
    speed: f64,
    protected: bool,
    obstacles: &'a [Obstacle],
) -> TickInput<'a> {
    TickInput {
        skier_start: start,
        skier_end: end,
        skier_distance: distance,
        skier_speed: speed,
        protected,
        obstacles,
    }
}

fn active(at: Point, speed: f64, heading: f64) -> Chase {
    Chase {
        phase: ChasePhase::Active,
        position: at,
        speed,
        heading,
        warning_ticks: 0,
        retry_ticks: 0,
        failed_retries: 0,
        detour: None,
        detour_ticks: 0,
    }
}

#[test]
fn threshold_emits_one_warning_then_spawns_after_simulation_ticks() {
    let skier = Point { x: 3., y: 1_000. };
    let below = Chase::default()
        .plan_tick(tick(
            skier,
            skier,
            TRIGGER_DISTANCE - 0.001,
            40.,
            false,
            &[],
        ))
        .finish();
    assert_eq!(below, Chase::default());

    let warning = below.plan_tick(tick(skier, skier, TRIGGER_DISTANCE, 40., false, &[]));
    assert_eq!(warning.events, vec![ChaseEvent::Warning]);
    let mut chase = warning.finish();
    assert_eq!(chase.phase, ChasePhase::Warning);
    assert_eq!(chase.warning_ticks, WARNING_TICKS);

    for _ in 0..WARNING_TICKS {
        let plan = chase.plan_tick(tick(skier, skier, skier.y, 40., false, &[]));
        assert!(plan.events.is_empty());
        chase = plan.finish();
    }
    let spawn = chase.plan_tick(tick(skier, skier, skier.y, 40., false, &[]));
    assert_eq!(spawn.events, vec![ChaseEvent::Spawned]);
    chase = spawn.finish();
    assert_eq!(chase.phase, ChasePhase::Active);
    assert!(chase.position.y < skier.y - CATCH_RADIUS);
    assert!(chase.valid());
}

#[test]
fn blocked_spawn_search_is_bounded_and_retries_without_invalid_state() {
    let skier = Point { x: 0., y: 1_100. };
    let mut blockers = Vec::new();
    for (id, gap) in [48., 54., 60., 42.].into_iter().enumerate() {
        for x_index in -20..=20 {
            blockers.push(Obstacle {
                id: id * 100 + (x_index + 20) as usize,
                at: Point {
                    x: x_index as f64 * 2.,
                    y: skier.y - gap,
                },
                kind: Kind::Tree,
            });
        }
    }
    let mut chase = Chase {
        phase: ChasePhase::Warning,
        ..Chase::default()
    };
    for expected in 1..=MAX_FAILED_RETRIES {
        let plan = chase.plan_tick(tick(skier, skier, skier.y, 0., false, &blockers));
        assert!(plan.events.is_empty());
        chase = plan.finish();
        assert_eq!(chase.failed_retries, expected);
        assert_eq!(chase.retry_ticks, SPAWN_RETRY_TICKS);
        for _ in 0..SPAWN_RETRY_TICKS {
            chase = chase
                .plan_tick(tick(skier, skier, skier.y, 0., false, &blockers))
                .finish();
        }
    }
    // The counter saturates and the actor remains a valid warning, rather than
    // overflowing or spawning inside blocked terrain.
    chase = chase
        .plan_tick(tick(skier, skier, skier.y, 0., false, &blockers))
        .finish();
    assert_eq!(chase.phase, ChasePhase::Warning);
    assert_eq!(chase.failed_retries, MAX_FAILED_RETRIES);
    assert!(chase.valid());
}

#[test]
fn relative_sweep_detects_crossing_between_tick_endpoints() {
    let chase = active(Point { x: 0., y: 100. }, 56., 0.);
    let skier_start = Point { x: -2., y: 100. };
    let skier_end = Point { x: 2., y: 100. };
    let plan = chase.plan_tick(tick(skier_start, skier_end, 1_200., 50., false, &[]));
    let contact = plan.catch_fraction.expect("paths cross within the tick");
    assert!((0. ..=1.).contains(&contact));
    let caught = plan.commit(contact);
    assert!(caught.position.x.abs() <= 0.1);
}

#[test]
fn obstacle_contacts_stop_the_actor_instead_of_phasing() {
    let tree = Obstacle {
        id: 7,
        at: Point { x: 0., y: 102.5 },
        kind: Kind::Tree,
    };
    let chase = active(Point { x: 0., y: 100. }, 56., 0.);
    let skier = Point { x: 0., y: 180. };
    let plan = chase.plan_tick(tick(skier, skier, 1_200., 50., false, &[tree]));
    let end = plan.creature_end.expect("active actor moves");
    assert!(end.y < tree.at.y);
    assert!((end.x - tree.at.x).hypot(end.y - tree.at.y) >= tree.radius() + 0.95 - 1e-5);
    assert!(plan.finish().speed < chase.speed);
}

#[test]
fn protection_holds_a_physical_gap_and_expiry_is_not_an_instant_catch() {
    let skier = Point { x: 0., y: 200. };
    let mut chase = active(
        Point {
            x: 0.,
            y: skier.y - RECOVERY_SAFE_GAP - 0.2,
        },
        56.,
        0.,
    );
    for _ in 0..120 {
        let plan = chase.plan_tick(tick(skier, skier, 1_200., 0., true, &[]));
        assert!(plan.catch_fraction.is_none());
        chase = plan.finish();
        assert!(
            (chase.position.x - skier.x).hypot(chase.position.y - skier.y)
                >= RECOVERY_SAFE_GAP - 1e-4
        );
    }
    let first_unprotected = chase.plan_tick(tick(skier, skier, 1_200., 0., false, &[]));
    assert_ne!(first_unprotected.catch_fraction, Some(0.));
}

#[test]
fn protected_gap_wins_when_terrain_contact_is_later_in_the_tick() {
    let skier = Point { x: 0., y: 114.5 };
    let tree = Obstacle {
        id: 71,
        at: Point { x: 0., y: 103.5 },
        kind: Kind::Tree,
    };
    let chase = active(Point { x: 0., y: 100. }, MAX_SPEED_PURSUER, 0.);
    let plan = chase.plan_tick(tick(skier, skier, 1_200., 0., true, &[tree]));
    assert!(plan.catch_fraction.is_none());
    let end = plan.creature_end.expect("active actor moves");
    assert!(
        (end.x - skier.x).hypot(end.y - skier.y) >= RECOVERY_SAFE_GAP - 1e-4,
        "later terrain contact must not cross the protected gap: {end:?}"
    );
    assert!(end.y < tree.at.y - tree.radius() - 0.95);
    assert_eq!(plan.finish().speed, 0.);
}

#[test]
fn normal_speed_pursuit_catches_straight_and_zigzagging_skiers() {
    fn run(evasive: bool) -> (bool, usize, f64, f64) {
        let mut skier = Point { x: 0., y: 1_100. };
        let mut chase = active(Point { x: 0., y: 1_052. }, 56., 0.);
        let initial = (skier.x - chase.position.x).hypot(skier.y - chase.position.y);
        let mut widest = initial;
        for frame in 0_usize..1_800 {
            let start = skier;
            let heading: f64 = if evasive && (frame / 90) % 2 == 0 {
                0.55
            } else if evasive {
                -0.55
            } else {
                0.
            };
            skier.x = (skier.x + heading.sin() * MAX_SPEED / 60.)
                .clamp(-HALF_WIDTH + 0.65, HALF_WIDTH - 0.65);
            skier.y += heading.cos() * MAX_SPEED / 60.;
            let plan = chase.plan_tick(tick(start, skier, skier.y, MAX_SPEED, false, &[]));
            if let Some(contact) = plan.catch_fraction {
                chase = plan.commit(contact);
                let separation = (skier.x - chase.position.x).hypot(skier.y - chase.position.y);
                return (true, frame, widest, separation);
            }
            chase = plan.finish();
            widest = widest.max((skier.x - chase.position.x).hypot(skier.y - chase.position.y));
        }
        (
            false,
            1_800,
            widest,
            (skier.x - chase.position.x).hypot(skier.y - chase.position.y),
        )
    }

    let straight = run(false);
    let evasive = run(true);
    assert!(
        straight.0,
        "higher straight speed must eventually close the gap"
    );
    assert!(evasive.0, "ordinary zigzagging must not defeat pursuit");
    assert!(
        straight.1 <= 240 && evasive.1 <= 240,
        "normal-speed catches should close a 48 m gap within four seconds: {straight:?} / {evasive:?}"
    );
}

#[test]
fn actor_cannot_claim_infinite_immunity_at_a_slope_edge() {
    let edge = HALF_WIDTH - 0.65;
    let mut skier = Point { x: edge, y: 1_100. };
    let mut chase = active(
        Point {
            x: edge - 8.,
            y: 1_055.,
        },
        30.,
        0.,
    );
    let mut caught = false;
    for _ in 0..1_800 {
        let start = skier;
        skier.y += MAX_SPEED / 60.;
        let plan = chase.plan_tick(tick(start, skier, skier.y, MAX_SPEED, false, &[]));
        if let Some(contact) = plan.catch_fraction {
            chase = plan.commit(contact);
            caught = true;
            break;
        }
        chase = plan.finish();
    }
    assert!(
        caught,
        "edge corridor must not make pursuit impossible: skier={skier:?} chase={chase:?}"
    );
    assert!(chase.position.x.abs() <= HALF_WIDTH - 0.95 + 1e-8);
}

#[test]
fn serialized_pursuit_round_trips_and_invalid_numbers_are_rejected() {
    let chase = active(Point { x: 4., y: 1_234. }, 41., -0.4);
    let bytes = serde_json::to_vec(&chase).unwrap();
    let restored: Chase = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored, chase);
    assert!(restored.valid());

    let mut invalid = restored;
    invalid.position.x = f64::NAN;
    assert!(!invalid.valid());

    let mut invalid_detour = chase;
    invalid_detour.detour = Some(Point {
        x: f64::NAN,
        y: 1_240.,
    });
    invalid_detour.detour_ticks = 10;
    assert!(!invalid_detour.valid());
}

#[test]
fn blocked_route_keeps_one_bounded_detour_across_ticks_and_restore() {
    let tree = Obstacle {
        id: 88,
        at: Point { x: 0., y: 106. },
        kind: Kind::Tree,
    };
    let skier = Point { x: 0., y: 220. };
    let mut chase = active(Point { x: 0., y: 100. }, 60., 0.);

    chase = chase
        .plan_tick(tick(skier, skier, 1_200., 60., false, &[tree]))
        .finish();
    let waypoint = chase.detour.expect("blocked route selects a detour");
    assert!(chase.detour_ticks > 0);
    assert_ne!(waypoint.x, 0.);

    // The first physical contact may replace the anticipatory probe with a
    // tangent that points away from the obstacle. Once selected, that side
    // must remain stable rather than alternating every tick.
    for _ in 0..5 {
        chase = chase
            .plan_tick(tick(skier, skier, 1_200., 60., false, &[tree]))
            .finish();
    }
    let detour_side = chase
        .detour
        .map(|point| (point.x - chase.position.x).signum())
        .expect("contact keeps a physical escape detour");
    for _ in 0..25 {
        chase = chase
            .plan_tick(tick(skier, skier, 1_200., 60., false, &[tree]))
            .finish();
        if let Some(point) = chase.detour {
            assert_eq!(
                (point.x - chase.position.x).signum(),
                detour_side,
                "detour must not oscillate"
            );
        }
    }

    let restored: Chase = serde_json::from_slice(&serde_json::to_vec(&chase).unwrap()).unwrap();
    assert_eq!(restored, chase);
    assert!(restored.valid());
}

#[test]
fn normal_speed_is_overhauled_while_fast_mode_can_open_distance() {
    fn separation_after(skier_speed: f64, ticks: usize) -> (f64, bool) {
        let mut skier = Point { x: 0., y: 1_100. };
        let mut chase = active(Point { x: 0., y: 1_052. }, MAX_SPEED_PURSUER, 0.);
        for _ in 0..ticks {
            let start = skier;
            skier.y += skier_speed * DT;
            let plan = chase.plan_tick(tick(start, skier, skier.y, skier_speed, false, &[]));
            if let Some(contact) = plan.catch_fraction {
                chase = plan.commit(contact);
                return (
                    (skier.x - chase.position.x).hypot(skier.y - chase.position.y),
                    true,
                );
            }
            chase = plan.finish();
        }
        (
            (skier.x - chase.position.x).hypot(skier.y - chase.position.y),
            false,
        )
    }

    let normal = separation_after(60., 300);
    let fast = separation_after(90., 300);
    assert!(
        normal.1,
        "ordinary straight skiing must be caught: {normal:?}"
    );
    assert!(
        !fast.1,
        "clean fast skiing should earn an escape window: {fast:?}"
    );
    assert!(
        fast.0 > 80.,
        "fast mode should open meaningful space: {fast:?}"
    );
}

#[test]
fn creature_routes_around_the_obstacle_that_stalled_a_live_chase() {
    let skier = Point {
        x: 9.907_216_461_272_625,
        y: 4_127.612_807_737_195,
    };
    let mut chase = active(
        Point {
            x: -0.811_865_154_729_933_3,
            y: 1_477.808_252_123_179,
        },
        0.041_664_817_448_891_55,
        0.004_073_432_536_034_538,
    );
    let start = chase.position;
    // Recreate the exact failure geometry independently of generator revisions:
    // the actor starts on the combined collision boundary of a rock directly
    // between it and the skier.
    let obstacles = vec![Obstacle {
        id: 705,
        at: Point {
            x: start.x,
            y: start.y + 2.15,
        },
        kind: Kind::Rock,
    }];
    let mut stationary = 0;
    let mut longest_stationary = 0;

    for _ in 0..600 {
        let previous = chase.position;
        chase = chase
            .plan_tick(tick(skier, skier, skier.y, 50., false, &obstacles))
            .finish();
        if (chase.position.x - previous.x).hypot(chase.position.y - previous.y) <= 1e-6 {
            stationary += 1;
            longest_stationary = longest_stationary.max(stationary);
        } else {
            stationary = 0;
        }
        assert!(chase.valid(), "escape produced invalid pursuit state");
        assert!(
            (chase.position.x - previous.x).hypot(chase.position.y - previous.y)
                <= MAX_SPEED_PURSUER * DT + 1e-9,
            "escape must use ordinary bounded actor motion"
        );
        for obstacle in obstacles
            .iter()
            .filter(|obstacle| obstacle.kind != Kind::Ramp)
        {
            assert!(
                (chase.position.x - obstacle.at.x).hypot(chase.position.y - obstacle.at.y)
                    >= obstacle.radius() + 0.95 - 1e-5,
                "creature crossed obstacle {}",
                obstacle.id
            );
        }
    }

    assert!(
        chase.position.y > start.y + 20.,
        "creature remained stalled at {:?} with speed {}",
        chase.position,
        chase.speed
    );
    assert!(
        longest_stationary <= 90,
        "physical detour took too long to clear contact: {longest_stationary} ticks"
    );
}

#[test]
fn creature_escapes_the_overlapping_tree_cusp_from_the_seed_corpus() {
    let obstacles = vec![
        Obstacle {
            id: 1_226,
            at: Point {
                x: 19.502_379_864_449_09,
                y: 2_447.804_688_127_233_5,
            },
            kind: Kind::Tree,
        },
        Obstacle {
            id: 1_232,
            at: Point {
                x: 22.566_163_282_746_23,
                y: 2_450.975_535_189_299,
            },
            kind: Kind::Tree,
        },
    ];
    let start = Point {
        x: 21.619_497_248_068_452,
        y: 2_448.824_646_001_051,
    };
    let skier = Point { x: 0., y: 11_714. };
    let mut chase = active(start, 18., -0.487_997_409_082_274_74);
    let mut stationary = 0;
    let mut longest_stationary = 0;

    for _ in 0..300 {
        let previous = chase.position;
        chase = chase
            .plan_tick(tick(skier, skier, skier.y, MAX_SPEED, false, &obstacles))
            .finish();
        if distance_for_test(chase.position, previous) <= 1e-6 {
            stationary += 1;
            longest_stationary = longest_stationary.max(stationary);
        } else {
            stationary = 0;
        }
        for obstacle in &obstacles {
            assert!(
                distance_for_test(chase.position, obstacle.at) >= obstacle.radius() + 0.95 - 1e-5,
                "creature crossed obstacle {}",
                obstacle.id
            );
        }
    }

    assert!(
        distance_for_test(chase.position, start) > 20.,
        "creature remained trapped in overlapping trees: {chase:?}"
    );
    assert!(
        longest_stationary <= 100,
        "overlapping-tree escape stalled for {longest_stationary} ticks"
    );
}

fn distance_for_test(a: Point, b: Point) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

fn chase_session() -> Session {
    let mut state = Save::default();
    state.select_chase(true, 17);
    state.run = Sim {
        phase: Phase::Running,
        position: Point { x: 0., y: 1_100. },
        speed: MAX_SPEED,
        distance: 1_100.,
        ..Sim::default()
    };
    state.chase = active(Point { x: 0., y: 1_052. }, 56., 0.);
    let mut session = Session::new(state);
    // A clear derived window isolates pursuit policy while retaining the exact
    // production Session and skier physics path.
    session.obstacles.clear();
    session
}

#[test]
fn production_session_freezes_warning_and_pursuit_while_paused() {
    let mut state = Save::default();
    state.select_chase(true, 11);
    state.run.phase = Phase::Running;
    state.run.position.y = TRIGGER_DISTANCE;
    state.run.distance = TRIGGER_DISTANCE;
    let mut session = Session::new(state);
    assert_eq!(session.step(Input::default()), vec![Event::Warning]);
    let warning = session.state.chase.clone();
    session.state.run.pause();
    for _ in 0..600 {
        assert!(session.step(Input::default()).is_empty());
    }
    assert_eq!(session.state.chase, warning);

    session.state.run.start();
    for _ in 0..WARNING_TICKS {
        session.step(Input::default());
    }
    assert_eq!(session.state.chase.phase, ChasePhase::Warning);
    assert_eq!(session.step(Input::default()), vec![Event::Spawn]);
    let pursuit = session.state.chase.clone();
    session.state.run.pause();
    session.step(Input::default());
    assert_eq!(session.state.chase, pursuit);
}

#[test]
fn production_session_normal_speed_pursuit_is_formidable() {
    fn run(evasive: bool) -> (bool, usize, f64, usize) {
        let mut session = chase_session();
        let mut widest: f64 = 0.;
        let mut prior = 48.;
        let mut gain_streak = 0;
        let mut longest_gain = 0;
        for tick in 0_usize..2_400 {
            session.obstacles.clear();
            let heading = if !evasive {
                0.
            } else if (tick / 90) % 2 == 0 {
                0.55
            } else {
                -0.55
            };
            let events = session.step(Input {
                heading,
                brake: false,
            });
            let separation = (session.state.run.position.x - session.state.chase.position.x)
                .hypot(session.state.run.position.y - session.state.chase.position.y);
            widest = widest.max(separation);
            if separation > prior + 1e-6 {
                gain_streak += 1;
                longest_gain = longest_gain.max(gain_streak);
            } else {
                gain_streak = 0;
            }
            prior = separation;
            if events.contains(&Event::Caught) {
                return (true, tick, widest, longest_gain);
            }
        }
        (false, 2_400, widest, longest_gain)
    }

    let straight = run(false);
    let evasive = run(true);
    assert!(straight.0, "straight capped-speed skiing should be caught");
    assert!(evasive.0, "ordinary zigzagging must not defeat pursuit");
    assert!(
        straight.1 <= 240 && evasive.1 <= 240,
        "normal-speed catches should close a 48 m gap within four seconds: {straight:?} / {evasive:?}"
    );
}

#[test]
fn serialized_warning_and_pursuit_continue_tick_for_tick() {
    fn round_trip(state: &Save) -> Save {
        serde_json::from_slice(&serde_json::to_vec(state).unwrap()).unwrap()
    }

    let mut warning_state = Save::default();
    warning_state.select_chase(true, 29);
    warning_state.run.phase = Phase::Running;
    warning_state.run.position.y = TRIGGER_DISTANCE;
    warning_state.run.distance = TRIGGER_DISTANCE;
    let mut warning_a = Session::new(warning_state);
    warning_a.step(Input::default());
    for _ in 0..47 {
        warning_a.step(Input::default());
    }
    let mut warning_b = Session::new(round_trip(&warning_a.state));
    for _ in 0..WARNING_TICKS + 30 {
        let a = warning_a.step(Input::default());
        let b = warning_b.step(Input::default());
        assert_eq!(a, b);
        assert_eq!(warning_a.state, warning_b.state);
    }

    let mut pursuit_a = chase_session();
    let mut pursuit_b = Session::new(round_trip(&pursuit_a.state));
    for tick in 0..240 {
        pursuit_a.obstacles.clear();
        pursuit_b.obstacles.clear();
        let input = Input {
            heading: if (tick / 60) % 2 == 0 { 0.45 } else { -0.45 },
            brake: false,
        };
        assert_eq!(pursuit_a.step(input), pursuit_b.step(input));
        assert_eq!(pursuit_a.state, pursuit_b.state);
        if pursuit_a.state.run.ended() {
            break;
        }
    }
}

#[test]
fn production_session_preserves_recovery_gap_until_protection_expires() {
    let mut session = chase_session();
    session.state.run.speed = 0.;
    session.state.run.tumble = 42;
    session.state.run.protection = 90;
    session.state.chase = active(
        Point {
            x: 0.,
            y: session.state.run.position.y - RECOVERY_SAFE_GAP - 0.2,
        },
        56.,
        0.,
    );
    for _ in 0..132 {
        session.obstacles.clear();
        let events = session.step(Input::default());
        assert!(!events.contains(&Event::Caught));
        let gap = (session.state.run.position.x - session.state.chase.position.x)
            .hypot(session.state.run.position.y - session.state.chase.position.y);
        assert!(gap >= RECOVERY_SAFE_GAP - 1e-4);
    }
    session.obstacles.clear();
    assert!(!session.step(Input::default()).contains(&Event::Caught));
    // Protection explains the temporary wait, but cannot leave the creature
    // parked beside an unprotected skier indefinitely after the timer expires.
    for _ in 0..150 {
        session.obstacles.clear();
        if session
            .step(Input {
                heading: 0.,
                brake: true,
            })
            .contains(&Event::Caught)
        {
            return;
        }
    }
    panic!("pursuit did not resume after crash protection expired");
}

#[test]
fn production_run_from_rest_quantifies_straight_and_evasive_pursuit() {
    let mut state = Save::default();
    state.select_chase(true, 17);
    state.run.start();
    let mut baseline = Session::new(state);
    let mut warnings = 0;
    let mut spawn_tick = None;
    for _ in 0..8_000 {
        let heading = endless::reference_heading(17, &baseline.state.run);
        let events = baseline.step(Input {
            heading,
            brake: false,
        });
        warnings += events
            .iter()
            .filter(|event| **event == Event::Warning)
            .count();
        if events.contains(&Event::Spawn) {
            spawn_tick = Some(baseline.state.run.ticks);
            break;
        }
        assert!(!baseline.state.run.ended());
    }
    assert_eq!(warnings, 1);
    assert!(
        spawn_tick.is_some(),
        "chase did not spawn from a normal run"
    );
    assert_eq!(baseline.state.run.crashes, 0);

    fn branch(mut session: Session, evasive: bool) -> (u64, f64, Phase, usize) {
        let start_tick = session.state.run.ticks;
        let mut closest = f64::INFINITY;
        let mut prior = (session.state.run.position.x - session.state.chase.position.x)
            .hypot(session.state.run.position.y - session.state.chase.position.y);
        let mut gain_streak = 0;
        let mut longest_gain = 0;
        for _ in 0_u64..2_400 {
            let base = endless::reference_heading(session.state.seed, &session.state.run);
            let heading = if evasive { base } else { 0. };
            session.step(Input {
                heading,
                brake: false,
            });
            let separation = (session.state.run.position.x - session.state.chase.position.x)
                .hypot(session.state.run.position.y - session.state.chase.position.y);
            closest = closest.min(separation);
            if separation > prior + 1e-6 {
                gain_streak += 1;
                longest_gain = longest_gain.max(gain_streak);
            } else {
                gain_streak = 0;
            }
            prior = separation;
            if session.state.run.ended() {
                break;
            }
        }
        (
            session.state.run.ticks - start_tick,
            closest,
            session.state.run.phase,
            longest_gain,
        )
    }

    let straight = branch(Session::new(round_trip_save(&baseline.state)), false);
    let evasive = branch(Session::new(round_trip_save(&baseline.state)), true);
    assert_eq!(straight.2, Phase::Caught, "straight: {straight:?}");
    assert!(
        evasive.0 > straight.0,
        "deliberate turns should still buy time against the tighter interception: {straight:?} / {evasive:?}"
    );
    assert!(
        evasive.3 >= 30,
        "the evasive route should grow separation for at least half a second: {evasive:?}"
    );
}

fn round_trip_save(state: &Save) -> Save {
    serde_json::from_slice(&serde_json::to_vec(state).unwrap()).unwrap()
}

#[test]
fn close_pursuer_catches_a_braked_skier_instead_of_orbiting() {
    let mut session = chase_session();
    session.state.run.speed = 0.;
    session.state.chase = active(Point { x: 8., y: 1_092. }, 60., 0.);
    for _ in 0..150 {
        session.obstacles.clear();
        if session
            .step(Input {
                heading: 0.,
                brake: true,
            })
            .contains(&Event::Caught)
        {
            return;
        }
    }
    panic!(
        "close pursuer circled a stationary unprotected skier for 2.5 seconds: {:?}",
        session.state.chase
    );
}

#[test]
fn close_pursuer_does_not_detour_around_terrain_beyond_the_skier() {
    let skier = Point { x: 0., y: 1_100. };
    let tree = Obstacle {
        id: 99,
        at: Point { x: 0., y: 1_108. },
        kind: Kind::Tree,
    };
    let mut chase = active(Point { x: 0., y: 1_094. }, 48., 0.);
    for _ in 0..30 {
        let plan = chase.plan_tick(tick(skier, skier, skier.y, 0., false, &[tree]));
        if plan.catch_fraction.is_some() {
            return;
        }
        chase = plan.finish();
        assert!(
            chase.detour.is_none(),
            "terrain beyond the skier must not divert a clear approach"
        );
    }
    panic!("clear approach abandoned because of a tree beyond the target: {chase:?}");
}

#[test]
fn close_pursuit_approach_matrix() {
    let mut worst = (0, String::new());
    let mut failures = Vec::new();
    for skier_speed in [0., 20., 60.] {
        for (x, y) in [(8., -8.), (-8., -8.), (8., 0.), (0., 8.), (14., -14.)] {
            for heading in [-1.5, 0., 1.5, 3.] {
                let mut skier = Point { x: 0., y: 1100. };
                let mut chase = active(Point { x, y: skier.y + y }, 60., heading);
                let mut caught = false;
                for t in 1..=600 {
                    let start = skier;
                    skier.y += skier_speed * DT;
                    let plan =
                        chase.plan_tick(tick(start, skier, skier.y, skier_speed, false, &[]));
                    if plan.catch_fraction.is_some() {
                        if t > worst.0 {
                            worst = (
                                t,
                                format!("speed {skier_speed} offset {x},{y} heading {heading}"),
                            );
                        }
                        caught = true;
                        break;
                    }
                    chase = plan.finish();
                }
                if !caught {
                    failures.push(format!("speed {skier_speed} offset {x},{y} heading {heading} final skier {skier:?} chase {chase:?}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "close approaches failed: {failures:?}");
    assert!(
        worst.0 <= 540,
        "close approaches must catch within nine seconds: {worst:?}"
    );
}
