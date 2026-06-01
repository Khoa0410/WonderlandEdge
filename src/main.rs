use image::{GrayImage, open};
use imageproc::edges::canny;

fn main() {
    // 1. Đọc file ảnh gốc (bạn để ảnh Yae Miko vào cùng thư mục chạy)
    let img_path = "yaemiko.jpg";
    let img = open(img_path).expect("Lỗi: Không tìm thấy file ảnh gốc!");

    // 2. Chuyển đổi ảnh sang hệ màu xám (Grayscale)
    let grayscale = img.to_luma8();

    // 3. Áp dụng thuật toán Canny Edge Detection
    // Tham số 1 (low_threshold): 50.0
    // Tham số 2 (high_threshold): 100.0
    // Tùy chỉnh hai tham số này để quyết định độ chi tiết của nét vẽ
    let edges: GrayImage = canny(&grayscale, 10.0, 30.0);

    // Thuật toán Canny mặc định sẽ trả về các nét màu Trắng trên nền Đen (True Black).
    // Đây chính xác là định dạng hoàn hảo nhất để tối ưu pin cho màn hình AMOLED của Watch 6!

    // 4. Xuất file kết quả
    let output_path = "yaemiko_v2.png";
    edges
        .save(output_path)
        .expect("Lỗi: Không thể lưu file output!");

    println!("Xử lý thành công! File đã được lưu tại: {}", output_path);
}
