use super::*;

fn flat() -> Game {
    let mut game = Game::new(42);
    game.terrain.heights.fill(400.0);
    game.tanks = [
        Tank::new(140.0, &game.terrain, 45),
        Tank::new(860.0, &game.terrain, 135),
    ];
    game.active = 0;
    game.starter = 0;
    game.wind = 0.0;
    game
}
fn fly(game: &mut Game) {
    for _ in 0..=MAX_FLIGHT_TICKS {
        if game.phase != Phase::Flying {
            return;
        }
        game.tick();
    }
    panic!("unbounded flight");
}

#[test]
fn seeded_generation_is_repeatable_supported_and_varied() {
    let mut varied = 0;
    for seed in 0..1000 {
        let game = Game::new(seed);
        assert_eq!(game, Game::new(seed));
        assert!(game.terrain.valid());
        assert!((-20.0..=20.0).contains(&game.wind));
        for tank in &game.tanks {
            assert_eq!(tank.position.y, game.terrain.support(tank.position.x));
            assert!(FLOOR - tank.position.y >= 100.0);
        }
        if game.terrain.heights.windows(2).any(|p| p[0] != p[1]) {
            varied += 1;
        }
    }
    assert!(
        varied > 900,
        "generation must not normally use the fallback"
    );
}

#[test]
fn ready_and_flight_gate_commands_without_mutation() {
    let mut game = flat();
    let initial = game.clone();
    assert_eq!(game.fire(), Err(Rejected::WrongPhase));
    assert_eq!(game.move_one(true), Err(Rejected::WrongPhase));
    assert_eq!(game.aim(60, 80), Err(Rejected::WrongPhase));
    assert_eq!(game, initial);
    game.ready().unwrap();
    game.select(Weapon::Heavy).unwrap();
    game.fire().unwrap();
    let shot = game.clone();
    assert_eq!(game.fire(), Err(Rejected::WrongPhase));
    assert_eq!(game.select(Weapon::Digger), Err(Rejected::WrongPhase));
    assert_eq!(game.aim(60, 80), Err(Rejected::WrongPhase));
    assert_eq!(game.move_one(true), Err(Rejected::WrongPhase));
    assert_eq!(game, shot);
    assert_eq!(game.tanks[0].heavy, 2);
    fly(&mut game);
    assert_eq!(game.tanks[0].heavy, 2);
    assert_eq!(game.phase, Phase::Ready);
    assert_eq!(game.active, 1);
    assert_eq!(game.fire(), Err(Rejected::WrongPhase));
}

#[test]
fn aim_bounds_ammunition_and_unlimited_shells() {
    let mut game = flat();
    game.ready().unwrap();
    for (a, p) in [(4, 50), (176, 50), (90, 0), (90, 101)] {
        let before = game.clone();
        assert_eq!(game.aim(a, p), Err(Rejected::OutOfRange));
        assert_eq!(before, game);
    }
    game.aim(5, 1).unwrap();
    game.aim(175, 100).unwrap();
    game.tanks[0].heavy = 0;
    game.tanks[0].diggers = 0;
    assert_eq!(game.select(Weapon::Heavy), Err(Rejected::NoAmmo));
    assert_eq!(game.select(Weapon::Digger), Err(Rejected::NoAmmo));
    game.select(Weapon::Shell).unwrap();
    game.fire().unwrap();
    fly(&mut game);
    assert!(game.tanks[0].available(Weapon::Shell));
}

#[test]
fn launch_mapping_and_wind_use_fixed_ticks() {
    let mut game = flat();
    game.ready().unwrap();
    game.aim(90, 100).unwrap();
    game.wind = 12.0;
    game.fire().unwrap();
    let p = game.projectile.unwrap();
    assert!((p.velocity.y + 400.0).abs() < 1e-9);
    assert!((p.position.y - 373.0).abs() < 1e-9);
    game.tick();
    let q = game.projectile.unwrap();
    assert!((q.velocity.x - 12.0 * DT).abs() < 1e-9);
    assert!((q.velocity.y - p.velocity.y - GRAVITY * DT).abs() < 1e-9);
    assert_eq!(game.wind, 12.0);
}

