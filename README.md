# KSK Webapp — Hệ thống Quản lý Khám Sức Khỏe (KSK)

Ứng dụng web nội bộ xây dựng bằng **Rust (Axum + Tokio + SQLite)** kết hợp giao diện web nhúng (`rust-embed` với **Alpine.js** & **Tailwind CSS**), phục vụ quản lý **khám sức khỏe (KSK)** cho cơ sở khám bệnh, chữa bệnh trên máy tính, tablet và điện thoại qua trình duyệt.

> Tài liệu chi tiết:
> - **Hướng dẫn cài đặt & triển khai:** [`SETUP.md`](SETUP.md)
> - **Hướng dẫn sử dụng cho nhân viên y tế & quản trị viên:** [`userguide.md`](userguide.md)

---

## Chức năng chính

### 1. Quản lý hồ sơ khám (Khám lẻ & Khám đoàn)
- Tiếp nhận hồ sơ khám lẻ, nhận người bệnh từ HIS hoặc HIS Bridge (`his_tiep_nhan_bridge.ahk`).
- Tách biệt rõ **Khám lẻ** và **Khám đoàn** theo hợp đồng công ty.
- Theo dõi trạng thái hồ sơ (Chưa khóa / Đã ký khóa) và người cập nhật cuối.
- Hỗ trợ 6 mẫu biểu mẫu khám sức khỏe:
  - `TREN_18`: Khám sức khỏe định kỳ / người từ đủ 18 tuổi.
  - `LAI_XE`: Khám sức khỏe người lái xe.
  - `TU_6_DUOI_18`: Khám sức khỏe trẻ từ 6 đến dưới 18 tuổi.
  - `DUOI_6`: Khám sức khỏe trẻ dưới 6 tuổi.
  - `MAU_GIAO`: Phiếu kiểm tra sức khỏe trẻ mẫu giáo.
  - `HS_PHO_THONG`: Phiếu kiểm tra sức khỏe học sinh phổ thông.

### 2. Quy trình khám & Phân quyền chặt chẽ
- Hồ sơ chia theo các phân hệ: **Tiếp nhận** → **Tiền sử** → **Thể lực** → **Lâm sàng** (Nội, Ngoại, Da liễu, Sản, Mắt, TMH, RHM) → **Cận lâm sàng** → **Kết luận**.
- Kiểm soát quyền chỉnh sửa ở **cả Frontend và Backend**: khóa input/nút sửa khi sai role ở giao diện và kiểm tra `section` + từng trường dữ liệu (`merge_payload_by_roles`) tại API backend.
- Hỗ trợ **Bác sĩ Nhi khoa (`bs_nhi`)**: có quyền khám và lưu các mục Tiền sử, Thể lực, Lâm sàng, Kết luận cho 4 biểu mẫu nhi khoa (`DUOI_6`, `TU_6_DUOI_18`, `MAU_GIAO`, `HS_PHO_THONG`).
- Giao diện tối ưu responsive cho **PC, Tablet và Mobile** (drawer menu, thanh điều hướng chuyên khoa, bảng và form tự động thích ứng).

### 3. Khám đoàn & Hợp đồng
- Quản lý công ty, hợp đồng khám đoàn, file scan đính kèm, ngày khám và số lượng dự kiến.
- Import danh sách hàng loạt từ `.csv`, `.xlsx`, `.xls` với preview, chuẩn hóa ngày/giới tính/mã danh mục và phát hiện trùng mã tiếp nhận.
- Theo dõi tiến độ khám của từng hợp đồng, lọc và in hàng loạt.

### 4. In ấn & Xuất dữ liệu (CSV 123 trường & REST API Fetch)
- Tạo PDF A4 nhiều trang bằng Rust (`printpdf`) kèm mã vạch Code 128; xuất DOCX theo template trong `asset/`.
- Phân hệ **Xuất dữ liệu** hỗ trợ 2 chế độ:
  1. **Xuất file CSV**: 123 trường chuẩn theo QĐ 3176/QĐ-BYT & QĐ 1804/QĐ-BYT (xem [`CSV_export_docs.md`](CSV_export_docs.md)).
  2. **Cấu hình API (App ngoài Fetch)**: Mở endpoint `GET /api/export/records` (bật/tắt, bảo vệ bằng API Key qua header/Bearer/query, lọc trạng thái, phân trang `limit`/`offset`, hỗ trợ `format=json` hoặc `format=csv`) để các hệ thống/app khác fetch dữ liệu hồ sơ đã giải mã.

