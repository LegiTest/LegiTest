use actix_web::http::StatusCode;
use actix_web::{get, web, HttpResponse, Result};
use chrono::Utc;

use crate::config::structs::InstanceInfo;
use crate::database::models::InsertableResult;
use crate::database::structs::{Results, ResultsGroupes};
use crate::errors::{throw, ErrorKind, InstanceError};
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
