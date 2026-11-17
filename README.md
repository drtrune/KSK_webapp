# Ứng dụng KSK desktop
Ứng dụng desktop quản lý **khám sức khỏe (KSK)** cho cơ sở khám bệnh, chữa bệnh. Phần mềm hỗ trợ tiếp nhận hồ sơ, khám theo từng chuyên khoa, khám đoàn theo hợp đồng, in biểu mẫu và xuất dữ liệu theo định dạng của Bộ Y tế.
Ứng dụng hiện được xây dựng bằng **Rust**, giao diện native **GPUI**, cơ sở dữ liệu **SQLite** và chạy chính trên Windows.
## Chức năng chính
### Quản lý hồ sơ khám
- Tạo và tiếp nhận hồ sơ khám lẻ.
- Nhận người bệnh từ HIS hoặc HIS Bridge.
- Theo dõi trạng thái hồ sơ và người cập nhật cuối.
- Tách rõ hồ sơ **khám lẻ** và **khám đoàn**.
- Hỗ trợ nhiều mẫu phiếu:
  - `TREN_18`: từ đủ 18 tuổi.
  - `LAI_XE`: khám sức khỏe người lái xe.
  - `TU_6_DUOI_18`: từ 6 đến dưới 18 tuổi.
  - `DUOI_6`: trẻ dưới 6 tuổi.
  - `MAU_GIAO`: phiếu kiểm tra sức khỏe mẫu giáo.
  - `HS_PHO_THONG`: phiếu kiểm tra sức khỏe học sinh phổ thông.
