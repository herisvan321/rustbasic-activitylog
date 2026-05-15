use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::Response,
};
use rustbasic_core::server::AppState;
use rustbasic_core::axum_session::Session;
use rustbasic_core::session_manager::RustBasicSessionStore;
use sea_orm::DatabaseConnection;
use crate::service::ActivityLogger;
use std::time::Instant;
use serde_json::json;

/// Middleware to log HTTP requests and responses.
/// Requires the AppState to have a `db` field of type `DatabaseConnection`.
pub async fn activity_log_middleware<S>(
    state: State<S>,
    session: Session<RustBasicSessionStore>,
    request: Request<Body>,
    next: Next,
) -> Response 
where 
    S: HasDatabase + Clone + Send + Sync + 'static
{
    let db = state.db();
    let start = Instant::now();
    let method = request.method().to_string();
    let uri = request.uri().to_string();
    
    // Proceed to next middleware/handler
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status = response.status().as_u16();
    
    let description = format!("{} {}", method, uri);
    let properties = json!({
        "method": method,
        "uri": uri,
        "status": status,
        "duration_ms": duration.as_millis(),
    });

    let mut logger = ActivityLogger::new(db)
        .use_log("http_request")
        .with_properties(properties);
    
    // Coba ambil user_id dari session jika ada
    if let Some(user_id) = session.get::<i32>("user_id") {
        logger = logger.caused_by("users", user_id);
    }
    
    let _ = logger.log(&description).await;
    
    response
}

/// Trait to extract DatabaseConnection from State.
pub trait HasDatabase {
    fn db(&self) -> DatabaseConnection;
}

// Blanket implementation for RustBasic AppState
impl HasDatabase for AppState {
    fn db(&self) -> DatabaseConnection {
        self.db.clone()
    }
}

// Implement for any type that has a db field, or let the user implement it.
// For RustBasic, we can provide a default implementation if we know the structure.
// But better to keep it decoupled.
