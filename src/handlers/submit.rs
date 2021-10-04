use actix_session::Session;
use actix_web::http::StatusCode;
use actix_web::{post, web, HttpRequest, HttpResponse, Result};
use awc::Client;
use chrono::offset::Utc;
use chrono::DateTime;
use csrf::{AesGcmCsrfProtection, CsrfProtection};
use ipnetwork::IpNetwork;
use std::convert::TryFrom;
use std::net::AddrParseError;
use std::net::Ipv4Addr;

use crate::abuseipdb::query_abuse;
use crate::config::global::{AC_IPABUSEDB_FAIL, AC_IP_NOT_WHITELISTED};
use crate::config::structs::{InstanceInfo, Platform};
use crate::database::models::InsertableSubmissions;
use crate::database::structs::{Addresses, Submissions, SubmissionsChoices};
use crate::errors::{throw, ErrorKind, InstanceError};
use crate::matching::{calculate_score, UserOpinion};
use crate::DbConn;
use crate::DbPool;

#[derive(Serialize, Deserialize)]
pub struct MatchData {
    pub csrf_token: String,
    pub results: Vec<UserOpinion>,
}

// 2. check CSRF token validity
fn verify_csrf_token(s: &Session, post_csrf: &str) -> Result<bool, InstanceError> {
    let g_instance = InstanceInfo::global();

    // 2.a. attempt to retreive the cookie from user session
    let cookie_csrf = s
        .get::<String>("csrf_cookie")
        .map_err(|e| throw(ErrorKind::WarnGetCsrfCookie, e.to_string()))?;

    // 2.b. check if the cookie exists
    let cookie_csrf = match cookie_csrf {
        Some(v) => v,
        None => {
            return Err(throw(
                ErrorKind::WarnNoCsrfCookie,
                "CSRF cookie is missing".into(),
            ));
        }
    };

    // 2.c. base64 decode the cookie content
    let cookie_csrf = base64::decode_config(cookie_csrf.as_bytes(), base64::URL_SAFE_NO_PAD)
        .map_err(|e| throw(ErrorKind::WarnDecodeCsrfCookie, e.to_string()))?;

    // 2.d. parse csrf token from request
    let post_csrf = base64::decode_config(post_csrf.as_bytes(), base64::URL_SAFE_NO_PAD)
        .map_err(|e| throw(ErrorKind::WarnDecodeCsrfToken, e.to_string()))?;

    // 2.e. use the seed to parse cookies and token
    let seed = AesGcmCsrfProtection::from_key(g_instance.get_csrf_key());
    let cookie_csrf = seed
        .parse_cookie(&cookie_csrf)
        .map_err(|e| throw(ErrorKind::WarnParseCsrfCookie, e.to_string()))?;

    let post_csrf = seed
        .parse_token(&post_csrf)
        .map_err(|e| throw(ErrorKind::WarnParseCsrfToken, e.to_string()))?;

    // 2.f. check the token validity
    Ok(seed.verify_token_pair(&post_csrf, &cookie_csrf))
}

// 4. extract, check and define duration variable
fn check_duration(s: &Session) -> Result<i32, InstanceError> {
    // 4.a. parse cookie
    let poll_start = s
        .get::<DateTime<Utc>>("duration")
        .map_err(|e| throw(ErrorKind::WarnGetDurationCookie, e.to_string()))?;

    // 4.b. check if cookie exists
    let poll_start = match poll_start {
        Some(v) => v,
        None => {
            return Err(throw(
                ErrorKind::WarnNoDurationCookie,
                "duration cookie is missing".into(),
            ));
        }
    };

    // 4.c. calculate duration
    let duration = (Utc::now() - poll_start).num_seconds();

    // 4.d. parse to i32, ensure that it won't overflow
    if duration < 0 || duration > i64::from(i32::MAX) {
        return Err(throw(
            ErrorKind::SevereDurationOverflow,
            format!("duration is {}", duration),
        ));
    }

    // 4.e. cast to fit in db
    Ok(i32::try_from(duration).unwrap_or(i32::MAX))
}

// 5. transform ip in headers to the appropriate format
fn translate_ip(req: &HttpRequest) -> Result<(Ipv4Addr, IpNetwork), InstanceError> {
    // 5.a retreive client IP
    let client_inet = req.connection_info();
    let client_inet = match client_inet.realip_remote_addr() {
        Some(v) => v,
        None => {
            return Err(throw(
                ErrorKind::CritUnknownRemoteAddr,
                format!("{:?}", req.connection_info()),
            ));
        }
    };

    // 5.b convert client IP to IpAddr
    let client_inet: Ipv4Addr = client_inet.parse().map_err(|e: AddrParseError| {
        throw(
            ErrorKind::CritIpv4AddrConvert,
            format!("{} - {}", e.to_string(), client_inet),
        )
    })?;

    // 5.c convert client IP to IpNetwork
    let client_ipnetwork = IpNetwork::new(std::net::IpAddr::V4(client_inet), 32)
        .map_err(|e| throw(ErrorKind::CritIpNetworkConvert, e.to_string()))?;

    Ok((client_inet, client_ipnetwork))
}

