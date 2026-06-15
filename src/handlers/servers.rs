use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use tera::Context;

use crate::{auth::CurrentUser, db, models::Server, AppState};

// ── audit log view ─────────────────────────────────────────────────────────────

pub async fn audit(State(state): State<AppState>) -> impl IntoResponse {
    let entries = db::recent_audit(&state.db, 500).await.unwrap_or_default();
    let mut ctx = Context::new();
    ctx.insert("entries", &entries);
    Html(state.tera.render("audit.html", &ctx).unwrap_or_default())
}

pub async fn dashboard(State(state): State<AppState>) -> impl IntoResponse {
    let servers = sqlx::query_as::<_, Server>("SELECT * FROM servers ORDER BY name")
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    let mut ctx = Context::new();
    ctx.insert("servers", &servers);
    Html(state.tera.render("dashboard.html", &ctx).unwrap_or_default())
}

pub async fn server_detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let server = sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .unwrap();

    match server {
        None => Redirect::to("/").into_response(),
        Some(server) => {
            let mut ctx = Context::new();
            ctx.insert("server", &server);
            Html(state.tera.render("server_detail.html", &ctx).unwrap_or_default()).into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct ServerForm {
    pub name: String,
    pub ldap_url: String,
    pub bind_dn: String,
    pub bind_password: String,
    pub skip_tls: Option<String>,
}

pub async fn create_server(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Form(form): Form<ServerForm>,
) -> impl IntoResponse {
    let skip_tls = form.skip_tls.is_some();
    let res = sqlx::query(
        "INSERT INTO servers (name, ldap_url, bind_dn, bind_password, skip_tls) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&form.name)
    .bind(&form.ldap_url)
    .bind(&form.bind_dn)
    .bind(&form.bind_password)
    .bind(skip_tls)
    .execute(&state.db)
    .await;

    let result = res.map(|_| ()).map_err(|e| e.to_string());
    db::log_action(&state.db, &actor, "server.create", &form.name, None, &result).await;

    Redirect::to("/")
}

#[derive(Deserialize)]
pub struct UpdateServerForm {
    pub name: String,
    pub ldap_url: String,
    pub bind_dn: String,
    pub bind_password: String,
    pub skip_tls: Option<String>,
}

pub async fn update_server(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<i64>,
    Form(form): Form<UpdateServerForm>,
) -> impl IntoResponse {
    let skip_tls = form.skip_tls.is_some();

    let res = if form.bind_password.is_empty() {
        sqlx::query(
            "UPDATE servers SET name=?, ldap_url=?, bind_dn=?, skip_tls=? WHERE id=?",
        )
        .bind(&form.name)
        .bind(&form.ldap_url)
        .bind(&form.bind_dn)
        .bind(skip_tls)
        .bind(id)
        .execute(&state.db)
        .await
    } else {
        sqlx::query(
            "UPDATE servers SET name=?, ldap_url=?, bind_dn=?, bind_password=?, skip_tls=? WHERE id=?",
        )
        .bind(&form.name)
        .bind(&form.ldap_url)
        .bind(&form.bind_dn)
        .bind(&form.bind_password)
        .bind(skip_tls)
        .bind(id)
        .execute(&state.db)
        .await
    };

    let result = res.map(|_| ()).map_err(|e| e.to_string());
    db::log_action(&state.db, &actor, "server.update", &form.name, Some(id), &result).await;

    Redirect::to("/")
}

pub async fn delete_server(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let res = sqlx::query("DELETE FROM servers WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await;

    let result = res.map(|_| ()).map_err(|e| e.to_string());
    db::log_action(&state.db, &actor, "server.delete", &id.to_string(), Some(id), &result).await;

    Redirect::to("/")
}
