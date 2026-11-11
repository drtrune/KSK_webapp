/**
 * KSK WebApp - Frontend Logic (Alpine.js + Flatpickr + Tom Select)
 * Phân quyền độc lập: reception (hành chính), tiensu (tiền sử), các chuyên khoa
 */

function kskApp() {
  return {
    token: localStorage.getItem('ksk_token') || '',
    currentUser: null,
    activeTab: 'reception',
    loginForm: { username: '', password: '' },
    loginLoading: false,

    // Danh sách hồ sơ KSK (Sidebar bên trái màn hình 16:9)
    recordsList: [],
    recordsLoading: false,
    recordsFilterStatus: '', // '' (tất cả), '0' (chờ khám / đang khám), '1' (đã khóa)
    recordsSearchQ: '',
    sidebarOpen: true,

    // Tra cứu
    searchMaTn: '',
    searchLoading: false,

    // Hồ sơ hiện tại
    recordId: null,
    recordStatus: 0,
    updatedBy: '',
    updatedAt: '',
    saving: false,
    locking: false,
    unlocking: false,

    // Chế độ in
    printModeReception: false,

    // Form dữ liệu KSK (121 chỉ tiêu)
    form: {
      id: null,
      MA_TIEP_NHAN: '',
      MA_Y_TE: '',
      SO_PHIEU: '',
      MA_CSKCB: '52208',
      MA_GTIN_CSKCB: '8930000522080',
      NGAY_VAO: '',
      HO_TEN: '',
      GIOI_TINH: '1',
      NGAY_SINH: '',
      MA_DAN_TOC: '01',
      SO_CCCD: '',
      NGAYCAP_CCCD: '',
      NOICAP_CCCD: 'Cục Cảnh sát Quản lý hành chính về trật tự xã hội',
      MATINH_CU_TRU: '11221',
      MAXA_CU_TRU: '',
      DIA_CHI: '',
      MA_NGHE_NGHIEP: '00',
      NHOM_MAU: '',
      DOI_TUONG: '13',
      NGUON_CHI_TRA: '9',
      NOI_LAM_VIEC_HOC_TAP: '',
      DIEN_THOAI: '',
      LY_DO_VV: 'Khám sức khỏe',
      MA_LOAI_KCB: '15',

      // 22 Bệnh tiền sử bản thân & gia đình (0: Không, 1: Có)
      TSGD_MAC_BENH: '0',
      TSGD_MA_BENH: '',
      TSBT_BENH_TRONG_5_NAM_QUA: '0',
      TSBT_BENH_THAN_KINH: '0',
      TSBT_BENH_MAT: '0',
      TSBT_BENH_TAI: '0',
      TSBT_BENH_TIM: '0',
      TSBT_PHAU_THUAT_TIM: '0',
      TSBT_TANG_HUYET_AP: '0',
      TSBT_KHO_THO: '0',
      TSBT_BENH_PHOI: '0',
      TSBT_BENH_THAN: '0',
      TSBT_NGHIEN_RUOU: '0',
      TSBT_DAI_THAO_DUONG: '0',
      TSBT_BENH_TAM_THAN: '0',
      TSBT_MAT_Y_THUC: '0',
      TSBT_NGAT: '0',
      TSBT_BENH_TIEU_HOA: '0',
      TSBT_ROI_LOAN_GIAC_NGU: '0',
      TSBT_TAI_BIEN: '0',
      TSBT_BENH_COT_SONG: '0',
      TSBT_RUOU_THUONG_XUYEN: '0',
      TSBT_MA_TUY: '0',
      TSBT_BENH_KHAC: '0',
      TSBT_MA_BENH_KHAC: '',
      TSBT_DANG_DIEU_TRI_BENH: '0',
      TSBT_MA_BENH: '',
      TSBT_TEN_THUOC_LIEU_LUONG: '',
      TSBT_THAI_SAN: '0',
      TSBT_MA_BENH_THAI_SAN: '',
      TSBT_TEN_THUOC_THAI_SAN: '',

      // Thể lực
      CHIEU_CAO: '',
      CAN_NANG: '',
      CHI_SO_BMI: '',
      MACH: '',
      HUYET_AP: '',
      KHAM_THE_LUC_PL: '1',
      BS_THE_LUC: '',

      // Nội khoa
      NOI_KHOA_TUAN_HOAN: '',
      NOI_KHOA_TUAN_HOAN_PL: '1',
      NOI_KHOA_HO_HAP: '',
      NOI_KHOA_HO_HAP_PL: '1',
      NOI_KHOA_TIEU_HOA: '',
      NOI_KHOA_TIEU_HOA_PL: '1',
      NOI_KHOA_THAN_TN_SD: '',
      NOI_KHOA_THAN_TIETNIEU_PL: '1',
      NOI_KHOA_NOI_TIET: '',
      NOI_KHOA_NOI_TIET_PL: '1',
      NOI_KHOA_CO_XUONG_KHOP: '',
      NOI_KHOA_CO_XUONG_KHOP_PL: '1',
      NOI_KHOA_THAN_KINH: '',
      NOI_KHOA_THAN_KINH_PL: '1',
      NOI_KHOA_TAM_THAN: '',
      NOI_KHOA_TAM_THAN_PL: '1',
      BS_NOI: '',

      // Ngoại, Da liễu, Sản
      KET_QUA_KHAM_NGOAI_KHOA: '',
      KHAM_NGOAI_KHOA_PL: '1',
      BS_NGOAI: '',
      KET_QUA_KHAM_DA_LIEU: '',
      KHAM_DA_LIEU_PL: '1',
      BS_DA_LIEU: '',
      KET_QUA_KHAM_SAN_PHU_KHOA: '',
      KHAM_SAN_PHU_KHOA_PL: '1',
      BS_SAN: '',

      // Mắt
      KHONG_KINH_MAT_PHAI: '10/10',
      KHONG_KINH_MAT_TRAI: '10/10',
      CO_KINH_MAT_PHAI: '',
      CO_KINH_MAT_TRAI: '',
      BENH_KHAC_MAT: '',
      KHAM_MAT_PL: '1',
      BS_MAT: '',

      // Tai mũi họng
      TAI_TRAI_NOI_THUONG: '5.0',
      TAI_TRAI_NOI_THAM: '0.5',
      TAI_PHAI_NOI_THUONG: '5.0',
      TAI_PHAI_NOI_THAM: '0.5',
      BENH_KHAC_TAI_MUI_HONG: '',
      KHAM_TAI_MUI_HONG_PL: '1',
      BS_TMH: '',

      // Răng hàm mặt
      HAM_TREN: 'Răng bình thường',
      HAM_DUOI: 'Răng bình thường',
      BENH_KHAC_RANG_HAM_MAT: '',
      KHAM_RANG_HAM_MAT_PL: '1',
      BS_RHM: '',

      // Cận lâm sàng
      CLS_HUYET_HOC: '',
      CLS_DUONG_MAU: '',
      CLS_URE: '',
      CLS_CREATININ: '',
      CLS_ASAT_GOT: '',
      CLS_ALAT_GPT: '',
      CLS_NUOC_TIEU: '',
      CLS_XQ_TIM_PHOI: '',
      CLS_KHAC_CO: '0',
      CLS_KHAC_MO_TA: '',
      CLS_DINH_KY_KET_QUA: '',
      BS_CLS: '',

      // Kết luận
      PHAN_LOAI_SK: '1',
      KET_LUAN_BENH: 'Đủ sức khỏe công tác và học tập',
      CAC_BENH_TAT_NEU_CO: '',
      BS_KET_LUAN: ''
    },

    // Danh mục
    catalogs: {
      doi_tuong: [],
      nguon_chi_tra: [],
      dan_toc: [],
      nghe_nghiep: [],
      icd10: [],
      tinh: [],
      xa: [],
      noi_cap: [],
      nhom_mau: []
    },

    // Quản lý hệ thống
    adminUsers: [],
    adminSubTab: 'users', // 'users', 'his', 'database', 'catalogs'
    adminCatalogItems: [],
    adminCatalogCatType: 'ICD10',
    adminCatalogSearchQ: '',
    adminCatalogLimit: 100,
    adminCatalogLoading: false,
    selectedCatalogIds: [], // Danh sách ID được chọn để xóa nhiều
    showCatalogModal: false,
    catalogModalSaving: false,
    catalogModalForm: { id: null, cat_type: '', code: '', name: '', parent_code: '' },
    showUserModal: false,
    userModalMode: 'create',
    userModalForm: { id: null, username: '', password: '', full_name: '', roles: [] },
    adminSettings: {},
    adminDbInfo: {},
    newDbFilename: '',
    bulkCatalogText: '',
    bulkCatType: 'ICD10',
    facility: {
      ten_cskcb: 'BỆNH VIỆN BÌNH ĐỊNH',
      ma_cskcb: '52208',
      dia_danh: 'Gia Lai'
    },

    // Toasts
    toasts: [],

    // Danh sách 22 bệnh tiền sử bản thân
    historyDiseases: [
      { field: 'TSBT_BENH_TRONG_5_NAM_QUA', label: 'Bệnh hay bị thương trong 5 năm qua' },
      { field: 'TSBT_BENH_THAN_KINH', label: 'Bệnh thần kinh, động kinh' },
      { field: 'TSBT_BENH_MAT', label: 'Bệnh mắt, giảm thị lực nghiêm trọng' },
      { field: 'TSBT_BENH_TAI', label: 'Bệnh tai, nghe kém, điếc' },
      { field: 'TSBT_BENH_TIM', label: 'Bệnh tim mạch, hồi hộp, đánh trống ngực' },
      { field: 'TSBT_PHAU_THUAT_TIM', label: 'Đã phẫu thuật tim mạch' },
      { field: 'TSBT_TANG_HUYET_AP', label: 'Tăng huyết áp' },
      { field: 'TSBT_KHO_THO', label: 'Khó thở, hen suyễn, thở rít' },
      { field: 'TSBT_BENH_PHOI', label: 'Bệnh phổi mạn tính, lao phổi' },
      { field: 'TSBT_BENH_THAN', label: 'Bệnh thận, suy thận mạn' },
      { field: 'TSBT_NGHIEN_RUOU', label: 'Nghiện rượu bia' },
      { field: 'TSBT_DAI_THAO_DUONG', label: 'Đái tháo đường (Tiểu đường)' },
      { field: 'TSBT_BENH_TAM_THAN', label: 'Bệnh tâm thần, rối loạn tâm thần' },
      { field: 'TSBT_MAT_Y_THUC', label: 'Mất ý thức bất thường' },
      { field: 'TSBT_NGAT', label: 'Cơn ngất xỉu đột ngột' },
      { field: 'TSBT_BENH_TIEU_HOA', label: 'Bệnh tiêu hóa, loét dạ dày tá tràng' },
      { field: 'TSBT_ROI_LOAN_GIAC_NGU', label: 'Rối loạn giấc ngủ kéo dài' },
      { field: 'TSBT_TAI_BIEN', label: 'Tai biến mạch máu não / Đột quỵ' },
      { field: 'TSBT_BENH_COT_SONG', label: 'Bệnh cột sống, thoát vị đĩa đệm' },
      { field: 'TSBT_RUOU_THUONG_XUYEN', label: 'Uống rượu thường xuyên' },
      { field: 'TSBT_MA_TUY', label: 'Sử dụng chất gây nghiện, ma túy' },
      { field: 'TSBT_BENH_KHAC', label: 'Các bệnh khác chưa nêu ở trên', hasIcd: true }
    ],

    // Instance Tom Select để điều khiển
    tomInstances: {},

    async initApp() {
      if (this.token) {
        await this.loadMe();
        await this.loadCatalogs();
        await this.loadRecordsList();
      }
      this.$watch('activeTab', () => {
        this.$nextTick(() => {
          this.syncAllSelects();
        });
      });
    },

    toast(message, type = 'success') {
      const id = Date.now() + Math.random();
      this.toasts.push({ id, message, type });
      setTimeout(() => {
        this.toasts = this.toasts.filter(t => t.id !== id);
      }, 4000);
    },

    // Quyền người dùng
    hasRole(r) {
      if (!this.currentUser) return false;
      const roles = (this.currentUser.role || '').split(',').map(s => s.trim());
      return roles.includes('admin') || roles.includes(r);
    },

    get isReceptionist() { return this.hasRole('reception'); },
    get isTiensuRole() { return this.hasRole('tiensu'); },
    get isDoctorTheLuc() { return this.hasRole('bs_theluc'); },
    get isDoctorNoi() { return this.hasRole('bs_noi'); },
    get isDoctorNgoai() { return this.hasRole('bs_ngoai'); },
    get isDoctorDaLieu() { return this.hasRole('bs_dalieu'); },
    get isDoctorSan() { return this.hasRole('bs_san'); },
    get isDoctorMat() { return this.hasRole('bs_mat'); },
    get isDoctorTmh() { return this.hasRole('bs_tmh'); },
    get isDoctorRhm() { return this.hasRole('bs_rhm'); },
    get isDoctorCls() { return this.hasRole('bs_cls'); },
    get isDoctorKetLuan() { return this.hasRole('bs_ketluan'); },
    get isAdmin() { return this.hasRole('admin'); },

    // Phân quyền sửa từng Tab
    get canEditReception() { return !this.isLocked && (this.isAdmin || this.isReceptionist); },
    get canEditHistory() { return !this.isLocked && (this.isAdmin || this.isTiensuRole); },
    get canEditPhysical() { return !this.isLocked && (this.isAdmin || this.isDoctorTheLuc); },
    get canEditClinicalNoi() { return !this.isLocked && (this.isAdmin || this.isDoctorNoi); },
    get canEditClinicalNgoai() { return !this.isLocked && (this.isAdmin || this.isDoctorNgoai); },
    get canEditClinicalDaLieu() { return !this.isLocked && (this.isAdmin || this.isDoctorDaLieu); },
    get canEditClinicalSan() { return !this.isLocked && (this.isAdmin || this.isDoctorSan); },
    get canEditClinicalMat() { return !this.isLocked && (this.isAdmin || this.isDoctorMat); },
    get canEditClinicalTmh() { return !this.isLocked && (this.isAdmin || this.isDoctorTmh); },
    get canEditClinicalRhm() { return !this.isLocked && (this.isAdmin || this.isDoctorRhm); },
    get canEditParaclinical() { return !this.isLocked && (this.isAdmin || this.isDoctorCls); },
    get canEditConclusion() { return !this.isLocked && (this.isAdmin || this.isDoctorKetLuan); },

    get isLocked() { return this.recordStatus === 1; },
    get isSaved() { return !!this.recordId; },

    // Khởi tạo Flatpickr
    initDatePicker(el, fieldName) {
      if (typeof flatpickr === 'undefined') return;
      flatpickr(el, {
        dateFormat: 'd/m/Y',
        allowInput: true,
        locale: (window.flatpickr && flatpickr.l1ons && flatpickr.l1ons.vn) ? flatpickr.l1ons.vn : 'default',
        onChange: (selectedDates, dateStr) => {
          this.form[fieldName] = dateStr;
        }
      });
    },

    // Khởi tạo Tom Select cho ICD-10 (Đa chọn / Tìm kiếm nhanh từ xa Remote Query)
    initIcdMultiSelect(el, fieldName) {
      if (typeof TomSelect === 'undefined' || !el) return;
      if (this.tomInstances[fieldName]) {
        try { this.tomInstances[fieldName].destroy(); } catch (e) { }
      }

      const instance = new TomSelect(el, {
        plugins: ['remove_button'],
        valueField: 'value',
        labelField: 'text',
        searchField: ['value', 'text'],
        create: true,
        maxItems: 20,
        placeholder: 'Gõ mã ICD (ví dụ: I10, E11) hoặc tên bệnh tật...',
        load: (query, callback) => {
          if (!query || !query.trim().length) return callback();
          fetch(`/api/catalogs?cat_type=ICD10&q=${encodeURIComponent(query.trim())}&limit=30`, {
            headers: { 'Authorization': `Bearer ${this.token}` }
          })
            .then(response => response.json())
            .then(data => {
              const results = (data || []).map(item => ({
                value: item.code,
                text: `${item.code} - ${item.name}`
              }));
              callback(results);
            })
            .catch(() => callback());
        },
        render: {
          item: function (data, escape) {
            return `<div>${escape(data.text || data.value)}</div>`;
          },
          option: function (data, escape) {
            return `<div><span class="font-bold text-sky-700">${escape(data.value)}</span> - ${escape(data.text ? data.text.replace(data.value + ' - ', '') : '')}</div>`;
          }
        },
        onChange: (values) => {
          this.form[fieldName] = Array.isArray(values) ? values.join('; ') : (values || '');
        }
      });

      // Nếu đã có giá trị sẵn trong form (ví dụ khi nạp hồ sơ cũ)
      if (this.form[fieldName]) {
        const valArr = this.form[fieldName].split(/[,;]/).map(s => s.trim()).filter(Boolean);
        valArr.forEach(v => {
          instance.addOption({ value: v, text: v });
        });
        instance.setValue(valArr, true);
      }

      this.tomInstances[fieldName] = instance;
    },

    // Cập nhật / Khởi tạo Select Tìm kiếm Đơn (Tỉnh / Xã / Dân tộc / Nghề nghiệp / Đối tượng / Nguồn chi trả / Nơi cấp / Nhóm máu)
    updateSearchableSelect(selectId, fieldName, catalogList, defaultVal, isFreeText = false) {
      const el = document.getElementById(selectId);
      if (!el) return;

      const activeItems = (catalogList || []).filter(c => c.is_active !== 0);
      const formatted = activeItems.map(c => {
        const val = isFreeText ? (c.name || c.code) : String(c.code);
        const txt = isFreeText ? (c.name || c.code) : `${c.code} - ${c.name}`;
        return { value: String(val), text: String(txt) };
      });

      // 1. Điền sẵn option thuần vào thẻ <select> để đảm bảo luôn hiển thị
      let optionsHtml = '<option value="">-- Tìm kiếm hoặc chọn --</option>';
      for (const opt of formatted) {
        optionsHtml += `<option value="${opt.value}">${opt.text}</option>`;
      }
      el.innerHTML = optionsHtml;

      const valToSet = this.form[fieldName] || defaultVal || '';
      if (valToSet) {
        this.form[fieldName] = String(valToSet);
        el.value = String(valToSet);
      }

      // 2. Nếu có TomSelect thì khởi tạo nâng cao
      if (typeof TomSelect !== 'undefined') {
        let instance = this.tomInstances[fieldName];
        if (!instance) {
          instance = new TomSelect(el, {
            valueField: 'value',
            labelField: 'text',
            searchField: ['value', 'text'],
            create: isFreeText,
            maxOptions: 1200,
            placeholder: '-- Tìm kiếm hoặc chọn --',
            onChange: (val) => {
              this.form[fieldName] = val || '';
              if (fieldName === 'MATINH_CU_TRU') {
                this.onProvinceChange(val);
              }
            }
          });
          this.tomInstances[fieldName] = instance;
        }

        instance.clearOptions();
        instance.addOptions(formatted);
        if (isFreeText && valToSet && !formatted.some(f => f.value === valToSet)) {
          instance.addOption({ value: valToSet, text: valToSet });
        }
        instance.refreshOptions(false);

        if (valToSet) {
          instance.setValue(String(valToSet), true);
        } else {
          instance.clear(true);
        }
      }
    },

    // Khởi tạo tất cả Searchable Select ở Tiếp nhận
    initAllSearchableSelects() {
      this.updateSearchableSelect('select_noicap_cccd', 'NOICAP_CCCD', this.catalogs.noi_cap, 'Cục Cảnh sát QLHC về TTXH', true);
      this.updateSearchableSelect('select_nhom_mau', 'NHOM_MAU', this.catalogs.nhom_mau && this.catalogs.nhom_mau.length ? this.catalogs.nhom_mau : [{ name: 'A' }, { name: 'B' }, { name: 'AB' }, { name: 'O' }], '', true);
      this.updateSearchableSelect('select_ma_dan_toc', 'MA_DAN_TOC', this.catalogs.dan_toc, '01');
      this.updateSearchableSelect('select_ma_nghe_nghiep', 'MA_NGHE_NGHIEP', this.catalogs.nghe_nghiep, '00');
      this.updateSearchableSelect('select_matinh_cu_tru', 'MATINH_CU_TRU', this.catalogs.tinh, '11221');
      this.updateSearchableSelect('select_doi_tuong', 'DOI_TUONG', this.catalogs.doi_tuong, '13');
      this.updateSearchableSelect('select_nguon_chi_tra', 'NGUON_CHI_TRA', this.catalogs.nguon_chi_tra, '9');
      if (this.catalogs.xa && this.catalogs.xa.length) {
        this.updateSearchableSelect('select_maxa_cu_tru', 'MAXA_CU_TRU', this.catalogs.xa, this.form.MAXA_CU_TRU);
      }
    },

    // Đồng bộ giá trị tất cả TomSelect trên giao diện
    syncAllSelects() {
      setTimeout(() => {
        // 1. Single selects
        const singleFields = ['NOICAP_CCCD', 'NHOM_MAU', 'MA_DAN_TOC', 'MA_NGHE_NGHIEP', 'MATINH_CU_TRU', 'MAXA_CU_TRU', 'DOI_TUONG', 'NGUON_CHI_TRA'];
        for (const f of singleFields) {
          if (this.tomInstances[f]) {
            if (this.form[f]) {
              this.tomInstances[f].setValue(String(this.form[f]), true);
            } else {
              this.tomInstances[f].clear(true);
            }
          }
        }

        // 2. Multi selects ICD-10
        const multiFields = ['CAC_BENH_TAT_NEU_CO', 'TSGD_MA_BENH', 'TSBT_MA_BENH_KHAC', 'TSBT_MA_BENH', 'TSBT_MA_BENH_THAI_SAN'];
        for (const f of multiFields) {
          if (this.tomInstances[f]) {
            this.tomInstances[f].clear(true);
            if (this.form[f]) {
              const arr = this.form[f].split(/[,;]/).map(s => s.trim()).filter(Boolean);
              arr.forEach(code => {
                this.tomInstances[f].addOption({ value: code, text: code });
              });
              this.tomInstances[f].setValue(arr, true);
            }
          }
        }
      }, 50);
    },

    // Xử lý bật/tắt disabled cho các TomSelect
    setSelectsDisabled(disabled) {
      for (let k in this.tomInstances) {
        if (this.tomInstances[k]) {
          if (disabled) {
            this.tomInstances[k].disable();
          } else {
            this.tomInstances[k].enable();
          }
        }
      }
    },

    // Tính toán BMI
    calcBmi() {
      const cc = parseFloat((this.form.CHIEU_CAO || '').replace(',', '.'));
      const cn = parseFloat((this.form.CAN_NANG || '').replace(',', '.'));
      if (cc > 30 && cn > 2) {
        const h_m = cc / 100.0;
        const bmi = cn / (h_m * h_m);
        this.form.CHI_SO_BMI = bmi.toFixed(2);
      } else {
        this.form.CHI_SO_BMI = '';
      }
    },

    // API ME
    async loadMe() {
      try {
        const res = await fetch('/api/me', {
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        if (res.ok) {
          const data = await res.json();
          this.currentUser = data.user;
          this.facility = {
            ten_cskcb: data.facility?.ten_cskcb || data.cskcb?.ten_cskcb || 'BỆNH VIỆN BÌNH ĐỊNH',
            ma_cskcb: data.facility?.ma_cskcb || data.cskcb?.ma_cskcb || '52208',
            dia_danh: data.facility?.dia_danh || data.cskcb?.dia_danh || 'Gia Lai'
          };
          if (this.isAdmin) {
            await this.loadAdminUsers();
            await this.loadAdminSettings();
            await this.fetchAdminCatalogItems();
          }
        } else {
          this.logout();
        }
      } catch (err) {
        console.error('Lỗi load user me:', err);
      }
    },

    // Đăng nhập
    async handleLogin() {
      if (!this.loginForm.username || !this.loginForm.password) {
        this.toast('Vui lòng nhập tên đăng nhập và mật khẩu', 'error');
        return;
      }
      this.loginLoading = true;
      try {
        const res = await fetch('/api/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(this.loginForm)
        });
        const data = await res.json();
        if (res.ok) {
          this.token = data.token;
          localStorage.setItem('ksk_token', this.token);
          this.currentUser = data.user;
          this.toast(`Chào mừng ${data.user.full_name}!`);
          await this.loadCatalogs();
          await this.loadRecordsList();
          if (this.isAdmin) {
            await this.loadAdminUsers();
            await this.loadAdminSettings();
            await this.fetchAdminCatalogItems();
          }
        } else {
          this.toast(data.error || 'Đăng nhập không thành công', 'error');
        }
      } catch (err) {
        this.toast('Không thể kết nối đến máy chủ', 'error');
      } finally {
        this.loginLoading = false;
      }
    },

    logout() {
      this.token = '';
      this.currentUser = null;
      localStorage.removeItem('ksk_token');
      this.resetForm();
    },

    // Tải danh mục có cơ chế Caching LocalStorage & Timestamp Check
    async loadCatalogs() {
      if (!this.token) return;
      try {
        // 1. Lấy phiên bản/timestamp cập nhật từ server
        let serverVersions = {};
        const vRes = await fetch('/api/catalogs/versions', {
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        if (vRes.ok) {
          serverVersions = await vRes.json();
        }

        const localVersionsStr = localStorage.getItem('ksk_catalog_versions');
        let localVersions = {};
        try { localVersions = JSON.parse(localVersionsStr || '{}'); } catch (e) { }

        // Danh mục cần cho Tiếp nhận & Nghiệp vụ thông thường (không bao gồm ICD10, XA_PHUONG)
        const catalogDefs = [
          { key: 'tinh', cat_type: 'TINH_TP' },
          { key: 'dan_toc', cat_type: 'DAN_TOC' },
          { key: 'nghe_nghiep', cat_type: 'NGHE_NGHIEP' },
          { key: 'noi_cap', cat_type: 'NOI_CAP' },
          { key: 'nhom_mau', cat_type: 'NHOM_MAU' },
          { key: 'doi_tuong', cat_type: 'DOI_TUONG' },
          { key: 'nguon_chi_tra', cat_type: 'NGUON_CHI_TRA' }
        ];

        for (const item of catalogDefs) {
          const sVer = serverVersions[item.cat_type];
          const lVer = localVersions[item.cat_type];
          const cachedData = localStorage.getItem(`ksk_cat_${item.cat_type}`);

          // Nếu có cache và timestamp không đổi -> lấy trực tiếp từ localStorage
          if (cachedData && sVer && sVer === lVer) {
            try {
              this.catalogs[item.key] = JSON.parse(cachedData);
              continue;
            } catch (e) { }
          }

          // Ngược lại, fetch từ server và cập nhật cache
          try {
            const res = await fetch(`/api/catalogs?cat_type=${item.cat_type}&limit=2000`, {
              headers: { 'Authorization': `Bearer ${this.token}` }
            });
            if (res.ok) {
              const data = await res.json();
              this.catalogs[item.key] = data;
              localStorage.setItem(`ksk_cat_${item.cat_type}`, JSON.stringify(data));
              localVersions[item.cat_type] = sVer || new Date().toISOString();
            }
          } catch (e) {
            console.error(`Lỗi tải danh mục ${item.cat_type}:`, e);
          }
        }

        localStorage.setItem('ksk_catalog_versions', JSON.stringify(localVersions));

        // 2. Khởi tạo tất cả Searchable Select ở Tiếp nhận
        this.initAllSearchableSelects();

        // 3. Tải danh mục Xã/Phường theo Tỉnh hiện tại (On-demand/Lazy)
        if (this.form.MATINH_CU_TRU) {
          await this.loadWardsForProvince(this.form.MATINH_CU_TRU);
        }
      } catch (err) {
        console.error('Lỗi kiểm tra danh mục:', err);
      }
    },

    // Xử lý khi người dùng chọn Tỉnh khác
    async onProvinceChange(matinh) {
      const pCode = matinh || this.form.MATINH_CU_TRU;
      this.form.MAXA_CU_TRU = '';
      if (pCode) {
        await this.loadWardsForProvince(pCode);
      } else {
        this.catalogs.xa = [];
        this.updateSearchableSelect('select_maxa_cu_tru', 'MAXA_CU_TRU', [], '');
      }
    },

    // Tải Xã/Phường theo Tỉnh (On-demand kèm Cache theo từng Tỉnh)
    async loadWardsForProvince(matinh) {
      if (!matinh) {
        this.catalogs.xa = [];
        this.updateSearchableSelect('select_maxa_cu_tru', 'MAXA_CU_TRU', [], '');
        return;
      }
      const cacheKey = `ksk_wards_${matinh}`;
      const cached = sessionStorage.getItem(cacheKey);
      if (cached) {
        try {
          this.catalogs.xa = JSON.parse(cached);
          this.updateSearchableSelect('select_maxa_cu_tru', 'MAXA_CU_TRU', this.catalogs.xa, this.form.MAXA_CU_TRU);
          return;
        } catch (e) { }
      }

      try {
        const res = await fetch(`/api/catalogs?cat_type=XA_PHUONG&parent_code=${encodeURIComponent(matinh)}&limit=1000`, {
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        if (res.ok) {
          const list = await res.json();
          this.catalogs.xa = list;
          try { sessionStorage.setItem(cacheKey, JSON.stringify(list)); } catch (e) { }
          this.updateSearchableSelect('select_maxa_cu_tru', 'MAXA_CU_TRU', this.catalogs.xa, this.form.MAXA_CU_TRU);
        }
      } catch (e) {
        console.error('Lỗi tải xã phường theo tỉnh:', e);
      }
    },

    // Tải Danh sách Khám (Sidebar danh sách bệnh nhân)
    async loadRecordsList() {
      if (!this.token) return;
      this.recordsLoading = true;
      try {
        let url = `/api/records?limit=100`;
        if (this.recordsFilterStatus !== '') {
          url += `&status=${this.recordsFilterStatus}`;
        }
        if (this.recordsSearchQ.trim()) {
          url += `&q=${encodeURIComponent(this.recordsSearchQ.trim())}`;
        }
        const res = await fetch(url, {
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        if (res.ok) {
          this.recordsList = await res.json();
        }
      } catch (e) {
        console.error('Lỗi tải danh sách hồ sơ:', e);
      } finally {
        this.recordsLoading = false;
      }
    },

    // Chọn 1 hồ sơ từ danh sách để nạp vào form nhập liệu
    selectRecord(rec) {
      this.loadRecordIntoForm(rec);
      this.searchMaTn = rec.ma_tiep_nhan;
      // Nếu đang là nhân viên chuyên khoa thì tự động chuyển tab phù hợp
      if (this.isDoctorTheLuc && this.activeTab === 'reception') {
        this.activeTab = 'physical';
      } else if (this.isTiensuRole && this.activeTab === 'reception') {
        this.activeTab = 'history';
      } else if ((this.isDoctorNoi || this.isDoctorNgoai || this.isDoctorDaLieu || this.isDoctorSan || this.isDoctorMat || this.isDoctorTmh || this.isDoctorRhm) && this.activeTab === 'reception') {
        this.activeTab = 'clinical';
      } else if (this.isDoctorCls && this.activeTab === 'reception') {
        this.activeTab = 'paraclinical';
      } else if (this.isDoctorKetLuan && this.activeTab === 'reception') {
        this.activeTab = 'conclusion';
      }
      this.toast(`Đã chọn bệnh nhân ${rec.ho_ten} (${rec.ma_tiep_nhan})`);
    },

    // Tìm kiếm / Tải dữ liệu từ HIS
    async handleSearchOrHis(inputMaTn = null) {
      const maTn = (inputMaTn || this.form.MA_TIEP_NHAN || this.searchMaTn || '').trim();
      if (!maTn) {
        this.toast('Vui lòng nhập Mã tiếp nhận để tra cứu', 'error');
        return;
      }
      this.searchMaTn = maTn;
      this.form.MA_TIEP_NHAN = maTn;

      this.searchLoading = true;
      try {
        // 1. Kiểm tra xem mã này đã có trong CSDL KSK chưa
        const resDb = await fetch(`/api/records/by-ma-tn/${encodeURIComponent(maTn)}`, {
          headers: { 'Authorization': `Bearer ${this.token}` }
        });

        if (resDb.ok) {
          const record = await resDb.json();
          this.loadRecordIntoForm(record);
          this.toast(`Đã tải hồ sơ KSK [${maTn}] từ cơ sở dữ liệu.`);
          this.searchLoading = false;
          return;
        }

        // 2. Nếu chưa có trong DB, gọi API HIS để lấy dữ liệu hành chính
        const resHis = await fetch(`/api/his?ma_tn=${encodeURIComponent(maTn)}`, {
          headers: { 'Authorization': `Bearer ${this.token}` }
        });

        if (resHis.ok) {
          const hisData = await resHis.json();
          if (hisData && hisData.data) {
            const p = hisData.data;
            this.resetForm();
            this.form.MA_TIEP_NHAN = p.ma_tn || maTn;
            this.form.MA_Y_TE = p.ma_y_te || '';
            this.form.HO_TEN = p.ho_ten || '';
            this.form.GIOI_TINH = String(p.gioi_tinh || '1');
            this.form.NGAY_SINH = p.ngay_sinh || '';
            this.form.SO_CCCD = p.so_cccd || '';
            this.form.NGAYCAP_CCCD = p.ngaycap_cccd || '';
            this.form.NOICAP_CCCD = p.noicap_cccd || 'Cục Cảnh sát QLHC về TTXH';
            this.form.MA_DAN_TOC = p.ma_dan_toc || '01';
            this.form.MA_NGHE_NGHIEP = p.ma_nghe_nghiep || '00';
            this.form.DIEN_THOAI = p.so_dien_thoai || '';
            this.form.DIA_CHI = p.dia_chi || '';
            this.form.MATINH_CU_TRU = p.matinh_cu_tru || '11221';
            this.form.MAXA_CU_TRU = p.maxa_cu_tru || '';
            this.form.NGAY_VAO = p.ngay_vao || this.todayFormatted();

            if (this.form.MATINH_CU_TRU) {
              await this.loadWardsForProvince(this.form.MATINH_CU_TRU);
            }
            this.syncAllSelects();

            this.toast(`Đã lấy dữ liệu hành chính từ HIS cho bệnh nhân ${this.form.HO_TEN}! Vui lòng bấm "Lưu tiếp nhận".`);
          } else {
            this.toast('Không tìm thấy thông tin bệnh nhân trên HIS', 'error');
          }
        } else {
          const err = await resHis.json().catch(() => ({}));
          this.toast(err.error || 'Không thể kết nối đến API HIS', 'error');
        }
      } catch (e) {
        this.toast('Lỗi khi tra cứu thông tin tiếp nhận', 'error');
      } finally {
        this.searchLoading = false;
      }
    },

    async loadRecordIntoForm(rec) {
      this.recordId = rec.id;
      this.recordStatus = rec.status;
      this.updatedBy = rec.updated_by || '';
      this.updatedAt = rec.updated_at || '';

      const payload = rec.payload
        ? rec.payload
        : (typeof rec.payload_json === 'string' ? JSON.parse(rec.payload_json || '{}') : (rec.payload_json || {}));

      // Chuẩn hóa mã tỉnh/xã cũ (nếu có) sang mã mới trong danh mục
      if (payload.MATINH_CU_TRU === '52' || payload.MATINH_CU_TRU === '64' || !payload.MATINH_CU_TRU) {
        payload.MATINH_CU_TRU = '11221';
      }
      if (payload.MAXA_CU_TRU === '21907' || payload.MAXA_CU_TRU === '21592' || payload.MAXA_CU_TRU === '21562') {
        payload.MAXA_CU_TRU = rec.ma_tiep_nhan === 'TN.2611.0004019' ? '13339' : '13338';
      }

      this.form = { ...this.form, ...payload, id: rec.id };
      this.form.MA_TIEP_NHAN = rec.ma_tiep_nhan || this.form.MA_TIEP_NHAN;

      // Nạp xã phường của tỉnh tương ứng nếu có
      if (this.form.MATINH_CU_TRU) {
        await this.loadWardsForProvince(this.form.MATINH_CU_TRU);
      }

      this.syncAllSelects();
    },

    resetForm() {
      this.recordId = null;
      this.recordStatus = 0;
      this.updatedBy = '';
      this.updatedAt = '';
      this.searchMaTn = '';
      this.activeTab = 'reception';

      for (let k in this.form) {
        if (k.startsWith('TSBT_') || k.startsWith('TSGD_')) {
          this.form[k] = (k === 'TSBT_MA_BENH_KHAC' || k === 'TSGD_MA_BENH' || k === 'TSBT_MA_BENH' || k === 'TSBT_TEN_THUOC_LIEU_LUONG' || k === 'TSBT_MA_BENH_THAI_SAN' || k === 'TSBT_TEN_THUOC_THAI_SAN') ? '' : '0';
        } else if (k === 'GIOI_TINH' || k === 'KHAM_THE_LUC_PL' || k === 'PHAN_LOAI_SK') {
          this.form[k] = '1';
        } else if (k === 'MA_DAN_TOC') {
          this.form[k] = '01';
        } else if (k === 'MA_NGHE_NGHIEP') {
          this.form[k] = '00';
        } else if (k === 'DOI_TUONG') {
          this.form[k] = '13';
        } else if (k === 'NGUON_CHI_TRA') {
          this.form[k] = '9';
        } else if (k === 'MATINH_CU_TRU') {
          this.form[k] = '11221';
        } else if (k === 'MAXA_CU_TRU') {
          this.form[k] = '';
        } else if (k === 'MA_LOAI_KCB') {
          this.form[k] = '15';
        } else if (k.endsWith('_PL')) {
          this.form[k] = '1';
        } else if (k === 'MA_CSKCB') {
          this.form[k] = this.facility.ma_cskcb || '52208';
        } else if (k === 'MA_GTIN_CSKCB') {
          this.form[k] = this.facility.ma_gtin_cskcb || '8930000522080';
        } else if (k === 'NOICAP_CCCD') {
          this.form[k] = 'Cục Cảnh sát QLHC về TTXH';
        } else if (k === 'LY_DO_VV') {
          this.form[k] = 'Khám sức khỏe';
        } else {
          this.form[k] = '';
        }
      }
      this.form.NGAY_VAO = this.todayFormatted();

      // Nạp xã cho Gia Lai mặc định
      if (this.form.MATINH_CU_TRU) {
        this.loadWardsForProvince(this.form.MATINH_CU_TRU);
      }
      this.syncAllSelects();
    },

    todayFormatted() {
      const d = new Date();
      const dd = String(d.getDate()).padStart(2, '0');
      const mm = String(d.getMonth() + 1).padStart(2, '0');
      return `${dd}/${mm}/${d.getFullYear()}`;
    },

    // Lưu hồ sơ
    async handleSave() {
      if (!this.form.MA_TIEP_NHAN) {
        this.toast('Vui lòng nhập Mã tiếp nhận trước khi lưu!', 'error');
        return;
      }
      if (!this.form.HO_TEN) {
        this.toast('Vui lòng nhập Họ và tên bệnh nhân!', 'error');
        return;
      }

      this.saving = true;
      try {
        const res = await fetch('/api/records', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${this.token}`
          },
          body: JSON.stringify({
            ma_tiep_nhan: this.form.MA_TIEP_NHAN,
            payload: this.form
          })
        });

        const data = await res.json();
        if (res.ok) {
          const rec = data.record || data;
          this.recordId = rec.id;
          this.recordStatus = rec.status;
          this.form.id = rec.id;
          this.updatedBy = rec.updated_by;
          this.updatedAt = rec.updated_at;
          this.toast('Đã lưu hồ sơ thành công! Đã mở quyền In phiếu.');
          await this.loadRecordsList();
        } else {
          this.toast(data.error || 'Lỗi khi lưu hồ sơ', 'error');
        }
      } catch (err) {
        this.toast('Không thể kết nối đến máy chủ khi lưu', 'error');
      } finally {
        this.saving = false;
      }
    },

    // Ký / Khóa hồ sơ
    async handleLock() {
      if (!this.recordId) return;
      if (!confirm('Bạn có chắc chắn muốn KÝ & KHÓA hồ sơ này? Khi khóa, tất cả chuyên khoa sẽ bị khóa cứng chỉ đọc.')) {
        return;
      }

      this.locking = true;
      try {
        const res = await fetch(`/api/records/${this.recordId}/lock`, {
          method: 'POST',
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        const data = await res.json();
        if (res.ok) {
          this.recordStatus = 1;
          this.toast('Hồ sơ đã được KÝ VÀ KHÓA THÀNH CÔNG (status = 1)!');
          await this.loadRecordsList();
        } else {
          this.toast(data.error || 'Lỗi khi khóa hồ sơ', 'error');
        }
      } catch (e) {
        this.toast('Lỗi kết nối khi khóa hồ sơ', 'error');
      } finally {
        this.locking = false;
      }
    },

    // Mở khóa hồ sơ (Admin only)
    async handleUnlock() {
      if (!this.recordId) return;
      if (!confirm('Bạn là Quản trị viên. Bạn có chắc chắn muốn MỞ KHÓA hồ sơ này để các khoa chỉnh sửa?')) {
        return;
      }

      this.unlocking = true;
      try {
        const res = await fetch(`/api/records/${this.recordId}/unlock`, {
          method: 'POST',
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        const data = await res.json();
        if (res.ok) {
          this.recordStatus = 0;
          this.toast('Đã MỞ KHÓA hồ sơ thành công (status = 0)!');
          await this.loadRecordsList();
        } else {
          this.toast(data.error || 'Lỗi khi mở khóa', 'error');
        }
      } catch (e) {
        this.toast('Lỗi kết nối khi mở khóa', 'error');
      } finally {
        this.unlocking = false;
      }
    },

    // In Phiếu Tiếp Nhận (In phôi 5 trang) - Ủy quyền xử lý cho PrintService tại print.html
    async printBlank() {
      if (window.PrintService && window.PrintService.printBlank) {
        await window.PrintService.printBlank(this);
      } else {
        window.print();
      }
    },

    printReceptionBlank() {
      if (window.PrintService && window.PrintService.printReceptionBlank) {
        window.PrintService.printReceptionBlank(this);
      } else {
        this.printBlank();
      }
    },

    // In Phiếu KSK Hoàn chỉnh
    printFullRecord() {
      if (window.PrintService && window.PrintService.printFullRecord) {
        window.PrintService.printFullRecord(this);
      } else {
        this.printBlank();
      }
    },

    // Tải file DOCX (Word)
    downloadDocx(mode = 'full') {
      if (!this.recordId) {
        this.toast('Vui lòng lưu hồ sơ trước khi xuất Word!', 'error');
        return;
      }
      const url = `/api/records/${this.recordId}/docx?mode=${mode}`;
      window.open(url, '_blank');
    },

    // Xuất CSV 89 cột QĐ 3176
    downloadCsv() {
      window.open('/api/export/csv', '_blank');
    },

    // =========================================================================
    // QUẢN TRỊ ADMIN (USERS, CATALOGS, SETTINGS, DB)
    // =========================================================================
    async loadAdminUsers() {
      try {
        const res = await fetch('/api/users', {
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        if (res.ok) {
          this.adminUsers = await res.json();
        }
      } catch (e) {
        console.error('Lỗi tải người dùng:', e);
      }
    },

    openCreateUserModal() {
      this.userModalMode = 'create';
      this.userModalForm = { id: null, username: '', password: '', full_name: '', roles: ['reception'] };
      this.showUserModal = true;
    },

    openEditUserModal(u) {
      this.userModalMode = 'edit';
      const roleArr = (u.role || '').split(',').map(s => s.trim()).filter(Boolean);
      this.userModalForm = { id: u.id, username: u.username, password: '', full_name: u.full_name, roles: roleArr };
      this.showUserModal = true;
    },

    async saveUserModal() {
      if (!this.userModalForm.full_name || this.userModalForm.roles.length === 0) {
        this.toast('Vui lòng nhập họ tên và chọn ít nhất 1 quyền!', 'error');
        return;
      }

      if (this.userModalMode === 'create') {
        if (!this.userModalForm.username || !this.userModalForm.password) {
          this.toast('Vui lòng nhập tên đăng nhập và mật khẩu!', 'error');
          return;
        }
        try {
          const res = await fetch('/api/users', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
            body: JSON.stringify(this.userModalForm)
          });
          const data = await res.json();
          if (res.ok) {
            this.toast('Đã tạo người dùng mới thành công!');
            this.showUserModal = false;
            await this.loadAdminUsers();
          } else {
            this.toast(data.error || 'Lỗi tạo người dùng', 'error');
          }
        } catch (e) {
          this.toast('Lỗi kết nối khi tạo người dùng', 'error');
        }
      } else {
        try {
          const res = await fetch(`/api/users/${this.userModalForm.id}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
            body: JSON.stringify({
              full_name: this.userModalForm.full_name,
              roles: this.userModalForm.roles,
              password: this.userModalForm.password ? this.userModalForm.password : undefined
            })
          });
          const data = await res.json();
          if (res.ok) {
            this.toast('Đã cập nhật thông tin người dùng thành công!');
            this.showUserModal = false;
            await this.loadAdminUsers();
          } else {
            this.toast(data.error || 'Lỗi cập nhật người dùng', 'error');
          }
        } catch (e) {
          this.toast('Lỗi kết nối khi cập nhật người dùng', 'error');
        }
      }
    },

    async toggleLockUser(u) {
      if (!confirm(`Bạn có chắc muốn ${u.is_locked ? 'Mở khóa' : 'Khóa'} tài khoản ${u.username}?`)) return;
      try {
        const res = await fetch(`/api/users/${u.id}/toggle-lock`, {
          method: 'POST',
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        if (res.ok) {
          this.toast(`Đã ${u.is_locked ? 'mở khóa' : 'khóa'} tài khoản ${u.username}`);
          await this.loadAdminUsers();
        }
      } catch (e) {
        this.toast('Lỗi thao tác khóa tài khoản', 'error');
      }
    },

    // Cấu hình Settings
    async loadAdminSettings() {
      try {
        const res = await fetch('/api/admin/settings', {
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        if (res.ok) {
          const data = await res.json();
          this.adminSettings = data.config || {};
          this.adminDbInfo = data.db_info || {};
        }
      } catch (e) {
        console.error('Lỗi tải cấu hình Admin:', e);
      }
    },

    async saveAdminSettings() {
      try {
        const res = await fetch('/api/admin/settings', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
          body: JSON.stringify(this.adminSettings)
        });
        if (res.ok) {
          this.toast('Đã lưu cấu hình hệ thống thành công!');
          await this.loadAdminSettings();
        } else {
          this.toast('Lỗi khi lưu cấu hình', 'error');
        }
      } catch (e) {
        this.toast('Lỗi kết nối khi lưu cấu hình', 'error');
      }
    },

    setToDefaultMockHis() {
      this.adminSettings.his_use_mock = true;
      this.adminSettings.his_api_url = 'http://127.0.0.1:8080/api/his/mock';
      this.adminSettings.his_http_method = 'POST';
      this.adminSettings.his_auth_header = 'x-api-key';
      this.adminSettings.his_api_key = 'HIS_SECRET_API_KEY_2026';
      this.adminSettings.his_payload_template = '{"ma_tiep_nhan": "{{ma_tn}}"}';
      this.adminSettings.his_timeout_seconds = 10;
      this.toast('Đã thiết lập về Mock API HIS nội bộ mặc định!');
    },

    async testHisConnection() {
      try {
        const res = await fetch('/api/admin/his/test', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
          body: JSON.stringify({
            api_url: this.adminSettings.his_api_url,
            api_key: this.adminSettings.his_api_key,
            auth_header: this.adminSettings.his_auth_header,
            http_method: this.adminSettings.his_http_method,
            payload_template: this.adminSettings.his_payload_template,
            ma_tn: 'TN.2611.0004018'
          })
        });
        const data = await res.json();
        if (res.ok && data.success) {
          this.toast(`Kết nối API HIS thành công! Mã phản hồi: ${data.status}`);
        } else {
          this.toast(`Kiểm tra HIS thất bại: ${data.message || data.error}`, 'error');
        }
      } catch (e) {
        this.toast('Không thể kết nối đến máy chủ để test HIS', 'error');
      }
    },

    async switchDatabase(targetPath) {
      if (!confirm(`Bạn có chắc muốn chuyển hoặc tạo database tại "${targetPath}"?`)) return;
      try {
        const res = await fetch('/api/admin/db/switch', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
          body: JSON.stringify({ database_path: targetPath })
        });
        const data = await res.json();
        if (res.ok) {
          this.toast(`Đã chuyển database thành công! Đường dẫn mới: ${data.current_database_path}`);
          this.newDbFilename = '';
          await this.loadAdminSettings();
          await this.loadCatalogs();
          await this.loadRecordsList();
          this.resetForm();
        } else {
          this.toast(data.error || 'Lỗi khi chuyển database', 'error');
        }
      } catch (e) {
        this.toast('Lỗi kết nối khi chuyển database', 'error');
      }
    },

    async handleBulkImportCatalogs() {
      if (!this.bulkCatalogText.trim()) {
        this.toast('Vui lòng dán dữ liệu danh mục vào ô trống!', 'error');
        return;
      }
      try {
        const res = await fetch('/api/catalogs/bulk', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
          body: JSON.stringify({
            cat_type: this.bulkCatType,
            raw_text: this.bulkCatalogText,
            replace_all: false
          })
        });
        const data = await res.json();
        if (res.ok) {
          this.toast(data.message || `Đã import thành công vào danh mục [${this.bulkCatType}]!`);
          this.bulkCatalogText = '';
          await this.loadCatalogs();
          await this.loadAdminSettings();
          await this.fetchAdminCatalogItems();
        } else {
          this.toast(data.error || 'Lỗi import danh mục', 'error');
        }
      } catch (e) {
        this.toast('Lỗi kết nối khi import danh mục', 'error');
      }
    },

    // Quản lý và tra cứu danh mục trong Admin
    async fetchAdminCatalogItems() {
      if (!this.token || !this.isAdmin) return;
      try {
        const lim = this.adminCatalogLimit !== undefined ? this.adminCatalogLimit : 100;
        let url = `/api/catalogs?cat_type=${encodeURIComponent(this.adminCatalogCatType)}&limit=${lim}`;
        if (this.adminCatalogSearchQ.trim()) {
          url += `&q=${encodeURIComponent(this.adminCatalogSearchQ.trim())}`;
        }
        const res = await fetch(url, {
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        if (res.ok) {
          this.adminCatalogItems = await res.json();
          this.selectedCatalogIds = []; // Reset lựa chọn khi chuyển danh mục hoặc tìm kiếm
        }
      } catch (e) {
        console.error('Lỗi tải danh mục admin:', e);
      }
    },

    async deleteCatalogItem(id) {
      if (!confirm('Bạn có chắc muốn xóa mục danh mục này?')) return;
      try {
        const res = await fetch(`/api/catalogs/${id}`, {
          method: 'DELETE',
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        if (res.ok) {
          this.toast('Đã xóa mục danh mục!');
          await this.fetchAdminCatalogItems();
          await this.loadAdminSettings();
        } else {
          const data = await res.json();
          this.toast(data.error || 'Không thể xóa', 'error');
        }
      } catch (e) {
        this.toast('Lỗi kết nối khi xóa danh mục', 'error');
      }
    },

    // Chọn / Bỏ chọn tất cả danh mục đang hiển thị
    toggleSelectAllCatalogs(e) {
      if (e.target.checked) {
        this.selectedCatalogIds = this.adminCatalogItems.map(item => item.id);
      } else {
        this.selectedCatalogIds = [];
      }
    },

    // Xóa nhiều mục danh mục đã chọn
    async bulkDeleteCatalogItems() {
      if (this.selectedCatalogIds.length === 0) return;
      if (!confirm(`Bạn có chắc chắn muốn xóa ${this.selectedCatalogIds.length} mục danh mục đã chọn?`)) return;

      try {
        const res = await fetch('/api/catalogs/bulk-delete', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${this.token}`
          },
          body: JSON.stringify({ ids: this.selectedCatalogIds })
        });
        const data = await res.json();
        if (res.ok) {
          this.toast(data.message || `Đã xóa thành công ${data.deleted_count} mục danh mục!`);
          this.selectedCatalogIds = [];
          await this.fetchAdminCatalogItems();
          await this.loadAdminSettings();
        } else {
          this.toast(data.error || 'Lỗi khi xóa nhiều danh mục', 'error');
        }
      } catch (e) {
        this.toast('Lỗi kết nối khi xóa nhiều danh mục', 'error');
      }
    },

    // Tạm ngưng hoặc Kích hoạt nhiều mục danh mục đã chọn
    async bulkUpdateCatalogStatus(isActive) {
      if (this.selectedCatalogIds.length === 0) return;
      const actionName = isActive === 0 ? 'tạm ngưng' : 'kích hoạt';
      if (!confirm(`Bạn có chắc chắn muốn ${actionName} ${this.selectedCatalogIds.length} mục danh mục đã chọn không?`)) {
        return;
      }
      try {
        const res = await fetch('/api/catalogs/bulk-status', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${this.token}`
          },
          body: JSON.stringify({ ids: this.selectedCatalogIds, is_active: isActive })
        });
        const data = await res.json();
        if (res.ok) {
          this.toast(data.message || `Đã ${actionName} thành công!`);
          this.selectedCatalogIds = [];
          localStorage.removeItem(`ksk_cat_${this.adminCatalogCatType}`);
          await this.fetchAdminCatalogItems();
          await this.loadCatalogs();
        } else {
          this.toast(data.error || `Lỗi khi ${actionName} danh mục`, 'error');
        }
      } catch (e) {
        this.toast(`Lỗi kết nối khi ${actionName} danh mục`, 'error');
      }
    },

    // Đặt 1 mục làm mặc định cho loại danh mục
    async setDefaultCatalog(catId) {
      try {
        const res = await fetch(`/api/catalogs/${catId}/set-default`, {
          method: 'POST',
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        const data = await res.json();
        if (res.ok) {
          this.toast(data.message || 'Đã đặt làm mặc định thành công!');
          localStorage.removeItem(`ksk_cat_${this.adminCatalogCatType}`);
          await this.fetchAdminCatalogItems();
          await this.loadCatalogs();
        } else {
          this.toast(data.error || 'Lỗi đặt mặc định', 'error');
        }
      } catch (e) {
        this.toast('Lỗi kết nối khi đặt mặc định', 'error');
      }
    },

    // Bật/tắt trạng thái Hoạt động / Tạm ngưng cho 1 mục
    async toggleCatalogActive(catId) {
      try {
        const res = await fetch(`/api/catalogs/${catId}/toggle-active`, {
          method: 'POST',
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        const data = await res.json();
        if (res.ok) {
          this.toast(data.message || 'Đã thay đổi trạng thái thành công!');
          localStorage.removeItem(`ksk_cat_${this.adminCatalogCatType}`);
          await this.fetchAdminCatalogItems();
          await this.loadCatalogs();
        } else {
          this.toast(data.error || 'Lỗi đổi trạng thái', 'error');
        }
      } catch (e) {
        this.toast('Lỗi kết nối khi đổi trạng thái', 'error');
      }
    },

    // Chỉnh sửa (Edit) mục danh mục
    openEditCatalogModal(item) {
      this.catalogModalForm = {
        id: item.id,
        cat_type: item.cat_type,
        code: item.code,
        name: item.name,
        parent_code: item.parent_code || ''
      };
      this.showCatalogModal = true;
    },

    async saveCatalogModal() {
      if (!this.catalogModalForm.code.trim() || !this.catalogModalForm.name.trim()) {
        this.toast('Vui lòng nhập đầy đủ Mã và Tên mục!', 'error');
        return;
      }
      this.catalogModalSaving = true;
      try {
        const res = await fetch(`/api/catalogs/${this.catalogModalForm.id}`, {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${this.token}`
          },
          body: JSON.stringify({
            code: this.catalogModalForm.code.trim(),
            name: this.catalogModalForm.name.trim(),
            parent_code: this.catalogModalForm.parent_code.trim()
          })
        });
        const data = await res.json();
        if (res.ok) {
          this.toast(data.message || 'Cập nhật mục danh mục thành công!');
          this.showCatalogModal = false;
          await this.fetchAdminCatalogItems();
          await this.loadCatalogs();
        } else {
          this.toast(data.error || 'Lỗi khi cập nhật danh mục', 'error');
        }
      } catch (e) {
        this.toast('Lỗi kết nối khi cập nhật danh mục', 'error');
      } finally {
        this.catalogModalSaving = false;
      }
    },

    async reloadCatalogsFromAsset() {
      if (!confirm('Nạp lại toàn bộ danh mục chuẩn từ file seed_catalogs.tsv vào CSDL (sẽ làm mới và đồng bộ lại danh mục)?')) return;
      this.adminCatalogLoading = true;
      try {
        const res = await fetch('/api/admin/catalogs/reload-asset', {
          method: 'POST',
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        const data = await res.json();
        if (res.ok) {
          this.toast(data.message || 'Đã nạp toàn bộ danh mục từ seed_catalogs.tsv!');
          await this.fetchAdminCatalogItems();
          await this.loadCatalogs();
          await this.loadAdminSettings();
        } else {
          this.toast(data.error || 'Lỗi nạp danh mục từ seed_catalogs.tsv', 'error');
        }
      } catch (e) {
        this.toast('Lỗi kết nối khi nạp danh mục', 'error');
      } finally {
        this.adminCatalogLoading = false;
      }
    },

    // Format ngày cho in
    formatDatePart(dateStr, partIndex) {
      if (!dateStr) return '';
      const parts = dateStr.split('/');
      return parts[partIndex] || '';
    },

    // Helper tra cứu tên từ danh mục cho bản in (luôn lấy Tên tỉnh/xã/nghề nghiệp thay vì mã code)
    getCatalogName(list, code, fieldName = '') {
      if (!code) return '';
      const s = String(code).trim();
      const item = (list || []).find(c => String(c.code).trim() === s || String(c.name).trim() === s);
      if (item && item.name) return item.name;
      if (fieldName && this.tomInstances[fieldName] && this.tomInstances[fieldName].options[s]) {
        const txt = String(this.tomInstances[fieldName].options[s].text || '');
        return txt.includes(' - ') ? txt.split(' - ').slice(1).join(' - ').trim() : txt;
      }
      if (s.includes(' - ')) return s.split(' - ').slice(1).join(' - ').trim();
      return '';
    },

    // Render Barcode Code128 bằng JsBarcode
    renderPrintBarcodes() {
      const code = this.form.MA_TIEP_NHAN || '0';
      if (typeof JsBarcode !== 'undefined') {
        try {
          ['p1', 'p2', 'p3', 'p4'].forEach(p => {
            const el = document.querySelector('#print_barcode_' + p);
            if (el) {
              JsBarcode('#print_barcode_' + p, code, {
                format: 'CODE128',
                displayValue: false,
                margin: 0,
                width: 1.4,
                height: 40
              });
            }
          });
        } catch (e) {
          console.error('Lỗi tạo Barcode:', e);
        }
      }
    }
  };
}

