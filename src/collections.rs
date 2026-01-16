// TODO: This module is planned for future collection system implementation
#![allow(dead_code)]

use bevy::prelude::*;
use crate::effect::{Value, Predicate};
use std::collections::HashMap;

/// Represents a collection that can be manipulated by effects
#[derive(Debug, Clone)]
pub struct Collection {
    pub items: Vec<Value>,
}

impl Collection {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn from_vec(items: Vec<Value>) -> Self {
        Self { items }
    }

    /// Filter items by predicate
    pub fn filter<F>(&mut self, predicate: F)
    where
        F: Fn(&Value) -> bool,
    {
        self.items.retain(predicate);
    }

    /// Remove N items (optionally random)
    pub fn remove(&mut self, count: usize, random: bool) -> Vec<Value> {
        let mut removed = Vec::new();
        let count = count.min(self.items.len());

        if random {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();

            for _ in 0..count {
                if !self.items.is_empty() {
                    let idx = (0..self.items.len()).collect::<Vec<_>>()
                        .choose(&mut rng)
                        .copied()
                        .unwrap_or(0);
                    removed.push(self.items.remove(idx));
                }
            }
        } else {
            for _ in 0..count {
                if let Some(item) = self.items.pop() {
                    removed.push(item);
                }
            }
        }

        removed
    }

    /// Append item to collection
    pub fn append(&mut self, item: Value) {
        self.items.push(item);
    }

    /// Insert item at index
    pub fn insert(&mut self, index: usize, item: Value) {
        if index <= self.items.len() {
            self.items.insert(index, item);
        } else {
            self.items.push(item);
        }
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get item by index
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.items.get(index)
    }

    /// Get mutable item by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        self.items.get_mut(index)
    }

    /// Iterate over items
    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.items.iter()
    }

    /// Convert to Vec
    pub fn to_vec(&self) -> Vec<Value> {
        self.items.clone()
    }
}

impl Default for Collection {
    fn default() -> Self {
        Self::new()
    }
}

/// Manager for collections in game state
#[derive(Resource, Default)]
pub struct CollectionManager {
    collections: HashMap<String, Collection>,
}

impl CollectionManager {
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
        }
    }

    /// Get collection by path
    pub fn get(&self, path: &str) -> Option<&Collection> {
        self.collections.get(path)
    }

    /// Get mutable collection by path
    pub fn get_mut(&mut self, path: &str) -> Option<&mut Collection> {
        self.collections.get_mut(path)
    }

    /// Create or get collection
    pub fn get_or_create(&mut self, path: &str) -> &mut Collection {
        self.collections
            .entry(path.to_string())
            .or_default()
    }

    /// Set collection
    pub fn set(&mut self, path: &str, collection: Collection) {
        self.collections.insert(path.to_string(), collection);
    }

    /// Remove collection
    pub fn remove(&mut self, path: &str) -> Option<Collection> {
        self.collections.remove(path)
    }

    /// Check if collection exists
    pub fn contains(&self, path: &str) -> bool {
        self.collections.contains_key(path)
    }

    /// Clear all collections
    pub fn clear(&mut self) {
        self.collections.clear();
    }

    /// Get collection length
    pub fn len(&self, path: &str) -> usize {
        self.get(path).map(|c| c.len()).unwrap_or(0)
    }

    /// Check if any collections exist
    pub fn is_empty(&self) -> bool {
        self.collections.is_empty()
    }
}

