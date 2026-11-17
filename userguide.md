# Hướng dẫn sử dụng KSK Webapp

## 1. Giới thiệu & Giao diện tổng quan

**KSK Webapp** là hệ thống web nội bộ quản lý quy trình khám sức khỏe, hoạt động trực tiếp trên trình duyệt của **Máy tính (PC)**, **Máy tính bảng (Tablet)** và **Điện thoại (Mobile)**.

Các phân hệ chính trên thanh điều hướng:

| Phân hệ | Chức năng chính |
|---|---|
| **Hồ sơ khám** | Tạo, tìm kiếm, tiếp nhận và nhập kết quả khám theo từng chuyên khoa |
| **Khám đoàn** | Quản lý công ty, hợp đồng khám đoàn, import danh sách Excel/CSV và theo dõi tiến độ |
| **In ấn** | Xem trước và in phiếu tiếp nhận, giấy khám sức khỏe (PDF A4, barcode Code 128, DOCX) |
| **Xuất dữ liệu** | Lọc hồ sơ, xuất file **CSV 123 trường** (QĐ 3176/1804) hoặc **Cấu hình & Xuất qua API JSON** |
| **Quản trị** | Quản lý người dùng, danh mục, phân loại bệnh đa cấp, sao lưu/khôi phục database |

> **Phân quyền hiển thị & chỉnh sửa:** Các tab hoặc nút bấm ngoài phạm vi role của tài khoản sẽ bị vô hiệu hóa (disable nút **Sửa**, khóa các ô nhập liệu và tắt tự động kiểm tra/sửa dữ liệu). Đồng thời backend cũng kiểm tra quyền theo từng section và từng trường dữ liệu.

---

## 2. Đăng nhập & Khôi phục tài khoản

### 2.1. Đăng nhập
1. Mở trình duyệt và truy cập địa chỉ máy chủ (mặc định `http://127.0.0.1:18080` hoặc IP LAN do IT cung cấp).
2. Nhập **Tên đăng nhập** và **Mật khẩu**, sau đó bấm **Đăng nhập**.
3. Nếu nhập sai thông tin hoặc tài khoản đang bị khóa, hệ thống sẽ hiển thị thông báo lỗi cụ thể ngay trên màn hình đăng nhập.

### 2.2. Quên mật khẩu Admin (Recovery Key)
1. Tại màn hình đăng nhập, chọn **Quên mật khẩu Admin? Khôi phục bằng Recovery Key**.
2. Nhập **Recovery Key** đã lưu khi khởi tạo database lần đầu.
3. Nhập mật khẩu Admin mới, xác nhận và đăng nhập lại.

---

## 3. Quy trình Khám lẻ

### 3.1. Tạo hoặc nhận hồ sơ mới
1. Đăng nhập bằng tài khoản `reception` (hoặc `admin`).
2. Vào **Hồ sơ khám** → chế độ **Khám lẻ** → bấm **Tạo hồ sơ mới**.
3. Nhập đầy đủ thông tin hành chính:
   - Mã tiếp nhận (duy nhất trong database), Mã y tế, Họ tên, Giới tính, Ngày sinh.
   - CCCD/Định danh, Địa chỉ, Điện thoại, Ngày khám, Lý do khám.
4. Chọn **Mẫu phiếu khám** phù hợp:
   - `TREN_18`: Từ đủ 18 tuổi
   - `LAI_XE`: Khám sức khỏe lái xe
   - `TU_6_DUOI_18`: Trẻ từ 6 đến dưới 18 tuổi
   - `DUOI_6`: Trẻ dưới 6 tuổi
   - `MAU_GIAO`: Mẫu giáo
   - `HS_PHO_THONG`: Học sinh phổ thông
5. Bấm **Lưu**.

> Nếu dùng **HIS Bridge**, khi quét/đẩy mã tiếp nhận từ HIS, hệ thống sẽ tự động tìm và mở hồ sơ tương ứng.

