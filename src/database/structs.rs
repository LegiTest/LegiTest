use chrono::NaiveDate;
use ipnetwork::IpNetwork;

use crate::database::schema::{
    addresses, results, resultsgroupes, submissions, submissionschoices,
};

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Choice {
    Pour,
    Contre,
    Abstention,
}

/* because diesel's enum support sucks atm */
impl Choice {
    pub fn from_db(i: i16) -> Option<Self> {
        match i {
            1 => Some(Choice::Pour),
            0 => Some(Choice::Abstention),
            -1 => Some(Choice::Contre),
            _ => None,
        }
    }

    pub fn to_db(&self) -> i16 {
        match self {
            Choice::Pour => 1,
            Choice::Abstention => 0,
            Choice::Contre => -1,
        }
    }
}

#[derive(Serialize, Deserialize, Queryable, Identifiable, Insertable, Debug, Clone)]
#[primary_key(ip, platform_id)]
#[table_name = "addresses"]
pub struct Addresses {
    pub ip: IpNetwork,
    pub platform_id: i16,
}

#[derive(Serialize, Queryable, Identifiable, Debug, Clone)]
#[table_name = "results"]
pub struct Results {
    pub id: i64,
    pub platform_id: i16,
    pub generated_at: NaiveDate,
    pub part_total: i64,
    pub part_valid: i64,
}

#[derive(Serialize, Queryable, Identifiable, Debug, Clone)]
#[table_name = "submissions"]
pub struct Submissions {
    pub id: i64,
    pub platform_id: i16,
    pub asn: i32,
    pub abuse_code: i16,
    pub sent_timestamp: NaiveDate,
    pub duration: i32,
}

#[derive(Serialize, Queryable, Identifiable, Insertable, Debug, Clone)]
#[primary_key(result_id, group_id)]
#[table_name = "resultsgroupes"]
pub struct ResultsGroupes {
    pub result_id: i64,
    pub group_id: i16,
    pub value_median: f32,
}

#[derive(Serialize, Queryable, Identifiable, Insertable, Debug, Clone, PartialEq)]
#[primary_key(submission_id, question_id)]
#[table_name = "submissionschoices"]
pub struct SubmissionsChoices {
    pub submission_id: i64,
    pub question_id: i16,
    pub userchoice: i16,
}
