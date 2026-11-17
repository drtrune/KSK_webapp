#Requires AutoHotkey v2.0
#SingleInstance Force

; ============================================================================
; AutoHotkey v2 UIA -> HTTP API Bridge (HIS Bình Định -> KSK Web Server)
; - Chế độ 2: Đọc thông tin hành chính từ cửa sổ HIS qua UIA và đẩy qua API
; - Kết nối tới Máy chủ Web KSK (POST /api/login & POST /api/his/push)
; - Nhấn F9 sẽ hiện hộp thoại Xác nhận (Confirm) trước khi đẩy lên máy chủ
; ============================================================================

global g_IniPath := A_ScriptDir "\bridge_config.ini"
global g_Config := {
    ServerUrl: "http://127.0.0.1:8080",
    LastUsername: "letan",
    AutoWatch: false,
    PollMs: 1500
}

global g_Session := {
    LoggedIn: false,
    Token: "",
    Username: "",
    FullName: ""
}

global g_LastPushedMaTN := ""
global g_SrvGui := ""
global g_LoginGui := ""
global g_ConfirmGui := ""

LoadBridgeConfig()
InitTrayMenu()

if (g_Config.ServerUrl = "") {
    ShowServerSettingsGui(true)
} else {
    ShowLoginGui()
}

; Phím tắt F9: Đọc HIS và hiện hộp thoại xác nhận đẩy lên máy chủ
F9::PushCurrentHisPatient(true, true)

; ============================================================================
; 1. CẤU HÌNH & TRAY MENU
; ============================================================================
LoadBridgeConfig() {
    global g_IniPath, g_Config
    g_Config.ServerUrl := RTrim(IniRead(g_IniPath, "Server", "ServerUrl", "http://127.0.0.1:8080"), "/")
    g_Config.LastUsername := IniRead(g_IniPath, "Auth", "LastUsername", "letan")
    g_Config.AutoWatch := IniRead(g_IniPath, "Bridge", "AutoWatch", "0") = "1"
    g_Config.PollMs := Integer(IniRead(g_IniPath, "Bridge", "PollMs", "1500"))
}

SaveBridgeConfig() {
    global g_IniPath, g_Config
    IniWrite(RTrim(g_Config.ServerUrl, "/"), g_IniPath, "Server", "ServerUrl")
    IniWrite(g_Config.LastUsername, g_IniPath, "Auth", "LastUsername")
    IniWrite(g_Config.AutoWatch ? "1" : "0", g_IniPath, "Bridge", "AutoWatch")
    IniWrite(String(g_Config.PollMs), g_IniPath, "Bridge", "PollMs")
}

InitTrayMenu() {
    global g_Config, g_Session
    A_TrayMenu.Delete()

    if (g_Session.LoggedIn) {
        A_TrayMenu.Add("Nhân viên: " g_Session.FullName " (" g_Session.Username ")", (*) => {})
        A_TrayMenu.Disable("Nhân viên: " g_Session.FullName " (" g_Session.Username ")")
        A_TrayMenu.Add("Máy chủ: " g_Config.ServerUrl, (*) => {})
        A_TrayMenu.Disable("Máy chủ: " g_Config.ServerUrl)
        A_TrayMenu.Add()
        A_TrayMenu.Add("Đẩy bệnh nhân hiện tại (F9 - Có xác nhận)", (*) => PushCurrentHisPatient(true, true))
        A_TrayMenu.Add("Tự động đẩy khi đổi Mã TN trên HIS", ToggleAutoWatch)
        if (g_Config.AutoWatch)
            A_TrayMenu.Check("Tự động đẩy khi đổi Mã TN trên HIS")
        A_TrayMenu.Add()
        A_TrayMenu.Add("Cài đặt Máy chủ KSK...", (*) => ShowServerSettingsGui(false))
        A_TrayMenu.Add("Đăng xuất / Đổi tài khoản", (*) => LogoutAndShowLogin())
    } else {
        A_TrayMenu.Add("Chưa đăng nhập Máy chủ KSK", (*) => {})
        A_TrayMenu.Disable("Chưa đăng nhập Máy chủ KSK")
        A_TrayMenu.Add("Đăng nhập...", (*) => ShowLoginGui())
        A_TrayMenu.Add("Cài đặt Máy chủ KSK...", (*) => ShowServerSettingsGui(false))
    }
    A_TrayMenu.Add()
    A_TrayMenu.Add("Thoát", (*) => ExitApp())
}