### 3.2. Nhập kết quả khám theo chuyên khoa
Hồ sơ được chia thành 6 tab quy trình:
1. **Tiếp nhận** (`reception`): Thông tin hành chính.
2. **Tiền sử** (`tiensu`): Tiền sử gia đình, bản thân, sản khoa (nếu có).
3. **Thể lực** (`bs_theluc`): Chiều cao, cân nặng, BMI, mạch, huyết áp, phân loại thể lực.
4. **Lâm sàng**: Nội khoa (`bs_noi`), Ngoại khoa (`bs_ngoai`), Da liễu (`bs_dalieu`), Sản phụ khoa (`bs_san`), Mắt (`bs_mat`), Tai - Mũi - Họng (`bs_tmh`), Răng - Hàm - Mặt (`bs_rhm`).
5. **Cận lâm sàng** (`bs_cls`): Xét nghiệm, chẩn đoán hình ảnh, mô tả kết quả.
6. **Kết luận** (`bs_ketluan`): Phân loại sức khỏe (Loại 1–5), bệnh phát hiện, khuyến nghị.

**Lưu ý quan trọng:**
- **Bác sĩ Nhi khoa (`bs_nhi`):** Có quyền xem, bấm **Sửa** và lưu kết quả tại các tab **Tiền sử, Thể lực, Lâm sàng, Kết luận** đối với 4 mẫu nhi khoa (`DUOI_6`, `TU_6_DUOI_18`, `MAU_GIAO`, `HS_PHO_THONG`).
- Với hồ sơ đã lưu trước đó, bấm nút **Sửa** tại tab đang xem để mở khóa nhập liệu (nút **Sửa** chỉ bật khi tài khoản có quyền tại tab đó và hồ sơ chưa bị khóa).
- Khi dùng nút điền **Mặc định bình thường**, thao tác chỉ áp dụng cho **tab đang mở**, tránh ghi đè nhầm sang chuyên khoa khác.
- Nhớ bấm **Lưu** trước khi chuyển tab hoặc chuyển sang bệnh nhân khác.

### 3.3. Ký và Khóa hồ sơ
1. Kiểm tra các chuyên khoa đã nhập đủ dữ liệu.
2. Vào tab **Kết luận**, chọn phân loại sức khỏe và nội dung kết luận.
3. Bấm **Ký / Khóa hồ sơ** (`bs_ketluan` hoặc `admin`).
4. Khi hồ sơ đã khóa (`status = 2`), mọi tài khoản không thể chỉnh sửa cho đến khi **Admin** thực hiện **Mở khóa**.

---

## 4. Quy trình Khám đoàn

### 4.1. Quản lý Công ty & Hợp đồng
1. Vào phân hệ **Khám đoàn** (tài khoản `business`, `reception` hoặc `admin`).
2. Thêm **Công ty** (mã, tên, MST, địa chỉ, liên hệ).
3. Trong công ty, bấm **Thêm hợp đồng**: nhập mã hợp đồng, tên đợt khám, ngày khám, số lượng dự kiến, mẫu phiếu mặc định và đính kèm file scan (nếu có).

### 4.2. Import danh sách đoàn từ Excel/CSV
1. Mở hợp đồng cần nhập → chọn **Import danh sách**.
2. Chọn file `.csv`, `.xlsx` hoặc `.xls` (khuyên dùng mẫu `mau_import_kham_doan.csv`).
3. Kiểm tra bảng **Preview**: hệ thống tự chuẩn hóa ngày sinh (`DD/MM/YYYY`), giới tính (`1` = Nam, `2` = Nữ), mã danh mục và cảnh báo dòng lỗi / trùng mã tiếp nhận.
4. Sửa các dòng lỗi trên file nguồn nếu có, sau đó bấm **Import**.

### 4.3. Theo dõi tiến độ & Khám
Trong màn hình hợp đồng, theo dõi tổng số hồ sơ, trạng thái từng hồ sơ và nhấn vào một bệnh nhân để mở sang giao diện khám chuyên khoa.

---

## 5. In ấn (PDF / DOCX)

1. Vào tab **In ấn** (hoặc nút in ngay trên hồ sơ).
2. Chọn một hoặc nhiều hồ sơ (lọc theo khám lẻ, khám đoàn, hợp đồng, trạng thái).
3. Chọn **In phiếu tiếp nhận** hoặc **In giấy KSK**.
4. Kiểm tra bản xem trước PDF A4 (có mã vạch Code 128) hoặc xuất **DOCX** theo mẫu trong `asset/`, sau đó in qua trình duyệt.

---

## 6. Xuất dữ liệu (CSV 123 trường & Xuất qua API)

