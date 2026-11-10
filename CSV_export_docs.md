# HƯỚNG DẪN ĐỊNH DẠNG DỮ LIỆU XUẤT CSV KHÁM SỨC KHỎE (QĐ 3176/QĐ-BYT & QĐ 1804/QĐ-BYT)

Tài liệu quy định chi tiết cấu trúc, danh sách trường, định dạng kiểu dữ liệu, ràng buộc bắt buộc và diễn giải nghiệp vụ của dữ liệu Khám sức khỏe (KSK) và Khám sức khỏe định kỳ dùng cho người đủ 18 tuổi trở lên.

---

## 1. Bảng đặc tả trường dữ liệu (123 chỉ tiêu)

| TT | Tên trường thông tin | Chỉ tiêu | Kiểu dữ liệu | Kích thước tối đa | Bắt buộc | Diễn giải & Quy tắc nghiệp vụ |
|:---|:---------------------|:---------|:-------------|:-------------------|:--------:|:------------------------------|
| **I** | **Thông tin hành chính** | | | | | |
| 1 | Họ và tên | `HO_TEN` | Chuỗi | 255 | **x** | Ghi IN HOA theo hướng dẫn tại QĐ 3176/QĐ-BYT. |
| 2 | Giới tính | `GIOI_TINH` | Số | 1 | **x** | `1` = Nam, `2` = Nữ. |
| 3 | Ngày sinh | `NGAY_SINH` | Chuỗi | 12 | **x** | Định dạng: `DD/MM/YYYY`. |
| 4 | Dân tộc | `MA_DAN_TOC` | Chuỗi | 2 | **x** | Mã danh mục 54 dân tộc Việt Nam (VD: `01` - Kinh). |
| 5 | Mã định danh (CCCD) | `SO_CCCD` | Chuỗi | n | **x** | Số CCCD / CMND / Mã định danh cá nhân (12 số). |
| 6 | Ngày cấp | `NGAYCAP_CCCD` | Chuỗi | 8 | | Định dạng: `DD/MM/YYYY`. |
| 7 | Nơi cấp | `NOICAP_CCCD` | Chuỗi | 1024 | | Nơi cấp CCCD (Cục Cảnh sát QLHC về TTXH / Công an tỉnh/TP). |
| 8 | Nơi ở hiện tại | `DIA_CHI` | Chuỗi | 1024 | **x** | Số nhà, thôn, xóm, đường phố nơi ở hiện tại. |
| 9 | Mã Tỉnh | `MATINH_CU_TRU` | Chuỗi | 3 | **x** | Mã Tỉnh / Thành phố cư trú (theo QĐ 3176/QĐ-BYT). |
| 10 | Mã Xã | `MAXA_CU_TRU` | Chuỗi | 5 | **x** | Mã Xã / Phường cư trú. |
| 11 | Nghề nghiệp | `MA_NGHE_NGHIEP` | Chuỗi | 2 | **x** | Mã nghề nghiệp theo danh mục nghề nghiệp Bộ Y tế. |
| 12 | Nơi làm việc, học tập | `NOI_LAM_VIEC_HOC_TAP` | Chuỗi | 1024 | | Đơn vị, công ty hoặc trường học đang công tác, học tập. |
| 13 | Điện thoại | `DIEN_THOAI` | Chuỗi | 15 | | Số điện thoại liên hệ của người đến khám KSK. |
| 14 | Nhóm máu | `NHOM_MAU` | Chuỗi | 5 | | Nhóm máu hệ ABO / Rh: `A`, `B`, `AB`, `O`. |
| 15 | Lý do khám sức khỏe | `LY_DO_VV` | Chuỗi | n | **x** | Lý do đến KSK (Khám sức khỏe đi làm, đi học, định kỳ...). |
| **II** | **Thông tin chung về lần khám** | | | | | |
| 16 | Lượt khám | `MA_LK` | Chuỗi | 100 | | Mã lượt khám bệnh / Lượt tiếp nhận. |
| 17 | Mã cơ sở khám bệnh, chữa bệnh | `MA_CSKCB` | Chuỗi | 5 | **x** | Mã 5 ký tự của CSKCB do Bộ Y tế cấp (VD: `52208`). |
| 18 | Mã cơ sở KCB theo chuẩn GLN | `MA_GTIN_CSKCB` | Chuỗi | 13 | **x** | Mã định danh toàn cầu GLN gồm 13 số của CSKCB. |
| 19 | Đối tượng | `DOI_TUONG` | Chuỗi | 50 | **x** | Mã đối tượng (`1`=Người cao tuổi, `2`=Người khuyết tật, `3`=Hộ nghèo/cận nghèo, `4`=Người có công, `5`=Mạn tính, `6`=Dân tộc thiểu số, `7`=Vùng khó khăn, `8`=Xã đảo, `9`=Đặc khu, `10`=Trẻ mầm non, `11`=Học sinh, `12`=Sinh viên, `13`=Người lao động, `14`=Lao động không chính thức, `15`=Chưa có BHYT, `16`=Khác). Phân tách `;` nếu có nhiều mã. |
| 20 | Nguồn chi trả | `NGUON_CHI_TRA` | Số | 1 | **x** | `1`=Ngân sách TW, `2`=Ngân sách địa phương, `3`=Quỹ BHYT, `4`=Người sử dụng lao động, `5`=Xã hội hóa, `9`=Khác. |
| 21 | Loại hình khám bệnh, chữa bệnh | `MA_LOAI_KCB` | Chuỗi | 2 | **x** | Mặc định: `15` (Khám sức khoẻ định kỳ) theo QĐ 1804/QĐ-BYT. |
| 22 | Ngày khám sức khỏe | `NGAY_VAO` | Chuỗi | 12 | **x** | Ngày khám (định dạng `DD/MM/YYYY`). |
| **III** | **Tiền sử bệnh / tật** | | | | | |
| **A** | **Tiền sử gia đình** | | | | | |
| 23 | Có ai trong gia đình mắc bệnh không | `TSGD_MAC_BENH` | Số | 1 | | `0` = Không, `1` = Có. |
| 24 | Tiền sử bệnh gia đình | `TSGD_MA_BENH` | Chuỗi | 255 | | Mã bệnh ICD-10 hoặc mã triệu chứng, ngăn cách bằng dấu `;`. |
| **B** | **Tiền sử bản thân** | | | | | |
| 25 | Bệnh hay bị thương trong 5 năm qua | `TSBT_BENH_TRONG_5_NAM_QUA` | Số | 1 | | `0` = Không, `1` = Có. |
| 26 | Bệnh thần kinh hay chấn thương đầu | `TSBT_BENH_THAN_KINH` | Số | 1 | | `0` = Không, `1` = Có. |
| 27 | Bệnh mắt hoặc giảm thị lực | `TSBT_BENH_MAT` | Số | 1 | | `0` = Không, `1` = Có. |
| 28 | Bệnh ở tai, giảm thính lực | `TSBT_BENH_TAI` | Số | 1 | | `0` = Không, `1` = Có. |
| 29 | Bệnh tim mạch, hồi hộp | `TSBT_BENH_TIM` | Số | 1 | | `0` = Không, `1` = Có. |
| 30 | Phẫu thuật can thiệp tim - mạch | `TSBT_PHAU_THUAT_TIM` | Số | 1 | | `0` = Không, `1` = Có. |
| 31 | Tăng huyết áp | `TSBT_TANG_HUYET_AP` | Số | 1 | | `0` = Không, `1` = Có. |
| 32 | Khó thở, hen suyễn | `TSBT_KHO_THO` | Số | 1 | | `0` = Không, `1` = Có. |
| 33 | Bệnh phổi, hen, viêm phế quản mạn | `TSBT_BENH_PHOI` | Số | 1 | | `0` = Không, `1` = Có. |
| 34 | Bệnh thận, lọc máu | `TSBT_BENH_THAN` | Số | 1 | | `0` = Không, `1` = Có. |
| 35 | Nghiện rượu, bia | `TSBT_NGHIEN_RUOU` | Số | 1 | | `0` = Không, `1` = Có. |
| 36 | Đái tháo đường | `TSBT_DAI_THAO_DUONG` | Số | 1 | | `0` = Không, `1` = Có. |
| 37 | Bệnh tâm thần | `TSBT_BENH_TAM_THAN` | Số | 1 | | `0` = Không, `1` = Có. |
| 38 | Mất ý thức, rối loạn ý thức | `TSBT_MAT_Y_THUC` | Số | 1 | | `0` = Không, `1` = Có. |
| 39 | Ngất, chóng mặt | `TSBT_NGAT` | Số | 1 | | `0` = Không, `1` = Có. |
| 40 | Bệnh tiêu hóa | `TSBT_BENH_TIEU_HOA` | Số | 1 | | `0` = Không, `1` = Có. |
| 41 | Rối loạn giấc ngủ | `TSBT_ROI_LOAN_GIAC_NGU` | Số | 1 | | `0` = Không, `1` = Có. |
| 42 | Tai biến mạch máu não / Liệt | `TSBT_TAI_BIEN` | Số | 1 | | `0` = Không, `1` = Có. |
| 43 | Bệnh cột sống | `TSBT_BENH_COT_SONG` | Số | 1 | | `0` = Không, `1` = Có. |
| 44 | Sử dụng rượu bia thường xuyên | `TSBT_RUOU_THUONG_XUYEN` | Số | 1 | | `0` = Không, `1` = Có. |
| 45 | Sử dụng ma túy, chất gây nghiện | `TSBT_MA_TUY` | Số | 1 | | `0` = Không, `1` = Có. |
| 46 | Bệnh khác | `TSBT_BENH_KHAC` | Số | 1 | | `0` = Không, `1` = Có. |
| 47 | Mã bệnh khác | `TSBT_MA_BENH_KHAC` | Chuỗi | 255 | | Mã ICD-10 ngăn cách bằng dấu `;`. |
| 48 | Đang điều trị bệnh gì không | `TSBT_DANG_DIEU_TRI_BENH` | Số | 1 | | `0` = Không, `1` = Có. |
| 49 | Tên bệnh tiền sử bản thân | `TSBT_MA_BENH` | Chuỗi | 255 | | Mã ICD-10 ngăn cách bằng dấu `;`. |
| 50 | Thuốc đang sử dụng và liều lượng | `TSBT_TEN_THUOC_LIEU_LUONG` | Chuỗi | 1024 | | Tên thuốc và phác đồ liều lượng đang dùng. |
| 51 | Tiền sử thai sản | `TSBT_THAI_SAN` | Số | 1 | | `0` = Không, `1` = Có. |
| 52 | Tên bệnh thai sản | `TSBT_MA_BENH_THAI_SAN` | Chuỗi | 255 | | Mã ICD-10 ngăn cách bằng dấu `;`. |
| 53 | Thuốc điều trị bệnh thai sản | `TSBT_TEN_THUOC_THAI_SAN` | Chuỗi | 1024 | | Tên thuốc đang sử dụng điều trị bệnh thai sản. |
| **IV** | **Khám thể lực** | | | | | |
| 54 | Chiều cao (cm) | `CHIEU_CAO` | Số | 10 | | Chiều cao đo bằng cm. |
| 55 | Cân nặng (kg) | `CAN_NANG` | Chuỗi | 6 | | Cân nặng đo bằng kg. |
| 56 | Chỉ số BMI | `CHI_SO_BMI` | Chuỗi | 10 | | BMI = Cân nặng (kg) / (Chiều cao (m))^2. |
| 57 | Mạch (lần/phút) | `MACH` | Chuỗi | 100 | | Tần số mạch đập trên phút. |
| 58 | Huyết áp (mmHg) | `HUYET_AP` | Chuỗi | 100 | | Trị số huyết áp (Huyết áp tâm thu/Huyết áp tâm trương). |
| 59 | Phân loại thể lực | `KHAM_THE_LUC_PL` | Số | 1 | | Phân loại thể lực từ 1 đến 5 (Loại I đến V). |
| **V** | **Khám lâm sàng** | | | | | |
| **A** | **Nội khoa** | | | | | |
| 60 | KQ khám tuần hoàn | `NOI_KHOA_TUAN_HOAN` | Chuỗi | n | | Kết quả khám hệ tuần hoàn. |
| 61 | Phân loại KQ khám tuần hoàn | `NOI_KHOA_TUAN_HOAN_PL` | Số | 1 | | Loại I - V. |
| 62 | CKĐT bác sỹ khám tuần hoàn | `CKDT_NOI_KHOA_TUAN_HOAN` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ tuần hoàn. |
| 63 | KQ Hô hấp | `NOI_KHOA_HO_HAP` | Chuỗi | n | | Kết quả khám hệ hô hấp. |
| 64 | Phân loại KQ khám hô hấp | `NOI_KHOA_HO_HAP_PL` | Số | 1 | | Loại I - V. |
| 65 | CKĐT bác sỹ khám Hô hấp | `CKDT_NOI_KHOA_HO_HAP` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ hô hấp. |
| 66 | KQ khám Tiêu hóa | `NOI_KHOA_TIEU_HOA` | Chuỗi | n | | Kết quả khám hệ tiêu hóa. |
| 67 | Phân loại KQ khám tiêu hoá | `NOI_KHOA_TIEU_HOA_PL` | Số | 1 | | Loại I - V. |
| 68 | CKĐT bác sỹ khám Tiêu hóa | `CKDT_NOI_KHOA_TIEU_HOA` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ tiêu hóa. |
| 69 | KQ khám Thận - TN - SD | `NOI_KHOA_THAN_TN_SD` | Chuỗi | n | | Thận - Tiết niệu - Sinh dục. |
| 70 | Phân loại Thận - TN - SD | `NOI_KHOA_THAN_TN_SD_PL` | Số | 1 | | Loại I - V. |
| 71 | CKĐT bác sỹ Thận - Tiết niệu | `CKDT_NOI_KHOA_THAN_TN_SD` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ thận - tiết niệu. |
| 72 | KQ khám Nội tiết | `NOI_KHOA_NOI_TIET` | Chuỗi | n | | Kết quả khám nội tiết. |
| 73 | Phân loại khám Nội tiết | `NOI_KHOA_NOI_TIET_PL` | Số | 1 | | Loại I - V. |
| 74 | CKĐT bác sỹ khám Nội tiết | `CKDT_NOI_KHOA_NOI_TIET` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ nội tiết. |
| 75 | KQ khám cơ - xương - khớp | `NOI_KHOA_CO_XUONG_KHOP` | Chuỗi | n | | Cơ - xương - khớp. |
| 76 | Phân loại cơ - xương - khớp | `NOI_KHOA_CO_XUONG_KHOP_PL` | Số | 1 | | Loại I - V. |
| 77 | CKĐT bác sỹ cơ - xương - khớp | `CKDT_NOI_KHOA_CO_XUONG_KHOP` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ CXK. |
| 78 | KQ khám thần kinh | `NOI_KHOA_THAN_KINH` | Chuỗi | n | | Kết quả khám thần kinh. |
| 79 | Phân loại KQ khám thần kinh | `NOI_KHOA_THAN_KINH_PL` | Số | 1 | | Loại I - V. |
| 80 | CKĐT bác sỹ khám thần kinh | `CKDT_NOI_KHOA_THAN_KINH` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ thần kinh. |
| 81 | KQ khám tâm thần | `NOI_KHOA_TAM_THAN` | Chuỗi | n | | Kết quả khám tâm thần. |
| 82 | Phân loại KQ khám tâm thần | `NOI_KHOA_TAM_THAN_PL` | Số | 1 | | Loại I - V. |
| 83 | CKĐT bác sỹ khám tâm thần | `CKDT_NOI_KHOA_TAM_THAN` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ tâm thần. |
| **B** | **Ngoại khoa** | | | | | |
| 84 | KQ khám ngoại khoa | `KET_QUA_KHAM_NGOAI_KHOA` | Chuỗi | n | | Kết quả khám ngoại khoa tổng quát. |
| 85 | Phân loại KQ khám ngoại khoa | `KHAM_NGOAI_KHOA_PL` | Số | 1 | | Loại I - V. |
| 86 | CKĐT bác sỹ khám ngoại khoa | `CKDT_KHAM_NGOAI_KHOA` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ ngoại khoa. |
| **C** | **Da liễu** | | | | | |
| 87 | KQ khám da liễu | `KET_QUA_KHAM_DA_LIEU` | Chuỗi | n | | Kết quả khám da liễu. |
| 88 | Phân loại KQ khám da liễu | `KHAM_DA_LIEU_PL` | Số | 1 | | Loại I - V. |
| 89 | CKĐT bác sỹ khám da liễu | `CKDT_KHAM_DA_LIEU` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ da liễu. |
| **D** | **Sản phụ khoa** | | | | | |
| 90 | KQ khám sản phụ khoa | `KET_QUA_KHAM_SAN_PHU_KHOA` | Chuỗi | n | | Theo phụ lục XXV kèm TT 32/2023/TT-BYT. |
| 91 | Phân loại KQ khám sản phụ khoa | `KHAM_SAN_PHU_KHOA_PL` | Số | 1 | | Loại I - V. |
| 92 | CKĐT bác sỹ khám sản phụ khoa | `CKDT_KHAM_SAN_PHU_KHOA` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ sản phụ khoa. |
| **E** | **Mắt** | | | | | |
| 93 | Thị lực không kính (phải) | `KHONG_KINH_MAT_PHAI` | Chuỗi | 5 | | Thị lực thang điểm 10 (VD: `10/10`). |
| 94 | Thị lực không kính (trái) | `KHONG_KINH_MAT_TRAI` | Chuỗi | 5 | | Thị lực thang điểm 10 (VD: `10/10`). |
| 95 | Thị lực có kính (phải) | `CO_KINH_MAT_PHAI` | Chuỗi | 5 | | Thị lực sau điều chỉnh kính. |
| 96 | Thị lực có kính (trái) | `CO_KINH_MAT_TRAI` | Chuỗi | 5 | | Thị lực sau điều chỉnh kính. |
| 97 | Các bệnh về mắt | `BENH_KHAC_MAT` | Chuỗi | n | | Bệnh lý về mắt (nếu có). |
| 98 | Phân loại KQ khám mắt | `KHAM_MAT_PL` | Số | 1 | | Loại I - V. |
| 99 | CKĐT bác sỹ khám mắt | `CKDT_KHAM_MAT` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ chuyên khoa mắt. |
| **F** | **Tai - Mũi - Họng** | | | | | |
| 100 | Thính lực tai trái (nói thường) (m) | `TAI_TRAI_NOI_THUONG` | Chuỗi | 10 | | Khoảng cách nghe nói thường (mét, VD: `5.0`). |
| 101 | Thính lực tai trái (nói thầm) (m) | `TAI_TRAI_NOI_THAM` | Chuỗi | 10 | | Khoảng cách nghe nói thầm (mét, VD: `0.5`). |
| 102 | Thính lực tai phải (nói thường) (m) | `TAI_PHAI_NOI_THUONG` | Chuỗi | 10 | | Khoảng cách nghe nói thường (mét, VD: `5.0`). |
| 103 | Thính lực tai phải (nói thầm) (m) | `TAI_PHAI_NOI_THAM` | Chuỗi | 10 | | Khoảng cách nghe nói thầm (mét, VD: `0.5`). |
| 104 | Các bệnh về TMH | `BENH_KHAC_TAI_MUI_HONG` | Chuỗi | n | | Bệnh lý tai mũi họng (nếu có). |
| 105 | Phân loại KQ khám TMH | `KHAM_TAI_MUI_HONG_PL` | Số | 1 | | Loại I - V. |
| 106 | CKĐT bác sỹ khám TMH | `CKDT_KHAM_TAI_MUI_HONG` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ TMH. |
| **G** | **Răng - Hàm - Mặt** | | | | | |
| 107 | Kết quả khám hàm trên | `HAM_TREN` | Chuỗi | n | | Mô tả tình trạng răng hàm trên. |
| 108 | Kết quả khám hàm dưới | `HAM_DUOI` | Chuỗi | n | | Mô tả tình trạng răng hàm dưới. |
| 109 | Các bệnh về RHM | `BENH_KHAC_RANG_HAM_MAT` | Chuỗi | n | | Bệnh lý răng hàm mặt (nếu có). |
| 110 | Phân loại KQ khám RHM | `KHAM_RANG_HAM_MAT_PL` | Số | 1 | | Loại I - V. |
| 111 | CKĐT bác sỹ khám RHM | `CKDT_KHAM_RANG_HAM_MAT` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ RHM. |
| **VI** | **Khám cận lâm sàng** | | | | | |
| 112 | Dịch vụ cận lâm sàng | `MA_DICH_VU` | Chuỗi | 50 | | Mã dịch vụ CLS theo QĐ 3176/QĐ-BYT. |
| 113 | Tên chỉ số cận lâm sàng | `MA_CHI_SO` | Chuỗi | 255 | | Tên chỉ số CLS (CTM, Đường máu, Ure...). |
| 114 | Giá trị chỉ số cận lâm sàng | `GIA_TRI` | Chuỗi | 255 | | Giá trị đo được từ xét nghiệm / CĐHA. |
| 115 | Đơn vị đo chỉ số CLS | `DON_VI_DO` | Chuỗi | 50 | | Đơn vị đo (mmol/L, mg/dL, ...). |
| 116 | Mô tả kết quả chỉ số CLS | `MO_TA` | Chuỗi | n | | Mô tả chi tiết kết quả. |
| 117 | Kết luận chỉ số CLS | `KET_LUAN` | Chuỗi | n | | Kết luận cận lâm sàng (Bình thường / Bất thường). |
| 118 | CKĐT bác sỹ Cận lâm sàng | `CKDT_KET_QUA_CLS` | Chuỗi | n | | Chữ ký điện tử / Họ tên bác sỹ CLS. |
| **VII** | **Kết luận** | | | | | |
| 119 | Phân loại sức khỏe | `PHAN_LOAI_SK` | Số | 1 | **x** | `1`=Loại I (Rất khỏe), `2`=Loại II (Khỏe), `3`=Loại III (Trung bình), `4`=Loại IV (Yếu), `5`=Loại V (Rất yếu). |
| 120 | Kết luận bệnh | `KET_LUAN_BENH` | Chuỗi | 255 | | Mã ICD-10 bệnh phát hiện (ngăn cách bằng dấu `;`). |
| 121 | Tình trạng sức khỏe, bệnh tật | `CAC_BENH_TAT_NEU_CO` | Chuỗi | n | | Tóm tắt tình trạng và khuyến nghị điều trị. |
| **VIII** | **Chữ ký số** | | | | | |
| 122 | Chữ ký số người kết luận | `CKS_NGUOI_KET_LUAN` | Chuỗi | n | | Chuỗi chữ ký số người kết luận. |
| 123 | Chữ ký số CSKCB | `CKS_BENH_VIEN` | Chuỗi | n | | Chuỗi chữ ký số của Bệnh viện / Cơ sở khám bệnh. |

