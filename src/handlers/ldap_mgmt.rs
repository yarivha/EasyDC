use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Redirect, Response},
};
use tera::Context;

use crate::{models::Server, AppState};

async fn get_server(state: &AppState, id: i64) -> Option<Server> {
    sqlx::query_as::<_, Server>("SELECT * FROM servers WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .unwrap_or(None)
}

async fn render_section(state: AppState, id: i64, section: &str) -> Response {
    match get_server(&state, id).await {
        None => Redirect::to("/").into_response(),
        Some(server) => {
            let mut ctx = Context::new();
            ctx.insert("server", &server);
            ctx.insert("section", section);
            Html(state.tera.render("ldap_mgmt.html", &ctx).unwrap_or_default()).into_response()
        }
    }
}

pub async fn users(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    render_section(state, id, "User Management").await
}

pub async fn groups(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    render_section(state, id, "Group Management").await
}

pub async fn computers(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    render_section(state, id, "Computer Management").await
}

pub async fn dns(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    render_section(state, id, "DNS Management").await
}

pub async fn gpo(State(state): State<AppState>, Path(id): Path<i64>) -> Response {
    render_section(state, id, "GPO Management").await
}
