# 📝 rustbasic-activitylog

Package **Activity Logging** yang elegan, cepat, dan terintegrasi penuh untuk framework **RustBasic**.

Dirancang khusus untuk ekosistem **RustBasic**, package ini memungkinkan Anda mencatat riwayat aktivitas pengguna, perubahan data, maupun log permintaan HTTP secara otomatis dan terstruktur langsung ke database.

---

## ✨ Fitur Utama

- ⚡ **Simple Activity Logger**: API berbasis builder pattern yang intuitif untuk mencatat aktivitas secara manual dari controller atau service.
- 🏗️ **Automatic Scaffolding (Magic Build)**: Secara otomatis membuat file migrasi database dan model RustBasic/Sea-ORM untuk tabel `activity_log` saat proyek dikompilasi (`cargo build`).
- 🌐 **Request Logging Middleware**: Middleware untuk mencatat otomatis setiap request HTTP (Method, Path, HTTP Status, dan Durasi eksekusi).
- 👤 **Automatic User Tracking**: Mendeteksi dan mengaitkan `user_id` secara otomatis dari session jika pengguna telah login.
- 🔍 **Subject & Causer Tracking**: Melacak aktor yang melakukan aksi (`causer`) dan objek/target yang dikenai aksi (`subject`).
- 📊 **Flexible JSON Properties**: Menyimpan metadata atau detail data tambahan dalam format JSON.

---

## 🛠️ Struktur Tabel Database

Secara default, scaffolding otomatis akan membuat tabel `activity_log` dengan struktur berikut:

| Nama Kolom | Tipe Data | Deskripsi |
| :--- | :--- | :--- |
| `id` | `INTEGER` (PK) | Auto-increment ID |
| `log_name` | `VARCHAR` (Nullable) | Kategori/nama log (e.g., `"default"`, `"auth"`, `"http_request"`) |
| `description` | `TEXT` (Not Null) | Penjelasan atau pesan aktivitas |
| `subject_type` | `VARCHAR` (Nullable) | Jenis model/objek target (e.g., `"users"`, `"products"`) |
| `subject_id` | `INTEGER` (Nullable) | ID unik dari objek target |
| `causer_type` | `VARCHAR` (Nullable) | Jenis pelaku aksi (e.g., `"users"`) |
| `causer_id` | `INTEGER` (Nullable) | ID unik dari pelaku aksi |
| `properties` | `TEXT` (Nullable) | Detail metadata tambahan yang disimpan dalam format string JSON |
| `created_at` | `DATETIME` | Waktu log dibuat |
| `updated_at` | `DATETIME` | Waktu log diperbarui |

---

## 🚀 Panduan Instalasi & Penggunaan

### 1. Instalasi

Tambahkan `rustbasic-activitylog` ke dalam berkas `Cargo.toml` proyek aplikasi RustBasic Anda:

```toml
[dependencies]
rustbasic-activitylog = "0.0"
```

---

### 2. Inisialisasi & Scaffolding Otomatis

Cukup jalankan kompilasi proyek Anda, dan build script akan mendeteksi serta menghasilkan file migrasi dan model secara otomatis jika belum ada:

```bash
cargo build
```

**Hasil Scaffolding:**
1. **Migration**: File migrasi baru di `database/migrations/mYYYYMMDD_HHMMSS_create_activity_log_table.rs`.
2. **Model**: File model Sea-ORM di `src/app/models/activity_log.rs`.

Setelah scaffolding selesai, jalankan migrasi database menggunakan CLI RustBasic:

```bash
rustbasic migrate
```

---

### 3. Penggunaan Manual di Controller / Service

Gunakan `ActivityLogger` untuk mencatat aktivitas secara manual dari handler atau service Anda:

```rust
use rustbasic_activitylog::ActivityLogger;
use serde_json::json;

pub async fn update_profile(state: AppState, user_id: i32) -> Result<(), Box<dyn std::error::Error>> {
    // Logika bisnis update profil...

    // Catat aktivitas ke database
    ActivityLogger::new(state.db.clone())
        .use_log("user_management")
        .caused_by("users", user_id)
        .performed_on("users", user_id)
        .with_properties(json!({
            "ip_address": "192.168.1.1",
            "changes": {
                "email": {
                    "old": "old-email@example.com",
                    "new": "new-email@example.com"
                }
            }
        }))
        .log("User memperbarui alamat email utama")
        .await?;

    Ok(())
}
```

---

### 4. Logging HTTP Request Secara Otomatis (Middleware)

Anda dapat menggunakan `activity_log_middleware` untuk mencatat log setiap request HTTP masuk secara otomatis.

#### A. Konfigurasi AppState
Middleware membutuhkan akses ke database pool. Pastikan `AppState` Anda mengimplementasikan trait `HasDatabase`. Jika Anda menggunakan standard `rustbasic_core::AppState`, trait ini sudah terimplementasi secara otomatis.

Jika Anda menggunakan custom state:
```rust
use rustbasic_activitylog::HasDatabase;
use rustbasic_core::sqlx::AnyPool;

#[derive(Clone)]
pub struct MyCustomState {
    pub db: AnyPool,
}

impl HasDatabase for MyCustomState {
    fn db(&self) -> AnyPool {
        self.db.clone()
    }
}
```

#### B. Pendaftaran Middleware ke Router
Daftarkan middleware saat mendefinisikan router Anda di `src/routes.rs` atau `main.rs`:

```rust
use rustbasic_core::router::Router;
use rustbasic_core::middleware::from_fn;
use rustbasic_activitylog::activity_log_middleware;

pub fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/dashboard", get(dashboard_handler))
        .layer(from_fn(activity_log_middleware))
}
```

*Catatan: Middleware secara otomatis mencoba membaca `user_id` (tipe `i32`) dari data session aktif untuk diisikan sebagai `causer_id` pada log.*

---

## 📄 Lisensi

Package ini dilisensikan di bawah lisensi **MIT**.
