mod auth;
mod db;
mod handlers;
mod models;

use std::sync::Arc;

use axum::{middleware, routing::{get, post}, Router};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use tera::Tera;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub tera: Arc<Tera>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let opts = SqliteConnectOptions::from_str("sqlite://easydc.db")
        .unwrap()
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(opts)
        .await
        .expect("Failed to connect to database");

    db::init_tables(&pool).await.expect("Failed to initialize database");

    let tera = Tera::new("templates/**/*").expect("Failed to load templates");

    let state = AppState {
        db: pool,
        tera: Arc::new(tera),
    };

    let public = Router::new()
        .route("/setup", get(handlers::setup::get_setup).post(handlers::setup::post_setup))
        .route("/login", get(handlers::auth_handlers::get_login).post(handlers::auth_handlers::post_login));

    let protected = Router::new()
        .route("/", get(handlers::servers::dashboard))
        .route("/logout", post(handlers::auth_handlers::logout))
        .route("/servers/new", post(handlers::servers::create_server))
        .route("/servers/:id", get(handlers::servers::server_detail))
        .route("/servers/:id/edit", post(handlers::servers::update_server))
        .route("/servers/:id/delete", post(handlers::servers::delete_server))
        .route("/servers/:id/users", get(handlers::ldap_mgmt::users))
        .route("/servers/:id/groups", get(handlers::ldap_mgmt::groups))
        .route("/servers/:id/computers", get(handlers::ldap_mgmt::computers))
        .route("/servers/:id/dns", get(handlers::ldap_mgmt::dns))
        .route("/servers/:id/gpo", get(handlers::ldap_mgmt::gpo))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("EasyDC running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
