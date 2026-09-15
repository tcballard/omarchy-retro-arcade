//! Run a bounded, tick-stamped production-input replay through `Session`.
use omarchy_freeski::{
    course,
    engine::{Event, Input, Mode, Phase, Sim},
    session::Session,
    storage::Save,
};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::PathBuf};

const FILE_LIMIT: u64 = 1024 * 1024;
const TICK_LIMIT: u64 = 1_000_000;

#[derive(Deserialize)]
struct Replay {
    initial: Save,
    inputs: Vec<TickInput>,
    tick_limit: u64,
    #[serde(default)]
    expected: Option<ExpectedFinal>,
}

#[derive(Deserialize)]
struct TickInput {
    tick: u64,
    input: Input,
    /// Equivalent to the fresh F/button action before this tick.
    #[serde(default)]
    toggle_fast: bool,
    #[serde(default)]
    expected: Option<Sim>,
}

#[derive(Deserialize)]
struct ExpectedFinal {
    phase: Phase,
    ticks: u64,
    distance: f64,
}

#[derive(Serialize)]
struct Summary {
    ticks: u64,
    phase: String,
    distance: f64,
    crashes: u8,
    events: Vec<(u64, Event)>,
}

fn main() -> Result<(), String> {
    let path = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("usage: cargo run -p omarchy-freeski --example replay -- replay.json")?,
    );
    let mut bytes = Vec::new();
    File::open(&path)
        .map_err(|error| format!("{}: {error}", path.display()))?
        .take(FILE_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > FILE_LIMIT {
        return Err("replay exceeds 1 MiB".into());
    }
    let replay: Replay =
        serde_json::from_slice(&bytes).map_err(|error| format!("invalid replay JSON: {error}"))?;
    if replay.tick_limit == 0 || replay.tick_limit > TICK_LIMIT {
        return Err(format!("tick_limit must be within 1..={TICK_LIMIT}"));
    }
    // Validate bounded identity/numbers before Session is allowed to derive an
    // endless terrain window from the position.
    let validation_obstacles = match replay.initial.mode {
        Mode::Practice => omarchy_freeski::world::practice(),
        Mode::FreeSki => Vec::new(),
        Mode::Slalom => course::course(replay.initial.course_index)
            .map(|course| course.obstacles)
            .unwrap_or_default(),
    };
    if !replay.initial.valid(&validation_obstacles) {
        return Err("initial save is incompatible or invalid".into());
    }
    let mut session = Session::new(replay.initial);
    session.state.run.start();
    let last_tick = session
        .state
        .run
        .ticks
        .checked_add(replay.tick_limit)
        .ok_or("tick_limit overflows the initial tick")?;
    let mut events = Vec::new();
    for scheduled in replay.inputs {
        let expected = session.state.run.ticks + 1;
        if scheduled.tick != expected {
            return Err(format!(
                "input order diverged: expected tick {expected}, got {}",
                scheduled.tick
            ));
        }
        if scheduled.tick > last_tick {
            return Err(format!(
                "input tick {} exceeds the {}-tick run limit",
                scheduled.tick, replay.tick_limit
            ));
        }
        if !scheduled.input.heading.is_finite() {
            return Err(format!("non-finite heading at tick {}", scheduled.tick));
        }
        if scheduled.toggle_fast {
            session.state.run.toggle_fast_mode();
        }
        for event in session.step(scheduled.input) {
            events.push((scheduled.tick, event));
        }
        if !session.state.valid(&session.obstacles) {
            return Err(format!(
                "invalid state first produced at tick {}",
                scheduled.tick
            ));
        }
        if let Some(expected) = scheduled.expected {
            if session.state.run != expected {
                return Err(format!(
                    "first divergence at tick {}: expected {:?}, got {:?}",
                    scheduled.tick, expected, session.state.run
                ));
            }
        }
        if session.state.run.ended() {
            break;
        }
    }
    if let Some(expected) = replay.expected {
        if session.state.run.phase != expected.phase
            || session.state.run.ticks != expected.ticks
            || session.state.run.distance != expected.distance
        {
            return Err(format!(
                "final divergence at tick {}: expected {:?}/{} ticks/{:.6} m, got {:?}/{} ticks/{:.6} m",
                session.state.run.ticks,
                expected.phase,
                expected.ticks,
                expected.distance,
                session.state.run.phase,
                session.state.run.ticks,
                session.state.run.distance,
            ));
        }
    }
    let summary = Summary {
        ticks: session.state.run.ticks,
        phase: format!("{:?}", session.state.run.phase),
        distance: session.state.run.distance,
        crashes: session.state.run.crashes,
        events,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&summary).map_err(|error| error.to_string())?
    );
    Ok(())
}
