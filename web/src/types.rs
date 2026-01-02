use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct TaskPreset {
    pub duration_minutes: i64,
    pub materials: HashMap<String, String>,
}