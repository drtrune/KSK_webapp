# Hướng dẫn Cài đặt & Triển khai (Setup Guide)

Tài liệu này hướng dẫn cài đặt, cấu hình, khởi tạo lần đầu và vận hành **KSK Webapp (`ksk_webapp.exe`)** trên Windows (máy đơn hoặc mạng LAN nội bộ).

---

## 1. Yêu cầu hệ thống

### Triển khai từ bản build sẵn (Production)
- **Hệ điều hành:** Windows 10 / 11 hoặc Windows Server 2016+.
- **Trình duyệt truy cập:** Chrome, Microsoft Edge hoặc trình duyệt di động/tablet hiện đại trong cùng mạng LAN.
- **Quyền thư mục:** Tài khoản Windows chạy `ksk_webapp.exe` phải có quyền đọc/ghi tại thư mục chứa ứng dụng, file database SQLite (`*.db`), thư mục `backups/` và file log `ksk_app.log`.

### Build từ mã nguồn (Development)
- **Rust toolchain** (stable) & **Cargo**.
- Kết nối mạng để tải crates khi build lần đầu (`cargo build --release`).

---

## 2. Cài đặt & Build

### Cách A: Build từ mã nguồn
```powershell
# Clone repository và vào thư mục dự án
cd D:\Work\KSK\KSK_webapp

# Chạy unit test kiểm tra
cargo test

# Build bản phát hành tối ưu
cargo build --release
```
Sau khi build xong, file thực thi nằm tại:
```text
target\release\ksk_webapp.exe
```
> Lưu ý: Giao diện web trong thư mục `ui/` đã được nhúng trực tiếp vào file `.exe` thông qua `rust-embed`. Mỗi khi sửa code hoặc giao diện, cần tắt `ksk_webapp.exe` đang chạy trước khi build lại để tránh lỗi khóa file (`Access is denied / os error 5`).

