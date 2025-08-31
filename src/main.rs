mod common;
mod config;
mod items;
mod templates;

#[macro_use]
extern crate rocket;

use std::env;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use dotenv::dotenv;
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::{Build, Rocket, Route};
use rocket_dyn_templates::Template;
use sqlx::postgres::PgPoolOptions;
use sqlx::Executor;

#[launch]
async fn launch() -> Rocket<Build> {
    // Load environment
    dotenv().ok();
    env_logger::init();

    // Load config
    let config = crate::config::Config::load().expect("Failed to load config properties");
    info!("Loaded configuration: {:?}", config);

    // Start DB connection pool
    let database_url = env::var("DATABASE_URL").expect("Failed to load database URL from env");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    // Import Schema
    let schema_path = Path::new("schema.sql");
    let schema = read_to_string(schema_path)
        .map_err(|e| format!("Failed to open DB schema file {:?}: {}", schema_path, e))
        .unwrap();
    info!("Loaded schema from {:?}", schema_path);
    pool.execute(schema.as_str()).await
        .map_err(|e| format!("Failed to import DB schema: {}", e))
        .unwrap();
    info!("Imported schema into database.");


    // Load navigation data and templates customization
    let templates_fairing = Template::custom(|engines| {
        engines.tera.autoescape_on(vec![".html", ".xml", ".js"]);
    });

    // Load S3 Config
    let s3_config = aws_config::from_env()
        .region("us-east-1")
        .endpoint_url("https://lon1.digitaloceanspaces.com")
        .load().await;
    let s3_client = aws_sdk_s3::Client::new(&s3_config);

    // Start Rocket
    rocket::build()
        .manage(config)
        .attach(templates_fairing)
        .manage(pool)
        .manage(s3_client)
        .mount("/", vec![
            crate::items::routes(),
            routes![static_files]
        ].into_iter().flatten().collect::<Vec<Route>>())
}

#[get("/static/<path..>")]
async fn static_files(
    path: PathBuf
) -> Result<NamedFile, Status> {
    let static_path = Path::new("static").join(&path);
    info!("No matching template for path {:?}, trying static path: {:?}", path, static_path);
    match NamedFile::open(static_path).await {
        Ok(file) => Ok(file),
        Err(_) => Err(Status::NotFound)
    }
}