use omarchy_freeski::{
    course::{
        course, crosses_finish, medal, reference_input, CourseProgress, GateOutcome, Medal,
        COURSE_COUNT, MISSED_GATE_PENALTY_TICKS,
    },
    engine::{Input, Mode, Phase, Point, Sim},
    session::Session,
    storage::Save,
    world::Kind,
};

#[test]
fn five_courses_are_distinct_ordered_and_physically_authored() {
    let mut signatures = Vec::new();
    for index in 0..COURSE_COUNT {
        let course = course(index).unwrap();
        assert_eq!(course.index, index);
        assert!(!course.name.is_empty());
        assert!(course.gold_ticks < course.silver_ticks);
        assert!(course.length > course.gates.last().unwrap().y + 40.);
        assert!(course.gates.windows(2).all(|pair| pair[0].y < pair[1].y));
        assert!(course.gates.iter().all(|gate| {
            gate.half_width >= 6.
                && gate.x.abs() + gate.half_width < 39.
                && gate.y > 0.
                && gate.y < course.length
        }));
        assert_eq!(
            course
                .obstacles
                .iter()
                .filter(|obstacle| obstacle.kind == Kind::Pole)
                .count(),
            course.gates.len() * 2
        );
        assert_eq!(
            course
                .obstacles
                .iter()
                .map(|obstacle| obstacle.id)
                .collect::<Vec<_>>(),
            (0..course.obstacles.len()).collect::<Vec<_>>()
        );
        signatures.push(
            course
                .gates
                .iter()
                .map(|gate| (gate.x as i32, gate.y as i32, gate.half_width as i32))
                .collect::<Vec<_>>(),
        );
    }
    for left in 0..signatures.len() {
        for right in left + 1..signatures.len() {
            assert_ne!(signatures[left], signatures[right]);
        }
    }
    assert!(course(COURSE_COUNT).is_none());
}

#[test]
fn gates_resolve_downhill_in_order_and_a_miss_is_charged_once() {
    let course = course(0).unwrap();
    let first = course.gates[0];
    let second = course.gates[1];
    let mut progress = CourseProgress::default();

    assert!(progress
        .cross_segment(
            &course,
            Point {
                x: first.x,
                y: first.y + 1.,
            },
            Point {
                x: first.x,
                y: first.y - 1.,
            },
        )
        .is_empty());
    assert_eq!(progress.next_gate, 0);

    let passed = progress.cross_segment(
        &course,
        Point {
            x: first.x - 1.,
            y: first.y - 1.,
        },
        Point {
            x: first.x + 1.,
            y: first.y + 1.,
        },
    );
    assert_eq!(passed.len(), 1);
    assert_eq!(passed[0].gate_index, 0);
    assert_eq!(passed[0].outcome, GateOutcome::Passed);

    let missed = progress.cross_segment(
        &course,
        Point {
            x: second.x + second.half_width + 2.,
            y: second.y - 1.,
        },
        Point {
            x: second.x + second.half_width + 2.,
            y: second.y + 1.,
        },
    );
    assert_eq!(missed.len(), 1);
    assert_eq!(missed[0].outcome, GateOutcome::Missed);
    assert_eq!(progress.missed, 1);
    assert_eq!(progress.penalty_ticks, MISSED_GATE_PENALTY_TICKS);

    assert!(progress
        .cross_segment(
            &course,
            Point {
                x: second.x,
                y: second.y - 1.,
            },
            Point {
                x: second.x,
                y: second.y + 1.,
            },
        )
        .is_empty());
    assert_eq!(progress.missed, 1);
    assert_eq!(progress.penalty_ticks, MISSED_GATE_PENALTY_TICKS);
}

#[test]
fn one_long_physical_segment_resolves_each_crossed_line_once() {
    let course = course(0).unwrap();
    let mut progress = CourseProgress::default();
    let resolutions = progress.cross_segment(
        &course,
        Point { x: 0., y: 0. },
        Point {
            x: 0.,
            y: course.gates[2].y + 1.,
        },
    );
    assert_eq!(resolutions.len(), 3);
    assert_eq!(progress.next_gate, 3);
    assert_eq!(
        progress.penalty_ticks,
        u64::from(progress.missed) * MISSED_GATE_PENALTY_TICKS
    );
    assert!(progress.valid(&course));
}

#[test]
fn progress_validation_and_medals_include_penalties() {
    let course = course(0).unwrap();
    let complete = CourseProgress {
        next_gate: course.gates.len(),
        missed: 0,
        penalty_ticks: 0,
    };
    assert_eq!(
        medal(&course, course.gold_ticks, &complete),
        Some(Medal::Gold)
    );
    assert_eq!(
        medal(&course, course.gold_ticks + 1, &complete),
        Some(Medal::Silver)
    );
    assert_eq!(
        medal(&course, course.silver_ticks + 1, &complete),
        Some(Medal::Bronze)
    );
    let penalized = CourseProgress {
        next_gate: course.gates.len(),
        missed: 1,
        penalty_ticks: MISSED_GATE_PENALTY_TICKS,
    };
    assert_eq!(
        medal(&course, course.gold_ticks, &penalized),
        Some(Medal::Bronze)
    );
    assert!(!CourseProgress {
        next_gate: 1,
        missed: 1,
        penalty_ticks: 0,
    }
    .valid(&course));
    assert_eq!(medal(&course, 0, &CourseProgress::default()), None);
}