### Quy trình khám
Hồ sơ được xử lý theo các phần:
1. Tiếp nhận và hành chính.
2. Tiền sử.
3. Khám thể lực.
4. Khám lâm sàng theo chuyên khoa.
5. Cận lâm sàng.
6. Kết luận.
Các trường dữ liệu được kiểm soát theo role. Bác sĩ chỉ chỉnh sửa phần được phân công; quản trị viên có quyền đầy đủ.
### Khám đoàn và hợp đồng
- Quản lý công ty và hợp đồng khám.
- Lưu ngày khám, số lượng dự kiến, trạng thái và file scan hợp đồng.
- Import danh sách từ `.csv`, `.xlsx` hoặc `.xls`.
- Hỗ trợ chuẩn hóa ngày, giới tính, mã danh mục và ánh xạ tên cột phổ biến.
- Tạo nhiều hồ sơ từ một hợp đồng.
- Lọc, in và xuất dữ liệu theo từng hợp đồng.
- Có thể tách dữ liệu các hợp đồng được chọn sang một file SQLite riêng.
### In và xuất dữ liệu
- Tạo PDF A4 nhiều trang bằng Rust/`printpdf`.
- In phiếu tiếp nhận hoặc giấy khám sức khỏe.
- Tạo mã vạch Code 128.
- Xuất DOCX theo các template trong `asset/`.
- Xuất CSV 123 trường theo QĐ 3176/QĐ-BYT và QĐ 1804/QĐ-BYT.
### Quản trị và vận hành
- Quản lý người dùng, khóa tài khoản và nhiều role trên một tài khoản.
- Quản lý danh mục hành chính, nghề nghiệp, bệnh lý và chuyên môn.
- Tạo database mới hoặc chuyển database đang sử dụng.
- Sao lưu SQLite thủ công hoặc tự động, có dọn các bản sao lưu cũ.
- Kiểm tra và cài đặt bản cập nhật nội bộ từ thư mục được cấu hình.
- Ghi log ứng dụng và panic vào `ksk_app.log`.
## Phân quyền
| Role | Phạm vi |
|---|---|
| `admin` | Toàn quyền hệ thống, người dùng, database, backup, cập nhật và danh mục |
| `business` | Công ty, hợp đồng khám đoàn, import danh sách và tiến độ đoàn |
| `reception` | Tiếp nhận, hành chính, quản lý hồ sơ và in phiếu tiếp nhận |
| `tiensu` | Tiền sử |
| `bs_theluc` | Khám thể lực |
| `bs_noi` | Nội khoa |
| `bs_ngoai` | Ngoại khoa |
| `bs_dalieu` | Da liễu |
| `bs_san` | Sản phụ khoa |
| `bs_mat` | Mắt |
| `bs_tmh` | Tai - Mũi - Họng |
| `bs_rhm` | Răng - Hàm - Mặt |
| `bs_cls` | Cận lâm sàng |
| `bs_ketluan` | Kết luận |
Một tài khoản có thể có nhiều role, được lưu trong trường `users.role` dưới dạng danh sách phân tách bằng dấu phẩy.
## Công nghệ và cấu trúc mã nguồn
- **Runtime:** Rust 2021, Tokio.
- **GUI:** GPUI, `gpui-component`, `gpui-kit`.
- **Database:** SQLite qua SQLx.
- **Import:** CSV và Excel qua `csv` và `calamine`.
- **Bảo mật dữ liệu:** AES-256-GCM, PBKDF2-HMAC-SHA256, Master Key và Recovery Key.
- **Tài liệu xuất:** PDF qua `printpdf`, DOCX qua template ZIP/XML.
- **Tích hợp:** Axum cho các handler HTTP và Local HIS IPC.
Các thư mục chính:
```text
src/
  main.rs             Khởi động runtime, database, IPC và cửa sổ GPUI
  config.rs           Đọc/ghi config.toml và database.toml
  db.rs               Khởi tạo schema, truy vấn, backup và chuyển database
  models.rs           DTO, role, trường dữ liệu và 123 cột CSV
  handlers.rs         Handler HTTP, HIS, truy vấn và xuất dữ liệu
  crypto.rs           Master Key, Recovery Key và mã hóa trường dữ liệu
  group_import.rs     Đọc và chuẩn hóa file import khám đoàn
  gui/                Các màn hình GPUI
  pdf_print/          Bộ dựng PDF A4 và các mẫu phiếu
  templates.rs        Định nghĩa trường theo từng mẫu khám
  updater.rs          Cập nhật ứng dụng
asset/
  seed_catalogs.tsv   Danh mục khởi tạo
  template*.docx      Template DOCX
  phan_loai_config.json  Cấu hình phân loại bệnh
```
Chi tiết luồng và ranh giới module được mô tả trong [`architect.md`](architect.md).
## Yêu cầu
- Rust stable và Cargo.
- Windows 10/11 là môi trường triển khai chính.
- Với nhiều máy trạm: thư mục chứa SQLite phải được chia sẻ qua LAN/SMB và có quyền đọc/ghi.
- Nếu dùng HIS Bridge, ứng dụng bên ngoài phải gửi request đến cổng Local IPC được cấu hình.
## Chạy và build
```bash
# Kiểm tra và chạy test
cargo test
# Chạy ứng dụng ở chế độ phát triển
cargo run
# Build bản phát hành
cargo build --release
```
Trên Windows, binary thường nằm tại:
```text
target/release/ksk_webapp.exe
```
## Cấu hình
Ứng dụng đọc `config.toml` từ thư mục hiện tại; nếu không có thì tìm cạnh file executable. Đường dẫn database được ưu tiên đọc từ `database.toml`.
Các nhóm cấu hình quan trọng:
| Nhóm | Trường |
|---|---|
| Database | `database_path` |
| HIS/IPC | `local_ipc_port`, `his_api_url`, `his_http_method`, `his_auth_header`, `his_api_key`, `his_payload_template` |
| Cơ sở KCB | `ma_cskcb`, `ma_gtin_cskcb`, `ten_cskcb`, `dia_danh` |
| Template | `template_docx_path` |
| Backup | `auto_backup_enabled`, `backup_dir_path`, `backup_interval_minutes`, `backup_keep_count` |
| Update | `update_dir_path`, `auto_check_update` |
Không commit API key, mật khẩu hoặc file database chứa dữ liệu thật vào repository. Sau lần khởi tạo đầu tiên, cần đổi thông tin tài khoản mặc định và cấu hình lại các secret theo môi trường triển khai.
## Local HIS IPC
Ứng dụng mở HTTP listener trên `127.0.0.1:<local_ipc_port>` và cung cấp các endpoint nội bộ:
```text
POST /api/his/push
POST /api/his/mock
```
HIS Bridge có thể đẩy mã bệnh nhân/mã tiếp nhận vào ứng dụng. Khi nhận được dữ liệu hợp lệ, giao diện sẽ tìm hoặc mở hồ sơ tương ứng.
## Kiểm thử
Các test hiện có tập trung vào:
- Mã hóa/giải mã trường và bọc Master Key.
- Tạo giá trị mẫu cho các biểu mẫu PDF.
- Sinh mã vạch và PDF cho các template.
- Chuẩn hóa dữ liệu import.
- Các helper phân quyền và điều hướng GUI.
## Tài liệu liên quan
- [`architect.md`](architect.md): kiến trúc và luồng dữ liệu.
- [`CSV_export_docs.md`](CSV_export_docs.md): đặc tả 123 trường CSV.
- [`change.md`](change.md): lịch sử thay đổi.
- [`asset/`](asset/): template, danh mục và dữ liệu hỗ trợ.
