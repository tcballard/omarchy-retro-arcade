//! Produce complete-game native fixtures through ordinary production inputs.
use omarchy_freeski::{
    course, endless,
    engine::{Event, Input, Mode, Phase},
    session::Session,
    storage::{self, Save},
    world::Kind,
};
use std::path::Path;

const SEED: u64 = 0x5945_5449_4348_4153;

fn write(output: &Path, name: &str, session: &Session) {
    storage::write(&output.join(name), &session.state, &session.obstacles).unwrap();
}

fn chase_session() -> Session {
    let mut save = Save::default();
    save.select_chase(true, SEED);
    save.run.start();
    Session::new(save)
}

fn reference_tick(session: &mut Session) -> Vec<Event> {
    let heading = endless::reference_heading(session.state.seed, &session.state.run);
    session.step(Input {
        heading,
        brake: false,
    })
}

fn main() {
    let output = std::path::PathBuf::from(
        std::env::args_os()
            .nth(1)
            .expect("output directory argument"),
    );
    std::fs::create_dir_all(&output).unwrap();

    let mut chase = chase_session();
    let mut wrote_warning = false;
    let mut wrote_active = false;
    for _ in 0..30_000 {
        let events = reference_tick(&mut chase);
        if events.contains(&Event::Warning) {
            write(&output, "chase-warning.json", &chase);
            wrote_warning = true;
        }
        if events.contains(&Event::Spawn) {
            write(&output, "chase-active.json", &chase);
            wrote_active = true;
            break;
        }
        assert!(!chase.state.run.ended());
    }
    assert!(wrote_warning && wrote_active);

    // From the active fixture, use normal steering to hit the nearest downhill
    // hazard and preserve the resulting protected recovery state.
    let mut recovery = chase;
    for _ in 0..12_000 {
        let target = recovery
            .obstacles
            .iter()
            .filter(|obstacle| {
                obstacle.kind != Kind::Ramp && obstacle.at.y > recovery.state.run.position.y + 2.
            })
            .min_by(|a, b| a.at.y.total_cmp(&b.at.y))
            .copied();
        let heading = target.map_or(0., |target| {
            (target.at.x - recovery.state.run.position.x)
                .atan2((target.at.y - recovery.state.run.position.y).max(2.))
        });
        recovery.step(Input {
            heading,
            brake: false,
        });
        if recovery.state.run.tumble > 0 {
            write(&output, "chase-recovery.json", &recovery);
            break;
        }
        assert!(
            !recovery.state.run.ended(),
            "caught before recovery fixture"
        );
    }
    assert!(recovery.state.run.tumble > 0);

    let mut caught = chase_session();
    for _ in 0..60_000 {
        caught.step(Input::default());
        if caught.state.run.phase == Phase::Caught {
            write(&output, "chase-caught.json", &caught);
            break;
        }
        assert_ne!(caught.state.run.phase, Phase::Crashed);
    }
    assert_eq!(caught.state.run.phase, Phase::Caught);

    let mut progress_save = Save::default();
    assert!(progress_save.select_course(0));
    progress_save.run.start();
    assert!(progress_save.run.toggle_fast_mode());
    let mut slalom = Session::new(progress_save);
    while slalom.state.run.distance < 360. {
        let input = course::reference_input(slalom.state.course_index, &slalom.state.run);
        slalom.step(input);
        assert!(!slalom.state.run.ended());
    }
    write(&output, "slalom-progress.json", &slalom);

    // Finish every course in sequence so unlocks and records use the same
    // production session transitions as the native game.
    for index in 0..course::COURSE_COUNT {
        if index != 0 {
            assert!(slalom.state.select_course(index));
            slalom.refresh();
            slalom.state.run.start();
            assert!(slalom.state.run.toggle_fast_mode());
        }
        for _ in 0..20_000 {
            let input = course::reference_input(index, &slalom.state.run);
            slalom.step(input);
            if slalom.state.run.ended() {
                break;
            }
        }
        assert_eq!(slalom.state.mode, Mode::Slalom);
        assert_eq!(slalom.state.run.phase, Phase::Finished, "course {index}");
        if index == 0 {
            write(&output, "slalom-finished.json", &slalom);
        }
    }
    write(&output, "slalom-cup-finished.json", &slalom);
    println!(
        "Generated chase and Slalom completion fixtures in {}",
        output.display()
    );
}
