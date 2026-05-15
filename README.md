# 📝 rustbasic-activitylog

Package **Activity Logging** yang elegan dan terintegrasi penuh untuk framework **RustBasic**, terinspirasi dari **Spatie Laravel ActivityLog**.

Dirancang khusus untuk ekosistem **Axum** dan **SeaORM**, package ini memungkinkan Anda mencatat riwayat aktivitas pengguna, perubahan model, maupun log permintaan HTTP secara otomatis dan terstruktur.

---

## ✨ Fitur Utama

- **Simple Activity Logger**: API yang intuitif untuk mencatat aktivitas secara manual dari Controller atau Service.
- **Auto-Migration & Scaffolding**: Inisialisasi tabel `activity_log` dan model Sea-ORM secara instan.
- **Request Logging Middleware**: Middleware opsional untuk mencatat setiap permintaan HTTP (Method, URI, Status, Durasi).
- **Auto User Tracking**: Middleware secara otomatis mendeteksi dan mencatat `user_id` dari session jika pengguna sudah login.
- **Subject & Causer Tracking**: Melacak siapa yang melakukan aksi (causer) dan pada objek apa aksi tersebut dilakukan (subject).
- **Flexible Properties**: Menyimpan data tambahan dalam format JSON.

---

## 🚀 Panduan Instalasi & Penggunaan

### 1. Instalasi
Tambahkan `rustbasic-activitylog` ke dalam berkas `Cargo.toml` proyek aplikasi Anda:

```toml
[dependencies]
rustbasic-activitylog = "0.0.2"
```

---

### 2. Inisialisasi Otomatis (Magic Scaffolding)
Cukup jalankan build pada proyek Anda, dan `rustbasic-activitylog` akan secara otomatis membuat migrasi dan model yang diperlukan jika belum ada:

```bash
cargo build
```

Perintah ini akan secara otomatis membuat:
- 📂 **Migration**: File migrasi baru di `database/migrations/` untuk tabel `activity_log`.
- 📂 **Models**: File model Sea-ORM di `src/app/models/activity_log.rs`.

Setelah menjalankan perintah di atas, jalankan migrasi database:
```bash
rustbasic migrate
```

---

### 3. Penggunaan Manual di Controller
Anda dapat mencatat aktivitas secara manual dengan `ActivityLogger`:

```rust
use rustbasic_activitylog::ActivityLogger;

pub async fn update_profile(State(state): State<AppState>, req: Request) -> impl IntoResponse {
    // Logika update profil...
    
    // Catat aktivitas
    let _ = ActivityLogger::new(state.db.clone())
        .use_log("user_activity")
        .caused_by("users", user_id)
        .performed_on("users", user_id)
        .with_properties(serde_json::json!({ "field": "email", "old": "a@b.com", "new": "c@d.com" }))
        .log("User memperbarui alamat email")
        .await;

    // ...
}
```

---

### 4. Logging HTTP Secara Otomatis (Middleware)
Daftarkan middleware di `main.rs` atau saat inisialisasi router untuk mencatat semua permintaan masuk:

```rust
use rustbasic_activitylog::{activity_log_middleware, HasDatabase};
use rustbasic_core::server::AppState;

// Implementasikan trait HasDatabase untuk AppState agar middleware bisa mengakses DB
impl HasDatabase for AppState {
    fn db(&self) -> DatabaseConnection {
        self.db.clone()
    }
}

// Tambahkan ke router
let app_router = Router::new()
    .route("/", get(index))
    .layer(axum::middleware::from_fn_with_state(state.clone(), activity_log_middleware));
```

---

## 📄 Lisensi

Package ini dirilis di bawah lisensi **MIT**.
