use std::fs::{self, OpenOptions};
use std::io::Write;
use rustbasic_core::chrono::Local;
use rustbasic_core::colored::Colorize;
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
    println!("  rustbasic-activitylog install    {}", "Scaffold Activity Log table and migration into your project".dimmed());
    println!();
}

pub fn make_activitylog_scaffolding() {
    println!("\n{} {}", "🚀".bold(), "Menyiapkan scaffolding Activity Log...".magenta().bold());

    // Cek apakah kita berada di root project RustBasic
    if !std::path::Path::new("Cargo.toml").exists() {
        println!("{}", "❌ Error: File Cargo.toml tidak ditemukan. Pastikan Anda menjalankan perintah ini di root proyek.".red().bold());
        return;
    }

    // 1. Buat Migration (menggunakan rustbasic-core Schema API)
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let migration_name = format!("m{}_create_activity_log_table", timestamp);
    let migration_path = format!("database/migrations/{}.rs", migration_name);

    // Pastikan folder migrations ada
    fs::create_dir_all("database/migrations").ok();

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

    fs::write(&migration_path, migration_template).expect("Gagal membuat migration Activity Log");
    update_migration_mod_rs(&migration_name);
    println!("   {} Migration dibuat: {}", "📦".blue(), migration_path.cyan());

    // 2. Buat Model (plain struct menggunakan rustbasic_core model! macro)
    fs::create_dir_all("src/app/models").ok();

    let model_file = "src/app/models/activity_log.rs";

    if !std::path::Path::new(model_file).exists() {
        let model_template =
r#"use rustbasic_core::model;

model! {
    table: "activity_log",
    Model {
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
        fs::write(model_file, model_template).expect("Gagal membuat model Activity Log");
        update_model_mod_rs("activity_log");
        println!("   {} Model dibuat: {}", "📄".blue(), model_file.cyan());
    }

    println!("\n{} {}", "✅".green(), "Scaffolding Activity Log berhasil diselesaikan!".green().bold());
    println!("{} Jalankan '{}' untuk menerapkan tabel ke database.", "💡".yellow(), "rustbasic migrate".cyan());
    println!("{} Tambahkan middleware di routes Anda:", "💡".yellow());
    println!("   {}", "use rustbasic_activitylog::activity_log_middleware;".cyan());
    println!("   {}", ".layer(from_fn(activity_log_middleware))".cyan());
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
    if let Some(pos) = content.find(search_pattern)
        && let Some(insert_offset) = content[pos..].find("        ]") {
        let absolute_insert_pos = pos + insert_offset;
        content.insert_str(absolute_insert_pos, &format!("            Box::new({}::Migration),\n", mod_name));
    }

    fs::write(mod_path, content).ok();
}

fn update_model_mod_rs(snake_name: &str) {
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
    writeln!(file, "pub use {}::Model as ActivityLog;", snake_name).ok();
}
