use axum::{
    extract::State,
    http::{header, HeaderValue},
    response::{Html, IntoResponse, Redirect},
    Form,
};
use serde::Deserialize;
use tera::Context;
use uuid::Uuid;

use crate::{auth::extract_session_cookie, db, AppState};

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

pub async fn get_login(State(state): State<AppState>) -> impl IntoResponse {
    if !db::has_users(&state.db).await.unwrap_or(true) {
        return Redirect::to("/setup").into_response();
    }
    let ctx = Context::new();
    Html(state.tera.render("login.html", &ctx).unwrap_or_default()).into_response()
}

pub async fn post_login(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    use sqlx::Row;

    let row = sqlx::query("SELECT password_hash FROM users WHERE username = ?")
        .bind(&form.username)
        .fetch_optional(&state.db)
        .await
        .unwrap();

    let authenticated = if let Some(row) = row {
        let hash: String = row.get("password_hash");
        bcrypt::verify(&form.password, &hash).unwrap_or(false)
    } else {
        false
    };

    if !authenticated {
        let mut ctx = Context::new();
        ctx.insert("error", "Invalid username or password");
        return Html(state.tera.render("login.html", &ctx).unwrap_or_default()).into_response();
    }

    let token = Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    sqlx::query("INSERT INTO sessions (token, created_at) VALUES (?, ?)")
        .bind(&token)
        .bind(now)
        .execute(&state.db)
        .await
        .unwrap();

    let cookie = format!("session={}; Path=/; HttpOnly; SameSite=Strict", token);
    let mut response = Redirect::to("/").into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    response
}

pub async fn logout(State(state): State<AppState>, req: axum::extract::Request) -> impl IntoResponse {
    if let Some(token) = extract_session_cookie(req.headers()) {
        let _ = sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(&token)
            .execute(&state.db)
            .await;
    }
    let mut response = Redirect::to("/login").into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static("session=; Path=/; HttpOnly; Max-Age=0"),
    );
    response
}
