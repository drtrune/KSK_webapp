# Hệ thống Quản lý Khám Sức Khỏe (KSK_webapp)

Ứng dụng web quản lý quy trình **Khám Sức Khỏe (KSK)** đóng gói dạng **Single Binary** (Rust + Axum + SQLite + Alpine.js), hỗ trợ đầy đủ từ khâu tiếp nhận, khám chuyên khoa, in phiếu khám / xuất file Word, đến xuất dữ liệu CSV 123 cột theo chuẩn QĐ 3176.

---

## Tính năng chính

- **1. Tiếp nhận hành chính & Tích hợp HIS**:
  - Tra cứu / lấy thông tin hành chính từ API HIS (hỗ trợ chế độ Mock HIS để thử nghiệm) hoặc quét mã tiếp nhận.
  - Chọn danh mục Tỉnh/TP, Xã/Phường, Dân tộc, Nghề nghiệp, Đối tượng KSK, Nguồn chi trả, Nơi cấp CCCD, Nhóm máu (hỗ trợ tìm kiếm nhanh với Tom Select).
- **2. Tiền sử bệnh**: Khai báo tiền sử gia đình và 22 bệnh tiền sử bản thân, hỗ trợ chọn mã bệnh ICD-10.
- **3. Khám thể lực**: Nhập chiều cao, cân nặng (tự tính BMI), mạch, huyết áp (tự động hỗ trợ chấm điểm/phân loại tim mạch) và phân loại thể lực.
- **4. Khám lâm sàng (7 chuyên khoa)**:
  - Tách gọn thành các sub-tab: *Nội khoa, Ngoại khoa, Da liễu, Sản phụ khoa (tự ẩn khi giới tính Nam), Mắt, Tai - Mũi - Họng, Răng - Hàm - Mặt* và chế độ *Xem tất cả*.
  - Tự động gán tên bác sĩ phụ trách theo tài khoản đăng nhập (`currentUser.full_name`).
  - Tự động điều hướng người dùng đến đúng chuyên khoa được phân quyền sau khi đăng nhập / chọn hồ sơ.
- **5. Cận lâm sàng & 6. Kết luận**:
  - Nhập kết quả xét nghiệm / chẩn đoán hình ảnh, phân loại sức khỏe (I–V), khóa/mở khóa hồ sơ.
- **In ấn & Xuất dữ liệu**:
  - In phôi giấy khám sức khỏe trực tiếp từ trình duyệt (tích hợp mã vạch Code128).
  - Xuất file Word (`.docx`) dựa trên mẫu `asset/template18.docx`.
  - Xuất CSV 123 cột chuẩn QĐ 3176 có bộ lọc theo ngày khám, trạng thái, đơn vị, phân loại sức khỏe.
- **Quản trị Hệ thống**:
  - Quản lý tài khoản & phân quyền đa vai trò (`admin`, `reception`, `tiensu`, `bs_theluc`, `bs_noi`, `bs_ngoai`, `bs_dalieu`, `bs_san`, `bs_mat`, `bs_tmh`, `bs_rhm`, `bs_cls`, `bs_ketluan`).
  - Cấu hình thông tin CSKCB, kết nối HIS, chuyển đổi/tạo file SQLite database.
  - Quản lý & tra cứu danh mục dùng chung: bật/tắt trạng thái, sửa/xóa, import hàng loạt, nạp lại từ `asset/seed_catalogs.tsv`, và **chọn / bỏ chọn giá trị mặc định** cho từng loại danh mục (không set cứng).
- **Hỗ trợ Desktop & Mobile**: Giao diện responsive; trên thiết bị di động, thanh danh sách khám bên trái thu gọn hoàn toàn thành drawer trượt khi cần mở.

---

## Công nghệ sử dụng

- **Backend**: Rust (Edition 2021), Axum `0.7`, Tokio, SQLx `0.8` (SQLite), `rust-embed`, `quick-xml`, `zip`, `csv`, `reqwest`, `serde`.
- **Frontend**: Alpine.js, TailwindCSS, Tom Select, Flatpickr (định dạng `DD/MM/YYYY`), JsBarcode.

---

## Cấu trúc thư mục

