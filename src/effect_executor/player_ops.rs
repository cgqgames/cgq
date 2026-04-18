use bevy::prelude::*;
use rand::seq::SliceRandom;
use std::time::{Duration, Instant};

use crate::effect::EffectOperation;
use crate::players::Player;

use super::{EffectError, EffectResult};

pub fn execute_player_operation(
    operation: &EffectOperation,
    world: &mut World,
) -> Option<EffectResult> {
    if let EffectOperation::TimeoutPlayer { target, seconds } = operation {
        Some(execute_timeout_player(target, *seconds, world))
    } else {
        None
    }
}

fn execute_timeout_player(target: &str, seconds: u64, world: &mut World) -> EffectResult {
    let until = Instant::now() + Duration::from_secs(seconds);
    let entities = select_targets(target, world);

    if entities.is_empty() {
        info!("timeout_player({}): no matching players", target);
        return Ok(());
    }

    for entity in entities {
        let Some(mut player) = world.get_mut::<Player>(entity) else { continue };
        // Stacking: longer-wins — a new shorter timeout never reduces an
        // existing longer one.
        player.timeout_until = match player.timeout_until {
            Some(existing) if existing > until => Some(existing),
            _ => Some(until),
        };
    }

    Ok(())
}

fn select_targets(target: &str, world: &mut World) -> Vec<Entity> {
    if let Some(n_str) = target.strip_prefix("random_") {
        let n = n_str.parse::<usize>().unwrap_or(0);
        return select_random(n, world);
    }

    match target {
        "all" => select_all(world),
        "last_answerer" => select_last_answerer(world),
        "most_correct" => select_most_correct(world),
        _ => {
            warn!("Unknown timeout_player target: {}", target);
            Vec::new()
        }
    }
}

fn select_all(world: &mut World) -> Vec<Entity> {
    world
        .query::<(Entity, &Player)>()
        .iter(world)
        .map(|(e, _)| e)
        .collect()
}

fn select_random(count: usize, world: &mut World) -> Vec<Entity> {
    let mut all = select_all(world);
    let mut rng = rand::thread_rng();
    all.shuffle(&mut rng);
    all.truncate(count);
    all
}

fn select_last_answerer(world: &mut World) -> Vec<Entity> {
    let mut best: Option<(Entity, Instant)> = None;
    let mut q = world.query::<(Entity, &Player)>();
    for (entity, player) in q.iter(world) {
        let Some(last) = player.answer_history.last().map(|r| r.at) else { continue };
        if best.map_or(true, |(_, t)| last > t) {
            best = Some((entity, last));
        }
    }
    best.map(|(e, _)| vec![e]).unwrap_or_default()
}

fn select_most_correct(world: &mut World) -> Vec<Entity> {
    let mut best: Option<(Entity, usize)> = None;
    let mut q = world.query::<(Entity, &Player)>();
    for (entity, player) in q.iter(world) {
        let correct = player
            .answer_history
            .iter()
            .filter(|r| r.was_correct)
            .count();
        if correct == 0 {
            continue;
        }
        if best.map_or(true, |(_, c)| correct > c) {
            best = Some((entity, correct));
        }
    }
    best.map(|(e, _)| vec![e]).unwrap_or_default()
}

// `EffectError` is only referenced through the function signatures above;
// `execute_timeout_player` currently never fails, but keeping the import
// stable means a future selector that validates arguments can return one.
#[allow(dead_code)]
fn _marker(_: EffectError) {}
