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
r#"use sea_orm_migration::prelude::*;
use async_trait::async_trait;

#[derive(Iden)]
pub enum ActivityLog {{
    Table, Id, LogName, Description, SubjectType, SubjectId, CauserType, CauserId, Properties, CreatedAt, UpdatedAt,
}}

#[derive(Iden)]
pub struct Migration;

impl MigrationName for Migration {{
    fn name(&self) -> &str {{
        "{migration_name}"
    }}
}}

#[async_trait]
impl MigrationTrait for Migration {{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager.create_table(
            Table::create()
                .table(ActivityLog::Table)
                .if_not_exists()
                .col(ColumnDef::new(ActivityLog::Id).integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(ActivityLog::LogName).string().null())
                .col(ColumnDef::new(ActivityLog::Description).text().not_null())
                .col(ColumnDef::new(ActivityLog::SubjectType).string().null())
                .col(ColumnDef::new(ActivityLog::SubjectId).integer().null())
                .col(ColumnDef::new(ActivityLog::CauserType).string().null())
                .col(ColumnDef::new(ActivityLog::CauserId).integer().null())
                .col(ColumnDef::new(ActivityLog::Properties).json().null())
                .col(ColumnDef::new(ActivityLog::CreatedAt).date_time().default(Expr::current_timestamp()))
                .col(ColumnDef::new(ActivityLog::UpdatedAt).date_time().default(Expr::current_timestamp()))
                .to_owned(),
        ).await?;

        Ok(())
    }}

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager.drop_table(Table::drop().table(ActivityLog::Table).to_owned()).await?;
        Ok(())
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
    let table_name = "activity_log";
    let file_path = models_dir.join(format!("{}.rs", model_name));
    
    if !file_path.exists() {
        let model_template = format!(
r#"use rustbasic_core::sea_orm::entity::prelude::*;
use serde::{{Deserialize, Serialize}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "{table_name}")]
pub struct Model {{
    #[sea_orm(primary_key)]
    pub id: i32,
    pub log_name: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub subject_type: Option<String>,
    pub subject_id: Option<i32>,
    pub causer_type: Option<String>,
    pub causer_id: Option<i32>,
    pub properties: Option<Json>,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
}}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {{}}

impl ActiveModelBehavior for ActiveModel {{}}
"#, table_name = table_name);

        if fs::write(&file_path, model_template).is_ok() {
            update_model_mod_rs(&project_root, "ActivityLog", model_name);
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
    if let Some(pos) = content.find(search_pattern) {
        if let Some(insert_pos) = content[pos..].find("        ]") {
            content.insert_str(pos + insert_pos, &format!("            Box::new({}::Migration),\n", mod_name));
        }
    }

    fs::write(mod_path, content).ok();
}

fn update_model_mod_rs(project_root: &std::path::Path, class_name: &str, snake_name: &str) {
    let mod_path = project_root.join("src/app/models/mod.rs");
    if !mod_path.exists() { return; }

    let content = fs::read_to_string(&mod_path).unwrap_or_default();
    if content.contains(&format!("pub mod {};", snake_name)) {
        return;
    }

    let mut file = fs::OpenOptions::new().append(true).open(mod_path).unwrap();
    use std::io::Write;
    writeln!(file, "pub mod {};", snake_name).ok();
    writeln!(file, "pub use {}::Entity as {};", snake_name, class_name).ok();
}
