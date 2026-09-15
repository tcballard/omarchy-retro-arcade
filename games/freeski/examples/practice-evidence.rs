//! Generate inspectable native save fixtures through ordinary production inputs.
use omarchy_freeski::{
    engine::{Input, Phase},
    session::Session,
    storage::{self, Save},
};
fn main() {
    let dir = std::path::PathBuf::from(std::env::args_os().nth(1).expect("output directory"));
    // A straight run reaches the braking-section tree and exercises actual recovery.
    let mut session = Session::new(Save::default());
    session.state.run.start();
    for _ in 0..12000 {
        session.step(Input::default());
        if session.state.run.tumble > 0 {
            storage::write(
                &dir.join("recovery.json"),
                &session.state,
                &session.obstacles,
            )
            .unwrap();
            break;
        }
    }
    assert!(session.state.run.tumble > 0);
    // Aim for the first ramp with bounded normal steering, then capture in flight.
    session = Session::new(Save::default());
    session.state.run.start();
    let mut captured = false;
    let mut approach = false;
    for _ in 0..12000 {
        let target_x = if session.state.run.position.y < 280. {
            0.
        } else {
            -12.
        };
        let heading = (target_x - session.state.run.position.x).atan2(24.);
        session.step(Input {
            heading,
            brake: false,
        });
        if !approach && session.state.run.position.y >= 370. {
            storage::write(
                &dir.join("ramp-approach.json"),
                &session.state,
                &session.obstacles,
            )
            .unwrap();
            approach = true;
        }
        if session.state.run.jump.is_some_and(|t| t > 0.3) {
            storage::write(
                &dir.join("mid-jump.json"),
                &session.state,
                &session.obstacles,
            )
            .unwrap();
            captured = true;
            break;
        }
        assert!(!session.state.run.ended());
    }
    assert!(captured && approach);
    assert_eq!(session.state.run.phase, Phase::Running);
    println!(
        "Generated ramp-approach, mid-jump and recovery saves through production controls in {}",
        dir.display()
    );
}
