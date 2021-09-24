use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Result};
use chrono::Utc;
use twapi_reqwest::v1;

use crate::config::structs::InstanceInfo;
use crate::database::models::InsertableResult;
use crate::database::structs::{Results, ResultsGroupes};
use crate::errors::{throw, ErrorKind, InstanceError};
use crate::handlers::results::fetch_results;
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
        .find(|p| &p.host == &params.hostname);

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

    let results_msg = if results_public.global.participations.total == 0 {
        format!(
            "Participations restantes avant la publication des résultats : {} / {}.\n#QuelParti / https://quelparti.fr",
            platform.minimum_participations - results_public.global.participations.valid as u32,
            platform.minimum_participations,
        )
    } else {
        let mut leading_group_stat = results_public.groupes.clone();
        leading_group_stat.sort_by(|a, b| {
            a.value_median
                .partial_cmp(&b.value_median)
                .expect("pubreport: There's no NaN in this set. Shouldnt' happen.")
        });

        let leading_group_stat = if let Some(v) = leading_group_stat.first() {
            v
        } else {
            return Err(throw(ErrorKind::CritNoLeadingGroup, format!("{:?}", leading_group_stat)));
        };

        // get group name
        let leading_group_info = g_instance
            .acteurs_list
            .organes
            .iter()
            .find(|x| x.id == leading_group_stat.id);

        let leading_group_info = if let Some(v) = leading_group_info {
            v
        } else {
            return Err(throw(ErrorKind::CritNoLeadingGroupName, format!("{:?}", leading_group_info)));
        };

        format!(
            "Statistiques de participation globales en date du {}\nComptabilisées : {} | Total : {}\nGroupe en tête : {} #{} ({} %)\n#QuelParti / https://quelparti.fr",
            results_public.global.generated_at.format("%d/%M/%Y"),
            results_public.global.participations.valid,
            results_public.global.participations.total,
            leading_group_info.name,
            leading_group_info.abrev,
            leading_group_stat.value_median,
        )
    };

    // now connects to the Twitter API
    // statuses/update
    let url = "https://api.twitter.com/1.1/statuses/update.json";
    let form_options = vec![("status", results_msg.as_str())];

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
    .map_err(|e| { throw(ErrorKind::CritTwitterReqFail, e.to_string()) })?
    .json()
    .await
    .map_err(|e| { throw(ErrorKind::CritTwitterRespFail, e.to_string()) })?;

    println!("Published: {:?}", results_msg);

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/plain; charset=utf-8")
        .body(results_msg))
}
