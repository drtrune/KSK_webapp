# Hướng dẫn sử dụng KSK Desktop
## 1. Giới thiệu
KSK Desktop là phần mềm desktop quản lý quy trình khám sức khỏe:
- Tiếp nhận khám lẻ và khám đoàn.
- Khám theo từng chuyên khoa.
- In phiếu tiếp nhận, giấy khám sức khỏe và xuất DOCX/PDF.
- Xuất dữ liệu CSV 123 trường theo chuẩn KSK.
- Quản lý công ty, hợp đồng, danh mục và người dùng.
Giao diện chính gồm bốn khu vực:
| Khu vực | Chức năng |
|---|---|
| **Hồ sơ khám** | Tạo, tìm kiếm và nhập kết quả khám |
| **Khám đoàn** | Quản lý công ty, hợp đồng và danh sách khám đoàn |
| **Xuất dữ liệu** | Xuất CSV/DOCX/PDF, lọc hồ sơ |
| **Quản trị** | Người dùng, danh mục, database, backup và cập nhật |
Các nút hoặc tab không thuộc quyền của tài khoản sẽ bị ẩn hoặc khóa.
## 2. Chuẩn bị trước khi sử dụng
### 2.1. Kiểm tra thư mục ứng dụng
Đảm bảo các file sau nằm đúng vị trí:
```text
ksk_webapp.exe
config.toml
database.toml
asset/
```
Database và thư mục backup phải có quyền đọc/ghi. Nếu dùng nhiều máy, nên đặt database trên thư mục LAN/SMB dùng chung và thư mục backup ở vị trí được sao lưu riêng.
### 2.2. Khởi tạo lần đầu
Ở lần chạy đầu tiên:
1. Chọn hoặc tạo file database.
2. Đặt mật khẩu Admin.
3. Ghi lại **Recovery Key** được hiển thị.
4. Xác nhận đã lưu Recovery Key.
5. Đăng nhập vào hệ thống.
Recovery Key dùng để khôi phục tài khoản Admin khi quên mật khẩu. Không lưu Recovery Key trong cùng thư mục với database và không gửi qua kênh không an toàn.
### 2.3. Tài khoản mặc định
Database mới có thể được tạo cùng các tài khoản mẫu trong mã nguồn. Nếu hệ thống của bạn hiển thị tài khoản mặc định, hãy đăng nhập lần đầu và đổi mật khẩu ngay trong khu vực **Quản trị**. Không sử dụng mật khẩu mặc định trong môi trường thật.
## 3. Đăng nhập và khôi phục Admin
### Đăng nhập
1. Nhập tên đăng nhập.
2. Nhập mật khẩu.
3. Chọn **Đăng nhập**.
Tài khoản bị khóa chỉ có thể được mở lại bởi Admin.
### Quên mật khẩu Admin
1. Tại màn hình đăng nhập, chọn **Quên mật khẩu Admin? Khôi phục bằng Recovery Key**.
2. Nhập Recovery Key.
3. Nhập mật khẩu Admin mới.
4. Xác nhận khôi phục.
5. Đăng nhập lại bằng mật khẩu mới.
Nếu Recovery Key bị mất, không thể khôi phục Master Key bằng cách thông thường. Hãy dừng thao tác trên database và liên hệ người quản trị hệ thống.
## 4. Quy trình khám lẻ
### 4.1. Tạo hồ sơ mới
1. Đăng nhập bằng tài khoản `reception` hoặc `admin`.
2. Chọn **Hồ sơ khám**.
3. Chọn chế độ **Khám lẻ**.
4. Chọn **Tạo hồ sơ mới**.
5. Nhập các thông tin tiếp nhận:
   - Mã tiếp nhận.
   - Mã y tế.
   - Họ và tên.
   - Giới tính, ngày sinh.
   - CCCD/định danh.
   - Địa chỉ, điện thoại.
   - Ngày khám và lý do khám.