### 5. Quản trị, Danh mục & Bảo mật
- Mã hóa dữ liệu nhạy cảm bằng **AES-256-GCM**, **Master Key** & **Recovery Key** (PBKDF2-HMAC-SHA256).
- Quản lý tài khoản đa role (một tài khoản có thể gán nhiều role), khóa/mở khóa tài khoản.
- Quản lý danh mục hành chính/chuyên môn và **Phân loại bệnh đa cấp** (cho phép thêm/sửa trực tiếp trên giao diện, lưu vào `phan_loai_config.json` / database).
- Tự động sao lưu SQLite (`auto_backup_enabled`) và ghi log chi tiết ra `ksk_app.log`.

---

## Danh sách Role phân quyền

| Role | Phạm vi quyền hạn |
|---|---|
| `admin` | Toàn quyền hệ thống, tài khoản, database, backup, mở khóa hồ sơ, danh mục |
| `business` | Quản lý công ty, hợp đồng khám đoàn, import danh sách |
| `reception` | Tiếp nhận, hành chính, tạo/tìm hồ sơ, in phiếu tiếp nhận |
| `tiensu` | Tab Tiền sử |
| `bs_theluc` | Tab Khám thể lực |
| `bs_noi` | Khám Nội khoa |
| `bs_ngoai` | Khám Ngoại khoa |
| `bs_dalieu` | Khám Da liễu |
| `bs_san` | Khám Sản phụ khoa |
| `bs_mat` | Khám Mắt |
| `bs_tmh` | Khám Tai - Mũi - Họng |
| `bs_rhm` | Khám Răng - Hàm - Mặt |
| `bs_nhi` | Bác sĩ Nhi khoa (Tiền sử, Thể lực, Lâm sàng, Kết luận cho 4 mẫu nhi khoa) |
| `bs_cls` | Tab Cận lâm sàng |
| `bs_ketluan` | Tab Kết luận & Ký/Khóa hồ sơ |

---

## Cấu trúc mã nguồn chính

```text
src/
  main.rs           Khởi động Axum HTTP server, logging (ksk_app.log), nhúng UI, auto-backup
  config.rs         Đọc/ghi config.toml và database.toml
  db.rs             Khởi tạo schema SQLite, seed tài khoản/danh mục mặc định, backup/restore
  models.rs         DTO, phân quyền role/field (can_user_roles_edit_field), 123 cột CSV
  handlers.rs       API handlers: Auth, hồ sơ, khám đoàn, HIS, xuất CSV & Export API Push
  crypto.rs         AES-256-GCM, Master Key & Recovery Key
  group_import.rs   Parse & chuẩn hóa CSV/Excel khi import khám đoàn
  templates.rs      Định nghĩa trường theo 6 biểu mẫu khám
  pdf_print/        Sinh PDF A4 & barcode Code 128
  updater.rs        Kiểm tra/cập nhật nội bộ
ui/
  index_header.html, index_footer.html, tabs/*.html   Giao diện HTML (Alpine.js + Tailwind)
  js/app.js                                           Logic frontend, phân quyền UI, API client
asset/
  seed_catalogs.tsv, template*.docx, phan_loai_config.json
```

---

## Khởi chạy nhanh (Dành cho Developer)

```bash
# 1. Kiểm tra và chạy unit tests
cargo test

# 2. Chạy ở chế độ phát triển
cargo run

# 3. Build bản phát hành (Windows: target/release/ksk_webapp.exe)
cargo build --release
```

Mặc định server chạy tại `http://127.0.0.1:18080` (cấu hình trong `config.toml`). Xem hướng dẫn triển khai chi tiết tại [`SETUP.md`](SETUP.md).

---

## Tài liệu liên quan
- [`SETUP.md`](SETUP.md): Hướng dẫn cài đặt, cấu hình `config.toml`, triển khai LAN và xử lý sự cố.
- [`userguide.md`](userguide.md): Hướng dẫn sử dụng đầy đủ các phân hệ.
- [`CSV_export_docs.md`](CSV_export_docs.md): Đặc tả 123 trường xuất CSV theo QĐ 3176 / QĐ 1804.
- [`ARCHITECT.MD`](ARCHITECT.MD): Kiến trúc hệ thống.
- [`change.md`](change.md): Lịch sử thay đổi.