/// Helper to evaluate predicate against a value
pub fn evaluate_item_predicate(
    predicate: &Predicate,
    item: &Value,
) -> bool {
    match predicate {
        Predicate::Equals { field, value } => {
            if let Some(item_value) = get_field_value(item, field) {
                &item_value == value
            } else {
                false
            }
        }

        Predicate::NotEquals { field, value } => {
            if let Some(item_value) = get_field_value(item, field) {
                &item_value != value
            } else {
                true
            }
        }

        Predicate::GreaterThan { field, value } => {
            if let Some(item_value) = get_field_value(item, field) {
                compare_values(&item_value, value, |a, b| a > b)
            } else {
                false
            }
        }

        Predicate::LessThan { field, value } => {
            if let Some(item_value) = get_field_value(item, field) {
                compare_values(&item_value, value, |a, b| a < b)
            } else {
                false
            }
        }

        Predicate::GreaterOrEqual { field, value } => {
            if let Some(item_value) = get_field_value(item, field) {
                compare_values(&item_value, value, |a, b| a >= b)
            } else {
                false
            }
        }

        Predicate::LessOrEqual { field, value } => {
            if let Some(item_value) = get_field_value(item, field) {
                compare_values(&item_value, value, |a, b| a <= b)
            } else {
                false
            }
        }

        Predicate::HasTag { tag } => {
            if let Some(Value::Array(tags)) = get_field_value(item, "tags") {
                tags.iter().any(|t| {
                    if let Value::String(s) = t {
                        s == tag
                    } else {
                        false
                    }
                })
            } else {
                false
            }
        }

        Predicate::IsType { card_type } => {
            if let Some(Value::String(item_type)) = get_field_value(item, "type") {
                &item_type == card_type
            } else {
                false
            }
        }

        Predicate::Contains { field, value } => {
            if let Some(item_value) = get_field_value(item, field) {
                match &item_value {
                    Value::Array(arr) => arr.contains(value),
                    Value::String(s) => {
                        if let Value::String(search) = value {
                            s.contains(search)
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            } else {
                false
            }
        }

        Predicate::And { predicates } => {
            predicates.iter().all(|p| evaluate_item_predicate(p, item))
        }

        Predicate::Or { predicates } => {
            predicates.iter().any(|p| evaluate_item_predicate(p, item))
        }

        Predicate::Not { predicate } => {
            !evaluate_item_predicate(predicate, item)
        }

        Predicate::Expression { expr: _ } => {
            warn!("Expression predicates not yet implemented");
            false
        }
    }
}

/// Get field value from an item (supports nested paths)
fn get_field_value(item: &Value, field: &str) -> Option<Value> {
    match item {
        Value::Object(_map) => {
            // Support nested paths like "user.name"
            let parts: Vec<&str> = field.split('.').collect();
            let mut current = item;

            for part in parts {
                if let Value::Object(obj) = current {
                    current = obj.get(part)?;
                } else {
                    return None;
                }
            }

            Some(current.clone())
        }
        _ => None,
    }
}

/// Compare numeric values
fn compare_values<F>(a: &Value, b: &Value, op: F) -> bool
where
    F: Fn(f64, f64) -> bool,
{
    let a_num = match a {
        Value::Int(v) => *v as f64,
        Value::Float(v) => *v as f64,
        _ => return false,
    };

    let b_num = match b {
        Value::Int(v) => *v as f64,
        Value::Float(v) => *v as f64,
        _ => return false,
    };

    op(a_num, b_num)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_basic_operations() {
        let mut collection = Collection::new();

        // Test append
        collection.append(Value::Int(1));
        collection.append(Value::Int(2));
        collection.append(Value::Int(3));
        assert_eq!(collection.len(), 3);

        // Test get
        assert_eq!(collection.get(0), Some(&Value::Int(1)));
        assert_eq!(collection.get(1), Some(&Value::Int(2)));

        // Test insert
        collection.insert(1, Value::Int(10));
        assert_eq!(collection.len(), 4);
        assert_eq!(collection.get(1), Some(&Value::Int(10)));
    }

    #[test]
    fn test_collection_remove() {
        let mut collection = Collection::from_vec(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ]);

        // Remove 2 items (non-random, from end)
        let removed = collection.remove(2, false);
        assert_eq!(removed.len(), 2);
        assert_eq!(collection.len(), 3);
    }

    #[test]
    fn test_collection_filter() {
        let mut collection = Collection::from_vec(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
            Value::Int(5),
        ]);

        // Filter even numbers
        collection.filter(|v| {
            if let Value::Int(n) = v {
                n % 2 == 0
            } else {
                false
            }
        });

        assert_eq!(collection.len(), 2);
        assert_eq!(collection.get(0), Some(&Value::Int(2)));
        assert_eq!(collection.get(1), Some(&Value::Int(4)));
    }

    #[test]
    fn test_predicate_evaluation() {
        let item = Value::Object({
            let mut map = HashMap::new();
            map.insert("id".to_string(), Value::String("test".to_string()));
            map.insert("value".to_string(), Value::Int(42));
            map.insert("tags".to_string(), Value::Array(vec![
                Value::String("tag1".to_string()),
                Value::String("tag2".to_string()),
            ]));
            map
        });

        // Test Equals
        let pred = Predicate::Equals {
            field: "value".to_string(),
            value: Value::Int(42),
        };
        assert!(evaluate_item_predicate(&pred, &item));

        // Test HasTag
        let pred = Predicate::HasTag {
            tag: "tag1".to_string(),
        };
        assert!(evaluate_item_predicate(&pred, &item));

        // Test And
        let pred = Predicate::And {
            predicates: vec![
                Predicate::Equals {
                    field: "value".to_string(),
                    value: Value::Int(42),
                },
                Predicate::HasTag {
                    tag: "tag2".to_string(),
                },
            ],
        };
        assert!(evaluate_item_predicate(&pred, &item));
    }

    #[test]
    fn test_collection_manager() {
        let mut manager = CollectionManager::new();

        // Create collection
        let collection = manager.get_or_create("test.items");
        collection.append(Value::Int(1));
        collection.append(Value::Int(2));

        assert_eq!(manager.len("test.items"), 2);
        assert!(manager.contains("test.items"));

        // Remove collection
        let removed = manager.remove("test.items");
        assert!(removed.is_some());
        assert!(!manager.contains("test.items"));
    }
}
