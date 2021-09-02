use chrono::NaiveDate;

use crate::database::schema::{results, submissions};

#[derive(Serialize, Insertable, Debug, Clone)]
#[table_name = "submissions"]
pub struct InsertableSubmissions {
    pub platform_id: i16,
    pub asn: i32,
    pub abuse_code: i16,
    pub sent_timestamp: NaiveDate,
    pub duration: i32,
}

#[derive(Serialize, Insertable, Debug, Clone)]
#[table_name = "results"]
pub struct InsertableResult {
    pub platform_id: i16,
    pub generated_at: NaiveDate,
    pub part_total: i64,
    pub part_valid: i64,
}
