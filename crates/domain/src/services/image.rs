use crate::repositories::image_repository::ImageRepository;
use core_app::AppResult;

pub struct ImageUseCase;

impl ImageUseCase {
  pub async fn upload_and_resize(
    repo: &dyn ImageRepository,
    data: &[u8],          // Dữ liệu ảnh thô
    content_type: &str,   // Loại file (image/jpeg, image/png)
    user_id: i64,         // ID người dùng để tạo tên file
    max_file_size: usize, // Kích thước tối đa (bytes)
    max_width: u32,       // Chiều rộng tối đa để resize
    quality: u8,
  ) -> AppResult<String> {
    repo.upload_and_resize(data, content_type, user_id, max_file_size, max_width, quality).await
  }

  pub async fn remove_old_image(
    repo: &dyn ImageRepository,
    image_path: &str,
  ) -> AppResult<()> {
    repo.remove_old_image(image_path).await
  }
}
