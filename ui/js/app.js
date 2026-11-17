/**
 * KSK WebApp - Frontend Logic (Alpine.js + Flatpickr + Tom Select)
 * Phân quyền độc lập: reception (hành chính), tiensu (tiền sử), các chuyên khoa
 */

function kskApp() {
  return {
    // --- CÀI ĐẶT CÁ NHÂN & GIAO DIỆN ---
    userSettings: {
      theme: 'light', // 'light' | 'dark' | 'system'
      fontSize: 'normal', // 'normal' | 'large' | 'xlarge'
      defaultInputMode: 'minimal', // 'minimal' | 'full'
      autoNavigateTab: true,
      soundOnSave: false
    },
    userSettingsTab: 'preferences', // 'preferences' | 'profile' | 'password'
    showUserSettingsModal: false,
    profileForm: { full_name: '' },
    profileLoading: false,
    pwForm: { old_password: '', new_password: '', confirm_password: '' },
    pwLoading: false,

    // --- 2 CHẾ ĐỘ KHÁM LÂM SÀNG (MINIMAL | FULL) ---
    clinicalInputMode: 'minimal', // 'minimal' | 'full'
    clinicalSubTab: 'noi',     // 'noi' | 'ngoai' | 'dalieu' | 'san' | 'mat' | 'tmh' | 'rhm' | 'all'
    activeInternalSubTab: 'CK_NOI_TIM_MACH', // Sub-tab con của Nội khoa (Tim mạch)
    
    // Trạng thái đo thị lực & sức nghe
    visionState: {
      rightAcuity: '10',
      leftAcuity: '10',
      rightGlass: '',
      leftGlass: '',
      baseVisionScore: 1
    },
    hearingState: {
      voice: 1,
      audiometer: 1,
      leftNormal: '5',
      leftWhisper: '0.5',
      rightNormal: '5',
      rightWhisper: '0.5'
    },

    // Bản đồ lưu các phân loại đã chọn: diseaseId -> classificationId
    selectedClassifications: {},
    expandedDiseases: {}, // diseaseId -> boolean
    conclusionExtra: {}, // specialtyId -> { notes: '', manualRows: [] }
    expandAllFullTree: false,

    // Modal nhập bệnh khác ngoài danh mục
    showCustomDiseaseModal: false,
    customDiseaseForm: {
      name: '',
      specialtyId: 'noi',
      score: 2,
      note: ''
    },

    provinceMap: {},
    wardMap: {},

    token: localStorage.getItem('ksk_token') || '',
    currentUser: null,
    activeTab: 'reception',
    loginForm: { username: '', password: '' },
    loginLoading: false,
    loginError: '',

    // Danh sách hồ sơ KSK (Sidebar bên trái màn hình 16:9)
    recordsList: [],
    recordsLoading: false,
    recordsFilterStatus: '', // '' (tất cả), '0' (chờ khám / đang khám), '1' (đã khóa)
    recordsSearchQ: '',
    recordsDateFrom: '',
    recordsDateTo: '',
    sidebarOpen: true,

    // Mô-đun Xuất CSV chuẩn QĐ 3176 (123 chỉ tiêu)
    exportFilters: {
      fromDate: '',
      toDate: '',
      status: '',
      q: '',
      workplace: '',
      classification: ''
    },
    exportList: [],
    exportLoading: false,
    exportSelectedIds: [],
    exportDatePickers: {},

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

    // Trạng thái mở khóa sửa theo từng đầu mục khi hồ sơ đã lưu
    editingSections: {},
    isDirty: false,

    // Setup lần đầu & Khôi phục Admin
    setupStatus: { initialized: true, unlocked: true },
    setupPassword: '',
    setupRecoveryKey: '',
    showRecoverForm: false,
    recoverForm: { recovery_key: '', new_password: '' },

    // 6 Mẫu Phiếu KSK & In PDF A4
    allTemplatesMeta: [],
    recordTemplates: ['TREN_18'],
    activeRecordTemplate: 'TREN_18',
    showPrintTemplateModal: false,
    printModalIsReception: false,
    printModalTemplates: ['TREN_18'],

    // Khám đoàn & Hợp đồng
    groupCompanies: [],
    groupContracts: [],
    selectedContractForView: null,
    groupContractRecords: [],
    showCompanyModal: false,
    companyForm: { id: null, code: '', name: '', tax_code: '', address: '', contact_person: '', contact_phone: '', note: '' },
    showContractModal: false,
    contractForm: { id: null, company_id: 0, contract_code: '', title: '', exam_date: '', expected_count: 0, templates_arr: ['TREN_18'], note: '' },
    showGroupImportModal: false,
    importTargetContract: null,
    importPreviewRows: [],

    // Sao lưu Database trên Máy chủ
    serverBackups: [],
    serverBackupDir: 'backups',
    serverBackupNote: '',
    serverBackupLoading: false,

    // Danh mục Phân loại Bệnh Đa cấp (phan_loai.xlsx)
    phanLoaiTree: [],
    phanLoaiStats: {},
    phanLoaiSearch: '',
    phanLoaiExpanded: {},
    phanLoaiReloading: false,

    isSectionEditing(secKey) {
      if (!this.isSaved) return true; // Hồ sơ mới chưa lưu: luôn ở chế độ nhập/sửa
      return !!this.editingSections[secKey];
    },

    startEditSection(sectionKey) {
      if (!this.canUnlockSection(sectionKey)) {
        this.toast(`Bạn không có quyền chỉnh sửa mục [${this.getSectionLabel(sectionKey)}]`, 'error');
        return;
      }
      this.editingSections[sectionKey] = true;
      this.toast(`Đã mở quyền chỉnh sửa mục [${this.getSectionLabel(sectionKey)}]. Nhập xong bấm Lưu để hoàn tất.`, 'info');
    },

    cancelEditSection(sectionKey) {
      this.editingSections[sectionKey] = false;
      this.toast(`Đã hủy/đóng chế độ sửa mục [${this.getSectionLabel(sectionKey)}]`, 'info');
    },

    toggleEditSection(sectionKey) {
      if (this.isSectionEditing(sectionKey) && this.isSaved) {
        this.cancelEditSection(sectionKey);
      } else {
        this.startEditSection(sectionKey);
      }
    },

    getSectionLabel(key) {
      const map = {
        'reception': '1. Hành chính & Tiếp nhận',
        'history': '2. Tiền sử bệnh',
        'physical': '3. Thể lực & Sinh tồn',
        'clinical_noi': '4.1 Nội khoa',
        'clinical_ngoai': '4.2 Ngoại khoa',
        'clinical_dalieu': '4.3 Da liễu',
        'clinical_san': '4.4 Sản phụ khoa',
        'clinical_mat': '4.5 Mắt',
        'clinical_tmh': '4.6 Tai Mũi Họng',
        'clinical_rhm': '4.7 Răng Hàm Mặt',
        'paraclinical': '5. Cận lâm sàng',
        'conclusion': '6. Kết luận'
      };
      return map[key] || key;
    },

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

    // Instance Tom Select & Flatpickr để điều khiển
    tomInstances: {},
    fpInstances: {},
    icdSelectKeys: [],

    async initApp() {
      if (typeof window !== 'undefined' && window.innerWidth < 1024) {
        this.sidebarOpen = false;
      }
      this.loadUserPreferences();
      this.initDefaultClassifications();
      this.recalcCardioScores();

      // Mặc định bộ lọc ngày là ngày hôm nay
      const today = this.todayIso();
      this.recordsDateFrom = today;
      this.recordsDateTo = today;
      const todayVn = this.todayFormatted();
      this.exportFilters.fromDate = todayVn;
      this.exportFilters.toDate = todayVn;

      await this.checkSetupStatus();
      await this.loadTemplatesMetadata();

      if (this.token) {
        if (this.setupStatus.initialized && !this.setupStatus.unlocked) {
          this.logout();
          this.toast('Máy chủ vừa khởi động lại. Vui lòng đăng nhập lại để mở khóa dữ liệu mã hóa!', 'error');
        } else {
          await this.loadMe();
          if (this.token) {
            await this.loadCatalogs();
            await this.loadRecordsList();
          }
        }
      }

      this.$watch('activeTab', (val) => {
        this.$nextTick(() => {
          this.syncAllSelects();
          if (val === 'export') {
            this.loadExportPreview();
          }
        });
      });

      // Tự động theo dõi số liệu Thể lực để phân loại Tim mạch (Huyết áp & Tần số mạch)
      this.$watch('form.HUYET_AP', () => this.recalcCardioScores());
      this.$watch('form.MACH', () => this.recalcCardioScores());
      this.$watch('form.NGAY_SINH', () => this.recalcCardioScores());

      // Lắng nghe sự kiện phím tắt toàn cục
      window.addEventListener('keydown', (e) => {
        if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
          e.preventDefault();
          this.handleSave();
        } else if (e.altKey && e.key === '1') {
          e.preventDefault();
          this.clinicalInputMode = 'minimal';
          this.saveUserPreferences();
          this.toast('Đã chuyển sang chế độ: Tối thiểu (Phiếu giấy)', 'success');
        } else if (e.altKey && e.key === '2') {
          e.preventDefault();
          this.clinicalInputMode = 'full';
          this.saveUserPreferences();
          this.toast('Đã chuyển sang chế độ: Chi tiết (Danh mục)', 'success');
        } else if (e.altKey && (e.key === 'p' || e.key === 'P' || e.code === 'KeyP')) {
          e.preventDefault();
          this.printFullRecord();
        }
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

    // Phân quyền sửa từng Tab (khi hồ sơ đã lưu thì phải bấm nút 'Sửa' của mục đó mới cho phép sửa)
    get canEditReception() {
      if (this.isLocked) return false;
      if (!this.isAdmin && !this.isReceptionist) return false;
      return !this.isSaved || !this.isSectionExamined('reception') || !!this.editingSections['reception'];
    },
    get canEditHistory() {
      if (this.isLocked) return false;
      if (!this.isAdmin && !this.isTiensuRole) return false;
      return !this.isSaved || !this.isSectionExamined('history') || !!this.editingSections['history'];
    },
    get canEditPhysical() {
      if (this.isLocked) return false;
      if (!this.isAdmin && !this.isDoctorTheLuc) return false;
      return !this.isSaved || !this.isSectionExamined('physical') || !!this.editingSections['physical'];
    },
    get canEditClinicalNoi() {
      if (this.isLocked) return false;
      if (!this.isAdmin && !this.isDoctorNoi) return false;
      return !this.isSaved || !this.isSectionExamined('clin_noi') || !!this.editingSections['clinical_noi'] || !!this.editingSections['clin_noi'];
    },
    get canEditClinicalNgoai() {
      if (this.isLocked) return false;
      if (!this.isAdmin && !this.isDoctorNgoai) return false;
      return !this.isSaved || !this.isSectionExamined('clin_ngoai') || !!this.editingSections['clinical_ngoai'] || !!this.editingSections['clin_ngoai'];
    },
    get canEditClinicalDaLieu() {
      if (this.isLocked) return false;
      if (!this.isAdmin && !this.isDoctorDaLieu) return false;
      return !this.isSaved || !this.isSectionExamined('clin_dalieu') || !!this.editingSections['clinical_dalieu'] || !!this.editingSections['clin_dalieu'];
    },
    get canEditClinicalSan() {
      if (this.isLocked) return false;
      if (!this.isAdmin && !this.isDoctorSan) return false;
      return !this.isSaved || !this.isSectionExamined('clin_san') || !!this.editingSections['clinical_san'] || !!this.editingSections['clin_san'];
    },
    get canEditClinicalMat() {
      if (this.isLocked) return false;
      if (!this.isAdmin && !this.isDoctorMat) return false;
      return !this.isSaved || !this.isSectionExamined('clin_mat') || !!this.editingSections['clinical_mat'] || !!this.editingSections['clin_mat'];
    },
    get canEditClinicalTmh() {
      if (this.isLocked) return false;
      if (!this.isAdmin && !this.isDoctorTmh) return false;
      return !this.isSaved || !this.isSectionExamined('clin_tmh') || !!this.editingSections['clinical_tmh'] || !!this.editingSections['clin_tmh'];
    },
    get canEditClinicalRhm() {
      if (this.isLocked) return false;
      if (!this.isAdmin && !this.isDoctorRhm) return false;
      return !this.isSaved || !this.isSectionExamined('clin_rhm') || !!this.editingSections['clinical_rhm'] || !!this.editingSections['clin_rhm'];
    },
    get canEditParaclinical() {
      if (this.isLocked) return false;
      if (!this.isAdmin && !this.isDoctorCls) return false;
      return !this.isSaved || !this.isSectionExamined('paraclinical') || !!this.editingSections['paraclinical'];
    },
    get canEditConclusion() {
      if (this.isLocked) return false;
      if (!this.isAdmin && !this.isDoctorKetLuan) return false;
      return !this.isSaved || !this.isSectionExamined('conclusion') || !!this.editingSections['conclusion'];
    },

    // Lấy key chuyên khoa lâm sàng hiện tại đang mở trên giao diện
    getCurrentClinicalSectionKey() {
      switch (this.clinicalSubTab) {
        case 'noi': return 'clinical_noi';
        case 'ngoai': return 'clinical_ngoai';
        case 'dalieu': return 'clinical_dalieu';
        case 'san': return 'clinical_san';
        case 'mat': return 'clinical_mat';
        case 'tmh': return 'clinical_tmh';
        case 'rhm': return 'clinical_rhm';
        case 'all': return 'clinical_noi';
        default: return 'clinical_noi';
      }
    },

    // Kiểm tra quyền sửa chuyên khoa lâm sàng đang xem
    get canEditCurrentClinicalSubTab() {
      if (this.isLocked) return false;
      if (this.isAdmin) return !this.isSaved || this.hasAnyClinicalEditing();
      const k = this.getCurrentClinicalSectionKey();
      return this.canUnlockSection(k) && (!this.isSaved || !!this.editingSections[k]);
    },

    get canUnlockCurrentClinicalSubTab() {
      if (this.isLocked) return false;
      if (this.isAdmin) return true;
      if (this.clinicalSubTab === 'all') {
        return this.isDoctorNoi || this.isDoctorNgoai || this.isDoctorDaLieu || this.isDoctorSan || this.isDoctorMat || this.isDoctorTmh || this.isDoctorRhm;
      }
      return this.canUnlockSection(this.getCurrentClinicalSectionKey());
    },

    get isCurrentClinicalEditing() {
      if (!this.isSaved) return true;
      if (this.clinicalSubTab === 'all') {
        return this.hasAnyClinicalEditing();
      }
      return !!this.editingSections[this.getCurrentClinicalSectionKey()];
    },

    hasAnyClinicalEditing() {
      return !!(this.editingSections['clinical_noi'] ||
        this.editingSections['clinical_ngoai'] ||
        this.editingSections['clinical_dalieu'] ||
        this.editingSections['clinical_san'] ||
        this.editingSections['clinical_mat'] ||
        this.editingSections['clinical_tmh'] ||
        this.editingSections['clinical_rhm']);
    },

    toggleCurrentClinicalEdit() {
      if (this.clinicalSubTab === 'all') {
        const any = this.hasAnyClinicalEditing();
        const clinicalKeys = ['clinical_noi', 'clinical_ngoai', 'clinical_dalieu', 'clinical_san', 'clinical_mat', 'clinical_tmh', 'clinical_rhm'];
        clinicalKeys.forEach(k => {
          if (this.canUnlockSection(k)) {
            this.editingSections[k] = !any;
          }
        });
        if (!any) {
          this.toast('Đã mở quyền chỉnh sửa các chuyên khoa lâm sàng bạn phụ trách', 'info');
        } else {
          this.toast('Đã đóng chế độ sửa các chuyên khoa lâm sàng', 'info');
        }
        return;
      }
      this.toggleEditSection(this.getCurrentClinicalSectionKey());
    },

    // Kiểm tra xem user có quyền mở sửa mục này hay không (bỏ qua điều kiện editingSections)
    canUnlockSection(secKey) {
      if (this.isLocked) return false;
      if (this.isAdmin) return true;
      switch (secKey) {
        case 'reception': return this.isReceptionist;
        case 'history': return this.isTiensuRole;
        case 'physical': return this.isDoctorTheLuc;
        case 'clinical_noi': return this.isDoctorNoi;
        case 'clinical_ngoai': return this.isDoctorNgoai;
        case 'clinical_dalieu': return this.isDoctorDaLieu;
        case 'clinical_san': return this.isDoctorSan;
        case 'clinical_mat': return this.isDoctorMat;
        case 'clinical_tmh': return this.isDoctorTmh;
        case 'clinical_rhm': return this.isDoctorRhm;
        case 'paraclinical': return this.isDoctorCls;
        case 'conclusion': return this.isDoctorKetLuan;
        default: return false;
      }
    },

    get isLocked() { return this.recordStatus === 2; },
    get isSaved() { return !!this.recordId; },

    /**
     * Xác định Tab chính và Tab con chuyên khoa dựa trên phân quyền người dùng (User Role)
     * Quy tắc: Nếu người dùng có nhiều vai trò (Multi-Role), ưu tiên chọn vai trò đầu tiên trong danh sách
     * @returns {{ tab: string, subTab: string }} Thông tin tab và sub-tab tương ứng
     */
    getDestinationTabForUser() {
      if (!this.currentUser) return { tab: 'reception', subTab: 'noi' };

      // Tách danh sách quyền của tài khoản hiện tại
      const rawRoles = (this.currentUser.role || '').split(',').map(s => s.trim()).filter(Boolean);
      if (rawRoles.length === 0) return { tab: 'reception', subTab: 'noi' };

      // Bảng tra cứu vai trò -> Tab quy trình và Tab con chuyên khoa
      const roleTabMap = {
        'reception': { tab: 'reception', subTab: 'noi' },
        'tiensu': { tab: 'history', subTab: 'noi' },
        'bs_theluc': { tab: 'physical', subTab: 'noi' },
        'bs_noi': { tab: 'clinical', subTab: 'noi' },
        'bs_ngoai': { tab: 'clinical', subTab: 'ngoai' },
        'bs_dalieu': { tab: 'clinical', subTab: 'dalieu' },
        'bs_san': { tab: 'clinical', subTab: 'san' },
        'bs_mat': { tab: 'clinical', subTab: 'mat' },
        'bs_tmh': { tab: 'clinical', subTab: 'tmh' },
        'bs_rhm': { tab: 'clinical', subTab: 'rhm' },
        'bs_cls': { tab: 'paraclinical', subTab: 'noi' },
        'bs_ketluan': { tab: 'conclusion', subTab: 'noi' },
      };

      // Duyệt danh sách quyền được phân, lấy vai trò đầu tiên có trong bảng cấu hình
      for (const r of rawRoles) {
        if (roleTabMap[r]) {
          return roleTabMap[r];
        }
      }

      // Trường hợp tài khoản chỉ có quyền admin đơn thuần
      return { tab: 'reception', subTab: 'noi' };
    },

    /**
     * Tự động điều hướng đến đúng màn hình chuyên khoa người dùng được phân công
     */
    navigateToUserDefaultTab() {
      const dest = this.getDestinationTabForUser();
      this.activeTab = dest.tab;
      if (dest.tab === 'clinical') {
        this.clinicalSubTab = dest.subTab;
      }
    },

    /**
     * Tự động gán họ tên bác sĩ khám theo tài khoản đăng nhập khi thực hiện lưu dữ liệu
     * Đảm bảo các chỉ tiêu BS_* được điền đầy đủ cho báo cáo và in ấn
     */
    autoFillDoctorNames() {
      if (!this.currentUser || !this.currentUser.full_name) return;
      const docName = this.currentUser.full_name;

      if (this.canEditClinicalNoi) {
        this.form.BS_NOI = docName;
      }
      if (this.canEditClinicalNgoai) {
        this.form.BS_NGOAI = docName;
      }
      if (this.canEditClinicalDaLieu) {
        this.form.BS_DA_LIEU = docName;
      }
      if (this.canEditClinicalSan) {
        this.form.BS_SAN = docName;
      }
      if (this.canEditClinicalMat) {
        this.form.BS_MAT = docName;
      }
      if (this.canEditClinicalTmh) {
        this.form.BS_TMH = docName;
      }
      if (this.canEditClinicalRhm) {
        this.form.BS_RHM = docName;
      }
      if (this.canEditPhysical) {
        this.form.BS_THE_LUC = docName;
      }
      const hasClsData = ['CLS_HUYET_HOC', 'CLS_DUONG_MAU', 'CLS_URE', 'CLS_CREATININ', 'CLS_ASAT_GOT', 'CLS_ALAT_GPT', 'CLS_NUOC_TIEU', 'CLS_XQ_TIM_PHOI', 'CLS_DINH_KY_KET_QUA']
        .some(k => !!(this.form[k] && String(this.form[k]).trim()));
      if (this.activeTab === 'paraclinical' || (this.canEditParaclinical && hasClsData)) {
        if (this.isAdmin || this.isDoctorCls) {
          this.form.BS_CLS = docName;
        }
      }
      const hasConcData = !!(this.form.KET_LUAN_BENH && String(this.form.KET_LUAN_BENH).trim())
        || !!(this.form.PHAN_LOAI_SK && String(this.form.PHAN_LOAI_SK).trim())
        || !!(this.form.LX_KET_LUAN_SK && String(this.form.LX_KET_LUAN_SK).trim());
      if (this.activeTab === 'conclusion' || (this.canEditConclusion && hasConcData)) {
        if (this.isAdmin || this.isDoctorKetLuan) {
          this.form.BS_KET_LUAN = docName;
          this.form.LX_BS_KET_LUAN = docName;
          this.form.BAC_SI_KHAM = docName;
        }
      }
    },

    // Khởi tạo Flatpickr
    initDatePicker(el, fieldName) {
      if (typeof flatpickr === 'undefined' || !el) return;
      if (this.fpInstances[fieldName]) {
        try { this.fpInstances[fieldName].destroy(); } catch (e) { }
      }
      const fp = flatpickr(el, {
        dateFormat: 'd/m/Y',
        allowInput: true,
        defaultDate: this.form[fieldName] || null,
        locale: (window.flatpickr && flatpickr.l10ns && flatpickr.l10ns.vn) ? flatpickr.l10ns.vn : 'default',
        onChange: (selectedDates, dateStr) => {
          this.form[fieldName] = dateStr;
          this.isDirty = true;
        }
      });
      this.fpInstances[fieldName] = fp;
    },

    initContractDatePicker(el) {
      if (typeof flatpickr === 'undefined' || !el) return;
      if (this.fpInstances['_contract_exam_date']) {
        try { this.fpInstances['_contract_exam_date'].destroy(); } catch (e) { }
      }
      const fp = flatpickr(el, {
        dateFormat: 'd/m/Y',
        allowInput: true,
        defaultDate: this.contractForm?.exam_date || null,
        locale: (window.flatpickr && flatpickr.l10ns && flatpickr.l10ns.vn) ? flatpickr.l10ns.vn : 'default',
        onChange: (selectedDates, dateStr) => {
          this.contractForm.exam_date = dateStr;
        }
      });
      this.fpInstances['_contract_exam_date'] = fp;
    },

    // Khởi tạo Tom Select cho ICD-10 (Đa chọn / Tìm kiếm nhanh từ xa Remote Query)
    initIcdMultiSelect(el, fieldName) {
      if (typeof TomSelect === 'undefined' || !el) return;
      if (!this.icdSelectKeys.includes(fieldName)) {
        this.icdSelectKeys.push(fieldName);
      }
      if (this.tomInstances[fieldName]) {
        try { this.tomInstances[fieldName].destroy(); } catch (e) { }
      }

      const instance = new TomSelect(el, {
        plugins: ['remove_button', 'clear_button'],
        valueField: 'value',
        labelField: 'text',
        searchField: ['value', 'text'],
        create: true,
        maxItems: 20,
        closeAfterSelect: false,
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
        onItemAdd: function () {
          this.setTextboxValue('');
          this.refreshOptions(false);
        },
        onChange: (values) => {
          this.form[fieldName] = Array.isArray(values) ? values.join('; ') : (values || '');
          this.isDirty = true;
        }
      });

      // Nếu đã có giá trị sẵn trong form (ví dụ khi nạp hồ sơ cũ)
      if (this.form[fieldName]) {
        const valArr = String(this.form[fieldName]).split(/[,;]/).map(s => s.trim()).filter(Boolean);
        valArr.forEach(v => {
          instance.addOption({ value: v, text: v });
        });
        instance.setValue(valArr, true);
      }

      if (!this.isCurrentTabEditable()) {
        instance.disable();
      } else {
        instance.enable();
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
      const emptyLabel = fieldName === 'NHOM_MAU' ? '-- Để trống --' : '-- Tìm kiếm hoặc chọn --';
      let optionsHtml = `<option value="">${emptyLabel}</option>`;
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
          const tsOpts = {
            valueField: 'value',
            labelField: 'text',
            searchField: ['value', 'text'],
            create: isFreeText,
            allowEmptyOption: fieldName === 'NHOM_MAU',
            maxOptions: 1200,
            placeholder: emptyLabel,
            onChange: (val) => {
              this.form[fieldName] = val || '';
              if (fieldName === 'MATINH_CU_TRU') {
                this.onProvinceChange(val);
              }
            }
          };
          if (fieldName === 'NHOM_MAU') {
            tsOpts.plugins = ['clear_button'];
          }
          instance = new TomSelect(el, tsOpts);
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

    clearBloodGroup() {
      this.form.NHOM_MAU = '';
      if (this.tomInstances['NHOM_MAU']) {
        this.tomInstances['NHOM_MAU'].clear();
      }
      this.isDirty = true;
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

    // Đồng bộ giá trị tất cả TomSelect & Flatpickr trên giao diện
    syncTomSelectDisabledState() {
      const canEdit = this.isCurrentTabEditable();
      for (const k in this.tomInstances) {
        if (this.tomInstances[k]) {
          if (canEdit) this.tomInstances[k].enable();
          else this.tomInstances[k].disable();
        }
      }
    },

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

        // 2. Multi selects ICD-10 (cho cả TREN_18 và 5 mẫu riêng)
        const baseIcd = ['CAC_BENH_TAT_NEU_CO', 'TSGD_MA_BENH', 'TSBT_MA_BENH_KHAC', 'TSBT_MA_BENH', 'TSBT_MA_BENH_THAI_SAN'];
        const multiFields = Array.from(new Set([...baseIcd, ...(this.icdSelectKeys || [])]));
        for (const f of multiFields) {
          if (this.tomInstances[f]) {
            this.tomInstances[f].clear(true);
            if (this.form[f]) {
              const arr = String(this.form[f]).split(/[,;]/).map(s => s.trim()).filter(Boolean);
              arr.forEach(code => {
                this.tomInstances[f].addOption({ value: code, text: code });
              });
              this.tomInstances[f].setValue(arr, true);
            }
          }
        }

        // 3. Đồng bộ Flatpickr ngày tháng
        for (const [k, fp] of Object.entries(this.fpInstances || {})) {
          if (!fp || k.startsWith('_')) continue;
          const v = this.form[k] || '';
          if (v) fp.setDate(v, false, 'd/m/Y');
          else fp.clear(false);
        }

        // 4. Đồng bộ trạng thái khóa/mở sửa
        this.syncTomSelectDisabledState();
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
          if (this.setupStatus.initialized && data.mk_unlocked === false) {
            this.logout();
            this.toast('Phiên làm việc cũ chưa mở khóa Master Key. Vui lòng đăng nhập lại!', 'error');
            return;
          }
          this.currentUser = data.user;
          if (this.currentUser) {
            this.profileForm.full_name = this.currentUser.full_name || '';
          }
          this.facility = {
            ten_cskcb: data.facility?.ten_cskcb || data.cskcb?.ten_cskcb || 'BỆNH VIỆN BÌNH ĐỊNH',
            ma_cskcb: data.facility?.ma_cskcb || data.cskcb?.ma_cskcb || '52208',
            dia_danh: data.facility?.dia_danh || data.cskcb?.dia_danh || 'Gia Lai'
          };
          // Tự động điều hướng trực tiếp đến tab/chuyên khoa được phân quyền của user
          this.navigateToUserDefaultTab();
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
      this.loginError = '';
      if (!this.loginForm.username || !this.loginForm.password) {
        this.loginError = 'Vui lòng nhập tên đăng nhập và mật khẩu.';
        this.toast(this.loginError, 'error');
        return;
      }
      this.loginLoading = true;
      try {
        const res = await fetch('/api/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(this.loginForm)
        });
        const data = await res.json().catch(() => ({}));
        if (res.ok) {
          this.loginError = '';
          this.token = data.token;
          localStorage.setItem('ksk_token', this.token);
          this.currentUser = data.user;
          this.toast(`Chào mừng ${data.user.full_name}!`);
          // Tự động điều hướng đến đúng chuyên khoa/tab được phân quyền (ưu tiên tab đầu tiên)
          this.navigateToUserDefaultTab();
          await this.loadCatalogs();
          await this.loadRecordsList();
          if (this.isAdmin) {
            await this.loadAdminUsers();
            await this.loadAdminSettings();
            await this.fetchAdminCatalogItems();
          }
        } else {
          this.loginError = data.error || 'Tên đăng nhập hoặc mật khẩu không đúng.';
          this.toast(this.loginError, 'error');
        }
      } catch (err) {
        this.loginError = 'Không thể kết nối đến máy chủ.';
        this.toast(this.loginError, 'error');
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
        let url = `/api/records?limit=200`;
        if (this.recordsFilterStatus !== '') {
          url += `&status=${this.recordsFilterStatus}`;
        }
        if (this.recordsDateFrom) {
          url += `&from_date=${encodeURIComponent(this.recordsDateFrom)}`;
        }
        if (this.recordsDateTo) {
          url += `&to_date=${encodeURIComponent(this.recordsDateTo)}`;
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

    // Xử lý tìm kiếm trong bộ lọc & tự động nạp khi quét mã tiếp nhận (Barcode/QR Scan)
    async handleRecordsSearchInput(event) {
      const q = (this.recordsSearchQ || '').trim();
      if (!q) {
        await this.loadRecordsList();
        return;
      }

      // Kiểm tra nếu chuỗi tìm kiếm khớp mẫu Mã tiếp nhận đầy đủ (vd: TN.2611.0004018, TN.2611.0001, hoặc kết thúc bởi Enter từ máy quét)
      const isEnter = event && event.key === 'Enter';
      const isFullMaTn = /^TN[\.\-\_][0-9A-Za-z\.\-\_]{4,}$/i.test(q) || (q.length >= 12 && q.toUpperCase().startsWith('TN'));

      if (isEnter || isFullMaTn) {
        // Ưu tiên 1: Xem đã có sẵn trong danh sách hiển thị chưa
        const foundInList = this.recordsList.find(r => (r.ma_tiep_nhan || '').toUpperCase() === q.toUpperCase());
        if (foundInList) {
          this.selectRecord(foundInList);
          this.toast(`⚡ Máy quét đã nhận diện: [${foundInList.ma_tiep_nhan}] ${foundInList.ho_ten}`, 'success');
          return;
        }

        // Ưu tiên 2: Gọi handleSearchOrHis để tải từ DB hoặc HIS ngay lập tức
        try {
          const resDb = await fetch(`/api/records/by-ma-tn/${encodeURIComponent(q)}`, {
            headers: { 'Authorization': `Bearer ${this.token}` }
          });
          if (resDb.ok) {
            const record = await resDb.json();
            this.selectRecord(record);
            this.toast(`⚡ Máy quét đã tải hồ sơ: [${record.ma_tiep_nhan}] ${record.ho_ten}`, 'success');
            return;
          } else {
            // Thử tra cứu từ HIS nếu chưa có trong DB
            await this.handleSearchOrHis(q);
            return;
          }
        } catch (err) {
          console.warn('Lỗi kiểm tra mã tiếp nhận tự động:', err);
        }
      }

      // Nếu chỉ gõ lọc thông thường
      await this.loadRecordsList();
    },

    // Chọn 1 hồ sơ từ danh sách để nạp vào form nhập liệu
    selectRecord(rec) {
      this.loadRecordIntoForm(rec);
      this.searchMaTn = rec.ma_tiep_nhan;
      if (typeof window !== 'undefined' && window.innerWidth < 1024) {
        this.sidebarOpen = false;
      }
      // Chạy thẳng tới vị trí chuyên khoa được phân công để nhập liệu ngay
      this.navigateToUserDefaultTab();
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
          // Chuyển thẳng tới chuyên khoa của user
          this.navigateToUserDefaultTab();
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
            this.navigateToUserDefaultTab();

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
      this.editingSections = {}; // Reset tất cả các mục về trạng thái chỉ xem ban đầu, yêu cầu bấm nút Sửa
      this.isDirty = false;
      const tpls = (rec.exam_templates || 'TREN_18').split(',').map(s => s.trim()).filter(Boolean);
      this.recordTemplates = tpls.length ? tpls : ['TREN_18'];
      if (!this.recordTemplates.includes(this.activeRecordTemplate)) {
        this.activeRecordTemplate = this.recordTemplates[0];
      }

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
      if (payload.PHAN_LOAI_SK !== undefined && payload.PHAN_LOAI_SK !== null && String(payload.PHAN_LOAI_SK).trim() !== '') {
        this.form.PHAN_LOAI_SK = String(payload.PHAN_LOAI_SK).trim();
      }

      // Khôi phục cây phân loại bệnh đã lưu (nếu có) trước khi tính toán hiển thị
      if (payload.SELECTED_CLASSIFICATIONS_JSON) {
        try {
          const parsedSel = typeof payload.SELECTED_CLASSIFICATIONS_JSON === 'string'
            ? JSON.parse(payload.SELECTED_CLASSIFICATIONS_JSON)
            : payload.SELECTED_CLASSIFICATIONS_JSON;
          if (parsedSel && typeof parsedSel === 'object') {
            this.selectedClassifications = parsedSel;
          }
        } catch (_) {}
      } else {
        this.selectedClassifications = {};
        this.initDefaultClassifications();
      }

      // Nạp xã phường của tỉnh tương ứng nếu có
      if (this.form.MATINH_CU_TRU) {
        await this.loadWardsForProvince(this.form.MATINH_CU_TRU);
      }

      this.syncAllSelects();
      this._loadingRecord = true;
      try {
        this.recalcVisionScore();
        this.recalcHearingScore();
        this.recalcCardioScores();
      } finally {
        this._loadingRecord = false;
      }
    },

    resetForm() {
      this.recordId = null;
      this.recordStatus = 0;
      this.updatedBy = '';
      this.updatedAt = '';
      this.searchMaTn = '';
      this.editingSections = {}; // Reset trạng thái sửa
      this.isDirty = false;
      this.recordTemplates = ['TREN_18'];
      this.activeRecordTemplate = 'TREN_18';
      // Quay về tab chuyên khoa được phân công thay vì cố định về reception
      this.navigateToUserDefaultTab();

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
      this.recalcVisionScore();
      this.recalcHearingScore();
      this.recalcCardioScores();
    },

    todayFormatted() {
      const d = new Date();
      const dd = String(d.getDate()).padStart(2, '0');
      const mm = String(d.getMonth() + 1).padStart(2, '0');
      return `${dd}/${mm}/${d.getFullYear()}`;
    },

    todayIso() {
      const d = new Date();
      const yyyy = d.getFullYear();
      const mm = String(d.getMonth() + 1).padStart(2, '0');
      const dd = String(d.getDate()).padStart(2, '0');
      return `${yyyy}-${mm}-${dd}`;
    },

    getCurrentActiveSectionKey() {
      if (this.activeTab === 'reception') return 'reception';
      if (this.activeTab === 'history') return 'history';
      if (this.activeTab === 'physical') return 'physical';
      if (this.activeTab === 'clinical') {
        return this.clinicalSubTab === 'all' ? 'clinical_all' : this.getCurrentClinicalSectionKey();
      }
      if (this.activeTab === 'paraclinical') return 'paraclinical';
      if (this.activeTab === 'conclusion') return 'conclusion';
      return null;
    },

    // Lưu hồ sơ
    async handleSave(sectionKey) {
      if (!this.form.MA_TIEP_NHAN) {
        this.toast('Vui lòng nhập Mã tiếp nhận trước khi lưu!', 'error');
        return;
      }
      if (!this.form.HO_TEN) {
        this.toast('Vui lòng nhập Họ và tên bệnh nhân!', 'error');
        return;
      }

      // Xác định mục cần lưu
      const sec = sectionKey || this.getCurrentActiveSectionKey();

      // Tự động điền tên bác sĩ chuyên khoa theo tài khoản đang thao tác
      this.autoFillDoctorNames();

      this.saving = true;
      try {
        const tplsStr = (this.recordTemplates && this.recordTemplates.length ? this.recordTemplates : [this.activeRecordTemplate]).join(',');
        this.form._EXAM_TEMPLATES = tplsStr;
        if (this.clinicalInputMode === 'full' && this.selectedClassifications) {
          this.form.SELECTED_CLASSIFICATIONS_JSON = JSON.stringify(this.selectedClassifications);
        }
        const bodyData = {
          id: this.form.id,
          ma_tiep_nhan: this.form.MA_TIEP_NHAN,
          exam_templates: tplsStr,
          payload: this.form
        };
        if (sec) {
          bodyData.section = sec;
        }

        const res = await fetch('/api/records', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${this.token}`
          },
          body: JSON.stringify(bodyData)
        });

        const data = await res.json();
        if (res.ok) {
          const rec = data.record || data;
          this.recordId = rec.id;
          this.recordStatus = rec.status;
          this.form.id = rec.id;
          this.updatedBy = rec.updated_by;
          this.updatedAt = rec.updated_at;
          // Đóng cờ sửa sau khi lưu thành công, đưa giao diện về trạng thái xem và hiển thị nút "Sửa"
          this.editingSections = {};
          this.toast('Đã lưu hồ sơ thành công! Đã mở quyền In phiếu.');
          this.playBeep();
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
          this.recordStatus = 2;
          this.toast('Hồ sơ đã được KÝ VÀ KHÓA THÀNH CÔNG (status = 2)!');
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

    printPhieuKsk() {
      this.printFullRecord();
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

    // Chuyển sang Phân hệ Xuất CSV (123 cột QĐ 3176)
    openExportTab() {
      this.activeTab = 'export';
      this.loadExportPreview();
    },

    initExportDatePicker(el, field) {
      if (typeof flatpickr === 'undefined' || !el) return;
      const fp = flatpickr(el, {
        dateFormat: 'd/m/Y',
        allowInput: true,
        defaultDate: this.exportFilters[field] || null,
        locale: (window.flatpickr && flatpickr.l10ns && flatpickr.l10ns.vn) ? flatpickr.l10ns.vn : 'default',
        onChange: (selectedDates, dateStr) => {
          this.exportFilters[field] = dateStr;
          this.loadExportPreview();
        }
      });
      this.exportDatePickers[field] = fp;
    },

    syncExportDatePickers() {
      ['fromDate', 'toDate'].forEach(f => {
        if (this.exportDatePickers[f]) {
          this.exportDatePickers[f].setDate(this.exportFilters[f] || '', false, 'd/m/Y');
        }
      });
    },

    setExportToday() {
      const todayVn = this.todayFormatted();
      this.exportFilters.fromDate = todayVn;
      this.exportFilters.toDate = todayVn;
      this.syncExportDatePickers();
      this.loadExportPreview();
    },

    setExportThisMonth() {
      const d = new Date();
      const firstDay = `01/${String(d.getMonth() + 1).padStart(2, '0')}/${d.getFullYear()}`;
      this.exportFilters.fromDate = firstDay;
      this.exportFilters.toDate = this.todayFormatted();
      this.syncExportDatePickers();
      this.loadExportPreview();
    },

    resetExportFilters() {
      this.exportFilters = {
        fromDate: '',
        toDate: '',
        status: '',
        q: '',
        workplace: '',
        classification: ''
      };
      this.exportSelectedIds = [];
      this.syncExportDatePickers();
      this.loadExportPreview();
    },

    buildExportQueryParams(selectedOnly = false) {
      const params = new URLSearchParams();
      if (selectedOnly && this.exportSelectedIds.length > 0) {
        params.set('selected_ids', this.exportSelectedIds.join(','));
      } else {
        if (this.exportFilters.fromDate) params.set('from_date', this.exportFilters.fromDate.trim());
        if (this.exportFilters.toDate) params.set('to_date', this.exportFilters.toDate.trim());
        if (this.exportFilters.status !== '') params.set('status', this.exportFilters.status);
        if (this.exportFilters.q && this.exportFilters.q.trim()) params.set('q', this.exportFilters.q.trim());
        if (this.exportFilters.workplace && this.exportFilters.workplace.trim()) params.set('workplace', this.exportFilters.workplace.trim());
        if (this.exportFilters.classification) params.set('classification', this.exportFilters.classification);
      }
      return params;
    },

    async loadExportPreview() {
      if (!this.token) return;
      this.exportLoading = true;
      try {
        const params = this.buildExportQueryParams(false);
        params.set('limit', '1000');
        const res = await fetch(`/api/records?${params.toString()}`, {
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        if (res.ok) {
          const rows = await res.json();
          this.exportList = rows || [];
          const validIds = new Set(this.exportList.map(r => r.id));
          this.exportSelectedIds = this.exportSelectedIds.filter(id => validIds.has(id));
        } else {
          this.toast('Không thể tải danh sách xem trước xuất CSV', 'error');
        }
      } catch (e) {
        console.error('Lỗi tải danh sách xuất CSV:', e);
        this.toast('Lỗi kết nối khi tải danh sách xuất CSV', 'error');
      } finally {
        this.exportLoading = false;
      }
    },

    toggleExportSelection(id) {
      if (this.exportSelectedIds.includes(id)) {
        this.exportSelectedIds = this.exportSelectedIds.filter(x => x !== id);
      } else {
        this.exportSelectedIds.push(id);
      }
    },

    toggleSelectAllExport() {
      if (this.exportList.length > 0 && this.exportSelectedIds.length === this.exportList.length) {
        this.exportSelectedIds = [];
      } else {
        this.exportSelectedIds = this.exportList.map(r => r.id);
      }
    },

    triggerExportCsv(selectedOnly = false) {
      if (selectedOnly && this.exportSelectedIds.length === 0) {
        this.toast('Vui lòng tích chọn ít nhất 1 hồ sơ để xuất!', 'error');
        return;
      }
      if (!selectedOnly && this.exportList.length === 0) {
        this.toast('Không có dữ liệu phù hợp bộ lọc để xuất CSV!', 'error');
        return;
      }
      const params = this.buildExportQueryParams(selectedOnly);
      if (this.token) {
        params.set('token', this.token);
      }
      const url = `/api/export/csv?${params.toString()}`;
      window.open(url, '_blank');
      const count = selectedOnly ? this.exportSelectedIds.length : this.exportList.length;
      this.toast(`Đang tải xuống file CSV chuẩn 123 chỉ tiêu (${count} hồ sơ)...`);
    },

    // Xuất CSV (Chuyển sang tab Xuất CSV)
    downloadCsv() {
      this.openExportTab();
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
      const item = (list || []).find(c =>
        String(c.code || c.ma || '').trim() === s ||
        String(c.name || c.ten || '').trim() === s
      );
      if (item && (item.name || item.ten)) return item.name || item.ten;

      if (fieldName && this.tomInstances && this.tomInstances[fieldName] && this.tomInstances[fieldName].options[s]) {
        const txt = String(this.tomInstances[fieldName].options[s].text || '');
        return txt.includes(' - ') ? txt.split(' - ').slice(1).join(' - ').trim() : txt;
      }

      if (fieldName === 'MATINH_CU_TRU' && this.provinceMap && this.provinceMap[s]) {
        return this.provinceMap[s];
      }
      if (fieldName === 'MAXA_CU_TRU' && this.wardMap && this.wardMap[s]) {
        return this.wardMap[s];
      }

      if (s.includes(' - ')) return s.split(' - ').slice(1).join(' - ').trim();
      return s;
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
    },
  
    // ===================================================================
    // MODULE CÀI ĐẶT DÀNH CHO CÁ NHÂN & PROFILE
    // ===================================================================
    openUserSettingsModal() {
      this.showUserSettingsModal = true;
      this.userSettingsTab = 'preferences';
      if (this.clinicalInputMode) {
        this.userSettings.defaultInputMode = this.clinicalInputMode;
      }
      this.pwForm = { old_password: '', new_password: '', confirm_password: '' };
      if (this.currentUser) {
        this.profileForm.full_name = this.currentUser.full_name || '';
      }
    },

    closeUserSettingsModal() {
      this.showUserSettingsModal = false;
    },

    loadUserPreferences() {
      try {
        const saved = localStorage.getItem('ksk_user_preferences');
        if (saved) {
          const parsed = JSON.parse(saved);
          this.userSettings = Object.assign(this.userSettings, parsed);
          if (this.userSettings.defaultInputMode === 'focus') {
            this.userSettings.defaultInputMode = 'minimal';
          }
          if (this.userSettings.defaultInputMode) {
            this.clinicalInputMode = this.userSettings.defaultInputMode;
          }
        }
      } catch (e) {
        console.warn('Lỗi đọc preferences từ localStorage:', e);
      }
      this.applyUserSettings();
    },

    setClinicalInputMode(mode) {
      if (!mode) return;
      if (mode === 'focus') mode = 'minimal';
      this.clinicalInputMode = mode;
      this.userSettings.defaultInputMode = mode;
      try {
        localStorage.setItem('ksk_user_preferences', JSON.stringify(this.userSettings));
        this.applyUserSettings();
      } catch (e) {
        console.warn('Lỗi lưu preferences:', e);
      }
      const labels = { minimal: 'Tối thiểu (Phiếu giấy)', full: 'Chi tiết (Danh mục)' };
      this.toast(`Đã chuyển sang chế độ: ${labels[mode] || mode}`, 'success');
    },

    saveUserPreferences() {
      try {
        if (this.userSettings.defaultInputMode === 'focus') {
          this.userSettings.defaultInputMode = 'minimal';
        }
        if (this.userSettings.defaultInputMode) {
          this.clinicalInputMode = this.userSettings.defaultInputMode;
        }
        localStorage.setItem('ksk_user_preferences', JSON.stringify(this.userSettings));
        this.applyUserSettings();
      } catch (e) {
        console.warn('Lỗi lưu preferences:', e);
      }
    },

    applyUserSettings() {
      const body = document.body;
      if (!body) return;

      // Theme
      if (this.userSettings.theme === 'dark') {
        body.classList.add('dark');
      } else if (this.userSettings.theme === 'system' && window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
        body.classList.add('dark');
      } else {
        body.classList.remove('dark');
      }

      // Font size
      body.classList.remove('font-size-large', 'font-size-xlarge');
      if (this.userSettings.fontSize === 'large') {
        body.classList.add('font-size-large');
      } else if (this.userSettings.fontSize === 'xlarge') {
        body.classList.add('font-size-xlarge');
      }
    },

    getBodyClasses() {
      const classes = [];
      if (this.userSettings.theme === 'dark' || (this.userSettings.theme === 'system' && window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
        classes.push('dark');
      }
      if (this.userSettings.fontSize === 'large') classes.push('font-size-large');
      if (this.userSettings.fontSize === 'xlarge') classes.push('font-size-xlarge');
      return classes.join(' ');
    },

    playBeep() {
      if (!this.userSettings.soundOnSave) return;
      try {
        const AudioContext = window.AudioContext || window.webkitAudioContext;
        if (!AudioContext) return;
        const ctx = new AudioContext();
        const osc = ctx.createOscillator();
        const gain = ctx.createGain();
        osc.type = 'sine';
        osc.frequency.setValueAtTime(587.33, ctx.currentTime); // Âm thanh D5 êm tai
        gain.gain.setValueAtTime(0.08, ctx.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.15);
        osc.connect(gain);
        gain.connect(ctx.destination);
        osc.start();
        osc.stop(ctx.currentTime + 0.15);
      } catch (e) {}
    },

    async handleChangePassword() {
      const f = this.pwForm;
      if (!f.old_password || !f.new_password) {
        this.toast('Vui lòng nhập mật khẩu hiện tại và mật khẩu mới!', 'error');
        return;
      }
      if (f.new_password.length < 6) {
        this.toast('Mật khẩu mới phải có ít nhất 6 ký tự!', 'error');
        return;
      }
      if (f.new_password !== f.confirm_password) {
        this.toast('Xác nhận mật khẩu mới không khớp!', 'error');
        return;
      }

      this.pwLoading = true;
      try {
        const token = localStorage.getItem('ksk_token') || sessionStorage.getItem('ksk_token');
        const res = await fetch('/api/me/change-password', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': 'Bearer ' + token
          },
          body: JSON.stringify({
            old_password: f.old_password,
            new_password: f.new_password
          })
        });

        const data = await res.json();
        if (!res.ok) {
          throw new Error(data.message || data.error || 'Đổi mật khẩu thất bại!');
        }

        this.toast('Đổi mật khẩu thành công! Vui lòng ghi nhớ mật khẩu mới.', 'success');
        this.pwForm = { old_password: '', new_password: '', confirm_password: '' };
        this.closeUserSettingsModal();
      } catch (err) {
        this.toast(err.message, 'error');
      } finally {
        this.pwLoading = false;
      }
    },

    async handleUpdateProfile() {
      if (!this.isAdmin) {
        this.toast('Chỉ Quản trị viên (Admin) mới có quyền đổi họ tên / chức danh người dùng!', 'error');
        return;
      }
      if (!this.profileForm.full_name || !this.profileForm.full_name.trim()) {
        this.toast('Vui lòng nhập họ tên / chức danh hiển thị!', 'error');
        return;
      }
      this.profileLoading = true;
      try {
        const token = localStorage.getItem('ksk_token') || sessionStorage.getItem('ksk_token');
        const res = await fetch('/api/me/profile', {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': 'Bearer ' + token
          },
          body: JSON.stringify({ full_name: this.profileForm.full_name.trim() })
        });
        const data = await res.json();
        if (!res.ok) throw new Error(data.message || data.error || 'Cập nhật thất bại');
        if (this.currentUser) {
          this.currentUser.full_name = data.full_name;
        }
        this.toast('Cập nhật họ tên & chức danh Bác sĩ ký thành công!', 'success');
      } catch (err) {
        this.toast(err.message, 'error');
      } finally {
        this.profileLoading = false;
      }
    },

    // ===================================================================
    // DANH MỤC PHÂN LOẠI & RULE THAM CHIẾU (PHAN_LOAI_CONFIG)
    // ===================================================================
    getPlConfig() {
      return window.PHAN_LOAI_CONFIG || { specialties: [], diseases: [], classifications: [] };
    },

    getSpecialties() {
      return this.getPlConfig().specialties || [];
    },

    getSpecialty(id) {
      return this.getSpecialties().find(s => s.id === id);
    },

    getChildren(parentId) {
      return this.getSpecialties().filter(s => s.parentId === parentId);
    },

    getDiseases(specId) {
      return (this.getPlConfig().diseases || []).filter(d => d.specialtyId === specId);
    },

    getClassifications(diseaseId) {
      return (this.getPlConfig().classifications || []).filter(c => c.diseaseId === diseaseId);
    },

    calcClassificationScore(c, depth = 0) {
      if (!c || depth > 5) return 1;
      if (c.score !== null && c.score !== undefined && c.score !== '') {
        return parseInt(c.score, 10) || 1;
      }
      if (c.refDiseaseId) {
        let base = 1;
        if (c.refDiseaseId === 'BT_MAT_THI_LUC') {
          if (c.refRule && c.refRule.toLowerCase().includes('có kính')) {
            const cr = parseFloat(this.visionState?.correctedRight || this.form.CO_KINH_MAT_PHAI || '');
            const cl = parseFloat(this.visionState?.correctedLeft || this.form.CO_KINH_MAT_TRAI || '');
            if (!isNaN(cr) || !isNaN(cl)) {
              const sum = (isNaN(cr) ? 10 : cr) + (isNaN(cl) ? 10 : cl);
              base = sum >= 19 ? 1 : (sum >= 16 ? 2 : (sum >= 13 ? 3 : (sum >= 10 ? 4 : (sum >= 6 ? 5 : 6))));
            } else {
              base = this.visionState?.baseVisionScore || 1;
            }
          } else {
            base = this.visionState?.baseVisionScore || 1;
          }
        } else {
          const refClsId = this.selectedClassifications?.[c.refDiseaseId];
          const refCls = (this.getPlConfig().classifications || []).find(x => x.id === refClsId);
          base = refCls ? this.calcClassificationScore(refCls, depth + 1) : 1;
        }
        let delta = 0;
        const adj = c.adjust !== undefined && c.adjust !== null ? c.adjust : c.refAdjustment;
        if (typeof adj === 'number') {
          delta = adj;
        } else if (typeof adj === 'string') {
          const m = adj.match(/[+-]?\d+/);
          if (m) delta = parseInt(m[0], 10) || 0;
        }
        return Math.min(6, Math.max(1, base + delta));
      }
      return 1;
    },

    initDefaultClassifications() {
      const diseases = this.getPlConfig().diseases || [];
      const classifications = this.getPlConfig().classifications || [];
      for (const d of diseases) {
        if (!this.selectedClassifications[d.id]) {
          const cls = classifications.filter(c => c.diseaseId === d.id);
          const defaultCls = cls.find(c => parseInt(c.score || '1', 10) === 1) || cls[0];
          if (defaultCls) {
            this.selectedClassifications[d.id] = defaultCls.id;
          }
        }
      }
    },

    toggleDisease(diseaseId) {
      this.expandedDiseases[diseaseId] = !this.expandedDiseases[diseaseId];
    },

    expandAllDiseases() {
      this.expandAllFullTree = true;
      const diseases = this.getPlConfig().diseases || [];
      for (const d of diseases) {
        this.expandedDiseases[d.id] = true;
      }
    },

    collapseAllDiseases() {
      this.expandAllFullTree = false;
      this.expandedDiseases = {};
    },

    updateSelection(diseaseId, clsId) {
      this.selectedClassifications[diseaseId] = clsId;
      this.syncClinicalToForm();
    },

    recalcVisionScore() {
      const r = parseFloat(this.visionState.rightAcuity || this.form.KHONG_KINH_MAT_PHAI || '10');
      const l = parseFloat(this.visionState.leftAcuity || this.form.KHONG_KINH_MAT_TRAI || '10');
      const rg = parseFloat(this.visionState.rightGlass || this.form.CO_KINH_MAT_PHAI || '');
      const lg = parseFloat(this.visionState.leftGlass || this.form.CO_KINH_MAT_TRAI || '');

      const bestR = (!isNaN(rg) && rg > 0) ? rg : (isNaN(r) ? 10 : r);
      const bestL = (!isNaN(lg) && lg > 0) ? lg : (isNaN(l) ? 10 : l);
      const total = bestR + bestL;
      const minEye = Math.min(bestR, bestL);

      let sc = 1;
      let clsId = 'PL_MAT_1';
      if (total >= 19 && minEye >= 9) {
        sc = 1; clsId = 'PL_MAT_1';
      } else if (total >= 16 && minEye >= 6) {
        sc = 2; clsId = 'PL_MAT_2';
      } else if (total >= 13 && minEye >= 5) {
        sc = 3; clsId = 'PL_MAT_3';
      } else if (total >= 8 && minEye >= 3) {
        sc = 4; clsId = 'PL_MAT_4';
      } else {
        sc = 5; clsId = 'PL_MAT_5';
      }

      this.visionState.baseVisionScore = sc;
      this.selectedClassifications['BT_MAT_THI_LUC'] = clsId;
      this.form.KHAM_MAT_PL = String(sc);
      this.syncClinicalToForm();
    },

    recalcHearingScore() {
      const lt = parseFloat(this.hearingState.leftNormal || this.form.TAI_TRAI_NOI_THUONG || '5');
      const rt = parseFloat(this.hearingState.rightNormal || this.form.TAI_PHAI_NOI_THUONG || '5');
      const lw = parseFloat(this.hearingState.leftWhisper || this.form.TAI_TRAI_NOI_THAM || '0.5');
      const rw = parseFloat(this.hearingState.rightWhisper || this.form.TAI_PHAI_NOI_THAM || '0.5');

      const minNormal = Math.min(isNaN(lt) ? 5 : lt, isNaN(rt) ? 5 : rt);
      const minWhisper = Math.min(isNaN(lw) ? 0.5 : lw, isNaN(rw) ? 0.5 : rw);

      let sc = 1;
      let clsId = 'PL_TMH_1';
      if (minNormal >= 5 && minWhisper >= 0.5) {
        sc = 1; clsId = 'PL_TMH_1';
      } else if (minNormal >= 4) {
        sc = 2; clsId = 'PL_TMH_2';
      } else if (minNormal >= 2.5) {
        sc = 3; clsId = 'PL_TMH_3';
      } else if (minNormal >= 1) {
        sc = 4; clsId = 'PL_TMH_4';
      } else {
        sc = 5; clsId = 'PL_TMH_5';
      }

      this.hearingState.voice = sc;
      this.selectedClassifications['BT_TMH_TAI_TRONG'] = clsId;
      this.form.KHAM_TAI_MUI_HONG_PL = String(sc);
      this.syncClinicalToForm();
    },

    // --- TỰ ĐỘNG PHÂN LOẠI HUYẾT ÁP & TẦN SỐ MẠCH THEO THỂ LỰC & TUỔI ---
    getPatientAge() {
      if (!this.form.NGAY_SINH) return null;
      const str = String(this.form.NGAY_SINH).trim();
      const currentYear = new Date().getFullYear();

      // Định dạng DD/MM/YYYY
      if (str.includes('/')) {
        const parts = str.split('/');
        if (parts.length === 3) {
          const year = parseInt(parts[2], 10);
          if (!isNaN(year) && year > 1900 && year <= currentYear) {
            return currentYear - year;
          }
        }
      }
      // Định dạng YYYY-MM-DD
      if (str.includes('-')) {
        const parts = str.split('-');
        if (parts.length === 3) {
          const year = parseInt(parts[0], 10);
          if (!isNaN(year) && year > 1900 && year <= currentYear) {
            return currentYear - year;
          }
        }
      }
      // Nhập năm trực tiếp
      const num = parseInt(str, 10);
      if (!isNaN(num) && num > 1900 && num <= currentYear) {
        return currentYear - num;
      }
      return null;
    },

    parseBloodPressure() {
      if (!this.form.HUYET_AP) return null;
      const str = String(this.form.HUYET_AP).trim();
      const match = str.match(/(\d{2,3})\s*[\/\-]\s*(\d{2,3})/);
      if (match) {
        return {
          systolic: parseInt(match[1], 10),
          diastolic: parseInt(match[2], 10)
        };
      }
      return null;
    },

    getHuyetApScore() {
      const bp = this.parseBloodPressure();
      if (!bp) return 1;
      const age = this.getPatientAge();
      const isUnder30 = (age !== null && age < 30);
      const sys = bp.systolic;
      const dia = bp.diastolic;

      if (isUnder30) {
        if (sys < 100 || dia < 60) return 4;
        if (sys > 140 || dia > 95) return 4;
        if ((sys >= 136 && sys <= 140) || (dia >= 90 && dia <= 95)) return 3;
        if ((sys >= 126 && sys <= 135) || (dia >= 60 && dia < 65) || (dia >= 86 && dia < 90) || (sys >= 100 && sys < 110)) return 2;
        return 1;
      } else {
        // >= 30 tuổi và > 50 tuổi tính như 30-50
        if (sys < 100 || dia < 60) return 4;
        if (sys > 145 || dia > 95) return 4;
        if ((sys >= 140 && sys <= 145) || (dia >= 90 && dia <= 95)) return 3;
        return 1;
      }
    },

    getHuyetApClsId() {
      const bp = this.parseBloodPressure();
      const age = this.getPatientAge();
      const isUnder30 = (age !== null && age < 30);

      if (!bp) {
        return isUnder30 ? 'CT_HA_01' : 'CT_HA_06';
      }
      const sys = bp.systolic;
      const dia = bp.diastolic;

      if (isUnder30) {
        if (sys < 100 || dia < 60) return 'CT_HA_05';
        if (sys > 140 || dia > 95) return 'CT_HA_04';
        if ((sys >= 136 && sys <= 140) || (dia >= 90 && dia <= 95)) return 'CT_HA_03';
        if ((sys >= 126 && sys <= 135) || (dia >= 60 && dia < 65) || (dia >= 86 && dia < 90) || (sys >= 100 && sys < 110)) return 'CT_HA_02';
        return 'CT_HA_01';
      } else {
        // >= 30 tuổi và > 50 tuổi tính như 30-50
        if (sys < 100 || dia < 60) return 'CT_HA_08';
        if (sys > 145 || dia > 95) return 'CT_HA_08';
        if ((sys >= 140 && sys <= 145) || (dia >= 90 && dia <= 95)) return 'CT_HA_07';
        return 'CT_HA_06';
      }
    },

    getMachScore() {
      const pulse = parseInt(this.form.MACH, 10);
      if (isNaN(pulse) || pulse <= 0) return 1;
      if (pulse > 95 || pulse < 55) return 4;
      if ((pulse >= 86 && pulse <= 95) || (pulse >= 55 && pulse < 60)) return 3;
      if (pulse >= 76 && pulse <= 85) return 2;
      return 1;
    },

    getMachClsId() {
      const pulse = parseInt(this.form.MACH, 10);
      if (isNaN(pulse) || pulse <= 0) return 'CT_MC_01';
      if (pulse > 95 || pulse < 55) return 'CT_MC_04';
      if ((pulse >= 86 && pulse <= 95) || (pulse >= 55 && pulse < 60)) return 'CT_MC_03';
      if (pulse >= 76 && pulse <= 85) return 'CT_MC_02';
      return 'CT_MC_01';
    },

    recalcCardioScores() {
      const haClsId = this.getHuyetApClsId();
      const mcClsId = this.getMachClsId();

      this.selectedClassifications['BT_TM_HUYET_AP'] = haClsId;
      this.selectedClassifications['BT_TM_MACH'] = mcClsId;

      this.syncClinicalToForm();
    },

    getDiseasesWithScoreAbove(specId, minScore = 1) {
      const diseases = this.getDiseases(specId);
      const rows = [];
      for (const d of diseases) {
        const clsId = this.selectedClassifications[d.id];
        const classifications = this.getClassifications(d.id);
        const sel = classifications.find(c => c.id === clsId) || classifications[0];
        const score = sel ? this.calcClassificationScore(sel) : 1;
        if (score > minScore) {
          rows.push({
            disease: d,
            classification: sel,
            score: score,
            isManual: false
          });
        }
      }
      return rows;
    },

    getConclusionExtra(specId) {
      if (!this.conclusionExtra[specId]) {
        this.conclusionExtra[specId] = { notes: '', manualRows: [] };
      }
      return this.conclusionExtra[specId];
    },

    getMaxScoreForSpecialty(specId) {
      const autoRows = this.getDiseasesWithScoreAbove(specId, 1);
      const extra = this.getConclusionExtra(specId);
      let maxSc = 1;
      for (const r of autoRows) {
        if (r.score > maxSc) maxSc = r.score;
      }
      for (const m of (extra.manualRows || [])) {
        if (m.score > maxSc) maxSc = m.score;
      }
      return maxSc;
    },

    openCustomDiseaseModal() {
      this.showCustomDiseaseModal = true;
      this.customDiseaseForm = {
        name: '',
        specialtyId: this.clinicalSubTab === 'noi' ? (this.activeInternalSubTab || 'CK_NOI_TIM_MACH') : (this.clinicalSubTab === 'san' ? 'CK_PHU_SAN' : this.getCurrentRenderSpecId()),
        score: 2,
        note: ''
      };
    },

    submitCustomDisease() {
      const f = this.customDiseaseForm;
      if (!f.name || !f.name.trim()) {
        this.toast('Vui lòng nhập tên bệnh tật / phát hiện bất thường!', 'error');
        return;
      }
      const specId = f.specialtyId || this.getCurrentRenderSpecId();
      const extra = this.getConclusionExtra(specId);
      const row = {
        id: 'MANUAL_' + Date.now(),
        name: f.name.trim(),
        score: parseInt(f.score, 10),
        note: f.note || ''
      };
      extra.manualRows.push(row);

      this.showCustomDiseaseModal = false;
      this.syncClinicalToForm();
      this.toast('Đã thêm bệnh: ' + row.name, 'success');
    },

    removeManualDisease(specId, manualId) {
      const extra = this.getConclusionExtra(specId);
      extra.manualRows = extra.manualRows.filter(m => m.id !== manualId);
      this.syncClinicalToForm();
    },

    getClinicalSubTabTitle(subTab) {
      const map = {
        noi: '1. Nội khoa',
        ngoai: '2. Ngoại khoa',
        dalieu: '3. Da liễu',
        san: '4. Sản phụ khoa',
        mat: '5. Mắt',
        tmh: '6. Tai - Mũi - Họng',
        rhm: '7. Răng - Hàm - Mặt',
        all: 'Tất cả chuyên khoa'
      };
      return map[subTab] || 'Khám lâm sàng';
    },

    // ===================================================================
    // ĐỒNG BỘ 2 CHIỀU VÀO 121 CHỈ TIÊU (FORM.*)
    // ===================================================================
    syncClinicalToForm() {
      // Không ghi đè các trường *_PL đã lưu khi đang nạp hồ sơ hoặc đang nhập ở chế độ Tối giản
      if (this._loadingRecord || this.clinicalInputMode !== 'full') {
        return;
      }
      // 1. Phân khoa Nội khoa (khớp chính xác 8 phân khoa trong danh mục)
      const noiMap = [
        { id: 'CK_NOI_TIM_MACH', field: 'NOI_KHOA_TUAN_HOAN', plField: 'NOI_KHOA_TUAN_HOAN_PL' },
        { id: 'CK_NOI_HO_HAP', field: 'NOI_KHOA_HO_HAP', plField: 'NOI_KHOA_HO_HAP_PL' },
        { id: 'CK_NOI_TIEU_HOA', field: 'NOI_KHOA_TIEU_HOA', plField: 'NOI_KHOA_TIEU_HOA_PL' },
        { id: 'CK_NOI_TIET_NIEU_SD', field: 'NOI_KHOA_THAN_TN_SD', plField: 'NOI_KHOA_THAN_TIETNIEU_PL' },
        { id: 'CK_NOI_NOITIET', field: 'NOI_KHOA_NOI_TIET', plField: 'NOI_KHOA_NOI_TIET_PL' },
        { id: 'CK_NOI_CO_XUONG_KHOP', field: 'NOI_KHOA_CO_XUONG_KHOP', plField: 'NOI_KHOA_CO_XUONG_KHOP_PL' },
        { id: 'CK_NOI_THAN_KINH', field: 'NOI_KHOA_THAN_KINH', plField: 'NOI_KHOA_THAN_KINH_PL' },
        { id: 'CK_NOI_TAM_THAN', field: 'NOI_KHOA_TAM_THAN', plField: 'NOI_KHOA_TAM_THAN_PL' },
      ];

      for (const item of noiMap) {
        const abn = this.getDiseasesWithScoreAbove(item.id, 1);
        const extra = this.getConclusionExtra(item.id);
        const manual = extra.manualRows || [];
        const maxScore = this.getMaxScoreForSpecialty(item.id);
        this.form[item.plField] = String(maxScore);

        if (abn.length === 0 && manual.length === 0) {
          if (!this.form[item.field]) this.form[item.field] = 'Bình thường';
        } else {
          const names = [
            ...abn.map(a => a.classification ? a.classification.name : a.disease.name),
            ...manual.map(m => m.name)
          ];
          this.form[item.field] = names.join('; ');
        }
      }

      // 2. Ngoại khoa
      const ngoaiAbn = this.getDiseasesWithScoreAbove('CK_NGOAI', 1);
      const ngoaiExtra = this.getConclusionExtra('CK_NGOAI');
      const ngoaiMax = this.getMaxScoreForSpecialty('CK_NGOAI');
      this.form.KHAM_NGOAI_KHOA_PL = String(ngoaiMax);
      if (ngoaiAbn.length > 0 || (ngoaiExtra.manualRows && ngoaiExtra.manualRows.length > 0)) {
        const names = [...ngoaiAbn.map(a => a.classification ? a.classification.name : a.disease.name), ...(ngoaiExtra.manualRows || []).map(m => m.name)];
        this.form.KET_QUA_KHAM_NGOAI_KHOA = names.join('; ');
      } else if (!this.form.KET_QUA_KHAM_NGOAI_KHOA) {
        this.form.KET_QUA_KHAM_NGOAI_KHOA = 'Bình thường';
      }

      // 3. Da liễu
      const dlAbn = this.getDiseasesWithScoreAbove('CK_DA_LIEU', 1);
      const dlExtra = this.getConclusionExtra('CK_DA_LIEU');
      const dlMax = this.getMaxScoreForSpecialty('CK_DA_LIEU');
      this.form.KHAM_DA_LIEU_PL = String(dlMax);
      if (dlAbn.length > 0 || (dlExtra.manualRows && dlExtra.manualRows.length > 0)) {
        const names = [...dlAbn.map(a => a.classification ? a.classification.name : a.disease.name), ...(dlExtra.manualRows || []).map(m => m.name)];
        this.form.KET_QUA_KHAM_DA_LIEU = names.join('; ');
      } else if (!this.form.KET_QUA_KHAM_DA_LIEU) {
        this.form.KET_QUA_KHAM_DA_LIEU = 'Bình thường';
      }

      // 4. Sản phụ khoa (khớp đúng mã CK_PHU_SAN)
      const sanAbn = this.getDiseasesWithScoreAbove('CK_PHU_SAN', 1);
      const sanExtra = this.getConclusionExtra('CK_PHU_SAN');
      const sanMax = this.getMaxScoreForSpecialty('CK_PHU_SAN');
      this.form.KHAM_SAN_PHU_KHOA_PL = String(sanMax);
      if (sanAbn.length > 0 || (sanExtra.manualRows && sanExtra.manualRows.length > 0)) {
        const names = [...sanAbn.map(a => a.classification ? a.classification.name : a.disease.name), ...(sanExtra.manualRows || []).map(m => m.name)];
        this.form.KET_QUA_KHAM_SAN_PHU_KHOA = names.join('; ');
      } else if (!this.form.KET_QUA_KHAM_SAN_PHU_KHOA) {
        this.form.KET_QUA_KHAM_SAN_PHU_KHOA = 'Bình thường';
      }

      // 5. Mắt
      const matMax = this.getMaxScoreForSpecialty('CK_MAT');
      this.form.KHAM_MAT_PL = String(matMax);
      const matAbn = this.getDiseasesWithScoreAbove('CK_MAT', 1).filter(a => a.disease.id !== 'BT_MAT_THI_LUC');
      if (matAbn.length > 0) {
        this.form.BENH_KHAC_MAT = matAbn.map(a => a.classification ? a.classification.name : a.disease.name).join('; ');
      }

      // 6. Tai - Mũi - Họng
      const tmhMax = this.getMaxScoreForSpecialty('CK_TMH');
      this.form.KHAM_TAI_MUI_HONG_PL = String(tmhMax);
      const tmhAbn = this.getDiseasesWithScoreAbove('CK_TMH', 1).filter(a => a.disease.id !== 'BT_TMH_TAI_TRONG');
      if (tmhAbn.length > 0) {
        this.form.BENH_KHAC_TAI_MUI_HONG = tmhAbn.map(a => a.classification ? a.classification.name : a.disease.name).join('; ');
      }

      // 7. Răng - Hàm - Mặt
      const rhmMax = this.getMaxScoreForSpecialty('CK_RHM');
      this.form.KHAM_RANG_HAM_MAT_PL = String(rhmMax);

      // Điểm tổng thể (Overall Conclusion)
      let overallMax = 1;
      const allSpecs = [
        'CK_NOI_TIM_MACH', 'CK_NOI_HO_HAP', 'CK_NOI_TIEU_HOA', 'CK_NOI_TIET_NIEU_SD',
        'CK_NOI_NOITIET', 'CK_NOI_CO_XUONG_KHOP', 'CK_NOI_THAN_KINH', 'CK_NOI_TAM_THAN',
        'CK_NGOAI', 'CK_DA_LIEU', 'CK_PHU_SAN', 'CK_MAT', 'CK_TMH', 'CK_RHM'
      ];
      for (const sp of allSpecs) {
        const sc = this.getMaxScoreForSpecialty(sp);
        if (sc > overallMax) overallMax = sc;
      }

      if (overallMax === 1) {
        if (!this.form.TINH_TRANG_SUC_KHOE) {
          this.form.TINH_TRANG_SUC_KHOE = 'Hiện tại sức khỏe bình thường, chưa phát hiện bệnh lý';
        }
        if (!this.form.KET_LUAN_PHAN_LOAI) {
          this.form.KET_LUAN_PHAN_LOAI = '1';
        }
      } else {
        if (!this.form.KET_LUAN_PHAN_LOAI || this.form.KET_LUAN_PHAN_LOAI === '1') {
          this.form.KET_LUAN_PHAN_LOAI = String(overallMax);
        }
      }
    },

    getOverallMaxScore() {
      const allSpecs = [
        'CK_NOI_TIM_MACH', 'CK_NOI_HO_HAP', 'CK_NOI_TIEU_HOA', 'CK_NOI_TIET_NIEU_SD',
        'CK_NOI_NOITIET', 'CK_NOI_CO_XUONG_KHOP', 'CK_NOI_THAN_KINH', 'CK_NOI_TAM_THAN',
        'CK_NGOAI', 'CK_DA_LIEU', 'CK_MAT', 'CK_TMH', 'CK_RHM'
      ];
      if (this.form.GIOI_TINH === '2') {
        allSpecs.push('CK_PHU_SAN');
      }
      let max = parseInt(this.form.KHAM_THE_LUC_PL || '1', 10);
      if (isNaN(max)) max = 1;

      for (const sp of allSpecs) {
        const sc = this.getMaxScoreForSpecialty(sp);
        if (sc > max) max = sc;
      }
      return max;
    },

    getSpecialtiesSummaryList() {
      const list = [
        {
          name: '1. Thể lực',
          doctor: this.form.BS_THE_LUC || '',
          result: `Chiều cao: ${this.form.CHIEU_CAO || '-'} cm; Cân nặng: ${this.form.CAN_NANG || '-'} kg; BMI: ${this.form.CHI_SO_BMI || '-'}; Huyết áp: ${this.form.HUYET_AP || '-'} mmHg; Mạch: ${this.form.MACH || '-'} ck/p`,
          score: parseInt(this.form.KHAM_THE_LUC_PL || '1', 10),
          tab: 'physical'
        },
        {
          name: '2. Nội khoa - Tuần hoàn',
          doctor: this.form.BS_NOI || '',
          result: this.form.NOI_KHOA_TUAN_HOAN || 'Bình thường',
          score: parseInt(this.form.NOI_KHOA_TUAN_HOAN_PL || '1', 10),
          tab: 'clinical', subTab: 'noi'
        },
        {
          name: '2. Nội khoa - Hô hấp',
          doctor: this.form.BS_NOI || '',
          result: this.form.NOI_KHOA_HO_HAP || 'Bình thường',
          score: parseInt(this.form.NOI_KHOA_HO_HAP_PL || '1', 10),
          tab: 'clinical', subTab: 'noi'
        },
        {
          name: '2. Nội khoa - Tiêu hóa',
          doctor: this.form.BS_NOI || '',
          result: this.form.NOI_KHOA_TIEU_HOA || 'Bình thường',
          score: parseInt(this.form.NOI_KHOA_TIEU_HOA_PL || '1', 10),
          tab: 'clinical', subTab: 'noi'
        },
        {
          name: '2. Nội khoa - Thận / Tiết niệu / Sinh dục',
          doctor: this.form.BS_NOI || '',
          result: this.form.NOI_KHOA_THAN_TN_SD || 'Bình thường',
          score: parseInt(this.form.NOI_KHOA_THAN_TIETNIEU_PL || '1', 10),
          tab: 'clinical', subTab: 'noi'
        },
        {
          name: '2. Nội khoa - Nội tiết',
          doctor: this.form.BS_NOI || '',
          result: this.form.NOI_KHOA_NOI_TIET || 'Bình thường',
          score: parseInt(this.form.NOI_KHOA_NOI_TIET_PL || '1', 10),
          tab: 'clinical', subTab: 'noi'
        },
        {
          name: '2. Nội khoa - Cơ xương khớp',
          doctor: this.form.BS_NOI || '',
          result: this.form.NOI_KHOA_CO_XUONG_KHOP || 'Bình thường',
          score: parseInt(this.form.NOI_KHOA_CO_XUONG_KHOP_PL || '1', 10),
          tab: 'clinical', subTab: 'noi'
        },
        {
          name: '2. Nội khoa - Thần kinh',
          doctor: this.form.BS_NOI || '',
          result: this.form.NOI_KHOA_THAN_KINH || 'Bình thường',
          score: parseInt(this.form.NOI_KHOA_THAN_KINH_PL || '1', 10),
          tab: 'clinical', subTab: 'noi'
        },
        {
          name: '2. Nội khoa - Tâm thần',
          doctor: this.form.BS_NOI || '',
          result: this.form.NOI_KHOA_TAM_THAN || 'Bình thường',
          score: parseInt(this.form.NOI_KHOA_TAM_THAN_PL || '1', 10),
          tab: 'clinical', subTab: 'noi'
        },
        {
          name: '3. Ngoại khoa',
          doctor: this.form.BS_NGOAI || '',
          result: this.form.KET_QUA_KHAM_NGOAI_KHOA || 'Bình thường',
          score: parseInt(this.form.KHAM_NGOAI_KHOA_PL || '1', 10),
          tab: 'clinical', subTab: 'ngoai'
        },
        {
          name: '4. Da liễu',
          doctor: this.form.BS_DA_LIEU || '',
          result: this.form.KET_QUA_KHAM_DA_LIEU || 'Bình thường',
          score: parseInt(this.form.KHAM_DA_LIEU_PL || '1', 10),
          tab: 'clinical', subTab: 'dalieu'
        }
      ];

      if (this.form.GIOI_TINH === '2') {
        list.push({
          name: '5. Sản phụ khoa',
          doctor: this.form.BS_SAN || '',
          result: this.form.KET_QUA_KHAM_SAN_PHU_KHOA || 'Bình thường',
          score: parseInt(this.form.KHAM_SAN_PHU_KHOA_PL || '1', 10),
          tab: 'clinical', subTab: 'san'
        });
      }

      list.push(
        {
          name: '6. Mắt',
          doctor: this.form.BS_MAT || '',
          result: `MP: ${this.form.KHONG_KINH_MAT_PHAI || '10'}/10, MT: ${this.form.KHONG_KINH_MAT_TRAI || '10'}/10${this.form.BENH_KHAC_MAT ? '; ' + this.form.BENH_KHAC_MAT : ''}`,
          score: parseInt(this.form.KHAM_MAT_PL || '1', 10),
          tab: 'clinical', subTab: 'mat'
        },
        {
          name: '7. Tai - Mũi - Họng',
          doctor: this.form.BS_TMH || '',
          result: `Tai trái: ${this.form.TAI_TRAI_NOI_THUONG || '5'}m, Tai phải: ${this.form.TAI_PHAI_NOI_THUONG || '5'}m${this.form.BENH_KHAC_TAI_MUI_HONG ? '; ' + this.form.BENH_KHAC_TAI_MUI_HONG : ''}`,
          score: parseInt(this.form.KHAM_TAI_MUI_HONG_PL || '1', 10),
          tab: 'clinical', subTab: 'tmh'
        },
        {
          name: '8. Răng - Hàm - Mặt',
          doctor: this.form.BS_RHM || '',
          result: `Hàm trên: ${this.form.HAM_TREN || 'Bình thường'}; Hàm dưới: ${this.form.HAM_DUOI || 'Bình thường'}${this.form.BENH_KHAC_RANG_HAM_MAT ? '; ' + this.form.BENH_KHAC_RANG_HAM_MAT : ''}`,
          score: parseInt(this.form.KHAM_RANG_HAM_MAT_PL || '1', 10),
          tab: 'clinical', subTab: 'rhm'
        },
        {
          name: '9. Cận lâm sàng',
          doctor: this.form.BS_CLS || '',
          result: [
            this.form.X_QUANG_TIM_PHOI ? 'X-quang: ' + this.form.X_QUANG_TIM_PHOI : '',
            this.form.CONG_THUC_MAU ? 'Huyết học: ' + this.form.CONG_THUC_MAU : '',
            this.form.DUONG_MAU ? 'Đường máu: ' + this.form.DUONG_MAU : '',
            this.form.NUOC_TIEU ? 'Nước tiểu: ' + this.form.NUOC_TIEU : ''
          ].filter(Boolean).join('; ') || 'Các chỉ tiêu cận lâm sàng trong giới hạn bình thường',
          score: 1,
          tab: 'paraclinical'
        }
      );

      return list;
    },

    getCurrentRenderSpecId() {
      if (this.clinicalSubTab === 'noi') return this.activeInternalSubTab || 'CK_NOI_TIM_MACH';
      if (this.clinicalSubTab === 'ngoai') return 'CK_NGOAI';
      if (this.clinicalSubTab === 'dalieu') return 'CK_DA_LIEU';
      if (this.clinicalSubTab === 'san') return 'CK_PHU_SAN';
      if (this.clinicalSubTab === 'mat') return 'CK_MAT';
      if (this.clinicalSubTab === 'tmh') return 'CK_TMH';
      if (this.clinicalSubTab === 'rhm') return 'CK_RHM';
      return this.activeInternalSubTab || 'CK_NOI_TIM_MACH';
    },

    getCurrentRenderSpec() {
      return this.getSpecialty(this.getCurrentRenderSpecId());
    },

    getDiseasesForCurrentTab() {
      if (this.clinicalSubTab === 'all') {
        return this.getPlConfig().diseases || [];
      }
      const specId = this.getCurrentRenderSpecId();
      return this.getDiseases(specId);
    },

    getSelectedClassification(diseaseId) {
      const clsId = this.selectedClassifications[diseaseId];
      const list = this.getClassifications(diseaseId);
      return list.find(c => c.id === clsId) || list[0];
    },

    getDiseaseScore(diseaseId) {
      const sel = this.getSelectedClassification(diseaseId);
      return sel ? this.calcClassificationScore(sel) : 1;
    },

    getSelectedStandardName(diseaseId) {
      const sel = this.getSelectedClassification(diseaseId);
      return sel ? sel.name : '';
    },

    // --- SETUP & RECOVERY ---
    async checkSetupStatus() {
      try {
        const res = await fetch('/api/setup/status');
        if (res.ok) this.setupStatus = await res.json();
      } catch (e) {}
    },

    async initServerSetup() {
      if (!this.setupPassword || this.setupPassword.length < 4) {
        this.toast('Mật khẩu Admin phải có ít nhất 4 ký tự!', 'error');
        return;
      }
      const res = await fetch('/api/setup/init', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ admin_password: this.setupPassword })
      });
      const data = await res.json();
      if (!res.ok) { this.toast(data.error || 'Lỗi khởi tạo', 'error'); return; }
      this.setupRecoveryKey = data.recovery_key;
      this.toast('Đã khởi tạo hệ thống thành công!', 'success');
    },

    async recoverAdminWithKey() {
      const res = await fetch('/api/auth/recover', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(this.recoverForm)
      });
      const data = await res.json();
      if (!res.ok) { this.toast(data.error || 'Khôi phục thất bại', 'error'); return; }
      this.showRecoverForm = false;
      this.toast(data.message || 'Khôi phục mật khẩu thành công!', 'success');
    },

    // --- 6 MẪU PHIẾU KSK ---
    async loadTemplatesMetadata() {
      try {
        const res = await fetch('/api/templates');
        if (res.ok) this.allTemplatesMeta = await res.json();
      } catch (e) {}
    },

    templateShortLabel(code) {
      const found = (this.allTemplatesMeta || []).find(t => t.code === code);
      return found ? found.short_label : code;
    },

    templateFullLabel(code) {
      const found = (this.allTemplatesMeta || []).find(t => t.code === code);
      return found ? found.full_label : code;
    },

    activeTemplateGroups() {
      const found = (this.allTemplatesMeta || []).find(t => t.code === this.activeRecordTemplate);
      if (!found || !found.groups) return [];
      return found.groups[this.activeTab] || [];
    },

    async toggleRecordTemplate(code) {
      if (!this.isCurrentTabEditable() && this.isSaved) return;
      let list = [...(this.recordTemplates || ['TREN_18'])];
      if (list.includes(code)) {
        if (list.length <= 1) return;
        list = list.filter(c => c !== code);
      } else {
        list.push(code);
      }
      this.recordTemplates = list;
      if (!list.includes(this.activeRecordTemplate)) {
        this.activeRecordTemplate = list[0];
      } else if (list.includes(code)) {
        this.activeRecordTemplate = code;
      }
      this.form._EXAM_TEMPLATES = list.join(',');
      if (this.recordId) {
        await fetch(`/api/records/${this.recordId}/templates`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
          body: JSON.stringify({ exam_templates: list.join(',') })
        });
      }
      this.$nextTick(() => this.syncAllSelects());
    },

    // --- TRẠNG THÁI ĐÃ KHÁM / CHƯA KHÁM & STICKY FOOTER ---
    currentSectionKey() {
      if (this.activeTab === 'reception') return 'reception';
      if (this.activeRecordTemplate !== 'TREN_18') {
        return `tpl_${this.activeRecordTemplate}_${this.activeTab}`;
      }
      if (this.activeTab === 'clinical') {
        const sub = this.clinicalSubTab === 'all' ? 'noi' : this.clinicalSubTab;
        return `clin_${sub}`;
      }
      return this.activeTab;
    },

    isSectionExamined(secKey) {
      if (!this.recordId) return false;
      const raw = this.form._EXAMINED_SECTIONS;
      if (typeof raw === 'string' && raw.length > 0) {
        return raw.split(',').map(s => s.trim()).includes(secKey);
      }
      if (secKey === 'reception') return !!(this.form.HO_TEN && this.form.HO_TEN.trim());
      return false;
    },

    isCurrentTabEditable() {
      if (this.isLocked) return false;
      const sec = this.currentSectionKey();
      if (!this.isSectionExamined(sec)) return true;
      return !!this.editingSections[sec];
    },

    enterCurrentTabEdit() {
      const sec = this.currentSectionKey();
      this.editingSections[sec] = true;
      this.$nextTick(() => this.syncTomSelectDisabledState());
    },

    cancelCurrentTabEdit() {
      const sec = this.currentSectionKey();
      this.editingSections[sec] = false;
      this.isDirty = false;
      if (this.recordId) this.selectRecord({ id: this.recordId });
      this.$nextTick(() => this.syncTomSelectDisabledState());
    },

    switchExamTab(tab) {
      this.activeTab = tab;
      this.isDirty = false;
      this.$nextTick(() => this.syncAllSelects());
    },

    cycleExamTab(delta) {
      const order = this.activeRecordTemplate === 'TREN_18'
        ? ['reception', 'history', 'physical', 'clinical', 'paraclinical', 'conclusion']
        : ['reception', 'history', 'physical', 'clinical', 'conclusion'];
      const idx = order.indexOf(this.activeTab);
      const next = order[(idx + delta + order.length) % order.length];
      this.switchExamTab(next);
    },

    async saveCurrentSection(moveNext) {
      const sec = this.currentSectionKey();
      const set = (this.form._EXAMINED_SECTIONS || '').split(',').map(s => s.trim()).filter(Boolean);
      if (!set.includes(sec)) set.push(sec);
      this.form._EXAMINED_SECTIONS = set.join(',');
      if (typeof this.handleSave === 'function') {
        await this.handleSave();
      }
      this.editingSections[sec] = false;
      this.isDirty = false;
      this.$nextTick(() => this.syncTomSelectDisabledState());
      if (moveNext) this.cycleExamTab(1);
    },

    getTren18NormalDefaultsMap() {
      const hist = { TSGD_MAC_BENH: '0', TSBT_DANG_DIEU_TRI_BENH: '0' };
      for (const it of (this.personalHistoryItems || [])) {
        hist[it.field] = '0';
      }
      if (this.form.GIOI_TINH === '2') {
        hist.TSBT_THAI_SAN = '0';
      }
      return {
        history: hist,
        physical: {
          MACH: '75',
          HUYET_AP: '120/80',
          NHIP_THO: '18',
          NHIET_DO: '36.8',
          KHAM_THE_LUC_PL: '1',
          THE_LUC_PL: '1'
        },
        paraclinical: {
          CLS_HUYET_HOC: 'Bình thường',
          CLS_DUONG_MAU: '5.2 mmol/L (Bình thường)',
          CLS_URE: '5.1 mmol/L',
          CLS_CREATININ: '78 µmol/L',
          CLS_ASAT_GOT: '24 U/L',
          CLS_ALAT_GPT: '22 U/L',
          CLS_NUOC_TIEU: 'Bình thường',
          CLS_XQ_TIM_PHOI: 'Hình tim phổi bình thường',
          CLS_KHAC_CO: '0',
          CLS_DINH_KY_KET_QUA: 'Bình thường'
        },
        conclusion: {
          PHAN_LOAI_SK: '1',
          KET_LUAN_BENH: 'Hiện tại đủ sức khỏe học tập và công tác'
        },
        clin_noi: {
          NOI_KHOA_TUAN_HOAN: 'Bình thường', NOI_KHOA_TUAN_HOAN_PL: '1',
          NOI_KHOA_HO_HAP: 'Bình thường', NOI_KHOA_HO_HAP_PL: '1',
          NOI_KHOA_TIEU_HOA: 'Bình thường', NOI_KHOA_TIEU_HOA_PL: '1',
          NOI_KHOA_THAN_TIETNIEU: 'Bình thường', NOI_KHOA_THAN_TIETNIEU_PL: '1',
          NOI_KHOA_NOI_TIET: 'Bình thường', NOI_KHOA_NOI_TIET_PL: '1',
          NOI_KHOA_CO_XUONG_KHOP: 'Bình thường', NOI_KHOA_CO_XUONG_KHOP_PL: '1',
          NOI_KHOA_THAN_KINH: 'Bình thường', NOI_KHOA_THAN_KINH_PL: '1',
          NOI_KHOA_TAM_THAN: 'Bình thường', NOI_KHOA_TAM_THAN_PL: '1'
        },
        clin_ngoai: { KET_QUA_KHAM_NGOAI_KHOA: 'Bình thường', KHAM_NGOAI_KHOA_PL: '1' },
        clin_dalieu: { KET_QUA_KHAM_DA_LIEU: 'Bình thường', KHAM_DA_LIEU_PL: '1' },
        clin_san: { KET_QUA_KHAM_SAN_PHU_KHOA: 'Bình thường', KHAM_SAN_PHU_KHOA_PL: '1' },
        clin_mat: {
          KHONG_KINH_MAT_PHAI: '10/10', KHONG_KINH_MAT_TRAI: '10/10',
          THINHLUC_SAU_KINH_PHAI: '10/10', THINHLUC_SAU_KINH_TRAI: '10/10',
          BENH_KHAC_MAT: 'Bình thường', KHAM_MAT_PL: '1'
        },
        clin_tmh: {
          TAI_PHAI_NOI_THUONG: '5m', TAI_PHAI_NOI_THAM: '0.5m',
          TAI_TRAI_NOI_THUONG: '5m', TAI_TRAI_NOI_THAM: '0.5m',
          BENH_KHAC_TAI_MUI_HONG: 'Bình thường', KHAM_TAI_MUI_HONG_PL: '1'
        },
        clin_rhm: {
          HAM_TREN: 'Bình thường', HAM_DUOI: 'Bình thường',
          BENH_KHAC_RANG_HAM_MAT: 'Bình thường', KHAM_RANG_HAM_MAT_PL: '1'
        }
      };
    },

    quickFillNormalDefaults() {
      if (this.activeRecordTemplate !== 'TREN_18') {
        for (const g of this.activeTemplateGroups()) {
          for (const f of g.fields) {
            if (f.default_normal && !(this.form[f.namespaced_key] || '').trim()) {
              this.form[f.namespaced_key] = f.default_normal;
            }
          }
        }
        this.isDirty = true;
        this.$nextTick(() => this.syncAllSelects());
        this.toast('Đã điền mặc định bình thường cho mục này!', 'success');
        return;
      }
      const map = this.getTren18NormalDefaultsMap();
      if (this.activeTab === 'clinical' && this.clinicalSubTab === 'all') {
        const clinKeys = ['clin_noi', 'clin_ngoai', 'clin_dalieu', 'clin_mat', 'clin_tmh', 'clin_rhm'];
        if (this.form.GIOI_TINH === '2') clinKeys.push('clin_san');
        for (const sk of clinKeys) {
          for (const [k, v] of Object.entries(map[sk] || {})) {
            if (!(this.form[k] || '').trim()) this.form[k] = v;
          }
        }
      } else {
        const defs = map[this.currentSectionKey()] || {};
        for (const [k, v] of Object.entries(defs)) {
          if (!(this.form[k] || '').trim()) this.form[k] = v;
        }
        if (this.currentSectionKey() === 'clin_noi') {
          this.initDefaultClassifications();
        }
      }
      this.autoFillDoctorNames();
      this.isDirty = true;
      this.$nextTick(() => this.syncAllSelects());
      this.toast('Đã điền mặc định bình thường cho mục này!', 'success');
    },

    quickFillAllNormalDefaults() {
      if (this.isLocked) {
        this.toast('Hồ sơ đã khóa, không thể sửa!', 'error');
        return;
      }
      this.enterCurrentTabEdit();
      if (this.activeRecordTemplate !== 'TREN_18') {
        const meta = (this.allTemplatesMeta || []).find(t => t.code === this.activeRecordTemplate);
        const tabs = ['history', 'physical', 'clinical', 'conclusion'];
        const examined = (this.form._EXAMINED_SECTIONS || '').split(',').map(s => s.trim()).filter(Boolean);
        const groupsMap = meta && (meta.groups || meta.groups_by_tab);
        if (groupsMap) {
          for (const tab of tabs) {
            for (const g of (groupsMap[tab] || [])) {
              for (const f of (g.fields || [])) {
                if (f.default_normal && !(this.form[f.namespaced_key] || '').trim()) {
                  this.form[f.namespaced_key] = f.default_normal;
                }
              }
            }
            const secKey = `tpl_${this.activeRecordTemplate}_${tab}`;
            if (!examined.includes(secKey)) examined.push(secKey);
          }
        }
        this.form._EXAMINED_SECTIONS = examined.join(',');
      } else {
        const map = this.getTren18NormalDefaultsMap();
        const examined = (this.form._EXAMINED_SECTIONS || '').split(',').map(s => s.trim()).filter(Boolean);
        for (const [secKey, defs] of Object.entries(map)) {
          if (secKey === 'clin_san' && this.form.GIOI_TINH !== '2') continue;
          for (const [k, v] of Object.entries(defs)) {
            if (!(this.form[k] || '').trim()) this.form[k] = v;
          }
          if (!examined.includes(secKey)) examined.push(secKey);
        }
        this.initDefaultClassifications();
        this.form._EXAMINED_SECTIONS = examined.join(',');
      }
      this.autoFillDoctorNames();
      this.isDirty = true;
      this.$nextTick(() => this.syncAllSelects());
      this.toast('Đã điền mặc định bình thường cho toàn bộ mẫu phiếu!', 'success');
    },

    canPrintFullKsk() {
      if (!this.recordId) return false;
      if (this.recordStatus >= 1) return true;
      const set = (this.form._EXAMINED_SECTIONS || '').split(',').map(s => s.trim()).filter(Boolean);
      if (this.activeRecordTemplate === 'TREN_18') {
        const req = ['reception', 'history', 'physical', 'clin_noi', 'clin_ngoai', 'clin_dalieu', 'clin_mat', 'clin_tmh', 'clin_rhm', 'paraclinical', 'conclusion'];
        if (this.form.GIOI_TINH === '2') req.push('clin_san');
        return req.every(s => set.includes(s));
      }
      const reqCustom = ['reception', 'history', 'physical', 'clinical', 'conclusion'];
      return reqCustom.every(s => set.includes(s));
    },

    async loadPhanLoaiTree() {
      try {
        const res = await fetch('/api/phan-loai/tree', {
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        if (res.ok) {
          const data = await res.json();
          this.phanLoaiTree = data.tree || [];
          this.phanLoaiStats = data.stats || {};
        }
      } catch (_) {}
    },

    async reloadPhanLoaiXlsx() {
      this.phanLoaiReloading = true;
      try {
        const res = await fetch('/api/admin/phan-loai/reload-xlsx', {
          method: 'POST',
          headers: { 'Authorization': `Bearer ${this.token}` }
        });
        const data = await res.json();
        if (res.ok) {
          this.toast(data.message || 'Đã đồng bộ từ phan_loai.xlsx');
          await this.loadPhanLoaiTree();
        } else {
          this.toast(data.error || 'Lỗi đồng bộ phan_loai.xlsx', 'error');
        }
      } catch (e) {
        this.toast('Lỗi kết nối khi đồng bộ phan_loai.xlsx', 'error');
      } finally {
        this.phanLoaiReloading = false;
      }
    },

    togglePhanLoaiNode(id) {
      this.phanLoaiExpanded[id] = !this.phanLoaiExpanded[id];
    },

    filteredPhanLoaiTree() {
      const q = (this.phanLoaiSearch || '').trim().toLowerCase();
      if (!q) return this.phanLoaiTree || [];
      return (this.phanLoaiTree || []).filter(root => {
        if ((root.name || '').toLowerCase().includes(q) || (root.note || '').toLowerCase().includes(q)) return true;
        if ((root.diseases || []).some(d => (d.name || '').toLowerCase().includes(q) || (d.classifications || []).some(c => (c.name || '').toLowerCase().includes(q) || (c.refRule || '').toLowerCase().includes(q)))) return true;
        return (root.children || []).some(sub =>
          (sub.name || '').toLowerCase().includes(q) ||
          (sub.note || '').toLowerCase().includes(q) ||
          (sub.diseases || []).some(d => (d.name || '').toLowerCase().includes(q) || (d.classifications || []).some(c => (c.name || '').toLowerCase().includes(q) || (c.refRule || '').toLowerCase().includes(q)))
        );
      });
    },

    printRecordPdf(isReception) {
      if (!this.recordId) return;
      const tpls = this.recordTemplates && this.recordTemplates.length ? this.recordTemplates : [this.activeRecordTemplate];
      if (tpls.length <= 1) {
        const url = `/api/records/${this.recordId}/pdf?reception=${isReception ? 1 : 0}&templates=${encodeURIComponent(tpls[0])}&token=${encodeURIComponent(this.token)}`;
        window.open(url, '_blank');
        return;
      }
      this.printModalIsReception = isReception;
      this.printModalTemplates = [...tpls];
      this.showPrintTemplateModal = true;
    },

    confirmPrintModalPdf() {
      if (!this.printModalTemplates.length) {
        this.toast('Vui lòng chọn ít nhất 1 mẫu!', 'error');
        return;
      }
      const url = `/api/records/${this.recordId}/pdf?reception=${this.printModalIsReception ? 1 : 0}&templates=${encodeURIComponent(this.printModalTemplates.join(','))}&token=${encodeURIComponent(this.token)}`;
      window.open(url, '_blank');
      this.showPrintTemplateModal = false;
    },

    // --- KHÁM ĐOÀN ---
    async openGroupTab() {
      this.activeTab = 'group';
      await this.loadGroupData();
    },

    async loadGroupData() {
      const h = { 'Authorization': `Bearer ${this.token}` };
      const [cRes, ctRes] = await Promise.all([
        fetch('/api/companies', { headers: h }),
        fetch('/api/contracts', { headers: h })
      ]);
      if (cRes.ok) this.groupCompanies = await cRes.json();
      if (ctRes.ok) this.groupContracts = await ctRes.json();
    },

    openCompanyModal(comp = null) {
      this.companyForm = comp
        ? { ...comp }
        : { id: null, code: '', name: '', tax_code: '', address: '', contact_person: '', contact_phone: '', note: '' };
      this.showCompanyModal = true;
    },

    async saveCompany() {
      const isEdit = !!this.companyForm.id;
      const res = await fetch(isEdit ? `/api/companies/${this.companyForm.id}` : '/api/companies', {
        method: isEdit ? 'PUT' : 'POST',
        headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
        body: JSON.stringify(this.companyForm)
      });
      const data = await res.json();
      if (!res.ok) { this.toast(data.error || 'Lỗi lưu công ty', 'error'); return; }
      this.showCompanyModal = false;
      await this.loadGroupData();
      this.toast('Đã lưu công ty!', 'success');
    },

    openContractModal(ct = null, companyId = null) {
      if (ct) {
        this.contractForm = {
          ...ct,
          templates_arr: (ct.default_templates || 'TREN_18').split(',').map(s => s.trim()).filter(Boolean)
        };
      } else {
        const defComp = companyId || (this.groupCompanies[0]?.id || 0);
        this.contractForm = { id: null, company_id: defComp, contract_code: '', title: '', exam_date: '', expected_count: 0, templates_arr: ['TREN_18'], note: '' };
      }
      this.showContractModal = true;
    },

    async saveContract() {
      const isEdit = !!this.contractForm.id;
      const body = {
        ...this.contractForm,
        default_templates: (this.contractForm.templates_arr || ['TREN_18']).join(',')
      };
      const res = await fetch(isEdit ? `/api/contracts/${this.contractForm.id}` : '/api/contracts', {
        method: isEdit ? 'PUT' : 'POST',
        headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
        body: JSON.stringify(body)
      });
      const data = await res.json();
      if (!res.ok) { this.toast(data.error || 'Lỗi lưu hợp đồng', 'error'); return; }
      this.showContractModal = false;
      await this.loadGroupData();
      this.toast('Đã lưu hợp đồng!', 'success');
    },

    async openContractRecords(ct) {
      this.selectedContractForView = ct;
      const res = await fetch(`/api/contracts/${ct.id}/records`, {
        headers: { 'Authorization': `Bearer ${this.token}` }
      });
      if (res.ok) this.groupContractRecords = await res.json();
    },

    printContractPdf(contractId, isReception) {
      window.open(`/api/contracts/${contractId}/pdf?reception=${isReception ? 1 : 0}&token=${encodeURIComponent(this.token)}`, '_blank');
    },

    openGroupImportModal(ct) {
      this.importTargetContract = ct;
      this.importPreviewRows = [];
      this.showGroupImportModal = true;
    },

    async previewGroupImportFile(ev) {
      const file = ev.target.files[0];
      if (!file || !this.importTargetContract) return;
      const fd = new FormData();
      fd.append('file', file);
      const res = await fetch(`/api/contracts/${this.importTargetContract.id}/import-preview`, {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${this.token}` },
        body: fd
      });
      const data = await res.json();
      if (!res.ok) { this.toast(data.error || 'Lỗi đọc file', 'error'); return; }
      this.importPreviewRows = data;
    },

    async commitGroupImport() {
      if (!this.importTargetContract || !this.importPreviewRows.length) return;
      const res = await fetch(`/api/contracts/${this.importTargetContract.id}/import-commit`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
        body: JSON.stringify({ rows: this.importPreviewRows })
      });
      const data = await res.json();
      if (!res.ok) { this.toast(data.error || 'Lỗi import', 'error'); return; }
      this.showGroupImportModal = false;
      await this.loadGroupData();
      await this.loadRecordsList();
      this.toast(`Đã import ${data.inserted} hồ sơ (bỏ qua ${data.skipped})!`, 'success');
    },

    // --- SAO LƯU & KHÔI PHỤC DB TRÊN MÁY CHỦ ---
    async loadServerBackups() {
      const res = await fetch('/api/admin/db/backups', {
        headers: { 'Authorization': `Bearer ${this.token}` }
      });
      if (res.ok) {
        const data = await res.json();
        this.serverBackupDir = data.backup_dir || 'backups';
        this.serverBackups = data.backups || [];
      }
    },

    async createServerBackup() {
      this.serverBackupLoading = true;
      try {
        const res = await fetch('/api/admin/db/backup', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
          body: JSON.stringify({ note: this.serverBackupNote || '' })
        });
        const data = await res.json();
        if (!res.ok) { this.toast(data.error || 'Lỗi sao lưu', 'error'); return; }
        this.serverBackupNote = '';
        await this.loadServerBackups();
        this.toast(data.message || 'Đã sao lưu trên máy chủ!', 'success');
      } finally {
        this.serverBackupLoading = false;
      }
    },

    async restoreServerBackup(filename) {
      if (!confirm(`Khôi phục CSDL máy chủ từ bản sao lưu '${filename}'?\nHệ thống sẽ tự động tạo 1 bản sao lưu an toàn trước khi khôi phục.`)) return;
      const res = await fetch('/api/admin/db/restore', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${this.token}` },
        body: JSON.stringify({ filename })
      });
      const data = await res.json();
      if (!res.ok) { this.toast(data.error || 'Lỗi khôi phục', 'error'); return; }
      await this.loadServerBackups();
      await this.loadRecordsList();
      this.toast(data.message || 'Đã khôi phục CSDL!', 'success');
    },

    downloadServerBackup(filename) {
      window.open(`/api/admin/db/backups/${encodeURIComponent(filename)}?token=${encodeURIComponent(this.token)}`, '_blank');
    },

    async deleteServerBackup(filename) {
      if (!confirm(`Xóa bản sao lưu '${filename}' trên máy chủ?`)) return;
      const res = await fetch(`/api/admin/db/backups/${encodeURIComponent(filename)}`, {
        method: 'DELETE',
        headers: { 'Authorization': `Bearer ${this.token}` }
      });
      if (res.ok) {
        await this.loadServerBackups();
        this.toast('Đã xóa bản sao lưu!', 'success');
      }
    },

  };
}

