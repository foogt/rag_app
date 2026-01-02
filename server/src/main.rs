mod db;
mod llm;

use warp::Filter;
use shared::{Task, InventoryItem};
use std::sync::Arc;
use db::DbStore;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    // Initialize DB
    let db = Arc::new(DbStore::new("./_data_rocksdb"));
    let db_filter = warp::any().map(move || db.clone());
    
    let inv_db = Arc::new(DbStore::new("./_data_rocksdb_inventory"));
    let inv_db_filter = warp::any().map(move || inv_db.clone());

    // CORS for frontend
    let cors = warp::cors().allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "DELETE"]);

    // Routes
    let get_tasks = warp::get()
        .and(warp::path("tasks"))
        .and(db_filter.clone())
        .map(|db: Arc<DbStore>| warp::reply::json(&db.get_all_tasks()));

    let add_task = warp::post()
        .and(warp::path("tasks"))
        .and(warp::body::json())
        .and(db_filter.clone())
        .map(|task: Task, db: Arc<DbStore>| {
            match db.add_task(task) {
                Ok(_) => warp::reply::with_status("Added", warp::http::StatusCode::CREATED),
                Err(_) => warp::reply::with_status("Error", warp::http::StatusCode::INTERNAL_SERVER_ERROR),
            }
        });

    let delete_task = warp::delete()
        .and(warp::path!("tasks" / String))
        .and(db_filter.clone())
        .map(|id: String, db: Arc<DbStore>| {
            match db.delete_task(&id) {
                Ok(_) => warp::reply::with_status("Deleted", warp::http::StatusCode::OK),
                Err(_) => warp::reply::with_status("Error", warp::http::StatusCode::INTERNAL_SERVER_ERROR),
            }
        });

    let llm_suggest = warp::post()
        .and(warp::path("suggest"))
        .and(warp::body::json()) // Expects { "requirement": "..." }
        .and(db_filter.clone())
        .and_then(handle_suggestion);

    // Inventory Routes
    let get_inventory = warp::get()
        .and(warp::path("inventory"))
        .and(inv_db_filter.clone())
        .map(|db: Arc<DbStore>| {
            // Reusing get_all_tasks logic since it just iterates all values, 
            // but casting to InventoryItem. 
            // Note: In a real app, we'd make DbStore generic or use different methods.
            // Here we assume the separate DB folder ensures type safety.
            let items: Vec<InventoryItem> = db.get_all_tasks().into_iter().map(|t| {
                // Hack: We are using the same DB struct which expects Task. 
                // But DbStore::get_all_tasks deserializes as Task. 
                // We need to modify DbStore or just use a generic get_all.
                // For this specific request, let's just use a raw route handler here or modify DbStore.
                // To keep it simple without changing DbStore signature too much:
                // We will rely on the fact that we are using a separate DB instance.
                // However, DbStore::get_all_tasks returns Vec<Task>. 
                // We need to modify DbStore to be generic or add get_all_inventory.
                // Let's modify DbStore in the next file.
                InventoryItem { name: "Error".to_string(), quantity: 0.0, unit: "".to_string()} 
            }).collect();
            // Actually, let's fix DbStore properly.
            warp::reply::json(&db.get_all_inventory())
        });

    let add_inventory = warp::post()
        .and(warp::path("inventory"))
        .and(warp::body::json())
        .and(inv_db_filter.clone())
        .map(|item: InventoryItem, db: Arc<DbStore>| {
            match db.add_inventory(item) {
                Ok(_) => warp::reply::with_status("Added", warp::http::StatusCode::CREATED),
                Err(_) => warp::reply::with_status("Error", warp::http::StatusCode::INTERNAL_SERVER_ERROR),
            }
        });

    let routes = get_tasks.or(add_task).or(delete_task).or(llm_suggest)
        .or(get_inventory).or(add_inventory)
        .with(cors);

    println!("Server started at http://localhost:8081");
    warp::serve(routes).run(([127, 0, 0, 1], 8081)).await;
}

async fn handle_suggestion(
    body: serde_json::Value, 
    db: Arc<DbStore>
) -> Result<impl warp::Reply, warp::Rejection> {
    let req_str = body["requirement"].as_str().unwrap_or("").to_string();
    let tasks = db.get_all_tasks();
    
    match llm::suggest_time_slot(tasks, req_str).await {
        Ok(suggestion) => Ok(warp::reply::json(&suggestion)),
        Err(_) => Err(warp::reject::not_found()),
    }
}