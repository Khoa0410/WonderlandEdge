# Tài liệu Hướng dẫn Tích hợp API - Wonderland Edge

Tài liệu này mô tả chi tiết API xử lý ảnh (thuật toán **XDoG - Extended Difference of Gaussians** để tách nét/biên ảnh) được phát triển bằng Rust (sử dụng thư viện `axum`, `tokio` và `rayon` để tối ưu hóa hiệu năng).

Mã nguồn của API được định nghĩa tại file [main.rs](file:///c:/Code/Projects/wonderland_edge/src/main.rs).

---

## 1. Thông tin chung (General Info)

- **Base URL:** `http://localhost:3001` (Hoặc địa chỉ IP của server chạy dịch vụ)
- **Cổng mặc định:** `3001`
- **Mã nguồn chính:** [src/main.rs](file:///c:/Code/Projects/wonderland_edge/src/main.rs)
- **Phương thức xử lý:** Bất đồng bộ (Async I/O) kết hợp Thread Pool (`rayon` song song hóa xử lý mức pixel).

---

## 2. API Endpoint: Xử lý Ảnh (`/process`)

Endpoint này nhận vào một ảnh gốc cùng các tham số cấu hình thuật toán XDoG và trả về trực tiếp dữ liệu nhị phân (binary) của ảnh đã xử lý dưới định dạng PNG.

- **URL:** `/process`
- **Method:** `POST`
- **Headers:** `Content-Type: multipart/form-data`

### Tham số Request (Multipart Form Data)

| Tên trường (Field) | Kiểu dữ liệu | Bắt buộc | Giá trị mặc định | Mô tả                                                                                                         |
| :----------------- | :----------- | :------- | :--------------- | :------------------------------------------------------------------------------------------------------------ |
| `image`            | `File`       | **Có**   | _N/A_            | File ảnh cần xử lý (Hỗ trợ PNG, JPEG, JPG...).                                                                |
| `s1`               | `float`      | Không    | `0.5`            | Tham số $\sigma_1$ (Gaussian Blur lần 1). Điều khiển độ chi tiết nhỏ của nét vẽ. Giá trị nhỏ giữ nét mịn hơn. |
| `s2`               | `float`      | Không    | `3.0`            | Tham số $\sigma_2$ (Gaussian Blur lần 2). Điều khiển cấu trúc nét lớn hơn.                                    |
| `t`                | `float`      | Không    | `0.98`           | Ngưỡng so sánh độ lệch (threshold factor). Quyết định độ nhạy khi nhận diện nét.                              |
| `phi`              | `float`      | Không    | `10.0`           | Hệ số khuếch đại cường độ nét (scaling factor). Tăng giá trị này giúp nét đậm và tương phản rõ hơn.           |

---

## 3. Định dạng phản hồi (Response Format)

### Thành công (HTTP Status: `200 OK`)

- **Headers:** `Content-Type: image/png`
- **Body:** Binary Stream chứa dữ liệu ảnh PNG đã được chuyển đổi thành nét vẽ (các pixel trong suốt/nền sẽ tự động chuyển thành màu đen, nét vẽ có màu xám/trắng tương ứng với biên độ nét).

### Thất bại

Các mã lỗi phổ biến trả về dạng text/plain để tiện debug:

- **HTTP Status: `400 Bad Request`**
  - Trả về khi thiếu trường `image` hoặc định dạng file ảnh gửi lên không hợp lệ.
  - Phản hồi mẫu: `Thiếu file ảnh!` hoặc chi tiết lỗi parse ảnh.
- **HTTP Status: `500 Internal Server Error`**
  - Lỗi phát sinh trong quá trình encode ảnh hoặc lỗi luồng xử lý phía backend.
  - Phản hồi mẫu: `Lỗi encode ảnh` hoặc `Lỗi Thread Pool`.

---

## 4. Hướng dẫn tích hợp từ Frontend (Frontend Examples)

Dưới đây là một số cách gọi API phổ biến từ phía Frontend (React / Vue / Vanilla JavaScript).

### Cách 1: Sử dụng Vanilla JS `fetch` (Tạo URL xem trước trực tiếp)

```javascript
async function processImage(
  imageFile,
  s1 = 0.5,
  s2 = 3.0,
  t = 0.98,
  phi = 10.0,
) {
  const formData = new FormData();
  formData.append("image", imageFile);
  formData.append("s1", s1);
  formData.append("s2", s2);
  formData.append("t", t);
  formData.append("phi", phi);

  try {
    const response = await fetch("http://localhost:3001/process", {
      method: "POST",
      body: formData, // Trình duyệt sẽ tự động thiết lập Content-Type với boundary phù hợp
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Server error: ${errorText}`);
    }

    // Nhận kết quả dưới dạng Blob
    const blob = await response.blob();

    // Tạo URL tạm thời để hiển thị trực tiếp lên thẻ <img>
    const imageUrl = URL.createObjectURL(blob);
    return imageUrl;
  } catch (error) {
    console.error("Lỗi khi gọi API xử lý ảnh:", error);
    throw error;
  }
}
```

### Cách 2: Tích hợp trong dự án Next.js (App Router)

Trong Next.js, bạn có thể gọi trực tiếp API từ phía Client (Browser) hoặc chuyển tiếp qua một API Route Handler phía Server của Next.js để bảo mật thông tin Server Rust và tránh các vấn đề liên quan tới CORS.

#### Lựa chọn A: Sử dụng Next.js Route Handler (Khuyên Dùng)

Cách này giúp định tuyến tất cả các request qua Server Next.js, che giấu cổng thực tế (`3001`) của backend Rust.

##### 1. Định nghĩa Route Handler tại `app/api/process/route.ts` hoặc `app/api/process/route.js`:

```typescript
import { NextRequest, NextResponse } from "next/server";

