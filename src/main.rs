use std::env;
use std::net::Ipv4Addr;
use pretty_env_logger::env_logger::DEFAULT_FILTER_ENV;
use warp::{Filter, Rejection};
use crate::model::{DB, QueryOptions};

mod handler;
mod model;

type WebResult<T> = std::result::Result<T, Rejection>;

#[tokio::main]
async fn main() {

    if std::env::var_os(DEFAULT_FILTER_ENV).is_none() {
        std::env::set_var(DEFAULT_FILTER_ENV, "api=info");
    }
    pretty_env_logger::init();

    let db = model::todo_db();

    let todo_router = warp::path!("api" / "todos");
    let todo_router_id = warp::path!("api" / "todos" / String);

    let health_checker = warp::path!("api" / "healthchecker")
        .and(warp::get())
        .and_then(handler::health_checker_handler);

    let cors = warp::cors()
        .allow_origins(vec!["http://localhost:3000/", "http://localhost:8000/", "http://localhost:7071/"])
        .allow_headers(vec!["content-type"])
        .allow_credentials(true);

    let todo_routes = todo_router
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(handler::create_todo_handler)
        .or(todo_router
            .and(warp::get())
            .and(warp::query::<QueryOptions>())
            .and(with_db(db.clone()))
            .and_then(handler::todos_list_handler));

    let todo_routes_id = todo_router_id
        .and(warp::patch())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(handler::edit_todo_handler)
        .or(todo_router_id
            .and(warp::get())
            .and(with_db(db.clone()))
            .and_then(handler::get_todo_handler))
        .or(todo_router_id
            .and(warp::delete())
            .and(with_db(db.clone()))
            .and_then(handler::delete_todo_handler));

    let routes = todo_routes
        .with(cors)
        .with(warp::log("api"))
        .or(todo_routes_id)
        .or(health_checker);

    let port_key = "FUNCTIONS_CUSTOMHANDLER_PORT";
    let port: u16 = match env::var(port_key) {
        Ok(val) => val.parse().expect("Custom Handler port is not a number!"),
        Err(_) => 3000,
    };

    warp::serve(routes).run((Ipv4Addr::LOCALHOST, port)).await

}

fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

