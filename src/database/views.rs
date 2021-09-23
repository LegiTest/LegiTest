use chrono::NaiveDate;

/* /api/results output */

#[derive(Serialize)]
pub struct ResultsPublic {
    pub global: ResultsPublicGlobal,
    pub groupes: Vec<ResultsPublicGroupes>,
}

#[derive(Serialize, Clone)]
pub struct ResultsPublicGroupes {
    pub id: i16,
    pub value_median: f32,
}

#[derive(Serialize)]
pub struct ResultsPublicGlobal {
    pub platform_id: i16,
    pub started_at: NaiveDate,
    pub generated_at: NaiveDate,
    pub participations: ResultsParticipations,
}

#[derive(Serialize)]
pub struct ResultsParticipations {
    pub total: i64,
    pub valid: i64,
}
