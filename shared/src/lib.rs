use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: String, // Unique ID (UUID)
    pub user_id: String,
    pub operation_id: String,
    pub expected_duration_minutes: i64,
    pub start_time: DateTime<Utc>,
    pub actual_start_time: Option<DateTime<Utc>>,
    pub actual_duration_minutes: Option<i64>,
    pub materials: HashMap<String, String>, // Dictionary of materials
}

impl Task {
    pub fn new(user_id: String, op_id: String, start: DateTime<Utc>, duration: i64, materials: HashMap<String, String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            operation_id: op_id,
            expected_duration_minutes: duration,
            start_time: start,
            actual_start_time: None,
            actual_duration_minutes: None,
            materials,
        }
    }
}

// Request payload for LLM scheduling
#[derive(Serialize, Deserialize)]
pub struct ScheduleRequest {
    pub current_tasks: Vec<Task>,
    pub new_operation_description: String,
}

// Response from LLM
#[derive(Serialize, Deserialize)]
pub struct ScheduleSuggestion {
    pub suggested_start_time: DateTime<Utc>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InventoryItem {
    pub name: String,
    pub quantity: f64,
    pub unit: String,
}