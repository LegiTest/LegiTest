use actix_session::Session;
use actix_web::{get, HttpRequest, HttpResponse, Result};
use chrono::offset::Utc;
use csrf::{AesGcmCsrfProtection, CsrfProtection};

use crate::config::global::CSRF_TTL;
use crate::config::structs::InstanceInfo;
use crate::errors::{throw, ErrorKind, InstanceError};

#[derive(Serialize, Deserialize)]
pub struct InitData {
    pub csrf_token: String,
}

// if the given platform (poll) is closed, sends an empty string.
#[get("/api/csrftoken")]
pub async fn csrftoken(s: Session, req: HttpRequest) -> Result<HttpResponse, InstanceError> {
    let g_instance = InstanceInfo::global();

    // if the platform doesn't exist or if the poll has ended
    if g_instance
        .check_open(req.connection_info().host())
        .is_none()
    {
        // returns an empty body.
        // Client interprets this as "poll is closed".
        return Ok(HttpResponse::Ok().json(InitData {
            csrf_token: String::new(),
        }));
    }

    let seed = AesGcmCsrfProtection::from_key(g_instance.get_csrf_key());

    // generates a new token pair
    let (csrf_token, csrf_cookie) = seed
        .generate_token_pair(None, CSRF_TTL)
        .map_err(|e| throw(ErrorKind::FatalGenerateCsrf, e.to_string()))?;

    s.insert(
        "csrf_cookie",
        &base64::encode_config(&csrf_cookie.value(), base64::URL_SAFE_NO_PAD),
    )
    .map_err(|e| throw(ErrorKind::WarnSetCsrfCookie, e.to_string()))?;

    s.insert("duration", Utc::now())
        .map_err(|e| throw(ErrorKind::WarnSetDurationCookie, e.to_string()))?;

    Ok(HttpResponse::Ok().json(InitData {
        csrf_token: base64::encode_config(csrf_token.value(), base64::URL_SAFE_NO_PAD),
    }))
}
