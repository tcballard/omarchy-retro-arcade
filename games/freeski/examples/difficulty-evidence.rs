//! Compare ordinary production inputs across seeds; this is not human acceptance.
use omarchy_freeski::{
    chase::ChasePhase,
    endless,
    engine::{Event, Input, Phase},
    session::Session,
    storage::Save,
};
use serde::Serialize;

#[derive(Default, Serialize)]
struct Summary {
    strategy: String,
    fast: bool,
    runs: u32,
    caught: u32,
    crashed: u32,
    surviving: u32,
    crashes: u32,
    maximum_unprotected_stationary_ticks: u32,
    minimum_catch_seconds: Option<f64>,
    maximum_catch_seconds: Option<f64>,
    mean_distance: f64,
}

fn main() {
    let mut summaries = Vec::new();
    for strategy in ["straight", "route", "left-edge", "right-edge"] {
        for fast in [false, true] {
            let mut summary = Summary {
                strategy: strategy.into(),
                fast,
                ..Summary::default()
            };
            for seed in 0..32 {
                let mut save = Save::default();
                save.select_chase(true, seed);
                save.run.start();
                if fast {
                    save.run.toggle_fast_mode();
                }
                let mut session = Session::new(save);
                let mut spawned = None;
                let mut stationary = 0;
                for _ in 0..12_000 {
                    let run = &session.state.run;
                    let heading = match strategy {
                        "route" => endless::reference_heading(seed, run),
                        "left-edge" => (-38.8 - run.position.x).atan2(20.),
                        "right-edge" => (38.8 - run.position.x).atan2(20.),
                        _ => 0.,
                    };
                    let protected = run.tumble > 0 || run.protection > 0;
                    let before = session.state.chase.position;
                    let events = session.step(Input {
                        heading,
                        brake: false,
                    });
                    assert!(
                        session.state.valid(&session.obstacles),
                        "seed {seed}, {strategy}, fast {fast}"
                    );
                    if events.contains(&Event::Spawn) {
                        spawned = Some(session.state.run.ticks);
                    }
                    let after = session.state.chase.position;
                    if session.state.chase.phase == ChasePhase::Active
                        && !protected
                        && (after.x - before.x).hypot(after.y - before.y) < 0.001
                    {
                        stationary += 1;
                        summary.maximum_unprotected_stationary_ticks =
                            summary.maximum_unprotected_stationary_ticks.max(stationary);
                    } else {
                        stationary = 0;
                    }
                    if session.state.run.ended() {
                        break;
                    }
                }
                let run = &session.state.run;
                summary.runs += 1;
                summary.crashes += u32::from(run.crashes);
                summary.mean_distance += run.distance;
                match run.phase {
                    Phase::Caught => {
                        summary.caught += 1;
                        let seconds =
                            (run.ticks - spawned.expect("catch requires spawn")) as f64 / 60.;
                        summary.minimum_catch_seconds = Some(
                            summary
                                .minimum_catch_seconds
                                .map_or(seconds, |v| v.min(seconds)),
                        );
                        summary.maximum_catch_seconds = Some(
                            summary
                                .maximum_catch_seconds
                                .map_or(seconds, |v| v.max(seconds)),
                        );
                    }
                    Phase::Crashed => summary.crashed += 1,
                    _ => summary.surviving += 1,
                }
            }
            summary.mean_distance /= f64::from(summary.runs);
            summaries.push(summary);
        }
    }
    println!("{}", serde_json::to_string_pretty(&summaries).unwrap());
}
