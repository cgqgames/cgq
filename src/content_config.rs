//! Recursive configuration loader.
//!
//! Walks a directory, parses every `*.toml` and `*.json` file as a partial
//! configuration tree, and deep-merges them into a single `toml::Value`.
//!
//! Merge rules: tables merge recursively; arrays and scalars replace.
//! Duplicate leaves are last-write-wins in lexicographic path order —
//! if two files both define `cards.foo`, that's a content bug, not a
//! mechanism we try to detect.

use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::Deserialize;
use std::path::Path;
use toml::Value;
use walkdir::WalkDir;

use crate::card_templates::YamlCardEffect;
use crate::components::{CardType, Permanence, QuestionOption};
use crate::ui_config::UiConfig;

/// Typed view over the merged configuration tree.
#[derive(Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub cards: IndexMap<String, CardConfig>,
    #[serde(default)]
    pub questions: IndexMap<String, QuestionConfig>,
    #[serde(default)]
    pub game: GameConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct GameConfig {
    pub title: Option<String>,
    pub passing_grade: Option<i32>,
    pub timer_seconds: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CardConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub card_type: CardType,
    pub permanence: Permanence,
    pub vote_requirement: usize,
    pub cost: i32,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub effects: Vec<YamlCardEffect>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuestionConfig {
    pub text: String,
    pub options: Vec<QuestionOption>,
    pub points: i32,
    pub explanation: Option<String>,
    pub source: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Load and merge every `*.toml` / `*.json` under `root` into a typed `AppConfig`.
pub fn load_app_config(root: &Path) -> Result<AppConfig> {
    let merged = load_config_dir(root)?;
    let app: AppConfig = merged
        .try_into()
        .context("Configuration tree failed typed validation")?;
    Ok(app)
}

fn load_config_dir(root: &Path) -> Result<Value> {
    let mut paths: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            let ext = e.path().extension().and_then(|s| s.to_str())?;
            matches!(ext, "toml" | "json").then(|| e.into_path())
        })
        .collect();
    paths.sort();

    let mut merged = Value::Table(Default::default());
    for path in paths {
        let fragment = parse_file(&path)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        deep_merge(&mut merged, fragment);
    }
    Ok(merged)
}

fn parse_file(path: &Path) -> Result<Value> {
    let content = std::fs::read_to_string(path)?;
    match path.extension().and_then(|s| s.to_str()) {
        Some("toml") => Ok(toml::from_str(&content)?),
        Some("json") => {
            let json: serde_json::Value = serde_json::from_str(&content)?;
            json_to_toml(json)
        }
        _ => unreachable!("Filtered in load_config_dir"),
    }
}

fn json_to_toml(value: serde_json::Value) -> Result<Value> {
    Ok(match value {
        serde_json::Value::Null => {
            anyhow::bail!("JSON null is not representable in TOML")
        }
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                anyhow::bail!("Unrepresentable JSON number: {}", n)
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(json_to_toml)
                .collect::<Result<Vec<_>>>()?,
        ),
        serde_json::Value::Object(obj) => {
            let mut table = toml::map::Map::new();
            for (k, v) in obj {
                table.insert(k, json_to_toml(v)?);
            }
            Value::Table(table)
        }
    })
}

fn deep_merge(into: &mut Value, from: Value) {
    match (into, from) {
        (Value::Table(a), Value::Table(b)) => {
            for (k, v) in b {
                match a.get_mut(&k) {
                    Some(existing) => deep_merge(existing, v),
                    None => {
                        a.insert(k, v);
                    }
                }
            }
        }
        (slot, other) => *slot = other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Value {
        toml::from_str(s).unwrap()
    }

    #[test]
    fn tables_merge() {
        let mut a = parse("[cards.foo]\nname = \"Foo\"\n");
        let b = parse("[cards.foo]\ncost = 5\n");
        deep_merge(&mut a, b);
        let expected = parse("[cards.foo]\nname = \"Foo\"\ncost = 5\n");
        assert_eq!(a, expected);
    }

    #[test]
    fn arrays_replace() {
        let mut a = parse("tags = [\"one\"]\n");
        let b = parse("tags = [\"two\"]\n");
        deep_merge(&mut a, b);
        assert_eq!(a, parse("tags = [\"two\"]\n"));
    }

    #[test]
    fn scalars_replace() {
        let mut a = parse("cost = 5\n");
        let b = parse("cost = 10\n");
        deep_merge(&mut a, b);
        assert_eq!(a, parse("cost = 10\n"));
    }

    #[test]
    fn deep_table_merge() {
        let mut a = parse("[cards.foo.effects.one]\ntype = \"add_time\"\n");
        let b = parse("[cards.foo.effects.two]\ntype = \"add_points\"\n");
        deep_merge(&mut a, b);
        let expected = parse(
            "[cards.foo.effects.one]\ntype = \"add_time\"\n\
             [cards.foo.effects.two]\ntype = \"add_points\"\n",
        );
        assert_eq!(a, expected);
    }

    #[test]
    fn type_mismatch_replaces() {
        let mut a = parse("foo = [1, 2, 3]\n");
        let b = parse("[foo]\nname = \"bar\"\n");
        deep_merge(&mut a, b);
        assert_eq!(a, parse("[foo]\nname = \"bar\"\n"));
    }
}
