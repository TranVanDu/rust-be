use async_trait::async_trait;

use core_app::AppResult;

#[async_trait]
pub trait PreProcess {
  async fn pre_process(&mut self) -> AppResult<()>;
}

#[async_trait]
pub trait PreProcessR {
  type Output;
  async fn pre_process_r(self) -> AppResult<Self::Output>;
}
