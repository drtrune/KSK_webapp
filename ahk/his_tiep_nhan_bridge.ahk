#Requires AutoHotkey v2.0
#SingleInstance Force

; ============================================================================
; AutoHotkey v2 UIA Bridge - HIS Bình Định -> KSK Native Desktop App
; Đọc trực tiếp màn hình [Tiếp Nhận Bệnh Nhân] qua Windows UI Automation COM
; Phím tắt: F9 = Đẩy bệnh nhân đang mở sang App KSK | Tray Menu = Bật/Tắt Tự động
; ============================================================================

global g_IpcPort := 18080
global g_ApiKey := "HIS_SECRET_API_KEY_2026"
global g_AutoWatch := false
global g_LastPushedMaTN := ""

LoadConfigFromToml() {
    global g_IpcPort, g_ApiKey
    candidates := [
        A_ScriptDir "\config.toml",
        A_ScriptDir "\..\config.toml"
    ]
    for cfgPath in candidates {
        if FileExist(cfgPath) {
            content := FileRead(cfgPath, "UTF-8")
            if RegExMatch(content, 'm)^\s*local_ipc_port\s*=\s*(\d+)', &mPort) {
                g_IpcPort := Integer(mPort[1])
            }
            if RegExMatch(content, 'm)^\s*his_api_key\s*=\s*"([^"]*)"', &mKey) {
                g_ApiKey := mKey[1]
            }
            break
        }
    }
}

LoadConfigFromToml()

; Cấu hình Tray Menu
A_TrayMenu.Delete()
A_TrayMenu.Add("Đẩy bệnh nhân hiện tại (F9)", (*) => PushCurrentHisPatient(true))
A_TrayMenu.Add("Tự động đẩy khi đổi Mã TN", ToggleAutoWatch)
A_TrayMenu.Add()
A_TrayMenu.Add("Thoát", (*) => ExitApp())
A_IconTip := "HIS -> KSK Bridge (F9 để đẩy hồ sơ)"

ToggleAutoWatch(*) {
    global g_AutoWatch
    g_AutoWatch := !g_AutoWatch
    if g_AutoWatch {
        A_TrayMenu.Check("Tự động đẩy khi đổi Mã TN")
        SetTimer(WatchHisTimer, 1500)
        TrayTip("KSK HIS Bridge", "Đã BẬT chế độ tự động theo dõi Mã tiếp nhận", 1)
    } else {
        A_TrayMenu.Uncheck("Tự động đẩy khi đổi Mã TN")
        SetTimer(WatchHisTimer, 0)
        TrayTip("KSK HIS Bridge", "Đã TẮT chế độ tự động", 1)
    }
}

WatchHisTimer() {
    global g_AutoWatch
    if g_AutoWatch {
        PushCurrentHisPatient(false)
    }
}

F9::PushCurrentHisPatient(true)

; ============================================================================
; UIA COM Helpers (Không cần thư viện ngoài)
; ============================================================================
class UiaHelper {
    static _uia := ""

    static Instance() {
        if !this._uia {
            this._uia := ComObject("{FF48DBA4-60EF-4201-AA87-54103EEF594E}", "{30CBE57D-D9D0-452A-AB13-7AC5AC4825EE}")
        }
        return this._uia
    }

    static ElementFromHandle(hwnd) {
        uia := this.Instance()
        el := ComValue(13, 0)
        ; IUIAutomation::ElementFromHandle (vtable 6)
        ComCall(6, uia, "Ptr", hwnd, "Ptr*", &pEl := 0)
        return pEl ? ComObjFromPtr(pEl) : ""
    }

    static FindByAutomationId(rootEl, autoId) {
        if !rootEl
            return ""
        uia := this.Instance()
        ; UIA_AutomationIdPropertyId = 30011
        varProp := Buffer(24, 0)
        NumPut("UShort", 8, varProp, 0) ; VT_BSTR
        bstr := DllCall("oleaut32\SysAllocString", "WStr", autoId, "Ptr")
        NumPut("Ptr", bstr, varProp, 8)
        ; CreatePropertyCondition (vtable 23)
        ComCall(23, uia, "Int", 30011, "Ptr", varProp.Ptr, "Ptr*", &pCond := 0)
        DllCall("oleaut32\SysFreeString", "Ptr", bstr)
        if !pCond
            return ""
        ; TreeScope_Descendants = 4, FindFirst (vtable 5 on IUIAutomationElement)
        ComCall(5, rootEl, "Int", 4, "Ptr", pCond, "Ptr*", &pFound := 0)
        ObjRelease(pCond)
        return pFound ? ComObjFromPtr(pFound) : ""
    }

