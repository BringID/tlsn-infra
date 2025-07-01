use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum Check {
    Gte(i64),
    Lte(i64),
    Eq(i64),
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
            Check::Any => true
        }
    }
}

impl CheckableValue for &str {
    fn check_against(&self, check: &Check) -> bool {
        match check {
            Check::Any => true,
            _ => false,
        }
    }
}