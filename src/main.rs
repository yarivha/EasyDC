mod schema; // Declare the schema module

use axum::{routing::get, Router, extract::State};
use sqlx::sqlite::{SqlitePool, SqliteConnectOptions};
use std::sync::Arc;
use std::str::FromStr;

struct AppState {
    db: SqlitePool,
    is_setup: bool,
}

#[tokio::main]
async fn main() {
    let db_url = "sqlite://easydc.db";

    // 1. Create the DB file if it is missing
    let opts = SqliteConnectOptions::from_str(db_url)
        .unwrap()
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(opts)
        .await
        .expect("Failed to connect to DB");

    // 2. Initialize the schema using your schema.rs file
    schema::init_tables(&pool)
        .await
        .expect("Failed to initialize database tables");

    // 3. Check for existing admin
    let row = sqlx::query("SELECT count(*) as count FROM web_users")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    use sqlx::Row;
    let count: i64 = row.get("count");

    let app_state = Arc::new(AppState {
        db: pool,
        is_setup: count > 0,
    });

    // 4. Setup Router
    let app = Router::new()
        .route("/", get(dashboard_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("EasyDC running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn dashboard_handler(State(state): State<Arc<AppState>>) -> String {
    if !state.is_setup {
        "Please go to /setup to create the admin account.".to_string()
    } else {
        "Welcome to EasyDC".to_string()
    }
}
