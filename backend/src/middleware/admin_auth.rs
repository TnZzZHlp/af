use axum::{extract::State, http::Request, middleware::Next, response::IntoResponse};
use uuid::Uuid;

use crate::{error::AppError, services::auth, state::AppState};

#[derive(Clone, Copy, Debug)]
pub struct AdminUserId(pub Uuid);

pub async fn admin_auth_middleware(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
    next: Next,
) -> impl IntoResponse {
    let token = auth::extract_api_key(req.headers());
    let Some(token) = token else {
        return AppError::Unauthorized.into_response();
    };

    let claims = match auth::verify_jwt(&token, &state.jwt_secret) {
        Ok(claims) => claims,
        Err(_) => return AppError::Unauthorized.into_response(),
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return AppError::Unauthorized.into_response(),
    };

    let mut req = req;
    req.extensions_mut().insert(AdminUserId(user_id));

    next.run(req).await
}

impl axum::extract::FromRequestParts<AppState> for AdminUserId {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &AppState,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        let id = parts.extensions.get::<AdminUserId>().copied();
        async move { id.ok_or(AppError::Unauthorized) }
    }
}