```text
KSK_webapp/
├── Cargo.toml                  # Khai báo package & dependencies Rust
├── config.toml                 # Cấu hình server, CSKCB, HIS, đường dẫn DB & template
├── asset/
│   ├── seed_catalogs.tsv       # Dữ liệu danh mục chuẩn (ICD-10, Tỉnh/Xã, Dân tộc, Nghề nghiệp...)
│   ├── template18.docx         # Mẫu xuất Word giấy khám sức khỏe
│   └── phan_loai.xlsx          # Dữ liệu chuẩn phân loại sức khỏe
├── src/
│   ├── main.rs                 # Entry point, router API & nhúng UI vào binary
│   ├── config.rs               # Đọc/ghi cấu hình từ config.toml
│   ├── db.rs                   # Khởi tạo SQLite schema, seed tài khoản & danh mục ban đầu
│   ├── models.rs               # Struct dữ liệu, request/response, danh sách cột CSV
│   └── handlers.rs             # Xử lý API (Auth, Records, HIS, DOCX, CSV, Catalogs, Admin)
└── ui/
    ├── index_header.html       # Khung HTML đầu, Header, Tab bar & Sidebar
    ├── index_footer.html       # Modal cài đặt cá nhân, Modal sửa danh mục & Footer
    ├── tabs/                   # Các màn hình chức năng (reception, history, physical, clinical, paraclinical, conclusion, export_csv, admin, print)
    ├── js/app.js               # Logic Alpine.js toàn ứng dụng
    └── css/style.css           # Stylesheet & cấu hình in ấn
```

---

## Yêu cầu & Hướng dẫn chạy

### Yêu cầu môi trường
- **Rust & Cargo** (stable, khuyến nghị phiên bản mới nhất qua [rustup](https://rustup.rs/)).

### Chạy ở chế độ phát triển (Development)
```bash
cargo run
```

### Biên dịch bản phát hành (Release Single Binary)
```bash
cargo build --release
```
File thực thi sẽ được tạo tại:
- Windows: `target/release/ksk_webapp.exe`
- Linux/macOS: `target/release/ksk_webapp`

Khi chạy, truy cập trình duyệt tại địa chỉ mặc định: `http://localhost:8080` (hoặc cổng được cấu hình trong `config.toml`).

---

## Cấu hình (`config.toml`)

```toml
database_path = "ksk_data.db"
server_host = "0.0.0.0"
server_port = 8080
ma_cskcb = "52208"
ma_gtin_cskcb = "8930000522080"
ten_cskcb = "BỆNH VIỆN BÌNH ĐỊNH"
dia_danh = "Gia Lai"
his_use_mock = true
his_api_url = "http://127.0.0.1:8080/api/his/mock"
his_http_method = "POST"
his_auth_header = "x-api-key"
his_api_key = "HIS_SECRET_API_KEY_2026"
his_payload_template = '{"ma_tiep_nhan": "{{ma_tn}}"}'
his_timeout_seconds = 10
template_docx_path = "asset/template18.docx"
```

---

## Tài khoản mặc định

Khi khởi tạo CSDL mới lần đầu (bảng `users` trống), hệ thống tự động tạo các tài khoản mẫu sau:

| Tên đăng nhập | Mật khẩu | Vai trò (`role`) |
|---|---|---|
| `admin` | `admin123` | `admin` (Quản trị viên Hệ thống) |
| `letan` | `123456` | `reception` |
| `nvtiensu` | `123456` | `tiensu` |
| `bstheluc` | `123456` | `bs_theluc` |
| `bsnoi` | `123456` | `bs_noi` |
| `bsngoai` | `123456` | `bs_ngoai` |
| `bsdalieu` | `123456` | `bs_dalieu` |
| `bssan` | `123456` | `bs_san` |
| `bsmat` | `123456` | `bs_mat` |
| `bstmh` | `123456` | `bs_tmh` |
| `bsrhm` | `123456` | `bs_rhm` |
| `bscls` | `123456` | `bs_cls` |
| `bsketluan` | `123456` | `bs_ketluan` |

> **Lưu ý bảo mật**: Hãy đổi mật khẩu tài khoản `admin` và các tài khoản mặc định trước khi đưa vào vận hành thực tế.

---

## Lưu ý vận hành

- **Danh mục & Giá trị mặc định**: Khi bảng `catalogs` trống, hệ thống tự nạp từ `asset/seed_catalogs.tsv` (ưu tiên file trên đĩa nếu có, nếu không sẽ dùng bản nhúng trong binary). Các giá trị mặc định của từng danh mục được cấu hình trực tiếp trong **Quản trị Hệ thống → Quản lý Danh mục**.
- **Chỉnh sửa giao diện (Live Edit vs Rebuild)**:
  - Các file tĩnh dưới `/static/*` (như `ui/js/app.js`, `ui/css/style.css`) ưu tiên đọc trực tiếp từ thư mục `ui/` trên đĩa nếu chạy cùng thư mục dự án.
  - Các file HTML (`ui/index_header.html`, `ui/index_footer.html`, `ui/tabs/*.html`) được nhúng vào binary lúc biên dịch qua `include_str!`, nên cần chạy `cargo build --release` lại sau khi chỉnh sửa.

