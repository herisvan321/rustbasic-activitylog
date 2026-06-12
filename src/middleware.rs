use rustbasic_core::router::Response;
use rustbasic_core::middleware::Next;
use rustbasic_core::requests::Request;
use rustbasic_core::sql::AnyPool;
use crate::service::ActivityLogger;
use std::time::Instant;
use rustbasic_core::serde_json::json;

/// Trait to extract database pool from your app state.
///
/// `rustbasic-core`'s `AppState` sudah mengimplementasikan trait ini
/// secara otomatis karena memiliki field `db: AnyPool`.
///
/// Jika Anda menggunakan custom state, implement trait ini:
/// ```rust,ignore
/// use rustbasic_activitylog::HasDatabase;
/// use rustbasic_core::sql::AnyPool;
///
/// #[derive(Clone)]
/// struct MyState { db: AnyPool }
///
/// impl HasDatabase for MyState {
///     fn db(&self) -> AnyPool { self.db.clone() }
/// }
/// ```
pub trait HasDatabase {
    fn db(&self) -> AnyPool;
}

/// Implementasi default untuk `rustbasic_core::AppState`.
impl HasDatabase for rustbasic_core::AppState {
    fn db(&self) -> AnyPool {
        self.db.clone()
    }
}

/// RustBasic middleware untuk mencatat setiap HTTP request ke tabel `activity_log`.
///
/// Mencatat method, URI, HTTP status, dan durasi. Jika `user_id` (i32) tersimpan
/// di session, akan dicatat sebagai causer aktivitas.
///
/// # Penggunaan
/// ```rust,ignore
/// use rustbasic_core::router::Router;
/// use rustbasic_core::middleware::from_fn;
/// use rustbasic_activitylog::activity_log_middleware;
///
/// let router = Router::new()
///     .route("/", get(home))
///     .layer(from_fn(activity_log_middleware));
/// ```
pub async fn activity_log_middleware(
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = req.method.to_string();
    let uri = req.path.clone();
    let db: AnyPool = req.state.db.clone();

    // Ambil user_id dari session sebelum req dikonsumsi
    let user_id: Option<i32> = req.session.get("user_id");

    let response = next.run(req).await;

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

    if let Some(uid) = user_id {
        logger = logger.caused_by("users", uid);
    }

    let _ = logger.log(&description).await;

    response
}