    static GetName(el) {
        if !el
            return ""
        try {
            ; CurrentName (vtable 23 on IUIAutomationElement)
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
            ; GetCurrentPattern UIA_ValuePatternId = 10002 (vtable 16)
            ComCall(16, el, "Int", 10002, "Ptr*", &pPat := 0)
            if pPat {
                ; CurrentValue on IUIAutomationValuePattern (vtable 4)
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
            ; UIA_SelectionItemPatternId = 10010 (vtable 16)
            ComCall(16, el, "Int", 10010, "Ptr*", &pPat := 0)
            if pPat {
                ; CurrentIsSelected (vtable 6)
                ComCall(6, pPat, "Int*", &isSel := 0)
                ObjRelease(pPat)
                return isSel != 0
            }
        }
        return false
    }
}

ComObjFromPtr(ptr) {
    return ComObjValue(ptr) ? ptr : ComValue(13, ptr)
}

JsonEscape(str) {
    str := StrReplace(str, "\", "\\")
    str := StrReplace(str, '"', '\"')
    str := StrReplace(str, "`r", "\r")
    str := StrReplace(str, "`n", "\n")
    str := StrReplace(str, "`t", "\t")
    return str
}

ReadControlText(grp, autoId) {
    el := UiaHelper.FindByAutomationId(grp, autoId)
    if !el
        return ""
    val := UiaHelper.GetValue(el)
    return val != "" ? val : UiaHelper.GetName(el)
}

PushCurrentHisPatient(showErrors := true) {
    global g_IpcPort, g_ApiKey, g_LastPushedMaTN

    hwnd := WinExist("HIS: Quản Lý Tổng Thể Bệnh Viện Bình Định")
    if !hwnd {
        if showErrors
            TrayTip("KSK HIS Bridge", "Không tìm thấy cửa sổ HIS Bình Định!", 2)
        return
    }

    rootEl := UiaHelper.ElementFromHandle(hwnd)
    grp := UiaHelper.FindByAutomationId(rootEl, "grpHanhChinh")
    if !grp {
        grp := UiaHelper.FindByAutomationId(rootEl, "TiepNhan")
    }
    if !grp {
        if showErrors
            TrayTip("KSK HIS Bridge", "Hãy mở màn hình [Tiếp Nhận Bệnh Nhân] trong HIS!", 2)
        return
    }

    maTN := ReadControlText(grp, "txtSoTiepNhan")
    if (maTN = "") {
        if showErrors
            TrayTip("KSK HIS Bridge", "Chưa có Mã tiếp nhận (txtSoTiepNhan)!", 2)
        return
    }

    if (!showErrors && maTN = g_LastPushedMaTN) {
        return
    }

    maYTe := ReadControlText(grp, "txtMaYTe")
    hoTen := ReadControlText(grp, "txtHoTen")
    ngaySinh := ReadControlText(grp, "txtNgaySinh")
    soCccd := ReadControlText(grp, "txtCCCD")
    ngayCap := SubStr(ReadControlText(grp, "dtpNgayCap"), 1, 10)
    noiCap := ReadControlText(grp, "txtNoiCap")
    dienThoai := ReadControlText(grp, "txtSoDienThoai")
    soNha := ReadControlText(grp, "txtSoNha")
    diaChiLienHe := ReadControlText(grp, "lblDiaChiLienHe")
    ngayVao := SubStr(ReadControlText(grp, "calThoiGianTiepNhan"), 1, 10)
    lyDo := ReadControlText(grp, "txtLyDoVaoKCB")
    tienSuRaw := ReadControlText(grp, "txtTienSuBenh")

    ; ComboBox dùng Name để lấy text hiển thị ("Kinh", "Nghề nghiệp khác", "Phường Quy Nhơn Nam, Tỉnh Gia Lai")
    elDanToc := UiaHelper.FindByAutomationId(grp, "cboDanToc")
    tenDanToc := UiaHelper.GetName(elDanToc)

    elNghe := UiaHelper.FindByAutomationId(grp, "cboNgheNghiep")
    tenNghe := UiaHelper.GetName(elNghe)

    elXa := UiaHelper.FindByAutomationId(grp, "cboXaPhuong")
    tenXaTinh := UiaHelper.GetName(elXa)

    gioiTinh := "1"
    elNu := UiaHelper.FindByAutomationId(grp, "optNu")
    if UiaHelper.IsSelected(elNu) {
        gioiTinh := "2"
    }

    json := '{'
        . '"ma_tiep_nhan":"' . JsonEscape(maTN) . '",'
        . '"ma_y_te":"' . JsonEscape(maYTe) . '",'
        . '"ho_ten":"' . JsonEscape(hoTen) . '",'
        . '"gioi_tinh":"' . gioiTinh . '",'
        . '"ngay_sinh":"' . JsonEscape(ngaySinh) . '",'
        . '"so_cccd":"' . JsonEscape(soCccd) . '",'
        . '"ngaycap_cccd":"' . JsonEscape(ngayCap) . '",'
        . '"noicap_cccd":"' . JsonEscape(noiCap) . '",'
        . '"so_dien_thoai":"' . JsonEscape(dienThoai) . '",'
        . '"so_nha":"' . JsonEscape(soNha) . '",'
        . '"dia_chi":"' . JsonEscape(diaChiLienHe != "" ? diaChiLienHe : soNha) . '",'
        . '"ten_dan_toc":"' . JsonEscape(tenDanToc) . '",'
        . '"ten_nghe_nghiep":"' . JsonEscape(tenNghe) . '",'
        . '"ten_xa_phuong_tinh":"' . JsonEscape(tenXaTinh) . '",'
        . '"ngay_vao":"' . JsonEscape(ngayVao) . '",'
        . '"ly_do_vv":"' . JsonEscape(lyDo) . '",'
        . '"tien_su_benh_raw":"' . JsonEscape(tienSuRaw) . '"'
        . '}'

    try {
        http := ComObject("WinHttp.WinHttpRequest.5.1")
        url := "http://127.0.0.1:" . g_IpcPort . "/api/his/push"
        http.Open("POST", url, false)
        http.SetRequestHeader("Content-Type", "application/json; charset=utf-8")
        http.SetRequestHeader("x-api-key", g_ApiKey)
        http.Send(json)

        if (http.Status >= 200 && http.Status < 300) {
            g_LastPushedMaTN := maTN
            TrayTip("KSK HIS Bridge", "Đã đẩy thành công: " . hoTen . " (" . maTN . ")", 1)
        } else if showErrors {
            TrayTip("KSK HIS Bridge", "Lỗi HTTP " . http.Status . ": " . http.ResponseText, 3)
        }
    } catch as err {
        if showErrors
            TrayTip("KSK HIS Bridge", "Không kết nối được App KSK (127.0.0.1:" . g_IpcPort . "): " . err.Message, 3)
    }
}