ToggleAutoWatch(*) {
    global g_Config, g_Session
    g_Config.AutoWatch := !g_Config.AutoWatch
    SaveBridgeConfig()
    InitTrayMenu()
    if (g_Session.LoggedIn && g_Config.AutoWatch)
        SetTimer(WatchHisTimer, g_Config.PollMs)
    else
        SetTimer(WatchHisTimer, 0)
    TrayTip("KSK HIS Bridge", g_Config.AutoWatch ? "Đã BẬT tự động đẩy khi đổi Mã TN" : "Đã TẮT tự động đẩy", 1)
}

; ============================================================================
; 2. CỬA SỔ CÀI ĐẶT MÁY CHỦ KSK (HTTP API)
; ============================================================================
ShowServerSettingsGui(openLoginAfter := false) {
    global g_Config, g_SrvGui
    if IsObject(g_SrvGui) {
        try g_SrvGui.Destroy()
    }

    g := Gui("+AlwaysOnTop -MaximizeBox", "Cài đặt Máy chủ KSK (API Bridge)")
    g.SetFont("s9", "Segoe UI")
    g.MarginX := 16
    g.MarginY := 14

    g.Add("Text", "w430", "Địa chỉ Máy chủ KSK Web Server (VD: http://192.168.1.10:8080):")
    edtUrl := g.Add("Edit", "w430 vServerUrl", g_Config.ServerUrl)

    g.Add("Text", "w430 cGray", "Bridge sẽ gọi POST /api/login và POST /api/his/push trực tiếp tới máy chủ.")

    btnTest := g.Add("Button", "w140 y+12", "Kiểm tra kết nối")
    btnSave := g.Add("Button", "w140 x+10 Default", "Lưu cài đặt")
    btnCancel := g.Add("Button", "w90 x+10", "Đóng")

    btnTest.OnEvent("Click", (*) => TestServerPing(edtUrl.Value))
    btnSave.OnEvent("Click", (*) => OnSaveServerSettings(g, edtUrl.Value, openLoginAfter))
    btnCancel.OnEvent("Click", (*) => g.Destroy())

    g.Show("AutoSize Center")
    g_SrvGui := g
}

TestServerPing(url) {
    u := RTrim(Trim(url), "/")
    if (u = "") {
        MsgBox("Vui lòng nhập URL máy chủ!", "Thiếu thông tin", 48)
        return
    }
    try {
        res := HttpRequestJson("GET", u "/api/setup/status", "", "")
        if (res.Status >= 200 && res.Status < 300) {
            MsgBox("Kết nối tới Máy chủ KSK thành công!`nURL: " u, "Thành công", 64)
        } else {
            MsgBox("Máy chủ phản hồi mã lỗi HTTP " res.Status ":`n" res.Body, "Cảnh báo", 48)
        }
    } catch as err {
        MsgBox("Không kết nối được tới máy chủ:`n" err.Message, "Lỗi kết nối", 16)
    }
}

OnSaveServerSettings(g, url, openLoginAfter) {
    global g_Config
    u := RTrim(Trim(url), "/")
    if (u = "") {
        MsgBox("Vui lòng nhập địa chỉ Máy chủ KSK!", "Thiếu thông tin", 48)
        return
    }
    g_Config.ServerUrl := u
    SaveBridgeConfig()
    g.Destroy()
    if (openLoginAfter)
        ShowLoginGui()
    else
        InitTrayMenu()
}

