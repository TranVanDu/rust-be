use infra::base::DB;

mod routes;
mod services_v0;
mod services_v1;

pub use routes::routes;
pub use services_v1::routes as routes_v1;

pub struct UserDmc;

impl DB for UserDmc {
  const SCHEMA: &'static str = "users";
  const TABLE: &'static str = "tbl_users";
  const ID_COLUMN: &'static str = "pk_user_id";
}
