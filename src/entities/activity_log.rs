use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Model activity log yang tersimpan di database.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: i32,
    pub log_name: Option<String>,
    pub description: String,
    pub subject_type: Option<String>,
    pub subject_id: Option<i32>,
    pub causer_type: Option<String>,
    pub causer_id: Option<i32>,
    pub properties: Option<Value>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
