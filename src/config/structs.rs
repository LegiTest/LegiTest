use chrono::naive::NaiveDate;
use serde_derive::Deserialize;
use std::collections::HashMap;

use crate::database::structs::Choice;

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    pub bind_address: String,
    pub bind_port: u16,
    pub show_stats: bool,
    pub debug_mode: bool,
    pub database_path: String,
    pub csrf_key: String,
    pub hide_errors: bool,
    pub hours_update_results: u16,
    pub abuseipdb_api_key: String,
    pub twitter_api_client_id: String,
    pub twitter_api_client_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct Platform {
    pub id: i16,
    pub host: String,
    pub name: String,
    pub begin_at: NaiveDate,
    pub end_at: NaiveDate,
    pub minimum_participations: u32,
    pub groups: Vec<i16>,
}

#[derive(Debug)]
pub struct InstanceInfo {
    pub config: ConfigFile,
    pub platforms_list: Vec<Platform>,
    pub scrutins_list: Vec<Scrutin>,
    pub ipv4_whitelist: Vec<IPV4WLEntry>,
    pub ipv4_blacklist: Vec<IPV4BLEntry>,
    pub asn_list: Vec<ASNLEntry>,
    pub asn_blacklist: Vec<ASNBLEntry>,
}

// IPv4 whitelist entry
#[derive(Debug)]
pub struct IPV4WLEntry {
    pub ip_start: u32,
    pub ip_end: u32,
}

// IPv4 blacklist entry
#[derive(Debug)]
pub struct IPV4BLEntry {
    pub ip_start: u32,
    pub ip_end: Option<u32>,
    pub abuse_code: u16,
}

// ASN list entry
#[derive(Debug)]
pub struct ASNLEntry {
    pub ip_start: u32,
    pub ip_end: u32,
    pub asn: u32,
}

// ASN blacklist entry
#[derive(Debug)]
pub struct ASNBLEntry {
    pub asn: u32,
    pub abuse_code: u16,
}

#[derive(Deserialize, Debug)]
pub struct Scrutin {
    pub id: String,
    pub question_id: i16,
    pub name: String,
    pub description: String,
    pub arguments: Vec<ScrutinArgument>,
    pub invert_votes: bool,
    #[serde(rename = "dateScrutin")]
    pub date_scrutin: String,
    #[serde(rename = "nbreVotants")]
    pub nbre_votants: i32,
    pub organes: HashMap<Choice, Vec<i16>>,
    pub deputes: HashMap<Choice, Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct ScrutinArgument {
    #[serde(rename = "type")]
    pub typ: String,
    pub comment: String,
}
