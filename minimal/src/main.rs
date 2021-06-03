use actix_web::{get, App, HttpResponse, HttpServer};

const WELCOME_MSG: &'static str = include_str!("./../strings/welcome.txt");

#[get("/")]
async fn index() -> HttpResponse {
    let response = HttpResponse::Ok()
        .content_type("text/html; charset=UTF-8")
        .body(WELCOME_MSG);
    response
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || App::new().service(index))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