export async function POST(request: NextRequest) {
  try {
    const data = await request.formData();

    // Chuyển tiếp (Proxy) FormData trực tiếp sang Rust Backend
    const backendResponse = await fetch("http://localhost:3001/process", {
      method: "POST",
      body: data, // Node.js fetch trên Next.js hỗ trợ chuyển tiếp FormData
    });

    if (!backendResponse.ok) {
      const errorText = await backendResponse.text();
      return new NextResponse(errorText, { status: backendResponse.status });
    }

    const imageBuffer = await backendResponse.arrayBuffer();

    // Trả về ảnh PNG đã xử lý trực tiếp cho Client
    return new NextResponse(Buffer.from(imageBuffer), {
      headers: {
        "Content-Type": "image/png",
        "Cache-Control": "no-store, max-age=0",
      },
    });
  } catch (error: any) {
    console.error("Lỗi API Route Proxy:", error);
    return new NextResponse(error.message || "Lỗi kết nối Backend Rust", {
      status: 500,
    });
  }
}
```

##### 2. Tạo Client Component gọi API tại `app/page.tsx`:

```tsx
"use client";

import React, { useState, useEffect } from "react";

export default function EdgeDetectorPage() {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [processedUrl, setProcessedUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  // Params thuật toán XDoG
  const [s1, setS1] = useState(0.5);
  const [s2, setS2] = useState(3.0);
  const [t, setT] = useState(0.98);
  const [phi, setPhi] = useState(10.0);

  // Giải phóng URL tránh rò rỉ bộ nhớ
  useEffect(() => {
    return () => {
      if (processedUrl) {
        URL.revokeObjectURL(processedUrl);
      }
    };
  }, [processedUrl]);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      setSelectedFile(e.target.files[0]);
    }
  };

  const handleUpload = async () => {
    if (!selectedFile) return;
    setLoading(true);

    const formData = new FormData();
    formData.append("image", selectedFile);
    formData.append("s1", s1.toString());
    formData.append("s2", s2.toString());
    formData.append("t", t.toString());
    formData.append("phi", phi.toString());

    try {
      // Gọi qua Route Handler cục bộ của Next.js
      const response = await fetch("/api/process", {
        method: "POST",
        body: formData,
      });

      if (!response.ok) {
        const errText = await response.text();
        throw new Error(errText || "Xử lý ảnh thất bại.");
      }

      // Nhận nhị phân của ảnh PNG
      const blob = await response.blob();

      // Tạo Object URL mới
      if (processedUrl) {
        URL.revokeObjectURL(processedUrl);
      }
      const url = URL.createObjectURL(blob);
      setProcessedUrl(url);
    } catch (err) {
      alert(err instanceof Error ? err.message : "Đã có lỗi xảy ra");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div
      style={{ padding: "30px", maxWidth: "700px", fontFamily: "sans-serif" }}
    >
      <h1>Next.js XDoG Edge Detector</h1>

      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: "15px",
          marginTop: "20px",
        }}
      >
        <div>
          <label style={{ display: "block", fontWeight: "bold" }}>
            Độ chi tiết nét (s1): {s1}
          </label>
          <input
            type="range"
            min="0.1"
            max="2.0"
            step="0.1"
            value={s1}
            onChange={(e) => setS1(parseFloat(e.target.value))}
            style={{ width: "100%" }}
          />
        </div>

        <div>
          <label style={{ display: "block", fontWeight: "bold" }}>
            Cấu trúc nét rộng (s2): {s2}
          </label>
          <input
            type="range"
            min="1.0"
            max="10.0"
            step="0.5"
            value={s2}
            onChange={(e) => setS2(parseFloat(e.target.value))}
            style={{ width: "100%" }}
          />
        </div>

        <div>
          <label style={{ display: "block", fontWeight: "bold" }}>
            Ngưỡng nét (t): {t}
          </label>
          <input
            type="range"
            min="0.80"
            max="1.0"
            step="0.01"
            value={t}
            onChange={(e) => setT(parseFloat(e.target.value))}
            style={{ width: "100%" }}
          />
        </div>

        <div>
          <label style={{ display: "block", fontWeight: "bold" }}>
            Cường độ nét (phi): {phi}
          </label>
          <input
            type="range"
            min="1.0"
            max="30.0"
            step="1.0"
            value={phi}
            onChange={(e) => setPhi(parseFloat(e.target.value))}
            style={{ width: "100%" }}
          />
        </div>
      </div>

      <div style={{ margin: "25px 0" }}>
        <input type="file" accept="image/*" onChange={handleFileChange} />
        <button
          onClick={handleUpload}
          disabled={loading || !selectedFile}
          style={{ padding: "8px 16px", cursor: "pointer" }}
        >
          {loading ? "Đang xử lý..." : "Bắt đầu xử lý"}
        </button>
      </div>

      {processedUrl && (
        <div style={{ marginTop: "20px" }}>
          <h3>Kết quả:</h3>
          {/* 
            LƯU Ý: Dùng thẻ <img> chuẩn thay vì Next.js <Image /> khi hiển thị blob URL tạm thời 
            để tránh lỗi tối ưu hóa hình ảnh SSR hoặc cấu hình domains của Next.js.
          */}
          <img
            src={processedUrl}
            alt="Processed Edge"
            style={{
              maxWidth: "100%",
              border: "1px solid #ccc",
              borderRadius: "4px",
            }}
          />
          <br />
          <a href={processedUrl} download="edge_detected.png">
            <button
              style={{
                marginTop: "12px",
                padding: "8px 16px",
                cursor: "pointer",
              }}
            >
              Tải ảnh xuống
            </button>
          </a>
        </div>
      )}
    </div>
  );
}
```

---

#### Lựa chọn B: Gọi trực tiếp từ Client Component & Sử dụng Next.js Rewrite Rules

Nếu bạn không muốn viết API Route Handler trung gian mà muốn Client gọi thẳng đến `http://localhost:3001/process` mà không bị lỗi CORS trong môi trường phát triển:

