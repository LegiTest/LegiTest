//use crate::database::schema::{addresses, submissions};
use crate::config::global::{AC_HIGHEST, AC_LOWEST, MAX_SUB_DURATION};
use crate::database::models::{InsertableResult, InsertableSubmissions};
use crate::database::structs::{
    Addresses, Results, ResultsGroupes, Submissions, SubmissionsChoices,
};
use crate::DbConn;

use chrono::NaiveDate;
use diesel::prelude::*;
use ipnetwork::IpNetwork;

impl Addresses {
    pub fn exists(
        i_ip: IpNetwork,
        i_platform_id: i16,
        conn: &DbConn,
    ) -> Result<bool, diesel::result::Error> {
        use crate::database::schema::addresses::dsl::addresses;

        let res: Option<Addresses> = addresses
            .find((i_ip, i_platform_id))
            .first(conn)
            .optional()?;

        Ok(res.is_some())
    }

    pub fn insert(i_addresses: Addresses, conn: &DbConn) -> Result<Self, diesel::result::Error> {
        use crate::database::schema::addresses::dsl::addresses;

        diesel::insert_into(addresses)
            .values(i_addresses)
            .get_result(conn)
    }
}

impl Submissions {
    pub fn insert(
        i_submissions: InsertableSubmissions,
        conn: &DbConn,
    ) -> Result<Self, diesel::result::Error> {
        use crate::database::schema::submissions::dsl::submissions;

        diesel::insert_into(submissions)
            .values(i_submissions)
            .get_result(conn)
    }

    pub fn count(i_platform_id: i16, conn: &DbConn) -> Result<i64, diesel::result::Error> {
        use crate::database::schema::submissions::dsl::{platform_id, submissions};

        submissions
            .filter(platform_id.eq(i_platform_id))
            .count()
            .get_result(conn)
    }

    // to check the minimal requirement to generate a report
    pub fn count_valid(
        i_platform_id: i16,
        conn: &DbConn,
        begin_at: NaiveDate,
        end_at: NaiveDate,
    ) -> Result<i64, diesel::result::Error> {
        use crate::database::schema::submissions::dsl::{
            abuse_code, duration, platform_id, sent_timestamp, submissions,
        };

        submissions
            .filter(platform_id.eq(i_platform_id))
            .filter(abuse_code.between(AC_LOWEST, AC_HIGHEST))
            .filter(sent_timestamp.between(begin_at, end_at))
            .filter(duration.le(MAX_SUB_DURATION))
            .count()
            .get_result(conn)
    }

    pub fn get_valid(
        i_platform_id: i16,
        conn: &DbConn,
        begin_at: NaiveDate,
        end_at: NaiveDate,
    ) -> Result<Vec<SubmissionsChoices>, diesel::result::Error> {
        use crate::database::schema::submissions::dsl::{
            abuse_code, duration, platform_id, sent_timestamp, submissions,
        };
        use crate::database::schema::submissionschoices::dsl::{
            question_id, submission_id, submissionschoices, userchoice,
        };

        submissions
            .inner_join(submissionschoices)
            .filter(platform_id.eq(i_platform_id))
            .filter(abuse_code.between(AC_LOWEST, AC_HIGHEST))
            .filter(sent_timestamp.between(begin_at, end_at))
            .filter(duration.le(MAX_SUB_DURATION))
            .order(submission_id)
            .select((submission_id, question_id, userchoice))
            .load(conn)
    }
}

impl SubmissionsChoices {
    pub fn insert(
        i_choices: Vec<SubmissionsChoices>,
        conn: &DbConn,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::database::schema::submissionschoices::dsl::submissionschoices;

        diesel::insert_into(submissionschoices)
            .values(i_choices)
            .get_results(conn)
    }
}

impl Results {
    pub fn get_latest(
        i_platform_id: i16,
        conn: &DbConn,
    ) -> Result<Option<Results>, diesel::result::Error> {
        use crate::database::schema::results::dsl::{generated_at, id, platform_id, results};

        results
            .filter(platform_id.eq(i_platform_id))
            .order((generated_at.desc(), id.desc()))
            .first(conn)
            .optional()
    }

    pub fn insert(
        i_result: &InsertableResult,
        conn: &DbConn,
    ) -> Result<Results, diesel::result::Error> {
        use crate::database::schema::results::dsl::results;

        diesel::insert_into(results)
            .values(i_result)
            .get_result(conn)
    }
}

impl ResultsGroupes {
    pub fn insert(
        i_resultgroupes: &[ResultsGroupes],
        conn: &DbConn,
    ) -> Result<Vec<ResultsGroupes>, diesel::result::Error> {
        use crate::database::schema::resultsgroupes::dsl::resultsgroupes;

        diesel::insert_into(resultsgroupes)
            .values(i_resultgroupes)
            .get_results(conn)
    }

    pub fn get_from(
        i_result_id: i64,
        conn: &DbConn,
    ) -> Result<Vec<ResultsGroupes>, diesel::result::Error> {
        use crate::database::schema::resultsgroupes::dsl::{result_id, resultsgroupes};

        resultsgroupes
            .filter(result_id.eq(i_result_id))
            .get_results(conn)
    }
}