### Cách B: Triển khai bằng thư mục phát hành
Chuẩn bị một thư mục triển khai (ví dụ `C:\KSK_Webapp\`) gồm tối thiểu:
```text
ksk_webapp.exe
config.toml
database.toml          (tùy chọn, dùng để ghi đè đường dẫn DB)
asset\                 (chứa template*.docx, seed_catalogs.tsv, phan_loai_config.json)
backups\               (tự tạo nếu chưa có)
```

---

## 3. Cấu hình (`config.toml` & `database.toml`)

Ứng dụng đọc `config.toml` ở thư mục hiện tại (hoặc cạnh file `ksk_webapp.exe`). Nếu có `database.toml`, đường dẫn database sẽ được ưu tiên đọc từ đó.

Ví dụ cấu hình `config.toml`:
```toml
# Đường dẫn database SQLite
database_path = 'D:\Work\KSK\KSK_webapp\ksk_data.db'

# Địa chỉ lắng nghe Web Server
# - Chỉ chạy trên máy hiện tại: "127.0.0.1"
# - Cho phép các máy khác / tablet / điện thoại trong mạng LAN truy cập: "0.0.0.0"
server_host = "127.0.0.1"
server_port = 18080
local_ipc_port = 18080

# Thông tin cơ sở khám chữa bệnh (dùng khi in phiếu và xuất CSV 123 trường)
ma_cskcb = "52208"
ma_gtin_cskcb = "8930000522080"
ten_cskcb = "BỆNH VIỆN BÌNH ĐỊNH"
dia_danh = "Gia Lai"

# Tích hợp HIS
his_use_mock = false
his_api_url = "http://<ip-his-server>/api/..."
his_http_method = "POST"
his_auth_header = "x-api-key"
his_api_key = "<YOUR_HIS_API_KEY>"
his_payload_template = '{"ma_tiep_nhan": "{{ma_tn}}"}'
his_timeout_seconds = 10

# Template DOCX mặc định
template_docx_path = "asset/template18.docx"

# Sao lưu tự động
auto_backup_enabled = true
backup_dir_path = "backups"
backup_interval_minutes = 60
backup_keep_count = 10

# Cập nhật nội bộ (nếu dùng thư mục chia sẻ bản cập nhật)
update_dir_path = ""
auto_check_update = true
```

---

## 4. Khởi chạy & Thiết lập lần đầu

1. Chạy `ksk_webapp.exe` (nhấp đúp hoặc chạy qua PowerShell/CMD để xem log trực tiếp).
2. Mở trình duyệt và truy cập:
   - Trên cùng máy chủ: `http://127.0.0.1:18080`
   - Từ máy khác / Tablet / Mobile trong LAN (khi `server_host = "0.0.0.0"`): `http://<IP-máy-chủ>:18080`
3. **Thiết lập Master Key & Recovery Key (lần đầu tiên):**
   - Hệ thống yêu cầu đặt mật khẩu Admin / khởi tạo khóa mã hóa.
   - Hệ thống sinh **Recovery Key** — **bắt buộc sao chép và cất giữ ở nơi an toàn ngoài thư mục database**. Recovery Key là cách duy nhất để khôi phục quyền truy cập dữ liệu mã hóa nếu quên mật khẩu Admin.

### Tài khoản mặc định khi khởi tạo database mới
Khi bảng `users` trống, hệ thống tự tạo các tài khoản mẫu sau:

| Username | Mật khẩu mặc định | Role |
|---|---|---|
| `admin` | `admin123` | `admin` |
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

> **Khuyến cáo bảo mật:** Đăng nhập bằng `admin`, đổi ngay mật khẩu của `admin` và toàn bộ tài khoản sử dụng thực tế trong mục **Quản trị → Quản lý người dùng**. Tài khoản Bác sĩ Nhi (`bs_nhi`) hoặc Kinh doanh (`business`) có thể tạo mới hoặc gán thêm role cho tài khoản hiện có.

---

## 5. Triển khai mạng LAN nhiều máy trạm / Tablet / Mobile

Mô hình khuyến nghị là **1 máy chủ chạy `ksk_webapp.exe` + SQLite cục bộ**, các phòng khám / lễ tân / bác sĩ dùng trình duyệt truy cập qua LAN:

1. Đặt `server_host = "0.0.0.0"` và `server_port = 18080` trong `config.toml`.
2. Mở **Windows Defender Firewall** cho phép Inbound TCP port `18080` trong mạng Private/Domain (không mở ra Internet công cộng).
3. Kiểm tra IP LAN của máy chủ (ví dụ `192.168.1.50` bằng lệnh `ipconfig`).
4. Trên các máy trạm, tablet, điện thoại cùng mạng Wi-Fi/LAN, mở trình duyệt truy cập `http://192.168.1.50:18080`.
5. Nếu cập nhật phiên bản mới, yêu cầu người dùng bấm `Ctrl + F5` (hoặc tải lại trang) để xóa cache JS/HTML cũ của trình duyệt.

---

## 6. Cấu hình HIS Bridge (Tùy chọn)

- File script AutoHotkey mẫu: `his_tiep_nhan_bridge.ahk` (và trong `ahk/`).
- Endpoint nhận dữ liệu từ HIS/Bridge:
  - `POST http://127.0.0.1:18080/api/his/push`
  - `POST http://127.0.0.1:18080/api/his/mock` (dùng khi test với `his_use_mock = true`)
- Khi tích hợp HIS thật, đặt `his_use_mock = false`, cấu hình đúng `his_api_url`, `his_auth_header`, `his_api_key` và `his_payload_template` trong `config.toml`.

---

## 7. Giám sát Log & Xử lý sự cố

- **File log:** `ksk_app.log` nằm cùng thư mục chạy ứng dụng (ghi lại thông tin khởi động, lỗi HTTP/API, lỗi SQLite, lỗi xuất dữ liệu và panic).
- **Lỗi thường gặp:**
  - *Không truy cập được từ máy khác trong LAN:* kiểm tra `server_host = "0.0.0.0"` trong `config.toml` và rule Firewall port `18080`.
  - *Lỗi `Access is denied` khi build/cập nhật:* vẫn còn tiến trình `ksk_webapp.exe` đang chạy ngầm — hãy tắt trong Task Manager trước khi ghi đè file `.exe`.
  - *Dữ liệu hiển thị `[Đã mã hóa]`:* phiên chưa mở khóa Master Key hoặc sai Recovery Key; đăng nhập lại bằng tài khoản có quyền hợp lệ.
  - *Lỗi `403 Forbidden` khi lưu hồ sơ:* tài khoản chưa được cấp đúng role phụ trách tab/mục đang sửa, hoặc hồ sơ đã bị khóa (`status = 2`).