#[test]
fn sweep_catches_thin_terrain_and_tanks_in_travel_order() {
    let mut game = flat();
    game.terrain.heights[500] = 200.0;
    let start = Point { x: 490.0, y: 300.0 };
    let end = Point { x: 880.0, y: 300.0 };
    let at = start.lerp(end, game.collision(start, end).unwrap());
    assert!((at.x - 499.5).abs() < 1e-9);
    // A direct tank hit is swept, even when a full tick crosses its body.
    game.terrain.heights.fill(400.0);
    let start = Point { x: 820.0, y: 390.0 };
    let end = Point { x: 900.0, y: 390.0 };
    assert_eq!(game.collision(start, end), Some(28.0 / 80.0));
    assert_eq!(game.collision(end, start), Some(28.0 / 80.0));
    assert_eq!(
        game.collision(Point { x: 500.0, y: 300.0 }, Point { x: 500.0, y: 500.0 }),
        Some(0.5)
    );
}

#[test]
fn direct_hit_uses_blast_falloff_without_bonus() {
    let mut game = flat();
    let at = Point { x: 848.0, y: 393.0 };
    let old = game.tanks[1].position.y;
    game.explode(at, Weapon::Shell);
    let fall = ((game.tanks[1].position.y - old - 20.0).max(0.0) / 2.0).floor() as u16;
    assert_eq!(game.tanks[1].health, 100 - 36 - fall);
    assert_eq!(game.tanks[0].health, 100);
}

#[test]
fn craters_have_exact_edges_never_raise_ground_and_stop_at_floor() {
    let mut terrain = flat().terrain;
    terrain.crater(Point { x: 500.0, y: 400.0 }, 45.0);
    assert_eq!(terrain.heights[500], 445.0);
    assert_eq!(terrain.heights[455], 400.0);
    assert_eq!(terrain.heights[454], 400.0);
    assert_eq!(terrain.heights[545], 400.0);
    let previous = terrain.clone();
    terrain.crater(Point { x: 500.0, y: 100.0 }, 45.0);
    assert_eq!(terrain, previous);
    for _ in 0..10 {
        terrain.crater(
            Point {
                x: 500.0,
                y: terrain.heights[500],
            },
            85.0,
        );
    }
    assert_eq!(terrain.heights[500], FLOOR);
    assert!(terrain.heights.iter().all(|&y| y <= FLOOR));
}

#[test]
fn self_damage_and_simultaneous_elimination_are_resolved_together() {
    let mut game = flat();
    game.tanks[1] = Tank::new(166.0, &game.terrain, 135);
    for tank in &mut game.tanks {
        tank.health = 1;
    }
    game.explode(Point { x: 153.0, y: 393.0 }, Weapon::Heavy);
    assert_eq!([game.tanks[0].health, game.tanks[1].health], [0, 0]);
    game.finish_shot();
    assert_eq!(game.phase, Phase::RoundOver { winner: None });
    assert_eq!(game.wins, [0, 0]);
    game.next_round().unwrap();
    assert_eq!(game.active, 1);
    assert_eq!(game.round, 2);
    assert_eq!(game.wins, [0, 0]);
}

#[test]
fn support_checks_whole_track_and_fall_damage_rounds_down() {
    let mut game = flat();
    game.terrain.heights[128..=152].fill(500.0);
    game.terrain.heights[140] = 410.0;
    assert_eq!(game.terrain.support(140.0), 410.0);
    // A remote blast triggers settling without direct blast damage.
    game.terrain.heights[140] = 425.0;
    game.explode(Point { x: 600.0, y: 400.0 }, Weapon::Shell);
    assert_eq!(game.tanks[0].position.y, 425.0);
    assert_eq!(game.tanks[0].health, 98);
    game.terrain.heights[128..=152].fill(FLOOR);
    game.explode(Point { x: 600.0, y: 400.0 }, Weapon::Shell);
    assert_eq!(game.tanks[0].position.y, FLOOR);
    assert_eq!(game.tanks[0].health, 31);
    game.explode(Point { x: 600.0, y: 400.0 }, Weapon::Shell);
    assert_eq!(game.tanks[0].health, 31);
}

