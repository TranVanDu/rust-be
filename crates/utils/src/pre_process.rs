use async_trait::async_trait;

use core_app::AppResult;

#[async_trait]
pub trait PreProcess {
  async fn pre_process(&mut self) -> AppResult<()>;
}
