use modql::SIden;
use sea_query::{IntoIden, TableRef};

pub trait DB {
  const SCHEMA: &'static str;
  const TABLE: &'static str;
  const ID_COLUMN: &'static str;

  fn table_ref() -> TableRef {
    TableRef::SchemaTable(SIden(Self::SCHEMA).into_iden(), SIden(Self::TABLE).into_iden())

    /*
     * cách 2
     * TableRef::Table(SIden("tbl_users").into_iden())
     */
  }
}

pub struct UserDmc;

impl DB for UserDmc {
  const SCHEMA: &'static str = "users";
  const TABLE: &'static str = "tbl_users";
  const ID_COLUMN: &'static str = "pk_user_id";
}

pub struct PhoneCodeDmc;

impl DB for PhoneCodeDmc {
  const SCHEMA: &'static str = "users";
  const TABLE: &'static str = "phone_codes";
  const ID_COLUMN: &'static str = "id";
}
