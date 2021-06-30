pub(crate) mod errors;
mod models;
pub(crate) mod routes;

const FIND_BY_PATTERN: &'static str = include_str!("./tasks/queries/find_by_pattern.sql");
const FIND_ONGOING: &'static str = include_str!("./tasks/queries/find_ongoing.sql");
const FIND_ALL: &'static str = include_str!("./tasks/queries/find_all.sql");
const FIND_BY_ID: &'static str = include_str!("./tasks/queries/find_by_id.sql");
const INSERT: &'static str = include_str!("./tasks/queries/insert.sql");
const UPDATE: &'static str = include_str!("./tasks/queries/update.sql");
const DELETE: &'static str = include_str!("./tasks/queries/delete.sql");

const COMPLETED: &'static str = include_str!("./tasks/queries/done.sql");
const UNDO: &'static str = include_str!("./tasks/queries/undo.sql");
