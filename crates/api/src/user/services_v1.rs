use core_app::AppResult;
use domain::user::request::{RequestCreateUser, RequestUpdateUser, User, UserFilter};
use infra::gen_com_fn;
use utils::pre_process::PreProcess;

use super::UserDmc;

gen_com_fn!(
  DMC: UserDmc,
  Entity: User,
  ReqCreate: RequestCreateUser,
  ResCreate: User,
  ReqUpdate: RequestUpdateUser,
  Filter: UserFilter,
  Route: "users",
);
