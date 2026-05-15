use crate::entities::activity_log;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde_json::Value;
use chrono::Local;

pub struct ActivityLogger {
    db: DatabaseConnection,
    log_name: String,
    subject_type: Option<String>,
    subject_id: Option<i32>,
    causer_type: Option<String>,
    causer_id: Option<i32>,
    properties: Option<Value>,
}

impl ActivityLogger {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            log_name: "default".to_string(),
            subject_type: None,
            subject_id: None,
            causer_type: None,
            causer_id: None,
            properties: None,
        }
    }

    pub fn use_log(mut self, name: &str) -> Self {
        self.log_name = name.to_string();
        self
    }

    pub fn performed_on(mut self, subject_type: &str, subject_id: i32) -> Self {
        self.subject_type = Some(subject_type.to_string());
        self.subject_id = Some(subject_id);
        self
    }

    pub fn caused_by(mut self, causer_type: &str, causer_id: i32) -> Self {
        self.causer_type = Some(causer_type.to_string());
        self.causer_id = Some(causer_id);
        self
    }

    pub fn with_properties(mut self, properties: Value) -> Self {
        self.properties = Some(properties);
        self
    }

    pub async fn log(self, description: &str) -> Result<activity_log::Model, sea_orm::DbErr> {
        let now = Local::now().naive_local();
        let active_model = activity_log::ActiveModel {
            log_name: Set(Some(self.log_name)),
            description: Set(description.to_string()),
            subject_type: Set(self.subject_type),
            subject_id: Set(self.subject_id),
            causer_type: Set(self.causer_type),
            causer_id: Set(self.causer_id),
            properties: Set(self.properties),
            created_at: Set(Some(now)),
            updated_at: Set(Some(now)),
            ..Default::default()
        };

        active_model.insert(&self.db).await
    }
}
