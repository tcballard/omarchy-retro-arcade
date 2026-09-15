//! Generate inspectable endless-mode saves through ordinary production inputs.
use omarchy_freeski::{
    endless,
    engine::{Input, Mode, Phase},
    session::Session,
    storage::{self, Save},
    world::Kind,
};

const EVIDENCE_SEED: u64 = 0x4652_4545_534b_4903;

fn fresh_run() -> Session {
    let mut save = Save::default();
    save.select_mode(Mode::FreeSki, EVIDENCE_SEED);
    save.run.start();
    Session::new(save)
}

fn write(output: &std::path::Path, name: &str, session: &Session) {
    storage::write(&output.join(name), &session.state, &session.obstacles).unwrap();
}

fn main() {
    let output = std::path::PathBuf::from(
        std::env::args_os()
            .nth(1)
            .expect("output directory argument"),
    );
    std::fs::create_dir_all(&output).unwrap();

    let mut session = fresh_run();
    let mut captured_jump = false;
    for _ in 0..20_000 {
        let heading = endless::reference_heading(EVIDENCE_SEED, &session.state.run);
        session.step(Input {
            heading,
            brake: false,
        });
        if !captured_jump && session.state.run.jump.is_some_and(|elapsed| elapsed > 0.3) {
            write(&output, "free-mid-jump.json", &session);
            captured_jump = true;
        }
        if session.state.run.distance > 1_600. {
            write(&output, "free-long-run.json", &session);
            break;
        }
        assert!(!session.state.run.ended());
    }
    assert!(captured_jump);
    assert!(session.state.run.distance > 1_600.);
    assert_eq!(session.state.run.phase, Phase::Running);

    // Deliberately steer from the ordinary start into the first non-ramp
    // obstacle, then capture the engine-selected clear recovery position.
    session = fresh_run();
    let target = endless::obstacles(EVIDENCE_SEED, 0.)
        .into_iter()
        .filter(|obstacle| obstacle.kind != Kind::Ramp)
        .min_by(|a, b| a.at.y.total_cmp(&b.at.y))
        .expect("opening chunk has a deliberate collision target");
    for _ in 0..8_000 {
        let heading = (target.at.x - session.state.run.position.x)
            .atan2((target.at.y - session.state.run.position.y).max(2.));
        session.step(Input {
            heading,
            brake: false,
        });
        if session.state.run.tumble > 0 {
            write(&output, "free-recovery.json", &session);
            break;
        }
        assert!(!session.state.run.ended());
    }
    assert!(session.state.run.tumble > 0);
    assert_eq!(session.state.run.crashes, 1);

    println!(
        "Generated endless long-run, mid-jump and recovery saves through production controls in {}",
        output.display()
    );
}
