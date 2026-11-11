# Lịch Sử Cập Nhật & Thay Đổi (Changelog)

## Phiên bản 1.2.0 (2026-11-11)
### 1. Tách chuyên khoa khám lâm sàng thành các Sub-tab nhỏ (`ui/tabs/clinical.html`, `ui/index.html`):
- Thiết kế thanh Sub-tabs chuyên khoa theo chuẩn Minimalist YouTube-style Chips (`design.md`):
  - 1. Nội khoa (`clinicalSubTab = 'noi'`)
  - 2. Ngoại khoa (`clinicalSubTab = 'ngoai'`)
  - 3. Da liễu (`clinicalSubTab = 'dalieu'`)
  - 4. Sản phụ khoa (`clinicalSubTab = 'san'`) - tự động ẩn nếu giới tính là Nam (`GIOI_TINH = 1`).
  - 5. Mắt (`clinicalSubTab = 'mat'`)
  - 6. Tai - Mũi - Họng (`clinicalSubTab = 'tmh'`)
  - 7. Răng - Hàm - Mặt (`clinicalSubTab = 'rhm'`)
  - 👁 Xem tất cả (`clinicalSubTab = 'all'`) để xem toàn bộ 7 chuyên khoa trên 1 trang khi cần đối chiếu.
- Mỗi tab con chỉ hiển thị nội dung chuyên khoa tương ứng, không bị cuộn trang dài, tăng tính tập trung cho bác sĩ khám.

### 2. Xóa ô nhập tên bác sĩ khám, tự động lấy theo tên user (`ui/tabs/clinical.html`, `ui/js/app.js`, `src/handlers.rs`):
- Đã xóa toàn bộ các ô input gõ tay tên bác sĩ ở 7 chuyên khoa: `BS_NOI`, `BS_NGOAI`, `BS_DA_LIEU`, `BS_SAN`, `BS_MAT`, `BS_TMH`, `BS_RHM`.
- Thay thế bằng nhãn thông tin tinh gọn (Badge/Pill) hiển thị tên bác sĩ phụ trách.
- Bổ sung hàm `autoFillDoctorNames()` trong `ui/js/app.js`: Tự động gán `currentUser.full_name` vào các trường `BS_*` tương ứng với quyền của tài khoản trước khi lưu.
- Cập nhật backend `src/handlers.rs`: Đảm bảo khi tài khoản có quyền chuyên khoa gửi lưu dữ liệu, backend tự động cập nhật trường `BS_*` theo `user.full_name`.

### 3. Tự động điều hướng trực tiếp đến tab/chuyên khoa được phân quyền (`ui/js/app.js`):
- Bổ sung hàm `getDestinationTabForUser()` và `navigateToUserDefaultTab()`:
  - Phân tích danh sách vai trò (`user.role`) của tài khoản.
  - Tự động nhảy thẳng đến tab/chuyên khoa tương ứng, không mặc định ở Tiếp nhận.
  - Nếu tài khoản được phân nhiều chuyên khoa (Multi-role), hệ thống ưu tiên nhảy vào chuyên khoa đầu tiên trong danh sách quyền.
- Tự động kích hoạt khi:
  - Đăng nhập thành công (`handleLogin`)
  - Mở ứng dụng đã có phiên đăng nhập (`loadMe`)
  - Chọn một hồ sơ bệnh nhân từ danh sách (`selectRecord`)
  - Tra cứu xong hồ sơ từ CSDL hoặc HIS (`handleSearchOrHis`)
  - Bấm nút làm mới / reset hồ sơ (`resetForm`)