; ============================================================================
; 3. CỬA SỔ ĐĂNG NHẬP MÁY CHỦ KSK
; ============================================================================
ShowLoginGui() {
    global g_Config, g_LoginGui
    if IsObject(g_LoginGui) {
        try g_LoginGui.Destroy()
    }

    g := Gui("+AlwaysOnTop -MaximizeBox", "Đăng nhập KSK HIS Bridge (API)")
    g.SetFont("s9", "Segoe UI")
    g.MarginX := 18
    g.MarginY := 14

    g.Add("Text", "w320 cGray", "Máy chủ: " g_Config.ServerUrl)
    g.Add("Text", "w320 y+8", "Tên đăng nhập:")
    edtUser := g.Add("Edit", "w320 vUsername", g_Config.LastUsername)

    g.Add("Text", "w320 y+8", "Mật khẩu:")
    edtPass := g.Add("Edit", "w320 Password vPassword")

    btnLogin := g.Add("Button", "w150 y+14 Default", "Đăng nhập")
    btnSrv := g.Add("Button", "w160 x+10", "Đổi Máy chủ...")

    btnLogin.OnEvent("Click", (*) => DoApiLogin(g, edtUser.Value, edtPass.Value))
    btnSrv.OnEvent("Click", (*) => (g.Destroy(), ShowServerSettingsGui(true)))

    g.Show("AutoSize Center")
    g_LoginGui := g
    edtPass.Focus()
}

DoApiLogin(g, username, password) {
    global g_Config, g_Session
    u := Trim(username)
    p := Trim(password)
    if (u = "" || p = "") {
        MsgBox("Vui lòng nhập tên đăng nhập và mật khẩu!", "Thiếu thông tin", 48)
        return
    }
    payload := '{"username":"' JsonEscape(u) '","password":"' JsonEscape(p) '"}'
    try {
        res := HttpRequestJson("POST", g_Config.ServerUrl "/api/login", payload, "")
        if (res.Status != 200) {
            errMsg := JsonGetStr(res.Body, "error")
            MsgBox(errMsg != "" ? errMsg : ("Đăng nhập thất bại (HTTP " res.Status ")"), "Lỗi đăng nhập", 16)
            return
        }
        tok := JsonGetStr(res.Body, "token")
        fnm := JsonGetStr(res.Body, "full_name")
        if (tok = "") {
            MsgBox("Máy chủ không trả về token hợp lệ!", "Lỗi", 16)
            return
        }
        g_Session.LoggedIn := true
        g_Session.Token := tok
        g_Session.Username := u
        g_Session.FullName := fnm != "" ? fnm : u
        g_Config.LastUsername := u
        SaveBridgeConfig()
        g.Destroy()
        InitTrayMenu()
        if (g_Config.AutoWatch)
            SetTimer(WatchHisTimer, g_Config.PollMs)
        TrayTip("KSK HIS Bridge", "Đăng nhập thành công: " g_Session.FullName "`nNhấn F9 trên cửa sổ HIS để đẩy bệnh nhân!", 1)
    } catch as err {
        MsgBox("Lỗi kết nối máy chủ: " err.Message, "Lỗi", 16)
    }
}

LogoutAndShowLogin() {
    global g_Session
    SetTimer(WatchHisTimer, 0)
    g_Session.LoggedIn := false
    g_Session.Token := ""
    g_Session.Username := ""
    g_Session.FullName := ""
    InitTrayMenu()
    ShowLoginGui()
}

; ============================================================================
; 4. ĐỌC HIS UIA, HỘP THOẠI XÁC NHẬN (F9) & ĐẨY QUA API /api/his/push
; ============================================================================
WatchHisTimer() {
    global g_Session, g_Config
    if (!g_Session.LoggedIn || !g_Config.AutoWatch)
        return
    PushCurrentHisPatient(false, false)
}