#[test]
fn finish_requires_all_gates_and_a_downhill_crossing() {
    let course = course(0).unwrap();
    let incomplete = CourseProgress::default();
    let complete = CourseProgress {
        next_gate: course.gates.len(),
        missed: 0,
        penalty_ticks: 0,
    };
    let above = Point {
        x: 0.,
        y: course.length - 1.,
    };
    let below = Point {
        x: 0.,
        y: course.length + 1.,
    };
    assert!(!crosses_finish(&course, &incomplete, above, below));
    assert!(crosses_finish(&course, &complete, above, below));
    assert!(!crosses_finish(&course, &complete, below, above));
}

#[test]
fn gate_poles_use_ordinary_swept_crash_physics() {
    let course = course(0).unwrap();
    let gate = course.gates[0];
    let mut sim = Sim {
        phase: Phase::Running,
        position: Point {
            x: gate.x - gate.half_width,
            y: gate.y - 3.,
        },
        speed: 30.,
        ..Sim::default()
    };
    let mut crashed = false;
    for _ in 0..20 {
        let outcome =
            sim.step_outcome_mode(Input::default(), &course.obstacles, Mode::Slalom, None);
        if outcome
            .events
            .contains(&omarchy_freeski::engine::Event::Crash)
        {
            assert!(outcome.segment.end.y < gate.y + 0.01);
            crashed = true;
            break;
        }
    }
    assert!(crashed);
    assert_eq!(sim.crashes, 1);
}

#[test]
fn session_cannot_finish_before_gate_objective_is_resolved() {
    let course = course(0).unwrap();
    let mut state = Save::default();
    assert!(state.select_course(0));
    state.run = Sim {
        phase: Phase::Running,
        position: Point {
            x: 0.,
            y: course.length - 1.,
        },
        speed: 50.,
        distance: course.length - 1.,
        ..Sim::default()
    };
    let mut session = Session::new(state);
    let mut events = Vec::new();
    for _ in 0..4 {
        events.extend(session.step(Input::default()));
    }
    assert!(!events.contains(&omarchy_freeski::engine::Event::Finish));
    assert_eq!(session.state.run.phase, Phase::Running);
    assert!(session.state.run.position.y > course.length);
}

#[test]
fn session_awards_a_slalom_result_and_unlock_exactly_once() {
    let course = course(0).unwrap();
    let mut state = Save::default();
    assert!(state.select_course(0));
    state.slalom = CourseProgress {
        next_gate: course.gates.len(),
        missed: 1,
        penalty_ticks: MISSED_GATE_PENALTY_TICKS,
    };
    state.run = Sim {
        phase: Phase::Running,
        position: Point {
            x: 0.,
            y: course.length - 0.2,
        },
        speed: 50.,
        ticks: 1_000,
        distance: course.length - 0.2,
        ..Sim::default()
    };
    let mut session = Session::new(state);
    let events = session.step(Input::default());
    assert!(events.contains(&omarchy_freeski::engine::Event::Finish));
    assert_eq!(session.state.run.phase, Phase::Finished);
    assert_eq!(session.state.unlocked_courses, 2);
    let recorded = session.state.slalom_best[0].unwrap();
    assert_eq!(
        recorded,
        session
            .state
            .run
            .ticks
            .saturating_add(MISSED_GATE_PENALTY_TICKS)
    );
    assert!(session.state.result_recorded);
    assert!(session.step(Input::default()).is_empty());
    assert_eq!(session.state.slalom_best[0], Some(recorded));
    assert_eq!(session.state.unlocked_courses, 2);
}

#[test]
fn every_course_has_clean_normal_and_fast_production_physics_reference_runs() {
    fn run(index: u8, fast: bool) -> (Sim, CourseProgress) {
        let course = course(index).unwrap();
        let mut sim = Sim::default();
        let mut progress = CourseProgress::default();
        sim.start();
        if fast {
            assert!(sim.toggle_fast_mode());
        }
        while !sim.ended() && sim.ticks < 12_000 {
            let finish = progress.all_resolved(&course).then_some(course.length);
            let outcome = sim.step_outcome_mode(
                reference_input(index, &sim),
                &course.obstacles,
                Mode::Slalom,
                finish,
            );
            if outcome
                .events
                .contains(&omarchy_freeski::engine::Event::Crash)
            {
                eprintln!(
                    "course {index} crash {} at {:?}, next gate {}",
                    sim.crashes, outcome.segment.end, progress.next_gate
                );
            }
            progress.cross_segment(&course, outcome.segment.start, outcome.segment.end);
        }
        (sim, progress)
    }

    let mut normal = Vec::new();
    let mut fast = Vec::new();
    for index in 0..COURSE_COUNT {
        let course = course(index).unwrap();
        let (sim, progress) = run(index, false);
        assert_eq!(sim.phase, Phase::Finished, "normal course {index}");
        assert_eq!(sim.crashes, 0, "normal course {index}");
        assert_eq!(progress.missed, 0, "normal course {index}");
        assert!(progress.all_resolved(&course));
        assert!(sim.ticks <= course.silver_ticks, "normal course {index}");
        normal.push(sim.ticks);

        let (sim, progress) = run(index, true);
        assert_eq!(sim.phase, Phase::Finished, "fast course {index}");
        assert_eq!(sim.crashes, 0, "fast course {index}");
        assert_eq!(progress.missed, 0, "fast course {index}");
        assert!(progress.all_resolved(&course));
        assert!(sim.ticks <= course.gold_ticks, "fast course {index}");
        fast.push(sim.ticks);
    }
    eprintln!("Slalom normal reference ticks: {normal:?}");
    eprintln!("Slalom fast reference ticks: {fast:?}");
    assert_eq!(normal, [1_088, 1_303, 1_374, 1_516, 1_668]);
    assert_eq!(fast, [1_042, 1_243, 1_293, 1_418, 1_551]);
}
