/// Demonstration of the CGQ Generic Effect System
///
/// This example shows how card effects work using primitive operations.
/// Run with: cargo run --example effect_demo

use cgq::effect::{CardEffect, EffectOperation, Predicate, Value, EffectContext};
use cgq::effect_executor::EffectExecutor;
use cgq::game_state::GameState;
use cgq::collections::{CollectionManager, Collection};
use bevy::prelude::*;
use bevy::app::App;
use std::collections::HashMap;

fn main() {
    println!("=== CGQ Generic Effect System Demo ===\n");

    // Create Bevy world
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Add resources
    app.insert_resource(cgq::GameTimer {
        timer: Timer::from_seconds(300.0, TimerMode::Once),
        paused: false,
    });
    app.insert_resource(cgq::Score {
        current: 0,
        passing_grade: 10,
        correct_answers: 0,
        total_answered: 0,
    });
    app.insert_resource(GameState::new());
    app.insert_resource(CollectionManager::new());

    // Demo 1: Simple value modification
    println!("Demo 1: Add Time Effect");
    println!("------------------------");
    demo_add_time(app.world_mut());
    println!();

    // Demo 2: Conditional effects
    println!("Demo 2: Conditional Effects");
    println!("---------------------------");
    demo_conditional(app.world_mut());
    println!();

    // Demo 3: Collection operations
    println!("Demo 3: Collection Operations");
    println!("-----------------------------");
    demo_collections(app.world_mut());
    println!();

    // Demo 4: Complex card effect
    println!("Demo 4: Complex Card Effect");
    println!("---------------------------");
    demo_complex_card(app.world_mut());
    println!();

    println!("=== Demo Complete ===");
}

fn demo_add_time(world: &mut World) {
    let mut state = GameState::new();
    let mut executor = EffectExecutor::new();
    let mut context = EffectContext::new("demo_card".to_string(), "add_time".to_string());

    // Get initial timer value
    let initial = state.get("timer.remaining", world).unwrap();
    println!("Initial timer: {:?}", initial);

    // Create "Add Time" effect
    let effect = CardEffect {
        id: "add_time".to_string(),
        name: Some("Add Time".to_string()),
        description: Some("Adds 60 seconds to the timer".to_string()),
        operations: vec![
            EffectOperation::Add {
                target: "timer.remaining".to_string(),
                amount: 60,
            }
        ],
        timing: cgq::EffectTiming::After,
        priority: 100,
        intercepts: None,
    };

    // Execute effect
    executor.execute_effect(&effect, &mut context, &mut state, world).unwrap();

    // Check result
    let final_value = state.get("timer.remaining", world).unwrap();
    println!("After effect: {:?}", final_value);
    println!("✓ Successfully added 60 seconds");
}

fn demo_conditional(world: &mut World) {
    let mut state = GameState::new();
    let mut executor = EffectExecutor::new();
    let mut context = EffectContext::new("demo_card".to_string(), "conditional".to_string());

    let initial = state.get("timer.remaining", world).unwrap();
    println!("Current timer: {:?}", initial);

    // Create conditional effect (like Palestine Action)
    let effect = CardEffect {
        id: "conditional".to_string(),
        name: Some("Conditional Effect".to_string()),
        description: Some("Different effects based on timer".to_string()),
        operations: vec![
            EffectOperation::IfCondition {
                condition: Predicate::GreaterThan {
                    field: "timer.remaining".to_string(),
                    value: Value::Int(200),
                },
                then: vec![
                    EffectOperation::Add {
                        target: "score.current".to_string(),
                        amount: 5,
                    }
                ],
                else_: Some(vec![
                    EffectOperation::Subtract {
                        target: "score.current".to_string(),
                        amount: 2,
                    }
                ]),
            }
        ],
        timing: cgq::EffectTiming::After,
        priority: 100,
        intercepts: None,
    };

    let score_before = state.get("score.current", world).unwrap();
    println!("Score before: {:?}", score_before);

    executor.execute_effect(&effect, &mut context, &mut state, world).unwrap();

    let score_after = state.get("score.current", world).unwrap();
    println!("Score after: {:?}", score_after);

    if let (Value::Int(timer), Value::Int(before), Value::Int(after)) = (initial, score_before, score_after) {
        if timer > 200 {
            println!("✓ Timer > 200, score increased by {}", after - before);
        } else {
            println!("✓ Timer ≤ 200, score decreased by {}", before - after);
        }
    }
}