PushCurrentHisPatient(showErrors := false, requireConfirm := true) {
    global g_Session, g_LastPushedMaTN

    if (!g_Session.LoggedIn) {
        if showErrors
            ShowLoginGui()
        return
    }

    hwnd := WinActive("A")
    if (!hwnd)
        return

    root := UiaHelper.ElementFromHandle(hwnd)
    if (!root)
        return

    grp := UiaHelper.FindByAutomationId(root, "groupBoxThongTinHanhChinh")
    if (!grp) {
        if showErrors
            TrayTip("KSK HIS Bridge", "Không tìm thấy khung Thông tin hành chính trên cửa sổ HIS đang mở!", 2)
        return
    }

    maTN := ReadControlText(grp, "txtMaTiepNhan")
    hoTen := ReadControlText(grp, "txtTenBenhNhan")
    if (maTN = "" || hoTen = "") {
        if showErrors
            TrayTip("KSK HIS Bridge", "Chưa có Mã tiếp nhận hoặc Họ tên trên HIS.", 2)
        return
    }

    if (!showErrors && maTN = g_LastPushedMaTN)
        return

    maYTe := ReadControlText(grp, "txtMaYTe")
    ngaySinh := ReadControlText(grp, "txtNgaySinh")
    soCccd := ReadControlText(grp, "txtSoCMND")
    ngayCap := ReadControlText(grp, "txtNgayCap")
    noiCap := ReadControlText(grp, "txtNoiCap")
    if (noiCap = "")
        noiCap := "Cục Cảnh sát QLHCVTTXH"
    dienThoai := ReadControlText(grp, "txtSoDienThoai")
    soNha := ReadControlText(grp, "txtSoNha")
    diaChiLienHe := ReadControlText(grp, "lblDiaChiLienHe")
    ngayVao := SubStr(ReadControlText(grp, "calThoiGianTiepNhan"), 1, 10)
    if (ngayVao = "")
        ngayVao := FormatTime(, "dd/MM/yyyy")
    lyDo := ReadControlText(grp, "txtLyDoVaoKCB")
    if (lyDo = "")
        lyDo := "Khám sức khỏe"
    tienSuRaw := ReadControlText(grp, "txtTienSuBenh")

    tenDanToc := UiaHelper.GetName(UiaHelper.FindByAutomationId(grp, "cboDanToc"))
    tenNghe := UiaHelper.GetName(UiaHelper.FindByAutomationId(grp, "cboNgheNghiep"))
    tenXaTinh := UiaHelper.GetName(UiaHelper.FindByAutomationId(grp, "cboXaPhuong"))

    gioiTinh := "1"
    if UiaHelper.IsSelected(UiaHelper.FindByAutomationId(grp, "optNu"))
        gioiTinh := "2"

    diaChi := diaChiLienHe != "" ? diaChiLienHe : (soNha != "" && tenXaTinh != "" ? soNha ", " tenXaTinh : soNha)

    patient := {
        ma_tiep_nhan: maTN,
        ma_y_te: maYTe,
        ho_ten: hoTen,
        gioi_tinh: gioiTinh,
        ngay_sinh: ngaySinh,
        so_cccd: soCccd,
        ngaycap_cccd: ngayCap,
        noicap_cccd: noiCap,
        so_dien_thoai: dienThoai,
        dia_chi: diaChi,
        so_nha: soNha,
        ten_dan_toc: tenDanToc,
        ten_nghe_nghiep: tenNghe,
        ten_xa_phuong_tinh: tenXaTinh,
        ngay_vao: ngayVao,
        ly_do_vv: lyDo,
        tien_su_benh_raw: tienSuRaw
    }

    if (requireConfirm) {
        ShowConfirmPushGui(patient)
    } else {
        SendPatientToServerApi(patient, showErrors)
    }
}

