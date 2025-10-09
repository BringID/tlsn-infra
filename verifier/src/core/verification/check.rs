use serde::{Deserialize, Serialize};
use serde_json::{Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum Check {
    LenGte(usize),
    Gte(i64),
    Lte(i64),
    Eq(i64),
    Contains(String),
    Custom,
    Any,
}

pub trait CheckableValue {
    fn check_against(&self, check: &Check) -> bool;
}

impl CheckableValue for i64 {
    fn check_against(&self, check: &Check) -> bool {
        match check {
            Check::Gte(threshold) => *self >= *threshold,
            Check::Lte(threshold) => *self <= *threshold,
            Check::Eq(threshold) => *self == *threshold,
            Check::Any | Check::Custom => true,
            _ => false,
        }
    }
}

impl CheckableValue for &str {
    fn check_against(&self, check: &Check) -> bool {
        match check {
            Check::Any | Check::Custom => true,
            Check::Contains(value) => {
                self.contains(value)
            },
            _ => false,
        }
    }
}

impl CheckableValue for &Vec<Value> {
    fn check_against(&self, check: &Check) -> bool {
        match check {
            Check::LenGte(value) => self.len() >= *value,
            Check::Any | Check::Custom => true,
            _ => false,
        }
    }
}