Vào phân hệ **Xuất dữ liệu** trên thanh điều hướng:

### 6.1. Lọc & Xem trước dữ liệu
- Lọc theo loại khám (**Tất cả / Khám lẻ / Khám đoàn**), hợp đồng, khoảng ngày, trạng thái hoặc chọn từng hồ sơ trong bảng xem trước.

### 6.2. Chế độ 1: Xuất file CSV
1. Chọn sub-tab **Xuất file CSV**.
2. Bấm **Xuất CSV** để tải về file CSV gồm **123 trường** chuẩn theo QĐ 3176/QĐ-BYT & QĐ 1804/QĐ-BYT (chi tiết tại [`CSV_export_docs.md`](CSV_export_docs.md)).

### 6.3. Chế độ 2: Cấu hình & Xuất qua API
1. Chọn sub-tab **Cấu hình & Xuất qua API**.
2. Nhập thông số kết nối API đích:
   - **API Endpoint URL**: địa chỉ API nhận dữ liệu JSON.
   - **Phương thức HTTP**: `POST` hoặc `PUT`.
   - **Chế độ gửi dữ liệu**:
     - `batch`: gửi 1 request chứa danh sách hồ sơ đã giải mã.
     - `single`: gửi lần lượt từng hồ sơ.
   - **Xác thực**: `Không` / `Bearer Token` / `API Key` (tùy chỉnh tên header và token/key).
   - **Custom Headers**: thêm các header bổ sung (nếu hệ thống đích yêu cầu).
3. Bấm **Lưu cấu hình** (cấu hình được lưu vào SQLite để tái sử dụng).
4. Bấm **Đẩy tất cả theo bộ lọc** hoặc **Đẩy hồ sơ đã chọn** và theo dõi kết quả trả về ngay tại khung thông báo bên dưới.

---

## 7. Quản trị hệ thống (Dành cho Admin)

### 7.1. Quản lý người dùng
- Vào **Quản trị → Quản lý người dùng** để tạo mới, sửa họ tên, cấp **nhiều role cùng lúc** cho một tài khoản, đặt lại mật khẩu hoặc khóa/mở khóa tài khoản.

### 7.2. Danh mục & Phân loại bệnh đa cấp
- **Danh mục:** Quản lý địa giới, dân tộc, nghề nghiệp, bệnh lý và các mã chuyên môn.
- **Phân loại bệnh đa cấp:** Nằm riêng tại mục **Phân loại bệnh đa cấp** trong khu vực Quản trị; hỗ trợ **Thêm mới** và **Chỉnh sửa** trực tiếp các nhóm bệnh/mức phân loại theo nhu cầu của cơ sở.

### 7.3. Sao lưu & Chuyển Database
- Bật **Sao lưu tự động** (chọn thư mục backup, chu kỳ phút và số bản giữ lại) hoặc bấm **Backup ngay** để tạo bản sao SQLite thủ công.
- Có thể **Tạo database mới** hoặc **Chuyển database** (nhớ lưu Recovery Key mới nếu tạo DB mới).

---

## 8. Xử lý lỗi thường gặp

| Hiện tượng | Nguyên nhân & Cách xử lý |
|---|---|
| Nút **Sửa** bị mờ / không nhập được | Tài khoản chưa có role phụ trách tab đó, hoặc hồ sơ đã bị **Ký/Khóa** (liên hệ Admin mở khóa). |
| Đăng nhập báo sai / bị khóa | Kiểm tra lại tên đăng nhập, mật khẩu hoặc nhờ Admin mở khóa tài khoản. |
| Hồ sơ hiển thị `[Đã mã hóa]` | Phiên làm việc chưa mở khóa Master Key; đăng nhập lại hoặc khôi phục bằng Recovery Key. |
| Import khám đoàn báo lỗi dòng | Kiểm tra trùng `MA_TIEP_NHAN`, định dạng ngày `DD/MM/YYYY`, giới tính `1`/`2`, mã danh mục. |
| Giao diện chưa cập nhật sau khi nâng cấp | Nhấn `Ctrl + F5` trên trình duyệt để xóa cache cũ. |
| Lỗi hệ thống / API không phản hồi | Kiểm tra file log `ksk_app.log` tại thư mục chạy server và gửi cho bộ phận IT. |
