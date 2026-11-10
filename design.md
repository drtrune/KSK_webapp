# Hệ Thống Thiết Kế Giao Diện (Design System Spec)

Tài liệu định dạng và quy chuẩn thiết kế giao diện cho nền tảng video/nội dung tối giản (Minimalist Video Platform).

---

## 1. Phong cách thiết kế tổng quan (Design Concept)
* **Cảm hứng chủ đạo:** YouTube layout (quen thuộc, tối ưu trải nghiệm duyệt nội dung), kết hợp phong cách **Minimalism** (bỏ viền thừa, khoảng trắng thoáng, tập trung vào nội dung).
* **Đặc trưng:**
  * Card bo tròn nhẹ (`border-radius: 8px - 12px`).
  * Đổ bóng cực nhẹ (`box-shadow: subtle / soft elevation`).
  * Hạn chế tối đa các chi tiết trang trí gây xao nhãng.

---

## 2. Bảng màu (Color Palette)

| Vai trò | Tên màu | Mã Hex | Ứng dụng |
| :--- | :--- | :--- | :--- |
| **Chủ đạo (Primary)** | Sky Blue | `#3498DB` | Nút bấm chính (CTA), thanh tiến trình video, active icon, link |
| **Phụ (Secondary / Accent)** | Emerald Green | `#2ECC71` | Nút trạng thái thành công, nhãn "Live", badge nổi bật, nút xác nhận |
| **Màu nền (Background)** | Light Gray | `#F4F6F8` | Nền toàn bộ trang web |
| **Nền phụ (Surface / Card)** | Pure White | `#FFFFFF` | Nền thẻ video, thanh điều hướng (Header/Sidebar), popover |
| **Màu chữ chính (Text Primary)**| Dark Slate | `#2C3E50` | Tiêu đề, văn bản chính, icon |
| **Màu chữ phụ (Text Secondary)**| Muted Slate| `#7F8C8D` | Thông tin phụ (lượt xem, thời gian, tên kênh phụ) |
| **Đường phân cách (Border)** | Soft Border | `#E2E8F0` | Đường chia line mỏng giữa các phần |

---

## 3. Cấu trúc bố cục (Layout Structure)

### 3.1. Thanh trên cùng (Top Navigation Bar - Header)
* **Chiều cao cố định:** `64px`, cố định vị trí (`sticky/fixed top`), nền `#FFFFFF`, bóng mờ phía dưới.
* **Thành phần:**
  * **Trái:** Nút toggle Menu (Hamburger Icon) + Logo nền tảng.
  * **Giữa:** Thanh tìm kiếm tối giản (Search input bo góc 20px, icon kính lúp, nút làm sạch văn bản).
  * **Phải:** Icon thông báo (chuông kèm chấm xanh `#2ECC71`), nút tải lên/tạo mới, avatar tài khoản người dùng & menu cài đặt.

### 3.2. Thanh bên trái (Collapsible Sidebar)
* **Trạng thái mở rộng (240px):**
  * Danh mục: Trang chủ, Khám phá, Kênh đăng ký, Lịch sử, Đã lưu.
  * Danh sách kênh theo dõi rút gọn.
* **Trạng thái thu gọn (Mini Sidebar - 72px):**
  * Chỉ hiển thị biểu tượng (icon) và nhãn chữ siêu ngắn phía dưới icon.

### 3.3. Khu vực trung tâm (Main Content Area)
* **Vị trí:** Co giãn tự động (`flex-1`) theo trạng thái Sidebar.
* **Cấu trúc:**
  * **Dải lọc nhanh (Chip tags bar):** Các nút bo tròn lọc chủ đề (Tất cả, Video mới, Âm nhạc, Tin tức...).
  * **Lưới nội dung (Content Grid):**
    * Desktop: 3 – 4 cột thẻ video.
    * Tablet: 2 cột.
    * Mobile: 1 cột.
  * **Thẻ video (Video Card):** Thumbnail tỉ lệ 16:9 bo góc 8px; bên dưới gồm Avatar kênh, Tiêu đề (tối đa 2 dòng, `#2C3E50`), Tên kênh & Lượt xem/thời gian (`#7F8C8D`).

### 3.4. Chân trang & Thông tin phụ (Footer / Bottom Bar)
* **Bản Desktop:** Dải liên kết phụ tối giản (Điều khoản, Chính sách, Giới thiệu, Bản quyền) nằm gọn dưới đáy danh mục Sidebar hoặc cuối trang.
* **Bản Mobile:** Thanh điều hướng đáy (Bottom Bar - `56px`) gồm 4–5 tab chính (Trang chủ, Khám phá, Đăng ký, Bạn).
* **Khu vực Toast / Thông báo nổi:** Nằm ở góc dưới cùng bên phải (`bottom: 24px`, `right: 24px`) hiển thị trạng thái tác vụ.

---

## 4. Typography & Quy chuẩn hiển thị

* **Font chữ khuyến nghị:** `Inter`, `Roboto`, hoặc `system-ui` (gọn gàng, trung tính, dễ đọc).
* **Cỡ chữ tiêu chuẩn:**
  * H1 / Header Title: `20px - 24px` (Semibold)
  * Card Video Title: `15px - 16px` (Medium)
  * Body Text: `14px` (Regular)
  * Metadata / Subtext: `12px` (Regular)
* **Kích cỡ màn hình:**
  * Desktop: 16:9 ngang
  * Mobile: 18:9 dọc