##### Cấu hình `next.config.mjs` hoặc `next.config.js`:

```javascript
/** @type {import('next').NextConfig} */
const nextConfig = {
  async rewrites() {
    return [
      {
        source: "/api/process",
        destination: "http://localhost:3001/process", // Chuyển tiếp request client-side đến Rust Server
      },
    ];
  },
};

export default nextConfig;
```

Sau khi cấu hình, tại Client Component bạn có thể gọi thẳng endpoint `/api/process`:

```javascript
const response = await fetch("/api/process", {
  method: "POST",
  body: formData,
});
```

---

## 5. Lưu ý quan trọng cho nhà phát triển Frontend Next.js

1. **Thẻ `<img>` và Next.js `<Image />`:**
   Khi hiển thị các ảnh được tạo bằng `URL.createObjectURL(blob)`, nên dùng thẻ `<img>` tiêu chuẩn thay vì thành phần `<Image />` của Next.js. Lý do là thành phần `<Image />` sẽ cố gắng chạy bộ tối ưu ảnh (image optimization) ở server-side, gây lỗi hoặc cảnh báo do URL `blob:` không khả dụng ở server Next.js.

2. **Thiết lập Content-Type:**
   Không thiết lập thủ công header `Content-Type: multipart/form-data`. Khi truyền đối tượng `FormData` trực tiếp vào `fetch` hoặc `axios`, trình duyệt (hoặc Node.js runtime) sẽ tự động thiết lập Content-Type kèm chuỗi `boundary`.

3. **Quản lý bộ nhớ (Memory Leaks):**
   Trong môi trường React/Next.js có Hot Reload, các Object URL cũ có thể tồn tại mãi mãi trong phiên làm việc của trình duyệt nếu không được giải phóng. Hãy sử dụng hàm `URL.revokeObjectURL(url)` trong hook cleanup của `useEffect` như ví dụ trên.
