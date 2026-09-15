//! Find a scenic, reproducible chase frame using ordinary production inputs.
//! The native capture script resumes this disposable save; it never edits actors.
use omarchy_freeski::{
    chase::ChasePhase,
    endless,
    engine::Input,
    session::Session,
    storage::{self, Save},
};

fn main() {
    let output = std::path::PathBuf::from(std::env::args_os().nth(1).expect("output save path"));
    // Matched practice poses reached through normal ticks, then the real toggle.
    let mut pose = Session::new(Save::default());
    pose.state.run.start();
    for _ in 0..180 {
        pose.step(Input::default());
    }
    storage::write(
        &output.with_file_name("pose-normal.json"),
        &pose.state,
        &pose.obstacles,
    )
    .unwrap();
    pose.state.run.toggle_fast_mode();
    storage::write(
        &output.with_file_name("pose-fast.json"),
        &pose.state,
        &pose.obstacles,
    )
    .unwrap();
    for seed in 0..128 {
        let mut save = Save::default();
        save.select_chase(true, seed);
        save.run.start();
        let mut session = Session::new(save);
        for _ in 0..12_000 {
            let run = &session.state.run;
            let chase = &session.state.chase;
            let dx = chase.position.x - run.position.x;
            let dy = chase.position.y - run.position.y;
            if chase.phase == ChasePhase::Active && dx.hypot(dy) < 10. && !run.fast_mode {
                session.state.run.toggle_fast_mode();
            }
            let heading = endless::reference_heading(seed, &session.state.run);
            session.step(Input {
                heading,
                brake: false,
            });
            if session.state.run.ended() {
                break;
            }
            let run = &session.state.run;
            let chase = &session.state.chase;
            let dx = chase.position.x - run.position.x;
            let dy = chase.position.y - run.position.y;
            if chase.phase == ChasePhase::Active
                && run.fast_mode
                && run.tumble == 0
                && run.protection == 0
                && run.height() == 0.
                && run.speed > 30.
                && dx.abs() < 16.
                && dy > -8.
                && dy < 18.
            {
                storage::write(&output, &session.state, &session.obstacles).unwrap();
                println!(
                    "seed {seed}, tick {}, speed {:.1}, heading {:.2}, yeti offset {:.1}, {:.1}",
                    run.ticks, run.speed, run.heading, dx, dy
                );
                return;
            }
        }
    }
    panic!("No suitable production chase frame found");
}
