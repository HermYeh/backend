use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct SaveData {
    key: String,
    value: String,
}

async fn save(data: web::Json<SaveData>, pool: web::Data<Mutex<Pool<Sqlite>>>) -> impl Responder {
    let pool = pool.lock().unwrap();
    sqlx::query("INSERT INTO data (key, value) VALUES (?, ?)")
        .bind(&data.key)
        .bind(&data.value)
        .execute(&*pool)
        .await
        .unwrap();
    println!("HttpResponse::Ok");
    HttpResponse::Ok().json("Data saved")
  
}

async fn load(data: web::Json<String>, pool: web::Data<Mutex<Pool<Sqlite>>>) -> impl Responder {
    let pool = pool.lock().unwrap();
    let result = sqlx::query_as::<_, (String,)>("SELECT value FROM data WHERE key = ?")
        .bind(&*data)
        .fetch_one(&*pool)
        .await
        .unwrap();
    HttpResponse::Ok().json(result.0)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite:order.db")
        .await
        .unwrap();

    sqlx::query("CREATE TABLE IF NOT EXISTS data (key TEXT PRIMARY KEY, value TEXT)")
        .execute(&pool)
        .await
        .unwrap();
    
    let pool = web::Data::new(Mutex::new(pool));
    
    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .route("/save", web::post().to(save))
            .route("/load", web::post().to(load))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}