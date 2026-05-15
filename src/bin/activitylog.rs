use std::fs::{self, OpenOptions};
use std::io::Write;
use chrono::Local;
use colored::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "install" => make_activitylog_scaffolding(),
        _ => {
            println!("{} {}", "❌ Error: Perintah tidak dikenal:".red().bold(), args[1].yellow());
            print_help();
        }
    }
}

fn print_help() {
    println!("\n{}", "📝 RustBasic Activity Log CLI".magenta().bold());
    println!("{}", "==============================".magenta());
    println!("{}", "Usage:".bold());
    println!("  rustbasic-activitylog install    {}", "Scaffold Activity Log table and model into your project".dimmed());
    println!();
}

pub fn make_activitylog_scaffolding() {
    println!("\n{} {}", "🚀".bold(), "Menyiapkan scaffolding Activity Log...".magenta().bold());

    // Cek apakah kita berada di root project RustBasic
    if !std::path::Path::new("Cargo.toml").exists() {
        println!("{}", "❌ Error: File Cargo.toml tidak ditemukan. Pastikan Anda menjalankan perintah ini di root proyek.".red().bold());
        return;
    }

    // 1. Buat Migration
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let migration_name = format!("m{}_create_activity_log_table", timestamp);
    let migration_path = format!("database/migrations/{}.rs", migration_name);

    // Pastikan folder migrations ada
    fs::create_dir_all("database/migrations").ok();

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

    fs::write(&migration_path, migration_template).expect("Gagal membuat migration Activity Log");
    update_migration_mod_rs(&migration_name);
    println!("   {} Migration dibuat: {}", "📦".blue(), migration_path.cyan());

    // 2. Buat Model
    fs::create_dir_all("src/app/models").ok();

    let model_name = "activity_log";
    let table_name = "activity_log";
    let file_path = format!("src/app/models/{}.rs", model_name);
    
    if !std::path::Path::new(&file_path).exists() {
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
        
        fs::write(&file_path, model_template).expect("Gagal membuat model Activity Log");
        update_model_mod_rs("ActivityLog", model_name);
        println!("   {} Model dibuat: {}", "📄".blue(), file_path.cyan());
    }

    println!("\n{} {}", "✅".green(), "Scaffolding Activity Log berhasil diselesaikan!".green().bold());
    println!("{} Jalankan '{}' untuk menerapkan tabel ke database.", "💡".yellow(), "rustbasic migrate".cyan());
}

fn update_migration_mod_rs(mod_name: &str) {
    let mod_path = "database/migrations/mod.rs";
    if !std::path::Path::new(mod_path).exists() { return; }

    let mut content = fs::read_to_string(mod_path).unwrap_or_default();

    // Tambahkan mod declaration
    if !content.contains(&format!("pub mod {};", mod_name)) {
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&format!("pub mod {};\n", mod_name));
    }

    // Tambahkan ke list migrations
    let search_pattern = "fn migrations() -> Vec<Box<dyn MigrationTrait>> {";
    if let Some(pos) = content.find(search_pattern) {
        if let Some(insert_pos) = content[pos..].find("        ]") {
            let absolute_insert_pos = pos + insert_pos;
            content.insert_str(absolute_insert_pos, &format!("            Box::new({}::Migration),\n", mod_name));
        }
    }

    fs::write(mod_path, content).ok();
}

fn update_model_mod_rs(class_name: &str, snake_name: &str) {
    let mod_path = "src/app/models/mod.rs";
    if !std::path::Path::new(mod_path).exists() { return; }

    let content = fs::read_to_string(mod_path).unwrap_or_default();

    let mod_line = format!("pub mod {};", snake_name);
    if content.contains(&mod_line) {
        return;
    }

    let mut file = OpenOptions::new()
        .append(true)
        .open(mod_path)
        .expect("Gagal membuka models/mod.rs");

    writeln!(file, "{}", mod_line).ok();
    writeln!(file, "pub use {}::Entity as {};", snake_name, class_name).ok();
}
