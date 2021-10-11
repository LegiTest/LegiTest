use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Result};
use chrono::Utc;
use twapi_reqwest::v1;
use twapi_reqwest::reqwest::multipart::{Part, Form};

use crate::config::structs::InstanceInfo;
use crate::database::models::InsertableResult;
use crate::database::structs::{Results, ResultsGroupes};
use crate::errors::{throw, ErrorKind, InstanceError};
use crate::handlers::results::fetch_results;
use crate::canvas::gen_results_image;
use crate::reports::generate_report;
use crate::DbPool;

#[get("/internal/generate_report")]
pub async fn int_genreport(dbpool: web::Data<DbPool>) -> Result<HttpResponse, InstanceError> {
    let conn = dbpool
        .get()
        .map_err(|e| throw(ErrorKind::CritReportPool, e.to_string()))?;

    let g_instance = InstanceInfo::global();

    // generate a report for all platforms
    for platform in &g_instance.platforms_list {
        // TODO: add possibility to pass a custom generated_at date in params
        let report = generate_report(platform, Utc::now().naive_utc().date(), &conn)?;
        // push to DB
        let insert_result = InsertableResult {
            platform_id: report.global.platform_id,
            generated_at: report.global.generated_at,
            part_total: report.global.participations.total,
            part_valid: report.global.participations.valid,
        };

        // first insert the result
        let single_result = Results::insert(&insert_result, &conn)
            .map_err(|e| throw(ErrorKind::CritReportInsert, e.to_string()))?;

        // then insert all the resultgroupes
        let mut resultsgroupes: Vec<ResultsGroupes> = Vec::new();
        for group in report.groupes {
            resultsgroupes.push(ResultsGroupes {
                result_id: single_result.id,
                group_id: group.id,
                value_median: group.value_median,
            });
        }

        ResultsGroupes::insert(&resultsgroupes, &conn)
            .map_err(|e| throw(ErrorKind::CritReportInsertGroups, e.to_string()))?;
        }
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/plain; charset=utf-8")
        .body("OK"))
}

#[derive(Deserialize, Clone)]
pub struct PublishReportData {
    pub hostname: String,
}

#[get("/internal/publish_report/{hostname}")]
pub async fn int_pubreport(
    dbpool: web::Data<DbPool>,
    params: web::Path<PublishReportData>,
) -> Result<HttpResponse, InstanceError> {
    let conn = dbpool
        .get()
        .map_err(|e| throw(ErrorKind::CritReportPool, e.to_string()))?;

    let g_instance = InstanceInfo::global();

    let results_public = fetch_results(g_instance, &params.hostname, &conn)?;

    let platform = g_instance
        .platforms_list
        .iter()
        .find(|p| p.host == params.hostname);

    let platform = match platform {
        Some(v) => v,
        None => {
            return Err(throw(
                    ErrorKind::WarnResultsNoPlatform,
                    params.hostname.clone(),
            ));
        }
    };

    // If the results aren't available yet, display a different message

    // results_msg is (String, Option<Vec<u8>>)
    // String: message (tweet) to send ; Vec<u8> is the media attachment
    let results_msg = if results_public.global.participations.total == 0 {
        (format!(
                "Participations restantes avant la publication des résultats : {} / {}.\n#QuelParti https://quelparti.fr\n",
                platform.minimum_participations - results_public.global.participations.valid as u32,
                platform.minimum_participations,
        ), None)
    } else {
        let generated_image = gen_results_image(&results_public)?;
        (generated_image.0, Some(generated_image.1))
    };

    if g_instance.config.do_not_publish {
        println!("Not publishing because do_not_publish is true: {:?}", results_msg.0);
    } else {

        // can't use Option here because form_options must be Vec<&str, &str>
        // so the variable, passed through a `if let`, won't live long enough
        let mut attachment: String = String::new();
        if let Some(img) = results_msg.1 {
            // publish attachment (if any)
            let part = Part::bytes(img);
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
                .map_err(|e| { throw(ErrorKind::CritTwitterUpReqFail, e.to_string())})?
                .json()
                .await
                .map_err(|e| { throw(ErrorKind::CritTwitterUpRespFail, e.to_string())})?;

            // get the media_id value from json
            if let Some(media_id_str) = res.get("media_id") {
                // convert media_id to u64
                attachment = if let Some(media_id_u64) = media_id_str.as_u64() {
                    media_id_u64.to_string()
                } else {
                    return Err(throw(ErrorKind::CritTwitterUpMediaInt, format!("{:?}", media_id_str)));
                }
            } else {
                return Err(throw(ErrorKind::CritTwitterUpMediaGet, format!("{:?}", res.get("media_id"))));
            }

            // convert the media_id to u64
        }

        // publish tweet
        println!("Publishing: {:?}", results_msg.0);
        // now connects to the Twitter API
        // statuses/update
        let url = "https://api.twitter.com/1.1/statuses/update.json";
        let mut form_options = vec![("status", results_msg.0.as_str())];

        // include the attachment if exists
        if !attachment.is_empty() {
            form_options.push(("media_ids", &attachment));
        }

        let _: serde_json::Value = v1::post(
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
        }


    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/plain; charset=utf-8")
        .body(results_msg.0))
}
