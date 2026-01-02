use shared::{Task, ScheduleSuggestion};
use serde_json::json;
use std::env;

pub async fn suggest_time_slot(tasks: Vec<Task>, requirement: String) -> Result<ScheduleSuggestion, String> {
    let api_key = env::var("GOOGLE_API_KEY").map_err(|_| "API Key not set")?;
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", 
        api_key
    );

    // Prompt engineering
    let task_summary = serde_json::to_string(&tasks).unwrap_or_default();
    let prompt = format!(
        "You are a scheduling assistant. Here are existing tasks: {}. 
        User wants to schedule: '{}'. 
        Suggest a start time (ISO 8601 format) that does not overlap and a brief reason. 
        Return ONLY valid JSON format: {{ \"suggested_start_time\": \"...\", \"reason\": \"...\" }}",
        task_summary, requirement
    );

    let client = reqwest::Client::new();
    let res = client.post(&url)
        .json(&json!({ "contents": [{ "parts": [{ "text": prompt }] }] }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
    
    // Extract JSON from Gemini response (simplified parsing)
    let text = body["candidates"][0]["content"]["parts"][0]["text"]
        .as_str().ok_or("No content")?;
    
    // Clean markdown code blocks if present
    let clean_json = text.replace("```json", "").replace("```", "");
    
    serde_json::from_str(&clean_json).map_err(|e| format!("Parse error: {}", e))
}