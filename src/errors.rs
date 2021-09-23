use actix_web::error::Error;
use actix_web::error::JsonPayloadError;
use actix_web::web::HttpRequest;
use actix_web::{error, http::StatusCode, HttpResponse};
use std::fmt;

use crate::config::structs::InstanceInfo;
use crate::debug;

pub fn throw(error_kind: ErrorKind, error_msg: String) -> InstanceError {
    InstanceError {
        kind: error_kind,
        msg: error_msg,
    }
}

pub fn invalid_form(json_e: JsonPayloadError, _: &HttpRequest) -> Error {
    throw(ErrorKind::SevereInvalidForm, format!("{:?}", json_e)).into()
}

/*
 * Error types:
 * Fatal: Something that *shouldn't* happen, maybe due to a logic flaw
 * Crit: Mostly DB failures or very unusual input handling failures
 * Severe: Due to client input, but shouldn't happen unless they try very hard
 * Warn: Mostly due to client input or kiddies poking around
 * Info: Not actually an error but rather a redirection
 */
#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    SevereInvalidForm,      // form answer deserialization error
    FatalGenerateCsrf,      // failed to generate csrf token
    WarnSetCsrfCookie,      // cookies disabled client side
    WarnSetDurationCookie,  // cookies disabled client side
    WarnGetCsrfCookie,      // failed to get the cookie from user session
    WarnNoCsrfCookie,       // csrf cookie is missing
    WarnDecodeCsrfCookie,   // failed to base64-decode csrf cookie
    WarnDecodeCsrfToken,    // failed to base64-deocde csrf token
    WarnParseCsrfCookie,    // failed to parse csrf cookie with csrf key
    WarnParseCsrfToken,     // failed to parse csrf token with csrf key
    WarnGetDurationCookie,  // failed to get duration cookie from user session
    WarnNoDurationCookie,   // duration cookie is missing
    SevereDurationOverflow, // duration is overflowing an i32
    CritUnknownRemoteAddr,  // failed to get client's remote address
    CritIpv4AddrConvert,    // failed to convert inet to ipv4addr
    CritIpNetworkConvert,   // failed to convert ipv4addr to ipnetwork
    CritInsertAddr,         // failed to insert a row in Addresses
    CritInsertSub,          // failed to insert a submission after addresses
    CritInsertSubChoices,   // failed to insert subchoices after submission
    CritSubmitPool,         // failed to get DB pool in submit handler
    WarnSubmitNoPlatform,   // answer submitted for a closed/unknown platform
    WarnCsrfTokenInvalid,   // invalid csrf token
    WarnInvalidAnswer,      // calculate_score failed to validate the submission
    CritCheckAddress,       // failed to contact DB to check if an address exists
    InfoAlreadyPolled,      // IP already participated to the test
    InfoAsnBlocklisted,     // IP is in the ASN blocklist
    InfoNotWhitelisted,     // IP isn't in the IP whitelist
    InfoIpBlocklisted,      // IP is in the IP blocklist
    CritAbuseIpDbApiFail,   // AbuseIPdb's API returned an unexpected value
    InfoPollClosed,         // can't generate report because poll is closed
    CritReportCountValid,   // couldn't count valid submissions on report gen
    InfoNotEnoughSubs,      // not enough submissions to generate a report
    CritReportCountTotal,   // couldn't count total submissions on report gen
    CritReportGetValid,     // couldn't get the valid submissions from DB
    FatalUnmatchedChoice,   // subc doesn't match with a Choice, DB corrupted?
    FatalMissingSubmission, // can't find the group when grouping submissions
    FatalInvalidScoreCalc,  // DB-sourced scores are invalid, shouldn't happen
    FatalInvalidScoreConv,  // fails to convert integers/floats for affinity
    CritResultsPool,        // failed to get the DB pool in results route
    WarnResultsNoPlatform,  // results asked for a nonexistent platform
    CritResultsGetLatest,   // failed to get the latest result from db
    CritResultsGetGroups,   // failed to get results groups from db
    CritResultsGetValid,    // couldn't get the valid submissions from DB (2)
    CritReportPool,         // failed to get the pool in internal report route
    CritReportInsert,       // failed to insert the report in DB
    CritReportInsertGroups, // failed to insert the resultsgroups in DB
}

impl ErrorKind {
    // returns true for items that must display only if debug mode is enabled
    // will return 200 OK for those items if config hide_errors == true
    pub fn is_info(&self) -> bool {
        matches!(
            self,
            ErrorKind::InfoNotWhitelisted
                | ErrorKind::InfoNotEnoughSubs
                | ErrorKind::InfoPollClosed
                | ErrorKind::InfoAlreadyPolled
                | ErrorKind::InfoIpBlocklisted
                | ErrorKind::InfoAsnBlocklisted
        )
    }

    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            ErrorKind::WarnSetCsrfCookie
                | ErrorKind::WarnSetDurationCookie
                | ErrorKind::WarnGetCsrfCookie
                | ErrorKind::WarnNoCsrfCookie
                | ErrorKind::WarnDecodeCsrfCookie
                | ErrorKind::WarnDecodeCsrfToken
                | ErrorKind::WarnGetDurationCookie
                | ErrorKind::WarnNoDurationCookie
                | ErrorKind::SevereDurationOverflow
                | ErrorKind::WarnSubmitNoPlatform
                | ErrorKind::WarnResultsNoPlatform
                | ErrorKind::WarnCsrfTokenInvalid
                | ErrorKind::WarnInvalidAnswer
        )
    }
}

#[derive(Debug)]
pub struct InstanceError {
    pub kind: ErrorKind,
    pub msg: String,
}

// implementing Display to avoid adding failure crate
impl fmt::Display for InstanceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} - {:?}", self.kind, self.msg)
    }
}

impl error::ResponseError for InstanceError {
    fn error_response(&self) -> HttpResponse {
        let g_instance = InstanceInfo::global();

        if self.kind.is_info() {
            debug(&format!("Error reached: {}", self));
        } else {
            eprintln!("Error reached: {}", self);
        }

        let body = if self.kind.is_info()
            || (self.kind.is_client_error() && g_instance.config.hide_errors)
        {
            "OK"
        } else {
            "Une erreur a \u{e9}t\u{e9} rencontr\u{e9}e lors de l'ex\u{e9}cution de votre requ\u{ea}te. Veuillez r\u{e9}essayer plus tard !"
        };
        HttpResponse::build(self.status_code())
            .content_type("text/plain; charset=utf-8")
            .body(body)
    }

    fn status_code(&self) -> StatusCode {
        if self.kind.is_info() {
            StatusCode::OK
        } else if self.kind.is_client_error() {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
