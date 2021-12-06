use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Result};
use chrono::Utc;

use crate::canvas::gen_results_image;
use crate::config::structs::{InstanceInfo, Platform};
use crate::database::models::InsertableResult;
use crate::database::structs::{Results, ResultsGroupes};
use crate::database::views::ResultsPublic;
use crate::errors::{throw, ErrorKind, InstanceError};
use crate::handlers::results::fetch_results;
use crate::reports::generate_report;
use crate::tweet::{publish_tweet, upload_attachment};
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
                value_average: group.value_average,
                value_uninominal: group.value_uninominal,
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

    let results_msg = format_tweet(&results_public, &platform, "value_median")?;

    if let Some(img) = results_msg.1 {
        let media_id = upload_attachment(&img).await;
    }
    else {

    }


    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/plain; charset=utf-8")
        .body(results_msg.0))
}
    
// returns results_msg, which is (String, Option<Vec<u8>>)
// String: message (tweet) to send ; Vec<u8> is the media attachment
fn format_tweet(results_public: &ResultsPublic, platform: &Platform, calc_method: &str) -> Result<(String, Option<Vec<u8>>), InstanceError> {
    // If the results aren't available yet, display a different message
    if results_public.global.participations.total == 0 {
        Ok((format!(
                    "Participations restantes avant la publication des résultats : {} / {}.\n#QuelParti https://quelparti.fr\n",
                    platform.minimum_participations - results_public.global.participations.valid as u32,
                    platform.minimum_participations,
        ), None))
    } else {
        let generated_image = gen_results_image(&results_public)?;
        Ok((generated_image.0, Some(generated_image.1)))
    }
}
