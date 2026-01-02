use rocksdb::{DB, Options, IteratorMode};
use shared::{Task, InventoryItem};
use std::sync::Arc;

// Wrapper for thread-safe DB access
pub struct DbStore {
    db: Arc<DB>,
}

impl DbStore {
    pub fn new(path: &str) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path).expect("Failed to open RocksDB");
        Self { db: Arc::new(db) }
    }

    pub fn add_task(&self, task: Task) -> Result<(), String> {
        let key = task.id.as_bytes();
        let value = serde_json::to_string(&task).map_err(|e| e.to_string())?;
        self.db.put(key, value.as_bytes()).map_err(|e| e.to_string())
    }

    pub fn add_inventory(&self, item: InventoryItem) -> Result<(), String> {
        let key = item.name.as_bytes();
        let value = serde_json::to_string(&item).map_err(|e| e.to_string())?;
        self.db.put(key, value.as_bytes()).map_err(|e| e.to_string())
    }

    pub fn delete_task(&self, id: &str) -> Result<(), String> {
        self.db.delete(id.as_bytes()).map_err(|e| e.to_string())
    }

    pub fn get_all_tasks(&self) -> Vec<Task> {
        let mut tasks = Vec::new();
        let iter = self.db.iterator(IteratorMode::Start);
        for item in iter {
            if let Ok((_, value)) = item {
                if let Ok(task) = serde_json::from_slice::<Task>(&value) {
                    tasks.push(task);
                }
            }
        }
        tasks
    }

    pub fn get_all_inventory(&self) -> Vec<InventoryItem> {
        let mut items = Vec::new();
        let iter = self.db.iterator(IteratorMode::Start);
        for item in iter {
            if let Ok((_, value)) = item {
                if let Ok(inv) = serde_json::from_slice::<InventoryItem>(&value) {
                    items.push(inv);
                }
            }
        }
        items
    }
}