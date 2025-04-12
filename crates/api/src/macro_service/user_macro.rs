use core_app::AppResult;
use domain::entities::user::{RequestCreateUser, RequestUpdateUser, User, UserFilter};
use infra::gen_com_fn;
use utils::pre_process::{PreProcess, PreProcessR};

use infra::database::schema::UserDmc;

gen_com_fn!(
  DMC: UserDmc,
  Entity: User,
  ReqCreate: RequestCreateUser,
  ResCreate: User,
  ReqUpdate: RequestUpdateUser,
  Filter: UserFilter,
  Route: "users",
  Tag: "User Service",
);