6. Chọn mẫu khám phù hợp.
7. Lưu hồ sơ.
Mã tiếp nhận phải là duy nhất trong database.
### 4.2. Tìm hồ sơ
Tìm theo mã tiếp nhận, mã y tế hoặc họ tên trong danh sách bên trái. Có thể chuyển giữa **Khám lẻ** và **Khám đoàn** để tránh chọn nhầm loại hồ sơ.
Nếu nhận dữ liệu từ HIS Bridge, hồ sơ mới hoặc hồ sơ tương ứng sẽ được tìm/mở tự động sau khi ứng dụng nhận mã tiếp nhận.
### 4.3. Nhập kết quả khám
Hồ sơ được nhập theo các phần:
1. **Tiếp nhận:** thông tin hành chính.
2. **Tiền sử:** tiền sử gia đình, bản thân và sản khoa khi phù hợp.
3. **Thể lực:** chiều cao, cân nặng, mạch, huyết áp và phân loại.
4. **Lâm sàng:** nội, ngoại, da liễu, sản, mắt, tai mũi họng, răng hàm mặt.
5. **Cận lâm sàng:** xét nghiệm, chẩn đoán hình ảnh và mô tả kết quả.
6. **Kết luận:** phân loại sức khỏe, bệnh phát hiện và khuyến nghị.
Chỉ role được cấp cho section mới có thể chỉnh sửa section đó. Sau khi nhập dữ liệu, chọn **Lưu** trước khi chuyển tab hoặc chuyển hồ sơ.
### 4.4. Ký và khóa hồ sơ
1. Kiểm tra các phần khám đã hoàn tất.
2. Mở tab **Kết luận**.
3. Nhập phân loại sức khỏe và nội dung kết luận.
4. Chọn **Ký / Khóa hồ sơ**.
5. Xác nhận thao tác.
Hồ sơ đã khóa không nên chỉnh sửa. Chỉ `admin` hoặc người có quyền kết luận theo thiết kế hệ thống mới được thực hiện thao tác ký/khóa.
## 5. Quy trình khám đoàn
### 5.1. Tạo công ty
1. Mở **Khám đoàn**.
2. Chọn **Công ty & Hợp đồng**.
3. Chọn **Thêm công ty**.
4. Nhập mã, tên, mã số thuế, địa chỉ và thông tin liên hệ.
5. Lưu công ty.
### 5.2. Tạo hợp đồng
1. Chọn công ty.
2. Chọn **Thêm hợp đồng**.
3. Nhập mã hợp đồng, tên đợt khám và ngày khám.
4. Nhập số lượng dự kiến.
5. Chọn mẫu phiếu mặc định.
6. Đính kèm file scan nếu cần.
7. Lưu hợp đồng.
### 5.3. Import danh sách
1. Mở hợp đồng cần nhập.
2. Chọn **Import danh sách**.
3. Chọn file `.csv`, `.xlsx` hoặc `.xls`.
4. Kiểm tra bảng preview.
5. Sửa các dòng báo lỗi trong file nguồn.
6. Chọn **Import** khi số dòng hợp lệ đúng yêu cầu.
Các cột cơ bản được hỗ trợ:
```text
MA_TIEP_NHAN, MA_Y_TE, HO_TEN, NGAY_SINH, GIOI_TINH,
SO_CCCD, NGAYCAP_CCCD, NOICAP_CCCD, DIEN_THOAI,
MATINH_CU_TRU, MAXA_CU_TRU, DIA_CHI, MA_DAN_TOC,
MA_NGHE_NGHIEP, NOI_LAM_VIEC_HOC_TAP, MA_LOAI_KCB,
NGAY_VAO, NHOM_MAU, LY_DO_VV
```
Hệ thống có thể nhận một số tên cột tương đương bằng tiếng Việt hoặc tiếng Anh. Nên dùng file mẫu `mau_import_kham_doan.csv` để tránh sai tên cột và định dạng.
Ngày nên dùng `DD/MM/YYYY`; giới tính dùng `1` = Nam, `2` = Nữ. Sau import, hồ sơ được gắn với hợp đồng và có nhãn **Khám đoàn**.
### 5.4. Theo dõi tiến độ
Trong hợp đồng, xem số hồ sơ đã tạo, trạng thái khám và các hồ sơ còn thiếu phần khám. Chọn một hồ sơ để mở quy trình khám giống hồ sơ khám lẻ.
## 6. In và xuất dữ liệu
### In phiếu
1. Chọn một hoặc nhiều hồ sơ.
2. Kiểm tra đúng loại khám và đúng template.
3. Chọn **In phiếu tiếp nhận** hoặc **In giấy KSK**.
4. Kiểm tra bản xem trước.
5. Chọn máy in và khổ giấy A4.
PDF được dựng theo template đã chọn và có thể chứa mã vạch Code 128.
### Xuất PDF/DOCX
Chọn chức năng xuất tương ứng, chọn thư mục lưu và kiểm tra file sau khi tạo. Với DOCX, không đổi tên hoặc di chuyển các template trong `asset/` khi ứng dụng đang sử dụng chúng.
### Xuất CSV
1. Mở **Xuất dữ liệu**.
2. Chọn loại khám: khám lẻ, khám đoàn hoặc tất cả.
3. Nếu là khám đoàn, chọn hợp đồng.
4. Lọc trạng thái hoặc chọn hồ sơ cần xuất.
5. Chọn **Xuất CSV**.
File CSV sử dụng 123 cột theo đặc tả trong [`CSV_export_docs.md`](CSV_export_docs.md). Hãy kiểm tra mã cơ sở KCB, ngày khám và phân loại sức khỏe trước khi gửi dữ liệu ra hệ thống khác.
## 7. Quản trị hệ thống
### 7.1. Quản lý người dùng
Trong **Quản trị**:
1. Chọn **Người dùng**.
2. Tạo tài khoản mới.
3. Nhập tên đăng nhập và họ tên.
4. Chọn một hoặc nhiều role.
5. Lưu tài khoản.
Chỉ cấp role cần thiết cho công việc. Khóa tài khoản khi nhân sự không còn sử dụng hệ thống.
### 7.2. Quản lý danh mục
Danh mục được dùng trong các ô chọn của hồ sơ, gồm địa giới, dân tộc, nghề nghiệp, bệnh lý và các mã chuyên môn. Có thể import hoặc cập nhật danh mục từ khu vực **Danh mục**.
Sau khi cập nhật, kiểm tra lại mã và tên hiển thị trước khi import dữ liệu hàng loạt.
### 7.3. Backup
Trong **Quản trị**, bật sao lưu tự động và cấu hình:
- Thư mục backup.
- Khoảng thời gian sao lưu.
- Số bản backup cần giữ.
Có thể chọn **Backup ngay** để tạo bản sao thủ công. Định kỳ nên thử mở một bản backup trên máy kiểm tra để xác nhận file có thể sử dụng.
### 7.4. Tạo hoặc chuyển database
Chỉ Admin nên thực hiện thao tác này:
1. Sao lưu database hiện tại.
2. Chọn **Tạo database mới** hoặc **Chuyển database**.
3. Kiểm tra đường dẫn đích.
4. Lưu Recovery Key mới nếu tạo database mới.
5. Xác nhận chuyển kết nối.
Recovery Key của database mới không dùng được cho database khác.
## 8. Tích hợp HIS Bridge
Local HIS IPC mặc định lắng nghe tại:
```text
http://127.0.0.1:18080
```
Endpoint:
```text
POST /api/his/push
POST /api/his/mock
```
HIS Bridge cần gửi đúng payload và API key theo `config.toml`. Khi tích hợp thật, tắt mock và dùng URL/API key của môi trường triển khai. Không mở Local IPC ra địa chỉ mạng công cộng.
## 9. Xử lý lỗi thường gặp
| Hiện tượng | Cách xử lý |
|---|---|
| Không mở được database | Kiểm tra đường dẫn, quyền đọc/ghi và việc database có đang bị khóa |
| Hồ sơ hiển thị `[Đã mã hóa]` | Đăng nhập bằng tài khoản có Master Key hoặc khôi phục bằng Recovery Key |
| Không sửa được một trường | Kiểm tra role và section được cấp quyền |
| Import báo lỗi dòng | Kiểm tra mã tiếp nhận trùng, ngày, giới tính, mã danh mục và tên cột |
| HIS không đẩy được hồ sơ | Kiểm tra ứng dụng đang chạy, port IPC, URL, method và API key |
| Không tạo được PDF/DOCX | Kiểm tra quyền ghi thư mục đích và các file trong `asset/` |
| Ứng dụng báo lỗi bất ngờ | Gửi nội dung lỗi và file `ksk_app.log` cho quản trị viên; không gửi kèm database thật nếu không được phép |
## 10. Quy tắc an toàn dữ liệu
- Không chia sẻ tài khoản hoặc Recovery Key.
- Không đặt database thật trong thư mục public.
- Sao lưu trước khi chuyển database, import số lượng lớn hoặc cập nhật phần mềm.
- Kiểm tra đúng loại khám và đúng hợp đồng trước khi in/xuất.
- Không sửa trực tiếp file SQLite bằng công cụ bên ngoài khi ứng dụng đang chạy.
- Khi nghi ngờ lộ mật khẩu, API key hoặc Recovery Key, đổi thông tin ngay và báo Admin.
Output
11.5 KB