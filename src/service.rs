use rustbasic_core::sql::{self, AnyPool};
use rustbasic_core::serde_json::Value;
use rustbasic_core::chrono::Local;

/// Builder untuk mencatat aktivitas ke tabel `activity_log`.
///
/// # Contoh
/// ```rust,ignore
/// use rustbasic_activitylog::ActivityLogger;
///
/// ActivityLogger::new(db)
///     .use_log("auth")
///     .performed_on("users", user_id)
///     .caused_by("users", actor_id)
///     .with_properties(json!({ "ip": "127.0.0.1" }))
///     .log("User login")
///     .await?;
/// ```
pub struct ActivityLogger {
    db: AnyPool,
    log_name: String,
    subject_type: Option<String>,
    subject_id: Option<i32>,
    causer_type: Option<String>,
    causer_id: Option<i32>,
    properties: Option<Value>,
}

impl ActivityLogger {
    pub fn new(db: AnyPool) -> Self {
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

    /// Simpan log aktivitas ke database.
    pub async fn log(self, description: &str) -> Result<(), sql::Error> {
        let now = Local::now().naive_local().to_string();
        let properties_str = self.properties
            .as_ref()
            .map(|p| p.to_string());

        sql::query(
            "INSERT INTO activity_log \
             (log_name, description, subject_type, subject_id, causer_type, causer_id, properties, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(self.log_name)
        .bind(description)
        .bind(self.subject_type)
        .bind(self.subject_id)
        .bind(self.causer_type)
        .bind(self.causer_id)
        .bind(properties_str)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
