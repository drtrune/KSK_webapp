# KSK Desktop Native — Hệ thống Quản lý Khám Sức Khỏe (QĐ 3176 / QĐ 1804 BYT)

Ứng dụng desktop viết bằng **Rust + GPUI**, dùng **SQLite** (hỗ trợ chạy chung qua ổ đĩa mạng LAN/SMB) và mã hóa đầu cuối **AES-256-GCM** cho toàn bộ hồ sơ bệnh nhân.

---

## Tính năng nổi bật

- **Khám lẻ & Khám đoàn tách biệt hoàn toàn**:
  - **Khám lẻ**: Tiếp nhận trực tiếp từng hồ sơ tại quầy hoặc nhận đẩy dữ liệu từ HIS Bridge.
  - **Khám đoàn & Hợp đồng**: Module 2 cấp (**Công ty → Hợp đồng khám**) dành cho Phòng kinh doanh / Tiếp nhận / Admin. Hỗ trợ đính kèm file scan hợp đồng (PDF/Ảnh), theo dõi tiến độ khám, và **Import danh sách hành chính từ Excel (`.xlsx`/`.xls`) hoặc `.csv`** (có sẵn Mã tiếp nhận, Mã y tế và 19 trường hành chính) để sinh hàng loạt phiếu khám đoàn.
  - **Chống trộn lẫn**: Sidebar có bộ chuyển **Khám lẻ / Khám đoàn** (khi chọn Khám đoàn phải chọn đúng hợp đồng), header hồ sơ gắn badge phân biệt, và tab Xuất CSV hỗ trợ lọc riêng theo loại khám & hợp đồng.
- **Giao diện khám 9 tab chuẩn BYT**:
  - Hành chính, Tiền sử (đủ 22 bệnh lý theo mẫu + tiền sử sản khoa cho nữ), Thể lực (tự tính BMI, phân loại thể lực/tim mạch/mắt/tai mũi họng), Lâm sàng (cây bệnh tật + rule tự động từ `asset/phan_loai_config.json`), Cận lâm sàng, Kết luận & Ký khóa.
- **In phiếu A4 5 trang giống mẫu web**:
  - Bấm **In Phiếu Tiếp Nhận (A4)** hoặc **In Giấy KSK (A4)** sẽ mở ngay giao diện bảng A4 5 trang kèm mã vạch Code128 và bật hộp thoại in trình duyệt (`window.print()`). Hỗ trợ thêm xuất `.docx` và xuất **CSV 123 cột** chuẩn QĐ 3176/QĐ 1804.
- **Bảo mật & Vận hành**:
  - Mã hóa AES-256-GCM các trường nhạy cảm (`ho_ten`, `ngay_sinh`, `so_cccd`, `payload_json`) bằng Master Key; mỗi tài khoản bọc Master Key riêng bằng mật khẩu đăng nhập + Recovery Key khôi phục cho Admin.
  - Tự động sao lưu (Auto-Backup) SQLite định kỳ về máy cục bộ.
  - Tạo DB mới & chuyển đổi DB trực tiếp trong giao diện Quản trị Admin.

---

## Phân quyền (Roles)

| Role | Tên hiển thị | Phạm vi chính |
|---|---|---|
| `admin` | Quản trị viên | Toàn quyền hệ thống, cấu hình DB/Backup/Update, quản lý người dùng & danh mục |
| `business` | Phòng kinh doanh | Quản lý Công ty, Hợp đồng khám đoàn, Import danh sách & theo dõi tiến độ đoàn |
| `reception` | Tiếp nhận | Tiếp nhận khám lẻ, quản lý khám đoàn, in phiếu tiếp nhận |
| `tiensu`, `bs_theluc`, `bs_noi`, `bs_ngoai`, `bs_san`, `bs_mat`, `bs_tmh`, `bs_rhm`, `bs_dalieu`, `bs_cls`, `bs_ketluan` | Bác sĩ / Chuyên khoa | Khám theo đúng phân hệ chuyên khoa được cấp |

---

## Yêu cầu & Build từ mã nguồn

### Yêu cầu môi trường
- Rust toolchain (`rustup` / `cargo` stable, edition 2021)
- Windows 10/11 (khuyến nghị) để build binary GPUI native (`ksk_webapp.exe`)

### Chạy kiểm thử & Build
```powershell
# Chạy unit tests
cargo test

# Chạy chế độ phát triển
cargo run

# Build bản phát hành (Release)
cargo build --release
```
File thực thi sau khi build nằm tại: `target/release/ksk_webapp.exe`.

---

## Cấu hình & Triển khai nhiều máy trạm (LAN / SMB)

Ứng dụng lưu cấu hình cục bộ tại `config.toml` cạnh file `.exe` (hoặc cấu hình trực tiếp ở màn hình Khởi tạo lần đầu / Quản trị Admin):

- `database_path`: Đường dẫn file `.db` trên ổ đĩa mạng chung (ví dụ: `\\SERVER\KSK_Share\ksk.db` hoặc `Z:\ksk.db`). SQLite sử dụng `JournalMode::Delete` + `Synchronous::Full` + `busy_timeout = 15s` để nhiều máy trạm truy cập đồng thời an toàn.
- `auto_backup_enabled` / `backup_dir_path` / `backup_interval_minutes`: Cấu hình sao lưu tự động về máy trạm.
- `update_dir_path`: Thư mục phát hành bản cập nhật nội bộ trên ổ mạng chung.