ShowConfirmPushGui(patient) {
    global g_ConfirmGui
    if IsObject(g_ConfirmGui) {
        try g_ConfirmGui.Destroy()
    }

    g := Gui("+AlwaysOnTop -MaximizeBox", "Xác nhận đẩy hồ sơ lên Máy chủ KSK")
    g.SetFont("s9", "Segoe UI")
    g.MarginX := 16
    g.MarginY := 14

    gtLabel := patient.gioi_tinh = "2" ? "Nữ" : "Nam"
    g.Add("Text", "w420 c0x0F766E Bold", "Thông tin đọc được từ phần mềm HIS:")
    infoText := "Mã tiếp nhận : " patient.ma_tiep_nhan "`n"
        . "Mã bệnh nhân : " patient.ma_y_te "`n"
        . "Họ và tên    : " patient.ho_ten " (" gtLabel ")`n"
        . "Ngày sinh    : " patient.ngay_sinh "`n"
        . "Số CCCD      : " patient.so_cccd "`n"
        . "Địa chỉ      : " patient.dia_chi "`n"
        . "Ngày vào     : " patient.ngay_vao
    g.Add("Edit", "w420 r7 ReadOnly -WantReturn", infoText)

    btnConfirm := g.Add("Button", "w180 y+12 Default", "Xác nhận đẩy lên Máy chủ")
    btnCancel := g.Add("Button", "w100 x+10", "Hủy")

    btnConfirm.OnEvent("Click", (*) => (g.Destroy(), SendPatientToServerApi(patient, true)))
    btnCancel.OnEvent("Click", (*) => g.Destroy())

    g.Show("AutoSize Center")
    g_ConfirmGui := g
}

SendPatientToServerApi(patient, showErrors := true) {
    global g_Config, g_Session, g_LastPushedMaTN

    jsonBody := "{"
        . '"ma_tiep_nhan":"' JsonEscape(patient.ma_tiep_nhan) '",'
        . '"ma_y_te":"' JsonEscape(patient.ma_y_te) '",'
        . '"ho_ten":"' JsonEscape(patient.ho_ten) '",'
        . '"gioi_tinh":"' JsonEscape(patient.gioi_tinh) '",'
        . '"ngay_sinh":"' JsonEscape(patient.ngay_sinh) '",'
        . '"so_cccd":"' JsonEscape(patient.so_cccd) '",'
        . '"ngaycap_cccd":"' JsonEscape(patient.ngaycap_cccd) '",'
        . '"noicap_cccd":"' JsonEscape(patient.noicap_cccd) '",'
        . '"so_dien_thoai":"' JsonEscape(patient.so_dien_thoai) '",'
        . '"dia_chi":"' JsonEscape(patient.dia_chi) '",'
        . '"so_nha":"' JsonEscape(patient.so_nha) '",'
        . '"ten_dan_toc":"' JsonEscape(patient.ten_dan_toc) '",'
        . '"ten_nghe_nghiep":"' JsonEscape(patient.ten_nghe_nghiep) '",'
        . '"ten_xa_phuong_tinh":"' JsonEscape(patient.ten_xa_phuong_tinh) '",'
        . '"ngay_vao":"' JsonEscape(patient.ngay_vao) '",'
        . '"ly_do_vv":"' JsonEscape(patient.ly_do_vv) '",'
        . '"tien_su_benh_raw":"' JsonEscape(patient.tien_su_benh_raw) '"'
        . "}"

    try {
        res := HttpRequestJson("POST", g_Config.ServerUrl "/api/his/push", jsonBody, g_Session.Token)
        if (res.Status = 401) {
            if showErrors
                MsgBox("Phiên đăng nhập đã hết hạn, vui lòng đăng nhập lại!", "Hết phiên", 48)
            LogoutAndShowLogin()
            return
        }
        if (res.Status < 200 || res.Status >= 300) {
            errMsg := JsonGetStr(res.Body, "error")
            if showErrors
                TrayTip("KSK HIS Bridge", "Lỗi đẩy API (" res.Status "): " (errMsg != "" ? errMsg : res.Body), 3)
            return
        }
        g_LastPushedMaTN := patient.ma_tiep_nhan
        TrayTip("KSK HIS Bridge", "Đã đẩy lên máy chủ (" g_Session.Username "):`n" patient.ho_ten " (" patient.ma_tiep_nhan ")", 1)
    } catch as err {
        if showErrors
            TrayTip("KSK HIS Bridge", "Lỗi kết nối máy chủ: " err.Message, 3)
    }
}

