use actix_web::http::StatusCode;
use actix_web::{get, web, HttpRequest, HttpResponse, Result};
use chrono::Utc;

use crate::DbConn;

use crate::config::structs::InstanceInfo;
use crate::database::structs::{Results, ResultsGroupes, Submissions};
use crate::database::views::{
    ResultsParticipations, ResultsPublic, ResultsPublicGlobal, ResultsPublicGroupes,
};
use crate::errors::{throw, ErrorKind, InstanceError};
use crate::reports::{groups_to_public, round_data};
use crate::DbPool;

#[get("/api/results")]
pub async fn results(
    dbpool: web::Data<DbPool>,
    req: HttpRequest,
) -> Result<HttpResponse, InstanceError> {
    let conn = dbpool
        .get()
        .map_err(|e| throw(ErrorKind::CritResultsPool, e.to_string()))?;

    let g_instance = InstanceInfo::global();

    let results_public = fetch_results(&g_instance, req.connection_info().host(), &conn)?;

    Ok(HttpResponse::build(StatusCode::OK).json(results_public))
}

pub fn fetch_results(
    g_instance: &InstanceInfo,
    hostname: &str,
    conn: &DbConn,
) -> Result<ResultsPublic, InstanceError> {
    let platform = g_instance
        .platforms_list
        .iter()
        .find(|p| p.host == hostname);

    let platform = match platform {
        Some(v) => v,
        None => {
            return Err(throw(ErrorKind::WarnResultsNoPlatform, hostname.into()));
        }
    };

    // get latest result from db
    let db_results = Results::get_latest(platform.id, &conn)
        .map_err(|e| throw(ErrorKind::CritResultsGetLatest, e.to_string()))?;

    let results_public = if let Some(res) = db_results {
        // gather results directly from the above query
        let resultsgroupes = ResultsGroupes::get_from(res.id, &conn)
            .map_err(|e| throw(ErrorKind::CritResultsGetGroups, e.to_string()))?;

        ResultsPublic {
            global: ResultsPublicGlobal {
                platform_id: res.platform_id,
                started_at: platform.begin_at,
                generated_at: res.generated_at,
                participations: ResultsParticipations {
                    total: res.part_total,
                    valid: res.part_valid,
                },
            },
            groupes: groups_to_public(&resultsgroupes),
        }
    } else {
        // display count of missing results before announcement
        let vsub_count = Submissions::count_valid(
            platform.id,
            &conn,
            platform.begin_at,
            Utc::now().naive_utc().date(),
        )
        .map_err(|e| throw(ErrorKind::CritResultsGetValid, e.to_string()))?;

        let mut placeholder_results: Vec<ResultsPublicGroupes> = Vec::new();

        for group in &platform.groups {
            placeholder_results.push(ResultsPublicGroupes {
                id: *group,
                value_median: 0_f32,
            });
        }

        ResultsPublic {
            // there's no generated_at value, so it returns begin_at
            // participations.total is rounded to prevent leaks
            global: ResultsPublicGlobal {
                platform_id: platform.id,
                started_at: platform.begin_at,
                generated_at: platform.begin_at,
                participations: ResultsParticipations {
                    total: 0,
                    valid: round_data(vsub_count),
                },
            },
            groupes: placeholder_results,
        }
    };

    Ok(results_public)
}
