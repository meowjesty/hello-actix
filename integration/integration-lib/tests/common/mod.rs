use actix_web::web;
use integration_lib::create_database;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};


pub async fn setup_data() -> web::Data<Pool<Sqlite>> {
    let db_options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(env!("DATABASE_FILE"))
        .create_if_missing(true);

    let database_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(db_options)
        .await
        .unwrap();

    create_database(&database_pool).await.unwrap();

    web::Data::new(database_pool)
}

// WARNING(alex): This macro doesn't check if there is an user register already, or if some user is
// logged in, so the tests must be run with:
// cargo test -- --test-threads=1
// Running them in parallel may fail!
#[macro_export]
macro_rules! setup_app {
    ($configure: expr) => {{
        let data = setup_data().await;
        let app = App::new()
            .app_data(data.clone())
            .configure($configure)
            .service(user_insert)
            .service(login)
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-cookie")
                    .login_deadline(Duration::minutes(10))
                    .secure(false),
            ));
        let mut app = test::init_service(app).await;

        let (cookies, bearer_token) = {
            let new_user = InsertUser {
                valid_username: "spike".to_string(),
                valid_password: "vicious".to_string(),
            };
            let register_user_request = test::TestRequest::post()
                .uri("/users/register")
                .set_json(&new_user)
                .to_request();
            let register_user_service_response: ServiceResponse =
                test::call_service(&mut app, register_user_request).await;
            assert!(register_user_service_response.status().is_success());

            let user: User = test::read_body_json(register_user_service_response).await;

            let login_user = LoginUser {
                username: user.username,
                password: user.password,
            };
            let login_request = test::TestRequest::post()
                .uri("/users/login")
                .set_json(&login_user)
                .to_request();
            let login_service_response: ServiceResponse =
                test::call_service(&mut app, login_request).await;
            assert!(login_service_response.status().is_success());

            let cookies = login_service_response.response().cookies();
            let cookies_str = cookies
                .flat_map(|cookie| cookie.to_string().chars().collect::<Vec<_>>())
                .collect::<String>();

            let logged_user: LoggedUser = test::read_body_json(login_service_response).await;

            let bearer_token = format!("Bearer {}", logged_user.token);
            let cookies = Cookie::parse_encoded(cookies_str).unwrap();

            (cookies, bearer_token)
        };

        (app, bearer_token, cookies)
    }};
}