fn demo_collections(world: &mut World) {
    // Create a collection of question options
    let mut collections = CollectionManager::new();
    let mut options = Collection::new();

    // Add 4 options (1 correct, 3 wrong)
    for i in 1..=4 {
        let mut option = HashMap::new();
        option.insert("id".to_string(), Value::String(format!("option_{}", i)));
        option.insert("text".to_string(), Value::String(format!("Answer {}", i)));
        option.insert("correct".to_string(), Value::Bool(i == 2)); // Option 2 is correct
        options.append(Value::Object(option));
    }

    collections.set("question.options", options);
    world.insert_resource(collections);

    let mut state = GameState::new();
    let mut executor = EffectExecutor::new();
    let mut context = EffectContext::new("demo_card".to_string(), "filter".to_string());

    println!("Initial options: 4 (A, B, C, D)");

    // Create "Eliminate Wrong Answer" effect
    let effect = CardEffect {
        id: "eliminate_wrong".to_string(),
        name: Some("Eliminate Wrong Answer".to_string()),
        description: Some("Removes incorrect options".to_string()),
        operations: vec![
            EffectOperation::Filter {
                target: "question.options".to_string(),
                predicate: Predicate::Equals {
                    field: "correct".to_string(),
                    value: Value::Bool(false),
                },
            },
            EffectOperation::Remove {
                target: "question.options".to_string(),
                count: 1,
                filter: None,
                random: Some(true),
            },
        ],
        timing: cgq::EffectTiming::After,
        priority: 100,
        intercepts: None,
    };

    executor.execute_effect(&effect, &mut context, &mut state, world).unwrap();

    // Check result
    let collections = world.get_resource::<CollectionManager>().unwrap();
    let options = collections.get("question.options").unwrap();
    println!("After filtering & removing: {} options", options.len());
    println!("✓ Successfully eliminated wrong answer");
}

fn demo_complex_card(world: &mut World) {
    let mut state = GameState::new();
    let mut executor = EffectExecutor::new();
    let mut context = EffectContext::new("handala".to_string(), "freeze_timer".to_string());

    // Create "Handala" effect - freezes timer until points earned
    let effect = CardEffect {
        id: "freeze_timer".to_string(),
        name: Some("Handala - Freeze Timer".to_string()),
        description: Some("Freezes timer until 1 point is earned".to_string()),
        operations: vec![
            EffectOperation::SetFlag {
                flag: "timer.paused".to_string(),
                value: true,
            },
            EffectOperation::SetVariable {
                name: "points_needed".to_string(),
                value: Value::Int(1),
            },
            // In a real implementation, we'd register an OnEvent listener here
            // For demo purposes, we'll just show the pause worked
        ],
        timing: cgq::EffectTiming::After,
        priority: 100,
        intercepts: None,
    };

    let paused_before = state.get("timer.paused", world).unwrap();
    println!("Timer paused before: {:?}", paused_before);

    executor.execute_effect(&effect, &mut context, &mut state, world).unwrap();

    let paused_after = state.get("timer.paused", world).unwrap();
    println!("Timer paused after: {:?}", paused_after);

    let points_threshold = context.get_variable("points_needed").unwrap();
    println!("Points threshold: {:?}", points_threshold);

    println!("✓ Timer successfully frozen until {} point(s) earned",
             if let Value::Int(p) = points_threshold { p } else { &0 });
}
