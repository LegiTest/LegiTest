use chrono::NaiveDate;

/* /api/results output */

#[derive(Serialize, Debug)]
pub struct ResultsPublic {
    pub global: ResultsPublicGlobal,
    pub groupes: Vec<ResultsPublicGroupes>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ResultsPublicGroupes {
    pub id: i16,
    pub value_median: f32,
}

#[derive(Serialize, Debug)]
pub struct ResultsPublicGlobal {
    pub platform_id: i16,
    pub started_at: NaiveDate,
    pub generated_at: NaiveDate,
    pub participations: ResultsParticipations,
}

#[derive(Serialize, Debug)]
pub struct ResultsParticipations {
    pub total: i64,
    pub valid: i64,
}
