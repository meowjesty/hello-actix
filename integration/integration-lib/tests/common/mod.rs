use actix_web::web;
use integration_lib::create_database;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};

pub async fn setup_data() -> web::Data<Pool<Sqlite>> {
    let db_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(env!("DATABASE_FILE"))
        .create_if_missing(true);

    let database_pool = SqlitePoolOptions::new()
        .max_connections(20)
        .connect_with(db_options)
        .await
        .unwrap();

    create_database(&database_pool).await.unwrap();

    web::Data::new(database_pool)
}
