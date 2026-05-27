use std::fs;
use std::path::PathBuf;
use std::env;

fn main() {
    // Hanya jalankan jika kita tidak sedang dalam proses rilis atau docs
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    // Ambil direktori kerja saat ini.
    let project_root = match env::var("PWD") {
        Ok(pwd) => PathBuf::from(pwd),
        Err(_) => match env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return,
        },
    };

    // Pastikan ini adalah proyek RustBasic (minimal ada Cargo.toml)
    if !project_root.join("Cargo.toml").exists() {
        return;
    }

    // JANGAN lakukan scaffolding jika kita sedang men-debug paket ini sendiri
    if project_root.join("src/bin/activitylog.rs").exists() {
        return;
    }

    println!("cargo:warning=📝 rustbasic-activitylog: Menyiapkan scaffolding otomatis...");

    // 1. Buat Migration
    let migrations_dir = project_root.join("database/migrations");
    fs::create_dir_all(&migrations_dir).ok();

    // Cek apakah sudah ada migrasi (hindari duplikasi)
    let existing_migrations = fs::read_dir(&migrations_dir)
        .map(|dir| dir.filter_map(|e| e.ok()).any(|e| e.file_name().to_string_lossy().contains("create_activity_log_table")))
        .unwrap_or(false);

    if !existing_migrations {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let migration_name = format!("m{}_create_activity_log_table", timestamp);
        let migration_path = migrations_dir.join(format!("{}.rs", migration_name));

        let migration_template = format!(
r#"use rustbasic_core::{{Schema, SchemaManager, MigrationTrait, DbErr}};
use rustbasic_core::async_trait;

pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {{
    fn name(&self) -> &str {{
        "{migration_name}"
    }}

    async fn up<'a>(&self, manager: &'a SchemaManager<'a>) -> Result<(), DbErr> {{
        Schema::create(manager, "activity_log", |table| {{
            table.id();
            table.string("log_name").nullable();
            table.text("description").not_null();
            table.string("subject_type").nullable();
            table.integer("subject_id").nullable();
            table.string("causer_type").nullable();
            table.integer("causer_id").nullable();
            table.text("properties").nullable(); // JSON disimpan sebagai TEXT
            table.timestamps();
        }}).await
    }}

    async fn down<'a>(&self, manager: &'a SchemaManager<'a>) -> Result<(), DbErr> {{
        Schema::drop(manager, "activity_log").await
    }}
}}
"#, migration_name = migration_name);

        if fs::write(&migration_path, migration_template).is_ok() {
            update_migration_mod_rs(&project_root, &migration_name);
        }
    }

    // 2. Buat Model
    let models_dir = project_root.join("src/app/models");
    fs::create_dir_all(&models_dir).ok();

    let model_name = "activity_log";
    let file_path = models_dir.join(format!("{}.rs", model_name));

    if !file_path.exists() {
        let model_template =
r#"use rustbasic_core::model;

model! {
    table: "activity_log",
    ActivityLog {
        pub id: i32,
        pub log_name: Option<String>,
        pub description: String,
        pub subject_type: Option<String>,
        pub subject_id: Option<i32>,
        pub causer_type: Option<String>,
        pub causer_id: Option<i32>,
        pub properties: Option<String>, // JSON string
    }
}
"#;

        if fs::write(&file_path, model_template).is_ok() {
            update_model_mod_rs(&project_root, model_name);
        }
    }
}

fn update_migration_mod_rs(project_root: &std::path::Path, mod_name: &str) {
    let mod_path = project_root.join("database/migrations/mod.rs");
    if !mod_path.exists() { return; }

    let mut content = fs::read_to_string(&mod_path).unwrap_or_default();

    if !content.contains(&format!("pub mod {};", mod_name)) {
        content.push_str(&format!("\npub mod {};\n", mod_name));
    }

    let search_pattern = "fn migrations() -> Vec<Box<dyn MigrationTrait>> {";
    if let Some(pos) = content.find(search_pattern)
        && let Some(insert_pos) = content[pos..].find("        ]") {
        content.insert_str(pos + insert_pos, &format!("            Box::new({}::Migration),\n", mod_name));
    }

    fs::write(mod_path, content).ok();
}

fn update_model_mod_rs(project_root: &std::path::Path, snake_name: &str) {
    let mod_path = project_root.join("src/app/models/mod.rs");
    if !mod_path.exists() { return; }

    let content = fs::read_to_string(&mod_path).unwrap_or_default();
    if content.contains(&format!("pub mod {};", snake_name)) {
        return;
    }

    let mut file = fs::OpenOptions::new().append(true).open(mod_path).unwrap();
    use std::io::Write;
    writeln!(file, "pub mod {};", snake_name).ok();
}
