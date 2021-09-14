// the abuseipdb crate wasn't satisfying (old, unmaintained and
// not handling server response very well).
// I need something way simpler and specific.

use actix_web::web;
use awc::Client;
use serde_json::Value;
use std::convert::TryFrom;

#[derive(Serialize, Debug)]
pub struct AbuseIpDbQuery {
    #[serde(rename = "ipAddress")]
    ip_address: String,
}

pub async fn query_abuse(
    client: &web::Data<Client>,
    client_inet: String,
    abuseipdb_key: &str,
) -> Result<u16, String> {
    let mut response = client
        .get("https://api.abuseipdb.com/api/v2/check")
        .insert_header(("User-Agent", "Actix-web"))
        .insert_header(("Accept", "application/json"))
        .insert_header(("Key", abuseipdb_key))
        .query(&AbuseIpDbQuery {
            ip_address: client_inet,
        })
        .map_err(|e| format!("Failed to insert query string: {:?}", e))?
        .send()
        .await
        .map_err(|e| format!("await on send failed: {:?}", e))?;

    let response_body = response
        .body()
        .await
        .map_err(|e| format!("await on response body failed: {:?}", e))?;

    let response_body = String::from_utf8(response_body.to_vec())
        .map_err(|e| format!("converting response body to utf8 failed: {:?}", e))?;

    let response_json: Value = serde_json::from_str(&response_body)
        .map_err(|e| format!("parsing json from response body failed: {:?}", e))?;

    let abuse_score = match response_json.get("data") {
        Some(r) => r,
        None => {
            return Err("getting data key from json failed".to_string());
        }
    };

    let abuse_score = match abuse_score.get("abuseConfidenceScore") {
        Some(r) => r,
        None => {
            return Err("getting abuseConfidenceScore key from json failed".to_string());
        }
    };

    let h_limit_remain = match response.headers().get("x-ratelimit-remaining") {
        Some(r) => r,
        None => {
            return Err("getting remaining ratelimit from headers failed".to_string());
        }
    }
    .to_str()
    .map_err(|e| format!("converting remaining ratelimit to str failed: {:?}", e))?;

    let h_limit_total = match response.headers().get("x-ratelimit-limit") {
        Some(r) => r,
        None => {
            return Err("getting max ratelimit from headers failed".to_string());
        }
    }
    .to_str()
    .map_err(|e| format!("converting max ratelimit to str failed: {:?}", e))?;

    // prints the rate limits
    println!(
        "Abuse Confidence Score: {}\nAbuseIPdb rate limit: {} / {}",
        abuse_score, h_limit_remain, h_limit_total
    );

    // cap abuse confidence score to 100 just in case
    let abuse_score = match abuse_score.as_u64() {
        Some(r) => r,
        None => {
            return Err("abuse score u64 conversion failed".to_string());
        }
    };

    let abuse_score = match abuse_score {
        0..=100 => u16::try_from(abuse_score).unwrap_or(100),
        _ => 100,
    };

    Ok(abuse_score)
}
