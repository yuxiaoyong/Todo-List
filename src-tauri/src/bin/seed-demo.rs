use std::fs;
use std::path::PathBuf;

use todo_list_lib::db::open_standalone;
use todo_list_lib::db::repositories::{clear_all_user_data, import_all_data, DataImportResult};
use todo_list_lib::demo_seed::build_demo_snapshot;

fn default_db_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").expect("APPDATA environment variable is required");
    PathBuf::from(appdata)
        .join("com.tx.todo-list")
        .join("todos.db")
}

fn reset_database(db_path: &PathBuf) -> DataImportResult {
    let conn = open_standalone(db_path).expect("failed to open database");
    clear_all_user_data(&conn).expect("failed to clear user data");
    import_all_data(&conn, build_demo_snapshot()).expect("failed to import demo data")
}

fn main() {
    let db_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_db_path);

    if !db_path.exists() {
        eprintln!(
            "Database not found at {} — creating a new database first.",
            db_path.display()
        );
        open_standalone(&db_path).expect("failed to initialize database");
    }

    let snapshot = build_demo_snapshot();
    let demo_json_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../demo/demo-data.json");
    if let Some(parent) = demo_json_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(&snapshot) {
        let _ = fs::write(&demo_json_path, json);
        println!("Exported snapshot to {}", demo_json_path.display());
    }

    let result = reset_database(&db_path);
    println!("Demo data loaded into {}", db_path.display());
    println!(
        "Imported {} tasks ({} categories, {} tags, {} kanban columns created).",
        result.todos_imported,
        result.categories_created,
        result.tags_created,
        result.kanban_columns_created
    );
}
