mod db;
mod middlewares;
mod models;
mod routes;

use dotenv::dotenv;
use middlewares::auth_middleware::Auth;
use std::{path::PathBuf, sync::Arc};

use db::init_db;
use poem::{delete, get, listener::TcpListener, middleware::Cors, post, put, EndpointExt, Route, Server};
use routes::{login::login, get_posts::get_posts, get_post::get_post, upload_post::upload_post};
use sqlx::{SqlitePool};
use crate::models::AppState;
use crate::routes::delete_post::delete_post;
use crate::routes::update_post::update_post;

fn configure_routes() -> Route {
    Route::new()
        .at("/posts", get(get_posts))
        .at("/post/:post_id", get(get_post))
        .at("/post/upload", post(upload_post).with(Auth))
        .at("/post/update/:post_id", put(update_post).with(Auth))
        .at("/post/delete/:post_id", delete(delete_post).with(Auth))
        .at("/login", post(login))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    let db_path = "./.db/database.db";
    let db_url = format!("sqlite:{}?mode=rwc", db_path);
    println!("Database URL: {}", db_url);

    let db = SqlitePool::connect(&db_url).await.map_err(|e| {
        eprintln!("Connection with Database: {:?}", e);
        std::io::Error::new(std::io::ErrorKind::Other, e)
    })?;

    init_db(&db)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let state = Arc::new(AppState {
        db,
        upload_dir: PathBuf::from("./uploads"),
    });

    let app = configure_routes().data(state).with(
        Cors::new()
            .allow_origin("http://localhost:3000")
            .allow_methods(vec!["GET", "POST"])
            .allow_credentials(true)
            .allow_headers(vec!["content-type"]),
    );

    Server::new(TcpListener::bind("0.0.0.0:8080"))
        .run(app)
        .await
}
