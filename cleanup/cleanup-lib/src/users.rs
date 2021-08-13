pub mod errors;
pub mod models;
pub mod routes;

const FIND_ALL: &'static str = include_str!("./users/queries/find_all.sql");
const FIND_BY_ID: &'static str = include_str!("./users/queries/find_by_id.sql");
const INSERT: &'static str = include_str!("./users/queries/insert.sql");
const UPDATE: &'static str = include_str!("./users/queries/update.sql");
const DELETE: &'static str = include_str!("./users/queries/delete.sql");
const LOGIN: &'static str = include_str!("./users/queries/login.sql");
