use async_trait::async_trait;
use core_app::AppResult;
use core_app::errors::AppError;
use domain::repositories::image_repository::ImageRepository;
use fast_image_resize::images::Image;
use fast_image_resize::{PixelType, ResizeOptions, Resizer};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::time::Instant;
use turbojpeg::{Image as TJImage, PixelFormat, decompress};
use uuid::Uuid;

pub struct LocalImageService;

#[async_trait]
impl ImageRepository for LocalImageService {
  /// Upload và resize ảnh, trả về đường dẫn file đã lưu
  async fn upload_and_resize(
    &self,
    data: &[u8],          // Dữ liệu ảnh thô
    content_type: &str,   // Loại file (image/jpeg, image/png)
    user_id: i64,         // ID người dùng để tạo tên file
    max_file_size: usize, // Kích thước tối đa (bytes)
    max_width: u32,       // Chiều rộng tối đa để resize
    quality: u8,          // Chất lượng ảnh (0-100)
    sub_dir: &str,        // Thư mục con trong uploads/ (ví dụ: "avatar", "service")
  ) -> AppResult<String> {
    // Kiểm tra kích thước file
    let start = Instant::now();
    if data.is_empty() {
      return Err(AppError::BadRequest("Empty image file".to_string()));
    }
    if data.len() > max_file_size {
      return Err(AppError::BadRequest(format!(
        "File size exceeds {}MB limit",
        max_file_size / (1024 * 1024)
      )));
    }
    let extension = match content_type {
      "image/jpeg" => "jpg",
      "image/png" => "png",
      "image/webp" => "webp",
      _ => {
        return Err(AppError::BadRequest("Only JPG, PNG and WebP files are allowed".to_string()));
      },
    };
    tracing::info!("Processing {} file, size: {} bytes", content_type, data.len());

    // let img = tokio::task::spawn_blocking({
    //   let data = data.to_vec();
    //   move || {
    //     image::load_from_memory(&data)
    //       .map_err(|err| AppError::BadRequest(format!("Failed to load image: {}", err)))
    //   }
    // })
    // .await
    // .map_err(|err| AppError::BadRequest(format!("Failed to spawn blocking task: {}", err)))??;

    // Chia sẻ dữ liệu với Arc
    let data = Arc::new(data.to_vec());

    // Tải ảnh từ dữ liệu thô
    let load_start = Instant::now();
    let img = match content_type {
      "image/jpeg" => tokio::task::spawn_blocking({
        let data = Arc::clone(&data);
        move || {
          let decode_start = Instant::now();
          let tj_image: TJImage<Vec<u8>> = decompress(&data, PixelFormat::RGBA)
            .map_err(|err| AppError::BadRequest(format!("Failed to decompress JPEG: {}", err)))?;
          tracing::info!("JPEG decode time: {:?}", decode_start.elapsed());
          let img = DynamicImage::ImageRgba8(
            image::RgbaImage::from_vec(
              tj_image.width as u32,
              tj_image.height as u32,
              tj_image.pixels,
            )
            .ok_or_else(|| AppError::BadRequest("Failed to create RGBA image".to_string()))?,
          );
          Ok::<_, AppError>(img)
        }
      })
      .await
      .map_err(|err| AppError::BadRequest(format!("Failed to spawn blocking task: {}", err)))??,
      "image/png" | "image/webp" => tokio::task::spawn_blocking({
        let data = Arc::clone(&data);
        move || {
          let decode_start = Instant::now();
          let img = image::load_from_memory(&data)
            .map_err(|err| AppError::BadRequest(format!("Failed to load image: {}", err)))?;
          tracing::info!("Image decode time: {:?}", decode_start.elapsed());
          Ok::<_, AppError>(img)
        }
      })
      .await
      .map_err(|err| AppError::BadRequest(format!("Failed to spawn blocking task: {}", err)))??,
      _ => unreachable!(),
    };
    tracing::info!("Load image took: {:?}", load_start.elapsed());

    // Tính toán kích thước mới (giữ tỷ lệ)
    let (width, height) = img.dimensions();
    tracing::info!(" image dimensions: {}x{}", width, height);
    let resized_width = if width > max_width { max_width } else { width };
    let resized_height = (height as f32 * (resized_width as f32 / width as f32)) as u32;

    let resize_start = Instant::now();

    // let resized_img: DynamicImage = tokio::task::spawn_blocking({
    //   let img = img.clone();
    //   move || {
    //     imageops::resize(&img, resized_width, resized_height, imageops::FilterType::Nearest).into()
    //   }
    // })
    // .await
    // .map_err(|err| AppError::BadRequest(format!("Failed to spawn blocking task: {}", err)))?;

    let resized_img = tokio::task::spawn_blocking({
      let img = img.clone();
      move || {
        // Chuyển đổi sang định dạng RGBA8 cho fast_image_resize
        let src_img = img.into_rgba8();
        let src_image = Image::from_vec_u8(width, height, src_img.into_vec(), PixelType::U8x4)
          .map_err(|err| AppError::BadRequest(format!("Failed to create source image: {}", err)))?;

        // Tạo buffer cho ảnh đích
        let mut dst_image = Image::new(resized_width, resized_height, PixelType::U8x4);

        // Thực hiện resize
        let mut resizer = Resizer::new();
        resizer
          .resize(&src_image, &mut dst_image, &ResizeOptions::new())
          .map_err(|err| AppError::BadRequest(format!("Failed to resize image: {}", err)))?;

        // Chuyển lại thành DynamicImage để encode
        let resized_img = DynamicImage::from(
          image::RgbaImage::from_vec(resized_width, resized_height, dst_image.into_vec())
            .ok_or_else(|| AppError::BadRequest("Failed to create resized image".to_string()))?,
        );

        Ok::<_, AppError>(resized_img)
      }
    })
    .await
    .map_err(|err| AppError::BadRequest(format!("Failed to spawn blocking task: {}", err)))??;
    tracing::info!("Resize image took: {:?}", resize_start.elapsed());

    let (resized_width, resized_height) = resized_img.dimensions();
    if resized_width == 0 || resized_height == 0 {
      return Err(AppError::BadRequest("Resized image has invalid dimensions".to_string()));
    }
    tracing::info!("Resized image dimensions: {}x{}", resized_width, resized_height);
    // Chuyển ảnh đã resize về định dạng JPEG hoặc PNG với chất lượng giảm

    let encode_start = Instant::now();
    let buffer = match extension {
      "jpg" => tokio::task::spawn_blocking({
        let resized_img = resized_img.clone();
        move || -> AppResult<Vec<u8>> {
          let mut buffer = Vec::new();
          let mut cursor = Cursor::new(&mut buffer);
          let mut encoder = JpegEncoder::new_with_quality(&mut cursor, quality);
          encoder
            .encode_image(&resized_img)
            .map_err(|err| AppError::BadRequest(format!("Failed to encode JPEG image: {}", err)))?;
          Ok(buffer)
        }
      })
      .await
      .map_err(|err| AppError::BadRequest(format!("Failed to spawn blocking task: {}", err)))??,
      "png" => tokio::task::spawn_blocking({
        let resized_img = resized_img.clone();
        move || -> AppResult<Vec<u8>> {
          let mut buffer = Vec::new();
          let mut cursor = Cursor::new(&mut buffer);
          resized_img
            .write_to(&mut cursor, ImageFormat::Png)
            .map_err(|err| AppError::BadRequest(format!("Failed to encode PNG image: {}", err)))?;
          Ok(buffer)
        }
      })
      .await
      .map_err(|err| AppError::BadRequest(format!("Failed to spawn blocking task: {}", err)))??,
      "webp" => tokio::task::spawn_blocking({
        let resized_img = resized_img.clone();
        move || -> AppResult<Vec<u8>> {
          let mut buffer = Vec::new();
          let mut cursor = Cursor::new(&mut buffer);
          resized_img
            .write_to(&mut cursor, ImageFormat::WebP)
            .map_err(|err| AppError::BadRequest(format!("Failed to encode WebP image: {}", err)))?;
          Ok(buffer)
        }
      })
      .await
      .map_err(|err| AppError::BadRequest(format!("Failed to spawn blocking task: {}", err)))??,
      _ => unreachable!(),
    };
    tracing::info!("Encode image took: {:?}", encode_start.elapsed());

    // Kiểm tra buffer sau khi encode
    if buffer.is_empty() {
      tracing::error!("Buffer is empty after encoding image (format: {})", extension);
      return Err(AppError::BadRequest("Failed to encode image: buffer is empty".to_string()));
    }
    tracing::info!("Encoded image buffer size: {} bytes", buffer.len());

    // Tạo thư mục uploads/ và thư mục con nếu chưa tồn tại
    let io_start = Instant::now();
    let uploads_dir = Path::new("uploads").join(sub_dir);
    if !uploads_dir.exists() {
      fs::create_dir_all(&uploads_dir)
        .await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;
    }

    // Tạo tên file duy nhất
    let file_name = format!("{}-{}.{}", user_id, Uuid::new_v4(), extension);
    let file_path = uploads_dir.join(&file_name);

    // Lưu ảnh đã resize vào thư mục uploads/sub_dir
    let mut file =
      fs::File::create(&file_path).await.map_err(|err| AppError::BadRequest(err.to_string()))?;
    file.write_all(&buffer).await.map_err(|err| AppError::BadRequest(err.to_string()))?;

    tracing::info!("I/O operations took: {:?}", io_start.elapsed());
    tracing::info!("Total processing time: {:?}", start.elapsed());

    Ok(file_path.to_string_lossy().to_string())
  }

  /// Xóa file ảnh cũ nếu tồn tại
  async fn remove_old_image(
    &self,
    image_path: &str,
  ) -> AppResult<()> {
    if image_path.is_empty() {
      tracing::info!("Old image path is empty, skipping removal");
      return Ok(());
    }

    let path = Path::new(image_path);
    if !path.starts_with("uploads/") {
      tracing::warn!("Invalid image path, must be in uploads/ directory: {:?}", path);
      return Ok(());
    }

    if path.exists() {
      match fs::remove_file(path).await {
        Ok(()) => {
          tracing::info!("Successfully removed old image: {:?}", path);
          Ok(())
        },
        Err(err) => {
          tracing::warn!("Failed to remove old image {:?}: {}", path, err);
          Ok(())
        },
      }
    } else {
      tracing::info!("Old image does not exist: {:?}", path);
      Ok(())
    }
  }
}
