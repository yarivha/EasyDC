use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};

use crate::AppState;

pub fn extract_session_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|s| s.trim().strip_prefix("session=").map(String::from))
}

pub async fn require_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if let Some(token) = extract_session_cookie(request.headers()) {
        let valid = sqlx::query("SELECT id FROM sessions WHERE token = ?")
            .bind(&token)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None)
            .is_some();
        if valid {
            return next.run(request).await;
        }
    }
    Redirect::to("/login").into_response()
}
