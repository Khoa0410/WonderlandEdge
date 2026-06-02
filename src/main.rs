use axum::{
    Router,
    extract::Multipart,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::post,
};
use image::{GrayImage, ImageFormat, load_from_memory};
use imageproc::filter::gaussian_blur_f32;
use rayon::prelude::*;
use std::io::Cursor;

struct XDogParams {
    s1: f32,
    s2: f32,
    t: f32,
    phi: f32,
}

impl Default for XDogParams {
    fn default() -> Self {
        Self {
            s1: 0.5,
            s2: 3.0,
            t: 0.98,
            phi: 10.0,
        }
    }
}

async fn process_image(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut img_bytes = None;
    let mut params = XDogParams::default();

    // 1. (I/O BOUND) Đọc dữ liệu nhanh bằng Tokio
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "image" {
            let bytes = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            img_bytes = Some(bytes);
        } else if name == "s1" {
            let text = field
                .text()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            if let Ok(val) = text.parse::<f32>() {
                params.s1 = val;
            }
        } else if name == "s2" {
            let text = field
                .text()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            if let Ok(val) = text.parse::<f32>() {
                params.s2 = val;
            }
        } else if name == "t" {
            let text = field
                .text()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            if let Ok(val) = text.parse::<f32>() {
                params.t = val;
            }
        } else if name == "phi" {
            let text = field
                .text()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            if let Ok(val) = text.parse::<f32>() {
                params.phi = val;
            }
        }
    }

    let bytes = img_bytes.ok_or((StatusCode::BAD_REQUEST, "Thiếu file ảnh!".to_string()))?;

    // 2. CHUYỂN GIAO SANG BLOCKING THREAD POOL
    let result = tokio::task::spawn_blocking(move || {
        // Parse ảnh trong blocking thread để không chặn Tokio
        let img = load_from_memory(&bytes).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        let img_rgba = img.to_rgba8();
        let img_gray = img.to_luma8();
        let (width, height) = img_rgba.dimensions();

        // Chạy Gaussian Blur
        let blur1 = gaussian_blur_f32(&img_gray, params.s1);
        let blur2 = gaussian_blur_f32(&img_gray, params.s2);

        // Tạo buffer kết quả trống
        let mut output = GrayImage::new(width, height);

        // 3. (CPU BOUND + PARALLEL) SỬ DỤNG RAYON
        // Lấy dữ liệu dạng mảng phẳng (flat array) để Rayon dễ chia nhỏ
        let out_pixels = &mut *output;
        let rgba_pixels = img_rgba.as_raw();
        let b1_pixels = blur1.as_raw();
        let b2_pixels = blur2.as_raw();

        // Xử lý song song từng pixel bằng par_iter_mut()
        // zipping các mảng lại với nhau để đọc ghi đồng thời
        out_pixels
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, out_pixel)| {
                // Lấy index kênh alpha của pixel thứ i trong mảng RGBA (1 pixel = 4 bytes)
                let alpha = rgba_pixels[i * 4 + 3];

                if alpha < 10 {
                    *out_pixel = 0; // Transparent -> nền đen
                    return;
                }

                let p1 = b1_pixels[i] as f32;
                let p2 = b2_pixels[i] as f32;
                let difference = p2 * params.t - p1;

                if difference > 0.0 {
                    let edge_intensity = (difference * params.phi).clamp(0.0, 255.0) as u8;
                    *out_pixel = edge_intensity;
                } else {
                    *out_pixel = 0;
                }
            });

        // 4. Encode ảnh song song
        let mut buffer = Cursor::new(Vec::new());
        output
            .write_to(&mut buffer, ImageFormat::Png)
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Lỗi encode ảnh".to_string(),
                )
            })?;

        Ok::<Vec<u8>, (StatusCode, String)>(buffer.into_inner())
    })
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Lỗi Thread Pool".to_string(),
        )
    })??;

    // 5. Trả về cho Client
    let headers = [(header::CONTENT_TYPE, "image/png")];
    Ok((headers, result))
}

#[tokio::main]
async fn main() {
    // Cấu hình router cho Axum
    let app = Router::new().route("/process", post(process_image));

    // Chạy server Axum lắng nghe cổng 3001
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("Server đang chạy tại http://localhost:3001");
    axum::serve(listener, app).await.unwrap();
}
