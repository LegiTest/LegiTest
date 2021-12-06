use twapi_reqwest::reqwest::multipart::{Form, Part};
use twapi_reqwest::v1;

use crate::config::structs::{InstanceInfo, Platform};
use crate::errors::{throw, ErrorKind, InstanceError};

fn print_pub(msg: &str, do_not_publish: bool) {
    if do_not_publish {
        println!(
            "Not publishing because do_not_publish is true: {:?}",
            msg
        );
    } else {
        println!(
            "Publishing: {:?}",
            msg
        );
    }
}

// returns the media_id
pub async fn upload_attachment(generated_img: &Vec<u8>) -> Result<u64, InstanceError> {
    let g_instance = InstanceInfo::global();

    print_pub("attachment", g_instance.config.do_not_publish);
    // early return if do_not_publish is true
    if g_instance.config.do_not_publish {
        return Ok(0);
    }

    // can't use Option here because form_options must be Vec<&str, &str>
    // so the variable, passed through a `if let`, won't live long enough
    let mut attachment: String = String::new();
    // publish attachment (if any)
    let part = Part::bytes(generated_img);
    let data = Form::new().part("media", part);
    let url = "https://upload.twitter.com/1.1/media/upload.json";
    let res: serde_json::Value = v1::multipart(
        url,
        &vec![],
        data,
        &g_instance.config.twitter_api_client_id,
        &g_instance.config.twitter_api_client_secret,
        &g_instance.config.twitter_api_oauth_token,
        &g_instance.config.twitter_api_oauth_secret,
    )
        .await
        .map_err(|e| throw(ErrorKind::CritTwitterUpReqFail, e.to_string()))?
        .json()
        .await
        .map_err(|e| throw(ErrorKind::CritTwitterUpRespFail, e.to_string()))?;

    // get the media_id value from json
    if let Some(media_id_str) = res.get("media_id") {
        // convert media_id to u64
        if let Some(media_id_u64) = media_id_str.as_u64() {
            Ok(media_id_u64)
        } else {
            return Err(throw(
                    ErrorKind::CritTwitterUpMediaInt,
                    format!("{:?}", media_id_str),
            ));
        }
    } else {
        return Err(throw(
                ErrorKind::CritTwitterUpMediaGet,
                format!("{:?}", res.get("media_id")),
        ));
    }
}

// returns the tweet ID
pub async fn publish_tweet(message: &str, media_id: Option<u64>) -> Result<u64, InstanceError> {
    let g_instance = InstanceInfo::global();

    // print tweet
    print_pub(message, g_instance.config.do_not_publish);
    // early return if do_not_publish is true
    if g_instance.config.do_not_publish {
        return Ok(0);
    }

    // now connects to the Twitter API
    // statuses/update
    let url = "https://api.twitter.com/1.1/statuses/update.json";
    let mut form_options = vec![("status", message)];

    // include the attachment if exists
    if let Some(id) = media_id {
        form_options.push(("media_ids", &id.to_string()));
    }

    let res: serde_json::Value = v1::post(
        url,
        &vec![],
        &form_options,
        &g_instance.config.twitter_api_client_id,
        &g_instance.config.twitter_api_client_secret,
        &g_instance.config.twitter_api_oauth_token,
        &g_instance.config.twitter_api_oauth_secret,
    )
        .await
        .map_err(|e| throw(ErrorKind::CritTwitterReqFail, e.to_string()))?
        .json()
        .await
        .map_err(|e| throw(ErrorKind::CritTwitterRespFail, e.to_string()))?;

    // get the media_id value from json
    if let Some(id_str) = res.get("id") {
        // convert media_id to u64
        if let Some(id_u64) = id_str.as_u64() {
            Ok(id_u64)
        } else {
            return Err(throw(
                    ErrorKind::CritTwitterUpMediaInt,
                    format!("{:?}", id_str),
            ));
        }
    } else {
        return Err(throw(
                ErrorKind::CritTwitterUpMediaGet,
                format!("{:?}", res.get("id")),
        ));
    }
}