// send to DB
fn save_to_db(
    conn: &DbConn,
    user_opinion: &[UserOpinion],
    platform: &Platform,
    asn: u32,
    abuse_code: u16,
    duration: i32,
    client_ipnet: IpNetwork,
) -> Result<(), InstanceError> {
    // inserts in the following order:
    // 1. One row in Addresses
    // 2. One row in Submissions
    // 3. (scrutins_list.len()) rows in SubmissionsChoices

    Addresses::insert(
        Addresses {
            ip: client_ipnet,
            platform_id: platform.id,
        },
        conn,
    )
    .map_err(|e| throw(ErrorKind::CritInsertAddr, e.to_string()))?;

    let submission = Submissions::insert(
        InsertableSubmissions {
            platform_id: platform.id,
            asn: i32::try_from(asn).unwrap_or(i32::MAX),
            abuse_code: i16::try_from(abuse_code).unwrap_or(i16::MAX),
            sent_timestamp: Utc::now().naive_utc().date(),
            duration,
        },
        conn,
    )
    .map_err(|e| throw(ErrorKind::CritInsertSub, e.to_string()))?;

    // add submission_id to UserOpinion
    // to get SubmissionChoices

    let mut choices: Vec<SubmissionsChoices> = Vec::new();

    for opinion in user_opinion {
        choices.push(SubmissionsChoices {
            submission_id: submission.id,
            question_id: opinion.question_id,
            userchoice: opinion.vote.to_db(),
        });
    }

    SubmissionsChoices::insert(choices, conn)
        .map_err(|e| throw(ErrorKind::CritInsertSubChoices, e.to_string()))?;

    Ok(())
}

#[post("/api/submit")]
pub async fn submit(
    dbpool: web::Data<DbPool>,
    web_client: web::Data<Client>,
    match_post: web::Json<MatchData>,
    req: HttpRequest,
    s: Session,
) -> Result<HttpResponse, InstanceError> {
    let g_instance = InstanceInfo::global();

    // connecting to DB
    let conn = dbpool
        .get()
        .map_err(|e| throw(ErrorKind::CritSubmitPool, e.to_string()))?;

    // 1. check if poll is still open
    let platform = match g_instance.check_open(req.connection_info().host()) {
        Some(p) => p,
        None => {
            return Err(throw(
                ErrorKind::WarnSubmitNoPlatform,
                req.connection_info().host().into(),
            ));
        }
    };

    // 2. check CSRF token validity
    if !verify_csrf_token(&s, &match_post.csrf_token)? {
        return Err(throw(
            ErrorKind::WarnCsrfTokenInvalid,
            match_post.csrf_token.clone(),
        ));
    }

    // 3. check answer consistency
    if calculate_score(platform, &match_post.results).is_none() {
        return Err(throw(
            ErrorKind::WarnInvalidAnswer,
            format!("{:?}", match_post.results),
        ));
    }

    // 4. extract, check and define duration variable
    let duration = check_duration(&s)?;

    // 5. check if IP is in Addresses table
    // (needs to transform the ip in headers to the appropriate formats)
    let (client_inet, client_ipnetwork) = translate_ip(&req)?;

    // 5.d. ask DB
    let is_in_db = Addresses::exists(client_ipnetwork, platform.id, &conn)
        .map_err(|e| throw(ErrorKind::CritCheckAddress, e.to_string()))?;

    // if the address is in db, safely returns "already voted"
    if is_in_db {
        return Err(throw(ErrorKind::InfoAlreadyPolled, client_inet.to_string()));
    }

    // 6. get AS from local DB
    let client_u32 = u32::from(client_inet);
    let client_asn = g_instance.find_asn(client_u32);

    // 6.a. display warning if asn is 0 (shouldn't happen)
    if client_asn == 0 {
        eprintln!("warn: AS number of {} is 0", client_inet);
    }

    // 7. check if AS is in the blocklist (doesn't apply to AS=0)
    if client_asn != 0 {
        let test_asnbl = g_instance.check_asn_blacklist(client_asn);
        if test_asnbl > 0 {
            save_to_db(
                &conn,
                &match_post.results,
                platform,
                client_asn,
                test_asnbl,
                duration,
                client_ipnetwork,
            )?;
            return Err(throw(
                ErrorKind::InfoAsnBlocklisted,
                client_inet.to_string(),
            ));
        }
    }

    // 8. check if IP is in the whitelist
    if !g_instance.check_ipv4_whitelist(client_u32) {
        save_to_db(
            &conn,
            &match_post.results,
            platform,
            client_asn,
            AC_IP_NOT_WHITELISTED,
            duration,
            client_ipnetwork,
        )?;
        return Err(throw(
            ErrorKind::InfoNotWhitelisted,
            client_inet.to_string(),
        ));
    }

    // 9. check if IP is in the blocklist
    let test_ipbl = g_instance.check_ipv4_blacklist(client_u32);
    if test_ipbl > 0 {
        save_to_db(
            &conn,
            &match_post.results,
            platform,
            client_asn,
            test_ipbl,
            duration,
            client_ipnetwork,
        )?;
        return Err(throw(ErrorKind::InfoIpBlocklisted, client_inet.to_string()));
    }

    // 10. ask ipabusedb for confidence score
    let ipdb_response = query_abuse(
        &web_client,
        client_inet.to_string(),
        &g_instance.config.abuseipdb_api_key,
    )
    .await;

    let ipdb_response = match ipdb_response {
        Ok(r) => r,
        Err(e) => {
            save_to_db(
                &conn,
                &match_post.results,
                platform,
                client_asn,
                AC_IPABUSEDB_FAIL,
                duration,
                client_ipnetwork,
            )?;
            return Err(throw(ErrorKind::CritAbuseIpDbApiFail, e));
        }
    };

    save_to_db(
        &conn,
        &match_post.results,
        platform,
        client_asn,
        ipdb_response,
        duration,
        client_ipnetwork,
    )?;

    // 11. Save to database
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/plain; charset=utf-8")
        .body("OK"))
}
