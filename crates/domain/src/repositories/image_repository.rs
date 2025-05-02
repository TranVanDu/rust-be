use core_app::AppResult;

#[async_trait::async_trait]
pub trait ImageRepository: Send + Sync {
  /// Upload và resize ảnh, trả về đường dẫn file đã lưu
  async fn upload_and_resize(
    &self,
    data: &[u8],          // Dữ liệu ảnh thô
    content_type: &str,   // Loại file (image/jpeg, image/png)
    user_id: i64,         // ID người dùng để tạo tên file
    max_file_size: usize, // Kích thước tối đa (bytes)
    max_width: u32,       // Chiều rộng tối đa để resize
    quality: u8,          // Chất lượng ảnh (0-100)
  ) -> AppResult<String>;

  /// Xóa file ảnh cũ nếu tồn tại
  async fn remove_old_image(
    &self,
    image_path: &str,
  ) -> AppResult<()>;
}
