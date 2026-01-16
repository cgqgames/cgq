/// Comprehensive tests for all effect operations
#[cfg(test)]
mod effect_operation_tests {
    use super::super::*;
    use crate::collections::{CollectionManager, Collection};
    use std::collections::HashMap;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::resources::GameTimer {
            timer: bevy::time::Timer::from_seconds(300.0, bevy::time::TimerMode::Once),
            paused: false,
        });
        world.insert_resource(crate::resources::Score {
            current: 10,
            passing_grade: 20,
            correct_answers: 0,
            total_answered: 0,
        });
        world.insert_resource(CollectionManager::new());
        world
    }

    // ==================== VALUE OPERATIONS ====================

    #[test]
    fn test_add_operation_detailed() {
        let mut world = setup_world();
        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "add".to_string());

        // Add to timer
        let op = EffectOperation::Add {
            target: "timer.remaining".to_string(),
            amount: 120,
        };
        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());

        if let Some(Value::Int(v)) = state.get("timer.remaining", &world) {
            assert!(v >= 419 && v <= 421); // 300 + 120 ± 1
        } else {
            panic!("Expected Int value");
        }

        // Add to score
        let op = EffectOperation::Add {
            target: "score.current".to_string(),
            amount: 5,
        };
        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());
        assert_eq!(state.get("score.current", &world), Some(Value::Int(15)));
    }

    #[test]
    fn test_subtract_operation() {
        let mut world = setup_world();
        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "subtract".to_string());

        let op = EffectOperation::Subtract {
            target: "score.current".to_string(),
            amount: 3,
        };
        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());
        assert_eq!(state.get("score.current", &world), Some(Value::Int(7)));
    }

    #[test]
    fn test_multiply_operation() {
        let mut world = setup_world();
        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "multiply".to_string());

        let op = EffectOperation::Multiply {
            target: "score.current".to_string(),
            factor: 2.0,
        };
        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());
        assert_eq!(state.get("score.current", &world), Some(Value::Int(20)));
    }

    #[test]
    fn test_set_operation() {
        let mut world = setup_world();
        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "set".to_string());

        let op = EffectOperation::Set {
            target: "score.current".to_string(),
            value: Value::Int(100),
        };
        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());
        assert_eq!(state.get("score.current", &world), Some(Value::Int(100)));
    }

    // ==================== FLAG OPERATIONS ====================

    #[test]
    fn test_set_flag_operation() {
        let mut world = setup_world();
        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "set_flag".to_string());

        // Set to true
        let op = EffectOperation::SetFlag {
            flag: "timer.paused".to_string(),
            value: true,
        };
        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());
        assert_eq!(state.get("timer.paused", &world), Some(Value::Bool(true)));

        // Set to false
        let op = EffectOperation::SetFlag {
            flag: "timer.paused".to_string(),
            value: false,
        };
        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());
        assert_eq!(state.get("timer.paused", &world), Some(Value::Bool(false)));
    }

    #[test]
    fn test_toggle_flag_operation() {
        let mut world = setup_world();
        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "toggle".to_string());

        // Initial state is false
        assert_eq!(state.get("timer.paused", &world), Some(Value::Bool(false)));

        // Toggle to true
        let op = EffectOperation::ToggleFlag {
            flag: "timer.paused".to_string(),
        };
        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());
        assert_eq!(state.get("timer.paused", &world), Some(Value::Bool(true)));

        // Toggle back to false
        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());
        assert_eq!(state.get("timer.paused", &world), Some(Value::Bool(false)));
    }

    // ==================== CONDITIONAL OPERATIONS ====================

    #[test]
    fn test_if_condition_true_branch() {
        let mut world = setup_world();
        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "if".to_string());

        let op = EffectOperation::IfCondition {
            condition: Predicate::GreaterThan {
                field: "score.current".to_string(),
                value: Value::Int(5),
            },
            then: vec![
                EffectOperation::Add {
                    target: "score.current".to_string(),
                    amount: 10,
                }
            ],
            else_: Some(vec![
                EffectOperation::Subtract {
                    target: "score.current".to_string(),
                    amount: 10,
                }
            ]),
        };

        // score.current is 10, which is > 5, so should execute 'then' branch
        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());
        assert_eq!(state.get("score.current", &world), Some(Value::Int(20)));
    }

    #[test]
    fn test_if_condition_false_branch() {
        let mut world = setup_world();
        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "if".to_string());

        let op = EffectOperation::IfCondition {
            condition: Predicate::LessThan {
                field: "score.current".to_string(),
                value: Value::Int(5),
            },
            then: vec![
                EffectOperation::Add {
                    target: "score.current".to_string(),
                    amount: 10,
                }
            ],
            else_: Some(vec![
                EffectOperation::Subtract {
                    target: "score.current".to_string(),
                    amount: 3,
                }
            ]),
        };

        // score.current is 10, which is NOT < 5, so should execute 'else' branch
        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());
        assert_eq!(state.get("score.current", &world), Some(Value::Int(7)));
    }

    #[test]
    fn test_while_operation() {
        let mut world = setup_world();
        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "while".to_string());

        context.set_variable("counter".to_string(), Value::Int(0));

        let op = EffectOperation::While {
            condition: Predicate::LessThan {
                field: "$counter".to_string(),
                value: Value::Int(5),
            },
            operations: vec![
                EffectOperation::SetVariable {
                    name: "counter".to_string(),
                    value: Value::Int(0), // This will be updated
                },
                EffectOperation::Add {
                    target: "score.current".to_string(),
                    amount: 2,
                },
            ],
            max_iterations: Some(10),
        };

        // This should loop, but we need to manually update counter
        // In practice, we'd use GetVariable and arithmetic, but for now test max iterations
        let result = executor.execute_operation(&op, &mut context, &mut state, &mut world);
        assert!(result.is_err()); // Should hit max iterations since counter doesn't increment
    }

    // ==================== VARIABLE OPERATIONS ====================

    #[test]
    fn test_variable_operations() {
        let mut world = setup_world();
        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "vars".to_string());

        // Set variable
        let op = EffectOperation::SetVariable {
            name: "my_value".to_string(),
            value: Value::Int(42),
        };
        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());
        assert_eq!(context.get_variable("my_value"), Some(&Value::Int(42)));

        // Set another variable
        let op = EffectOperation::SetVariable {
            name: "my_string".to_string(),
            value: Value::String("hello".to_string()),
        };
        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());
        assert_eq!(context.get_variable("my_string"), Some(&Value::String("hello".to_string())));
    }

    // ==================== COLLECTION OPERATIONS ====================

    #[test]
    fn test_filter_operation() {
        let mut world = setup_world();
        let mut collections = CollectionManager::new();
        let mut collection = Collection::new();

        for i in 1..=5 {
            let mut item = HashMap::new();
            item.insert("value".to_string(), Value::Int(i * 10));
            collection.append(Value::Object(item));
        }
        collections.set("items", collection);
        world.insert_resource(collections);

        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "filter".to_string());

        let op = EffectOperation::Filter {
            target: "items".to_string(),
            predicate: Predicate::GreaterThan {
                field: "value".to_string(),
                value: Value::Int(25),
            },
        };

        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());

        let collections = world.get_resource::<CollectionManager>().unwrap();
        let collection = collections.get("items").unwrap();
        assert_eq!(collection.len(), 3); // 30, 40, 50
    }

    #[test]
    fn test_remove_operation() {
        let mut world = setup_world();
        let mut collections = CollectionManager::new();
        let mut collection = Collection::new();

        collection.append(Value::Int(1));
        collection.append(Value::Int(2));
        collection.append(Value::Int(3));
        collections.set("numbers", collection);
        world.insert_resource(collections);

        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "remove".to_string());

        let op = EffectOperation::Remove {
            target: "numbers".to_string(),
            count: 2,
            filter: None,
            random: Some(false),
        };

        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());

        let collections = world.get_resource::<CollectionManager>().unwrap();
        let collection = collections.get("numbers").unwrap();
        assert_eq!(collection.len(), 1);

        // Check removed variable was set
        assert!(context.get_variable("removed").is_some());
    }

    #[test]
    fn test_append_operation() {
        let mut world = setup_world();
        let mut collections = CollectionManager::new();
        let collection = Collection::new();
        collections.set("numbers", collection);
        world.insert_resource(collections);

        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "append".to_string());

        let op = EffectOperation::Append {
            target: "numbers".to_string(),
            item: Value::Int(42),
        };

        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());

        let collections = world.get_resource::<CollectionManager>().unwrap();
        let collection = collections.get("numbers").unwrap();
        assert_eq!(collection.len(), 1);
        assert_eq!(collection.get(0), Some(&Value::Int(42)));
    }

    #[test]
    fn test_insert_operation() {
        let mut world = setup_world();
        let mut collections = CollectionManager::new();
        let mut collection = Collection::new();
        collection.append(Value::Int(1));
        collection.append(Value::Int(3));
        collections.set("numbers", collection);
        world.insert_resource(collections);

        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "insert".to_string());

        let op = EffectOperation::Insert {
            target: "numbers".to_string(),
            index: 1,
            item: Value::Int(2),
        };

        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());

        let collections = world.get_resource::<CollectionManager>().unwrap();
        let collection = collections.get("numbers").unwrap();
        assert_eq!(collection.len(), 3);
        assert_eq!(collection.get(0), Some(&Value::Int(1)));
        assert_eq!(collection.get(1), Some(&Value::Int(2)));
        assert_eq!(collection.get(2), Some(&Value::Int(3)));
    }

    #[test]
    fn test_for_each_operation() {
        let mut world = setup_world();
        let mut collections = CollectionManager::new();
        let mut collection = Collection::new();
        collection.append(Value::Int(5));
        collection.append(Value::Int(10));
        collection.append(Value::Int(15));
        collections.set("numbers", collection);
        world.insert_resource(collections);

        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "foreach".to_string());

        let op = EffectOperation::ForEach {
            collection: "numbers".to_string(),
            operations: vec![
                EffectOperation::SetVariable {
                    name: "last_seen".to_string(),
                    value: Value::Int(0), // Will be overwritten by item
                }
            ],
        };

        assert!(executor.execute_operation(&op, &mut context, &mut state, &mut world).is_ok());

        // Should have set item variable to last item (15)
        assert_eq!(context.get_variable("item"), Some(&Value::Int(15)));
        assert_eq!(context.get_variable("index"), Some(&Value::Int(2)));
    }

    // ==================== PREDICATE TESTS ====================

    #[test]
    fn test_predicate_equals() {
        let mut world = setup_world();
        let state = GameState::new();
        let executor = EffectExecutor::new();
        let context = EffectContext::new("test".to_string(), "pred".to_string());

        let pred = Predicate::Equals {
            field: "score.current".to_string(),
            value: Value::Int(10),
        };

        assert!(executor.evaluate_predicate(&pred, &context, &state, &world).unwrap());

        let pred = Predicate::Equals {
            field: "score.current".to_string(),
            value: Value::Int(999),
        };

        assert!(!executor.evaluate_predicate(&pred, &context, &state, &world).unwrap());
    }

    #[test]
    fn test_predicate_not_equals() {
        let mut world = setup_world();
        let state = GameState::new();
        let executor = EffectExecutor::new();
        let context = EffectContext::new("test".to_string(), "pred".to_string());

        let pred = Predicate::NotEquals {
            field: "score.current".to_string(),
            value: Value::Int(999),
        };

        assert!(executor.evaluate_predicate(&pred, &context, &state, &world).unwrap());
    }

    #[test]
    fn test_predicate_and() {
        let mut world = setup_world();
        let state = GameState::new();
        let executor = EffectExecutor::new();
        let context = EffectContext::new("test".to_string(), "pred".to_string());

        let pred = Predicate::And {
            predicates: vec![
                Predicate::GreaterThan {
                    field: "score.current".to_string(),
                    value: Value::Int(5),
                },
                Predicate::LessThan {
                    field: "score.current".to_string(),
                    value: Value::Int(15),
                },
            ],
        };

        assert!(executor.evaluate_predicate(&pred, &context, &state, &world).unwrap());
    }

    #[test]
    fn test_predicate_or() {
        let mut world = setup_world();
        let state = GameState::new();
        let executor = EffectExecutor::new();
        let context = EffectContext::new("test".to_string(), "pred".to_string());

        let pred = Predicate::Or {
            predicates: vec![
                Predicate::Equals {
                    field: "score.current".to_string(),
                    value: Value::Int(999),
                },
                Predicate::Equals {
                    field: "score.current".to_string(),
                    value: Value::Int(10),
                },
            ],
        };

        assert!(executor.evaluate_predicate(&pred, &context, &state, &world).unwrap());
    }

    #[test]
    fn test_predicate_not() {
        let mut world = setup_world();
        let state = GameState::new();
        let executor = EffectExecutor::new();
        let context = EffectContext::new("test".to_string(), "pred".to_string());

        let pred = Predicate::Not {
            predicate: Box::new(Predicate::Equals {
                field: "score.current".to_string(),
                value: Value::Int(999),
            }),
        };

        assert!(executor.evaluate_predicate(&pred, &context, &state, &world).unwrap());
    }

    // ==================== ERROR CASES ====================

    #[test]
    fn test_invalid_path_error() {
        let mut world = setup_world();
        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "error".to_string());

        let op = EffectOperation::Add {
            target: "invalid.path.that.does.not.exist".to_string(),
            amount: 10,
        };

        let result = executor.execute_operation(&op, &mut context, &mut state, &mut world);
        assert!(result.is_err());
    }

    #[test]
    fn test_while_max_iterations() {
        let mut world = setup_world();
        let mut state = GameState::new();
        let mut executor = EffectExecutor::new();
        let mut context = EffectContext::new("test".to_string(), "loop".to_string());

        // Infinite loop that should hit max iterations
        let op = EffectOperation::While {
            condition: Predicate::Equals {
                field: "score.current".to_string(),
                value: Value::Int(10),
            },
            operations: vec![
                EffectOperation::Add {
                    target: "score.passing_grade".to_string(),
                    amount: 1,
                }
            ],
            max_iterations: Some(5),
        };

        let result = executor.execute_operation(&op, &mut context, &mut state, &mut world);
        assert!(result.is_err());

        if let Err(EffectError::MaxIterationsReached) = result {
            // Expected error
        } else {
            panic!("Expected MaxIterationsReached error");
        }
    }
}
