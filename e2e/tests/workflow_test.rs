use thirtyfour::prelude::*;
use std::time::Duration;

#[tokio::test]
async fn test_add_task_flow() -> anyhow::Result<()> {
    // 1. Connect to WebDriver
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", caps).await?;

    // 2. Navigate to the App
    driver.goto("http://localhost:8080").await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 3. Locate Form Elements by ID (More Robust)
    let user_input = driver.find(By::Id("input-user-id")).await?;
    let op_input = driver.find(By::Id("input-op-id")).await?;
    let dur_hour = driver.find(By::Id("input-dur-hour")).await?;
    let add_task_btn = driver.find(By::Id("btn-add-task")).await?;

    // 4. Fill in the Form
    user_input.send_keys("TestUser_01").await?;
    op_input.send_keys("TestOp_A").await?;

    // Clear the default "1" value in duration before typing
    dur_hour.clear().await?; 
    dur_hour.send_keys("2").await?;

    // 5. Add a Material
    // (We didn't add an ID to this button in the snippet above, so keep using XPath or add an ID)
    let add_material_btn = driver.find(By::XPath("//button[contains(text(), '+ Add Material')]")).await?;
    add_material_btn.click().await?;
    
    // Find the new material inputs
    let mat_name_inputs = driver.find_all(By::Css("table tbody tr td:first-child input")).await?;
    if let Some(last_input) = mat_name_inputs.last() {
        last_input.send_keys("Wood").await?;
    }

    // 6. Submit the Task
    add_task_btn.click().await?;

    // 7. Verification / Assertion
    tokio::time::sleep(Duration::from_secs(1)).await; 
    
    let user_val = user_input.value().await?.unwrap_or_default();
    assert!(user_val.is_empty(), "Form should clear after adding task");

    let client = reqwest::Client::new();
    let resp = client.get("http://localhost:8081/tasks").send().await?;
    let body = resp.text().await?;
    
    assert!(body.contains("TestUser_01"), "Backend should have the new task");
    assert!(body.contains("TestOp_A"), "Backend should have the new operation");

    driver.quit().await?;

    Ok(())
}