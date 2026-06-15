use axum::{
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};

use crate::{db, AppState};

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

// ── current user extractor ──────────────────────────────────────────────────────

/// The username behind the request's session, for audit attribution.
/// Always succeeds (routes are already guarded by `require_auth`); falls back
/// to "unknown" if the session has no recorded username (legacy sessions).
pub struct CurrentUser(pub String);

#[axum::async_trait]
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let username = match extract_session_cookie(&parts.headers) {
            Some(token) => db::username_for_token(&state.db, &token)
                .await
                .unwrap_or_else(|| "unknown".to_string()),
            None => "unknown".to_string(),
        };
        Ok(CurrentUser(username))
    }
}
