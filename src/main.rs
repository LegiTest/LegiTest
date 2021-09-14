#![forbid(unsafe_code)]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

mod abuseipdb;
mod config;
mod database;
mod errors;
mod handlers;
mod matching;
mod reports;

use actix_files::Files;
use actix_session::CookieSession;
use actix_web::cookie::SameSite;
use actix_web::web::{Data, JsonConfig};
use actix_web::{App, HttpServer};
use awc::Client;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use crate::config::global::INSTANCE;
use crate::config::structs::{ASNBLEntry, IPV4BLEntry, IPV4WLEntry, InstanceInfo};
use crate::errors::invalid_form;
use crate::handlers::csrftoken::csrftoken;
use crate::handlers::reports::int_genreport;
use crate::handlers::results::results;
use crate::handlers::submit::submit;

type DbConn = PgConnection;
embed_migrations!("migrations/postgres");

type DbPool = r2d2::Pool<ConnectionManager<DbConn>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("QuelParti server, starting.");

    println!("Loading configuration files...");
    INSTANCE.set(InstanceInfo::init()).unwrap();

    let g_instance = InstanceInfo::global();

    println!("Checking input validity...");
    g_instance.check_validity();

    if InstanceInfo::global().config.show_stats {
        println!("{} platforms loaded.", g_instance.platforms_list.len());
        println!(
            "{} whitelisted IPv4 ranges loaded ({} individual IPs).",
            g_instance.ipv4_whitelist.len(),
            IPV4WLEntry::count(&g_instance.ipv4_whitelist)
        );

        println!(
            "{} blocked IPv4 ranges loaded ({} individual IPs).",
            g_instance.ipv4_blacklist.len(),
            IPV4BLEntry::count(&g_instance.ipv4_blacklist)
        );

        println!("{} ASNs loaded.", g_instance.asn_list.len());

        println!(
            "{} blocked ASNs loaded ({} individual IPs).",
            g_instance.asn_blacklist.len(),
            ASNBLEntry::count(&g_instance.asn_blacklist, &g_instance.asn_list)
        );
    }

    println!("Connecting to DB...");
    let manager = ConnectionManager::<DbConn>::new(&g_instance.config.database_path);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("ERROR: main: Failed to create the database pool.");

    let conn = pool.get().expect("ERROR: main: DB connection failed");

    println!("Running migrations...");
    embedded_migrations::run(&*conn).expect("ERROR: main: Failed to run database migrations");

    println!(
        "Running web server at {}:{}.",
        g_instance.config.bind_address, g_instance.config.bind_port
    );
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(Client::default()))
            .app_data(
                JsonConfig::default()
                    .limit(4096)
                    .error_handler(invalid_form),
            )
            .wrap(
                CookieSession::signed(&[0; 32])
                    .secure(true)
                    .same_site(SameSite::Strict)
                    .http_only(true),
            )
            .service(submit)
            .service(int_genreport)
            .service(results)
            .service(csrftoken)
            .service(Files::new("/data/", "./data/"))
            .service(Files::new("/", "./static/").index_file("index.html"))
    })
    .bind((
        g_instance.config.bind_address.as_str(),
        g_instance.config.bind_port,
    ))?
    .workers(16)
    .system_exit()
    .run()
    .await
}

pub fn debug(txt: &str) {
    if InstanceInfo::global().config.debug_mode {
        println!("{}", txt);
    }
}
