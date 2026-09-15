use omarchy_freeski::engine::{
    Event, Input, Phase, Point, Sim, ACCELERATION, DT, FAST_MAX_SPEED, MAX_SPEED,
    OVERSPEED_DECELERATION,
};
use omarchy_freeski::world::{Kind, Obstacle};

fn running() -> Sim {
    let mut sim = Sim::default();
    sim.start();
    sim
}

fn ski_for(sim: &mut Sim, ticks: usize) {
    for _ in 0..ticks {
        sim.step(Input::default(), &[]);
    }
}

#[test]
fn normal_and_fast_caps_build_gradually_at_fixed_tick_rate() {
    let mut normal = running();
    ski_for(&mut normal, 60);
    assert!((8.7..9.0).contains(&normal.speed));
    let mut normal_ticks = 60;
    while normal.speed < MAX_SPEED {
        ski_for(&mut normal, 1);
        normal_ticks += 1;
    }
    assert_eq!(normal.speed, MAX_SPEED);
    assert_eq!(normal_ticks, 487);

    let mut fast = running();
    assert!(fast.toggle_fast_mode());
    ski_for(&mut fast, 60);
    assert!((8.7..9.0).contains(&fast.speed));
    let mut fast_ticks = 60;
    while fast.speed < FAST_MAX_SPEED {
        ski_for(&mut fast, 1);
        fast_ticks += 1;
    }
    assert_eq!(fast.speed, FAST_MAX_SPEED);
    assert_eq!(fast_ticks, 832);
    assert!(fast.valid(&[]));

    assert_eq!(ACCELERATION, 9.);
    assert_eq!(DT, 1. / 60.);
}

#[test]
fn leaving_fast_mode_bleeds_excess_speed_without_a_snap() {
    let mut sim = running();
    sim.fast_mode = true;
    sim.speed = FAST_MAX_SPEED;
    assert!(sim.toggle_fast_mode());
    sim.step(Input::default(), &[]);
    assert_eq!(sim.speed, FAST_MAX_SPEED - OVERSPEED_DECELERATION * DT);
    assert!(sim.speed > MAX_SPEED);

    ski_for(&mut sim, 200);
    assert_eq!(sim.speed, MAX_SPEED);
    assert!(sim.valid(&[]));
}

#[test]
fn fast_mode_is_causal_state_and_only_toggles_during_a_run() {
    let mut sim = Sim::default();
    assert!(!sim.toggle_fast_mode());
    assert!(!sim.fast_mode);
    sim.start();
    assert!(sim.toggle_fast_mode());
    assert!(sim.fast_mode);
    sim.pause();
    assert!(!sim.toggle_fast_mode());
    assert!(sim.fast_mode);
    sim.start();
    ski_for(&mut sim, 30);

    let json = serde_json::to_vec(&sim).unwrap();
    let restored: Sim = serde_json::from_slice(&json).unwrap();
    assert_eq!(restored, sim);
    assert_eq!(restored.phase, Phase::Running);

    let legacy = serde_json::to_string(&sim)
        .unwrap()
        .replace(&format!("\"fast_mode\":{},", sim.fast_mode), "");
    assert!(!serde_json::from_str::<Sim>(&legacy).unwrap().fast_mode);
}

#[test]
fn fast_cap_remains_finite_and_sweeps_through_hazards() {
    let mut sim = running();
    sim.fast_mode = true;
    sim.speed = FAST_MAX_SPEED;
    let tree = Obstacle {
        id: 1,
        at: Point { x: 0., y: 3. },
        kind: Kind::Tree,
    };
    let outcome = sim.step_outcome_mode(Input::default(), &[tree], Default::default(), None);
    assert_eq!(outcome.events, vec![Event::Crash]);
    assert!(outcome
        .segment
        .crash_fraction
        .is_some_and(|fraction| (0. ..1.).contains(&fraction)));
    assert!(outcome.segment.proposed_end.y < tree.at.y);
    assert!(sim.valid(&[tree]));
}
