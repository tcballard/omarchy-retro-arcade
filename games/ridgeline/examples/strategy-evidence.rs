//! Play every map and difficulty with the reference strategy through the
//! production engine. With a directory argument, write each winning replay.
use omarchy_ridgeline::{battle::Phase, data::*, strategy};

fn main() {
    let out = std::env::args().nth(1).map(std::path::PathBuf::from);
    let mut failed = false;
    println!("| Map | Difficulty | Result | Base health | Towers | Credits left | Combat time |");
    println!("| --- | --- | --- | ---: | ---: | ---: | ---: |");
    for (i, map) in maps().iter().enumerate() {
        for d in Difficulty::ALL {
            let (replay, b) = strategy::play(i, d);
            let won = b.phase == Phase::Victory;
            failed |= !won;
            println!(
                "| {:02} {} | {} | {} (wave {}) | {} | {} | {} | {}:{:02} |",
                i + 1,
                map.name,
                d.name(),
                if won { "Victory" } else { "Defeat" },
                b.wave + 1,
                b.health,
                b.towers.len(),
                b.credits,
                b.tick / 3600,
                b.tick / 60 % 60
            );
            if let (Some(dir), true) = (&out, won) {
                std::fs::create_dir_all(dir).unwrap();
                let name = format!("{:02}-{}.json", i + 1, d.name().to_lowercase());
                std::fs::write(
                    dir.join(name),
                    serde_json::to_string(&replay).unwrap() + "\n",
                )
                .unwrap();
            }
        }
    }
    if failed {
        std::process::exit(1);
    }
}