; ============================================================================
; 5. HTTP & JSON HELPERS (WinHTTP)
; ============================================================================
HttpRequestJson(method, url, body := "", bearerToken := "") {
    req := ComObject("WinHttp.WinHttpRequest.5.1")
    req.Open(method, url, false)
    req.SetRequestHeader("Content-Type", "application/json; charset=utf-8")
    if (bearerToken != "")
        req.SetRequestHeader("Authorization", "Bearer " bearerToken)
    req.SetTimeouts(5000, 5000, 10000, 10000)
    if (body != "")
        req.Send(body)
    else
        req.Send()
    return { Status: req.Status, Body: req.ResponseText }
}

JsonGetStr(jsonText, key) {
    pattern := '"' key '"\s*:\s*"((?:\\.|[^"\\])*)"'
    if RegExMatch(jsonText, pattern, &m) {
        s := m[1]
        s := StrReplace(s, '\"', '"')
        s := StrReplace(s, "\\", "\")
        s := StrReplace(s, "\n", "`n")
        return s
    }
    return ""
}

JsonEscape(str) {
    str := StrReplace(str, "\", "\\")
    str := StrReplace(str, '"', '\"')
    str := StrReplace(str, "`r", "\r")
    str := StrReplace(str, "`n", "\n")
    str := StrReplace(str, "`t", "\t")
    return str
}

; ============================================================================
; 6. UIA COM HELPERS
; ============================================================================
class UiaHelper {
    static _uia := ""
    static Instance() {
        if !this._uia
            this._uia := ComObject("{FF48DBA4-60EF-4201-AA87-54103EEF594E}", "{30CBE57D-D9D0-452A-AB13-7AC5AC4825EE}")
        return this._uia
    }
    static ElementFromHandle(hwnd) {
        ComCall(6, this.Instance(), "Ptr", hwnd, "Ptr*", &pEl := 0)
        return pEl ? ComObjFromPtr(pEl) : ""
    }
    static FindByAutomationId(rootEl, autoId) {
        if !rootEl
            return ""
        uia := this.Instance()
        varProp := Buffer(24, 0)
        NumPut("UShort", 8, varProp, 0)
        bstr := DllCall("oleaut32\SysAllocString", "WStr", autoId, "Ptr")
        NumPut("Ptr", bstr, varProp, 8)
        ComCall(23, uia, "Int", 30011, "Ptr", varProp.Ptr, "Ptr*", &pCond := 0)
        DllCall("oleaut32\SysFreeString", "Ptr", bstr)
        if !pCond
            return ""
        ComCall(5, rootEl, "Int", 4, "Ptr", pCond, "Ptr*", &pFound := 0)
        ObjRelease(pCond)
        return pFound ? ComObjFromPtr(pFound) : ""
    }
    static GetName(el) {
        if !el
            return ""
        try {
            ComCall(23, el, "Ptr*", &pBstr := 0)
            if pBstr {
                s := StrGet(pBstr, "UTF-16")
                DllCall("oleaut32\SysFreeString", "Ptr", pBstr)
                return Trim(s)
            }
        }
        return ""
    }
    static GetValue(el) {
        if !el
            return ""
        try {
            ComCall(16, el, "Int", 10002, "Ptr*", &pPat := 0)
            if pPat {
                ComCall(4, pPat, "Ptr*", &pBstr := 0)
                ObjRelease(pPat)
                if pBstr {
                    s := StrGet(pBstr, "UTF-16")
                    DllCall("oleaut32\SysFreeString", "Ptr", pBstr)
                    return Trim(s)
                }
            }
        }
        return this.GetName(el)
    }
    static IsSelected(el) {
        if !el
            return false
        try {
            ComCall(16, el, "Int", 10010, "Ptr*", &pPat := 0)
            if pPat {
                ComCall(6, pPat, "Int*", &isSel := 0)
                ObjRelease(pPat)
                return isSel != 0
            }
        }
        return false
    }
}

ComObjFromPtr(ptr) {
    return ptr ? ComValue(13, ptr) : ""
}

ReadControlText(grp, autoId) {
    el := UiaHelper.FindByAutomationId(grp, autoId)
    if !el
        return ""
    val := UiaHelper.GetValue(el)
    return val != "" ? val : UiaHelper.GetName(el)
}
