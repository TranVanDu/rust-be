pub mod configs;
pub mod errors;
pub mod response;

pub type AppResult<T> = Result<T, errors::AppError>;