#[test]
fn movement_spends_travel_only_and_rejects_edges_slopes_and_tanks() {
    let mut game = flat();
    game.ready().unwrap();
    for _ in 0..60 {
        game.move_one(true).unwrap();
    }
    assert_eq!(game.tanks[0].fuel, 0.0);
    assert_eq!(game.tanks[0].position.x, 200.0);
    let before = game.clone();
    assert_eq!(game.move_one(true), Err(Rejected::NoFuel));
    assert_eq!(game, before);
    game.tanks[0].fuel = 60.0;
    game.terrain.heights[213] = 200.0;
    let before = game.clone();
    assert_eq!(game.move_one(true), Err(Rejected::Steep));
    assert_eq!(game, before);
    game.terrain.heights.fill(400.0);
    game.tanks[0].position.x = HALF_WIDTH;
    assert_eq!(game.move_one(false), Err(Rejected::Edge));
    game.tanks[0].position.x = 835.0;
    assert_eq!(game.move_one(true), Err(Rejected::Occupied));
}

#[test]
fn misses_above_screen_and_lifetime_are_bounded() {
    let mut game = flat();
    game.ready().unwrap();
    game.aim(90, 100).unwrap();
    game.fire().unwrap();
    let mut above = false;
    for _ in 0..MAX_FLIGHT_TICKS {
        if let Some(p) = game.projectile {
            above |= p.position.y < 0.0;
        }
        game.tick();
    }
    assert!(above);
    assert_ne!(game.phase, Phase::Flying);
    let mut game = flat();
    game.ready().unwrap();
    game.fire().unwrap();
    game.projectile.as_mut().unwrap().position = Point {
        x: 500.0,
        y: -10000.0,
    };
    game.projectile.as_mut().unwrap().ticks = MAX_FLIGHT_TICKS - 1;
    let before = game.terrain.clone();
    game.tick();
    assert_eq!(game.phase, Phase::Ready);
    assert_eq!(game.terrain, before);
    assert_eq!(
        exit_time(
            Point { x: 990.0, y: 100.0 },
            Point {
                x: 1010.0,
                y: 100.0
            }
        ),
        Some(0.5)
    );
    assert_eq!(
        exit_time(Point { x: 10.0, y: 100.0 }, Point { x: -10.0, y: 100.0 }),
        Some(0.5)
    );
    assert_eq!(
        exit_time(Point { x: 10.0, y: 590.0 }, Point { x: 10.0, y: 610.0 }),
        Some(0.5)
    );
}

#[test]
fn rounds_reset_resources_alternate_starters_and_count_wins_once() {
    let mut game = flat();
    for win in 1..=2 {
        game.ready().unwrap();
        game.fire().unwrap();
        game.tanks[1].health = 0;
        fly(&mut game);
        assert_eq!(game.wins, [win, 0]);
        let finished = game.clone();
        for _ in 0..100 {
            game.tick();
        }
        assert_eq!(game, finished);
        if win == 1 {
            game.next_round().unwrap();
            assert_eq!(game.active, 1);
            for tank in &game.tanks {
                assert_eq!(tank.health, 100);
                assert_eq!(tank.fuel, 60.0);
                assert_eq!((tank.heavy, tank.diggers), (3, 2));
            }
        }
    }
    assert_eq!(game.phase, Phase::MatchOver { winner: 0 });
    assert_eq!(game.next_round(), Err(Rejected::WrongPhase));
    assert_eq!(game.ready(), Err(Rejected::WrongPhase));
}

#[test]
fn clone_mid_flight_and_tick_batching_produce_identical_results() {
    let mut a = flat();
    a.ready().unwrap();
    a.select(Weapon::Digger).unwrap();
    a.fire().unwrap();
    for _ in 0..31 {
        a.tick();
    }
    let mut b = a.clone();
    // Rendering and pause own no engine time. Tick batches are equivalent.
    for _ in 0..600 {
        a.tick();
    }
    for _ in 0..100 {
        for _ in 0..6 {
            b.tick();
        }
    }
    assert_eq!(a, b);
    assert_eq!(a.tanks[0].diggers, 1);
    assert!(!a.traces[0].is_empty());
    let trace = a.traces[0].clone();
    b.ready().unwrap();
    b.fire().unwrap();
    assert_eq!(b.traces[0], trace);
    assert_eq!(b.traces[1].len(), 1);
}
