use rustbasic_core::model;
use rustbasic_core::serde_json::Value;

model! {
    table: "activity_logs",
    timestamps: true,
    fillable: [log_name, description, subject_type, subject_id, causer_type, causer_id, properties],
    guarded: [],
    Model {
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
}
