use dotenv::var;

// cach 1
pub fn get_dsn() -> String {
  var("DSN").expect("DSN must be set")
}

pub fn get_max_connections() -> u32 {
  var("JWT_SECRET_KEY")
    .expect("MAX_CONNS must be set")
    .parse::<u32>()
    .expect("MAX_CONNS must be a number")
}

pub fn get_port() -> String {
  var("PORT").expect("PORT must be set")
}
