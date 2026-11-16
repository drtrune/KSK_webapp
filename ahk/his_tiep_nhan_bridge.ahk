#Requires AutoHotkey v2.0
#SingleInstance Force

; ============================================================================
; AutoHotkey v2 UIA -> Direct Encrypted SQLite Bridge (HIS Bình Định -> KSK)
; - Cửa sổ Cài đặt Database riêng (chọn file .db Local/LAN, Mã CSKCB)
; - Cửa sổ Đăng nhập khi khởi động (xác thực SHA256 + mở khóa Master Key)
; - Tự động tạo hồ sơ tiếp nhận mới (exam_mode = 'le') vào SQLite (hỗ trợ LAN)
; ============================================================================

global g_IniPath := A_ScriptDir "\bridge_config.ini"
global g_SqliteDll := A_ScriptDir "\sqlite3.dll"
global g_Config := {
    DbPath: "",
    LastUsername: "letan",
    MaCskcb: "52112",
    MaGtinCskcb: "",
    DefaultTemplates: "TREN_18",
    AutoWatch: true,
    PollMs: 1500
}

; Trạng thái phiên đăng nhập hiện tại (chỉ lưu trên RAM)
global g_Session := {
    LoggedIn: false,
    UserId: 0,
    Username: "",
    FullName: "",
    Role: "",
    MasterKey: "" ; Buffer(32) nếu DB bật mã hóa E2EE
}

global g_LastPushedMaTN := ""
global g_DbGui := ""
global g_LoginGui := ""

LoadBridgeConfig()
InitTrayMenu()

; Luồng khởi động: Chưa có DB -> Mở Cài đặt DB | Đã có DB -> Mở Đăng nhập
if (g_Config.DbPath = "" || !FileExist(g_Config.DbPath)) {
    ShowDbSettingsGui(true)
} else {
    ShowLoginGui()
}

; Phím tắt đẩy thủ công (chỉ hoạt động khi đã đăng nhập)
F9::PushCurrentHisPatient(true)

; ============================================================================
; 1. QUẢN LÝ CẤU HÌNH & TRAY MENU
; ============================================================================
LoadBridgeConfig() {
    global g_IniPath, g_Config
    g_Config.DbPath := IniRead(g_IniPath, "Database", "DbPath", "")
    g_Config.LastUsername := IniRead(g_IniPath, "Auth", "LastUsername", "letan")
    g_Config.MaCskcb := IniRead(g_IniPath, "Hospital", "MaCskcb", "52112")
    g_Config.MaGtinCskcb := IniRead(g_IniPath, "Hospital", "MaGtinCskcb", "")
    g_Config.DefaultTemplates := IniRead(g_IniPath, "Hospital", "DefaultTemplates", "TREN_18")
    g_Config.AutoWatch := IniRead(g_IniPath, "Bridge", "AutoWatch", "1") = "1"
    g_Config.PollMs := Integer(IniRead(g_IniPath, "Bridge", "PollMs", "1500"))
}

SaveBridgeConfig() {
    global g_IniPath, g_Config
    IniWrite(g_Config.DbPath, g_IniPath, "Database", "DbPath")
    IniWrite(g_Config.LastUsername, g_IniPath, "Auth", "LastUsername")
    IniWrite(g_Config.MaCskcb, g_IniPath, "Hospital", "MaCskcb")
    IniWrite(g_Config.MaGtinCskcb, g_IniPath, "Hospital", "MaGtinCskcb")
    IniWrite(g_Config.DefaultTemplates, g_IniPath, "Hospital", "DefaultTemplates")
    IniWrite(g_Config.AutoWatch ? "1" : "0", g_IniPath, "Bridge", "AutoWatch")
    IniWrite(String(g_Config.PollMs), g_IniPath, "Bridge", "PollMs")
}

InitTrayMenu() {
    global g_Config, g_Session
    A_TrayMenu.Delete()

    if (g_Session.LoggedIn) {
        A_TrayMenu.Add("Nhân viên: " g_Session.FullName " (" g_Session.Username ")", (*) => {})
        A_TrayMenu.Disable("Nhân viên: " g_Session.FullName " (" g_Session.Username ")")
        A_TrayMenu.Add()
        A_TrayMenu.Add("Đẩy bệnh nhân hiện tại (F9)", (*) => PushCurrentHisPatient(true))
        A_TrayMenu.Add("Tự động tạo tiếp nhận khi đổi Mã TN", ToggleAutoWatch)
        if (g_Config.AutoWatch)
            A_TrayMenu.Check("Tự động tạo tiếp nhận khi đổi Mã TN")
        A_TrayMenu.Add()
        A_TrayMenu.Add("Cài đặt Database...", (*) => ShowDbSettingsGui(false))
        A_TrayMenu.Add("Đăng xuất / Đổi tài khoản", (*) => LogoutAndShowLogin())
    } else {
        A_TrayMenu.Add("Đăng nhập hệ thống KSK...", (*) => ShowLoginGui())
        A_TrayMenu.Add("Cài đặt Database...", (*) => ShowDbSettingsGui(false))
    }

    A_TrayMenu.Add()
    A_TrayMenu.Add("Thoát", (*) => ExitApp())
    A_IconTip := g_Session.LoggedIn
        ? "HIS -> KSK Bridge (" g_Session.Username " - F9)"
        : "HIS -> KSK Bridge (Chưa đăng nhập)"
}

ToggleAutoWatch(*) {
    global g_Config, g_Session
    if !g_Session.LoggedIn {
        ShowLoginGui()
        return
    }
    g_Config.AutoWatch := !g_Config.AutoWatch
    SaveBridgeConfig()
    if g_Config.AutoWatch {
        A_TrayMenu.Check("Tự động tạo tiếp nhận khi đổi Mã TN")
        SetTimer(WatchHisTimer, g_Config.PollMs)
        TrayTip("KSK HIS Bridge", "Đã BẬT tự động tạo tiếp nhận mới từ HIS", 1)
    } else {
        A_TrayMenu.Uncheck("Tự động tạo tiếp nhận khi đổi Mã TN")
        SetTimer(WatchHisTimer, 0)
        TrayTip("KSK HIS Bridge", "Đã TẮT chế độ tự động", 1)
    }
}

WatchHisTimer() {
    global g_Config, g_Session
    if (g_Config.AutoWatch && g_Session.LoggedIn) {
        PushCurrentHisPatient(false)
    }
}

LogoutAndShowLogin() {
    global g_Session, g_LastPushedMaTN
    SetTimer(WatchHisTimer, 0)
    g_Session.LoggedIn := false
    g_Session.UserId := 0
    g_Session.Username := ""
    g_Session.FullName := ""
    g_Session.Role := ""
    g_Session.MasterKey := ""
    g_LastPushedMaTN := ""
    InitTrayMenu()
    ShowLoginGui()
}

; ============================================================================
; 2. GIAO DIỆN CÀI ĐẶT DATABASE (CHỈ CÀI DATABASE & THÔNG SỐ CSKCB)
; ============================================================================
ShowDbSettingsGui(openLoginAfterSave := false) {
    global g_DbGui, g_Config
    if IsObject(g_DbGui) {
        try {
            g_DbGui.Show()
            return
        }
    }

    g := Gui("+AlwaysOnTop", "Cài đặt Cơ sở dữ liệu KSK")
    g.SetFont("s9.5", "Segoe UI")
    g.MarginX := 16
    g.MarginY := 14

    g.AddGroupBox("w510 h110 Section", "1. Kết nối File Cơ sở dữ liệu SQLite (Local hoặc Mạng LAN)")
    g.AddText("xs+14 ys+28", "Đường dẫn DB:")
    edtDb := g.AddEdit("xs+120 ys+25 w295", g_Config.DbPath)
    btnBrowse := g.AddButton("xs+422 ys+24 w74 h25", "Chọn file...")
    btnBrowse.OnEvent("Click", (*) => (
        sel := FileSelect(1, edtDb.Value, "Chọn file cơ sở dữ liệu KSK", "SQLite DB (*.db; *.sqlite; *.sqlite3)"),
        sel != "" ? edtDb.Value := sel : ""
    ))
    g.AddText("xs+14 ys+62 w480 c666666", "Hỗ trợ ổ đĩa mạng nội bộ (vd: \\192.168.1.10\KSK_Data\ksk.db).`nTự động cấu hình Journal DELETE + Busy Timeout 15s an toàn đa máy trạm.")

    g.AddGroupBox("xs y+20 w510 h120 Section", "2. Thông số Cơ sở KCB & Chế độ đồng bộ")
    g.AddText("xs+14 ys+28", "Mã CSKCB:")
    edtCskcb := g.AddEdit("xs+120 ys+25 w120", g_Config.MaCskcb)
    g.AddText("xs+260 ys+28", "Mã GTIN:")
    edtGtin := g.AddEdit("xs+325 ys+25 w170", g_Config.MaGtinCskcb)

    g.AddText("xs+14 ys+65", "Mẫu KSK mặc định:")
    ddlTpl := g.AddDropDownList("xs+120 ys+62 w145", ["TREN_18", "DUOI_18", "LAI_XE", "DINH_KY"])
    try ddlTpl.Text := g_Config.DefaultTemplates

    chkAuto := g.AddCheckBox("xs+280 ys+65 Checked" (g_Config.AutoWatch ? "1" : "0"), "Tự động tạo tiếp nhận mới")

    btnTest := g.AddButton("xs y+22 w155 h32", "Kiểm tra kết nối DB")
    btnSave := g.AddButton("x+12 w165 h32 Default", "Lưu && Tiếp tục")
    btnCancel := g.AddButton("x+12 w100 h32", "Đóng")

    btnTest.OnEvent("Click", (*) => TestDatabaseConnection(edtDb.Value))
    btnSave.OnEvent("Click", (*) => SaveDbSettingsFromUi(g, edtDb.Value, edtCskcb.Value, edtGtin.Value, ddlTpl.Text, chkAuto.Value, openLoginAfterSave))
    btnCancel.OnEvent("Click", (*) => g.Hide())
    g.OnEvent("Close", (*) => g.Hide())

    g_DbGui := g
    g.Show()
}

TestDatabaseConnection(dbPath) {
    p := Trim(dbPath)
    if (p = "" || !FileExist(p)) {
        MsgBox("Không tìm thấy file database tại:`n" p, "Lỗi đường dẫn", 16)
        return
    }
    try {
        db := SqliteDb(p)
        userCnt := db.QueryScalar("SELECT COUNT(*) FROM users;")
        recCnt := db.QueryScalar("SELECT COUNT(*) FROM ksk_records;")
        hasEnc := db.QueryScalar("SELECT value FROM app_settings WHERE key = 'recovery_wrapped_mk' LIMIT 1;")
        db.Close()

        encInfo := hasEnc != "" ? "Đang bật mã hóa E2EE (AES-256-GCM)" : "Chưa bật mã hóa (Plaintext)"
        MsgBox("Kết nối cơ sở dữ liệu KSK thành công!`n"
            . "- Số tài khoản hệ thống: " userCnt "`n"
            . "- Số hồ sơ KSK hiện có: " recCnt "`n"
            . "- Chế độ bảo mật: " encInfo, "Kiểm tra thành công", 64)
    } catch as err {
        MsgBox("Lỗi kết nối SQLite:`n" err.Message, "Lỗi Database", 16)
    }
}

SaveDbSettingsFromUi(g, dbPath, cskcb, gtin, tpl, autoW, openLoginAfterSave) {
    global g_Config, g_Session
    p := Trim(dbPath)
    if (p = "" || !FileExist(p)) {
        MsgBox("Vui lòng chọn đúng đường dẫn file database SQLite (.db) hợp lệ!", "Thiếu Database", 48)
        return
    }

    ; Kiểm tra bảng hợp lệ trước khi lưu
    try {
        db := SqliteDb(p)
        db.Close()
    } catch as err {
        MsgBox("File database không hợp lệ:`n" err.Message, "Lỗi Database", 16)
        return
    }

    dbChanged := (g_Config.DbPath != p)
    g_Config.DbPath := p
    g_Config.MaCskcb := Trim(cskcb)
    g_Config.MaGtinCskcb := Trim(gtin)
    g_Config.DefaultTemplates := tpl
    g_Config.AutoWatch := autoW ? true : false
    SaveBridgeConfig()

    g.Hide()

    if (dbChanged && g_Session.LoggedIn) {
        MsgBox("Đường dẫn Database đã thay đổi. Vui lòng đăng nhập lại vào Database mới!", "Đổi Database", 64)
        LogoutAndShowLogin()
        return
    }

    if (openLoginAfterSave || !g_Session.LoggedIn) {
        ShowLoginGui()
    } else {
        InitTrayMenu()
        if (g_Config.AutoWatch)
            SetTimer(WatchHisTimer, g_Config.PollMs)
        else
            SetTimer(WatchHisTimer, 0)
        TrayTip("KSK HIS Bridge", "Đã lưu cấu hình Database!", 1)
    }
}

; ============================================================================
; 3. GIAO DIỆN ĐĂNG NHẬP KHI KHỞI ĐỘNG (XÁC THỰC USERS & UNWRAP MASTER KEY)
; ============================================================================
ShowLoginGui() {
    global g_LoginGui, g_Config
    if IsObject(g_LoginGui) {
        try {
            g_LoginGui.Destroy()
        }
        g_LoginGui := ""
    }

    g := Gui("+AlwaysOnTop", "Đăng nhập Cầu nối HIS -> KSK")
    g.SetFont("s9.5", "Segoe UI")
    g.MarginX := 18
    g.MarginY := 14

    g.SetFont("s11 Bold", "Segoe UI")
    g.AddText("w380 c0F172A", "ĐĂNG NHẬP HỆ THỐNG TIẾP NHẬN KSK")
    g.SetFont("s8.5 Norm", "Segoe UI")
    dbShort := g_Config.DbPath
    if (StrLen(dbShort) > 55)
        dbShort := "..." SubStr(dbShort, -52)
    g.AddText("y+4 w380 c64748B", "Database: " dbShort)

    g.SetFont("s9.5 Norm", "Segoe UI")
    g.AddText("y+14", "Tên đăng nhập:")
    edtUser := g.AddEdit("y+4 w380", g_Config.LastUsername)

    g.AddText("y+10", "Mật khẩu:")
    edtPass := g.AddEdit("y+4 w380 Password")

    txtErr := g.AddText("y+8 w380 h20 cDC2626", "")

    btnLogin := g.AddButton("y+6 w245 h34 Default", "Đăng nhập")
    btnCfgDb := g.AddButton("x+10 w125 h34", "Cài đặt Database...")

    btnLogin.OnEvent("Click", (*) => HandleLoginSubmit(g, edtUser.Value, edtPass.Value, txtErr))
    btnCfgDb.OnEvent("Click", (*) => (g.Hide(), ShowDbSettingsGui(true)))
    g.OnEvent("Close", (*) => (
        g.Hide(),
        TrayTip("KSK HIS Bridge", "Ứng dụng đang chạy ẩn ở khay hệ thống (Chưa đăng nhập).", 2)
    ))

    g_LoginGui := g
    g.Show()
    if (g_Config.LastUsername != "")
        edtPass.Focus()
    else
        edtUser.Focus()
}

HandleLoginSubmit(g, usernameInput, passwordInput, txtErrCtrl) {
    global g_Config, g_Session

    uname := Trim(usernameInput)
    pass := passwordInput
    if (uname = "" || pass = "") {
        txtErrCtrl.Value := "Vui lòng nhập đầy đủ Tên đăng nhập và Mật khẩu!"
        return
    }

    if (g_Config.DbPath = "" || !FileExist(g_Config.DbPath)) {
        txtErrCtrl.Value := "Không tìm thấy file Database! Hãy bấm [Cài đặt Database...]"
        return
    }

    try {
        db := SqliteDb(g_Config.DbPath)
        rows := db.QueryRow(
            "SELECT id, username, password_hash, full_name, role, is_locked, wrapped_mk "
            . "FROM users WHERE username = '" SqlEsc(uname) "' LIMIT 1;",
            7
        )
        recWrapped := db.QueryScalar("SELECT value FROM app_settings WHERE key = 'recovery_wrapped_mk' LIMIT 1;")
        db.Close()

        if (rows.Length < 7 || rows[2] = "") {
            txtErrCtrl.Value := "Tài khoản '" uname "' không tồn tại trong hệ thống!"
            return
        }

        userId := Integer(rows[1])
        dbUser := rows[2]
        dbPassHash := rows[3]
        fullName := rows[4]
        roleStr := rows[5]
        isLocked := Integer(rows[6])
        userWrappedMk := rows[7]

        if (isLocked != 0) {
            txtErrCtrl.Value := "Tài khoản này đã bị khóa! Liên hệ Quản trị viên."
            return
        }

        ; Kiểm tra mật khẩu bằng SHA-256 (khớp với hash_password trong db.rs)
        inputHash := RustCrypto.Sha256Hex(pass)
        if (StrLower(inputHash) != StrLower(dbPassHash)) {
            txtErrCtrl.Value := "Mật khẩu không chính xác!"
            return
        }

        ; Kiểm tra quyền (chỉ cho phép role 'reception' hoặc 'admin')
        if !HasReceptionPermission(roleStr) {
            txtErrCtrl.Value := "Tài khoản không có quyền Tiếp nhận (reception/admin)!"
            return
        }

        ; Giải mã Master Key nếu hệ thống đang bật mã hóa E2EE
        mkBuf := ""
        if (recWrapped != "" || userWrappedMk != "") {
            if (userWrappedMk != "") {
                mkBuf := RustCrypto.UnwrapMasterKey(userWrappedMk, pass)
            } else {
                txtErrCtrl.Value := "Tài khoản chưa được cấp khóa giải mã (wrapped_mk)!"
                return
            }
        }

        ; Lưu phiên đăng nhập trên RAM
        g_Session.LoggedIn := true
        g_Session.UserId := userId
        g_Session.Username := dbUser
        g_Session.FullName := fullName
        g_Session.Role := roleStr
        g_Session.MasterKey := mkBuf

        g_Config.LastUsername := dbUser
        SaveBridgeConfig()

        g.Destroy()
        global g_LoginGui := ""

        InitTrayMenu()
        if (g_Config.AutoWatch) {
            SetTimer(WatchHisTimer, g_Config.PollMs)
        }

        TrayTip("KSK HIS Bridge", "Xin chào " fullName "! Cầu nối HIS -> KSK đã sẵn sàng.", 1)
    } catch as err {
        txtErrCtrl.Value := "Lỗi đăng nhập: " err.Message
    }
}

HasReceptionPermission(roleStr) {
    for r in StrSplit(roleStr, ",") {
        t := Trim(r)
        if (t = "admin" || t = "reception")
            return true
    }
    return false
}

; ============================================================================
; 4. CRYPTO ENGINE (SHA-256 + PBKDF2-HMAC-SHA256 + AES-256-GCM)
; ============================================================================
class RustCrypto {
    static PBKDF2_ITERS := 100000

    static Sha256Hex(str) {
        DllCall("bcrypt\BCryptOpenAlgorithmProvider", "Ptr*", &hAlg := 0, "WStr", "SHA256", "Ptr", 0, "UInt", 0)
        DllCall("bcrypt\BCryptCreateHash", "Ptr", hAlg, "Ptr*", &hHash := 0, "Ptr", 0, "UInt", 0, "Ptr", 0, "UInt", 0, "UInt", 0)
        dataBuf := StrToUtf8Buf(str)
        if (dataBuf.Size > 0)
            DllCall("bcrypt\BCryptHashData", "Ptr", hHash, "Ptr", dataBuf.Ptr, "UInt", dataBuf.Size, "UInt", 0)
        digest := Buffer(32, 0)
        DllCall("bcrypt\BCryptFinishHash", "Ptr", hHash, "Ptr", digest.Ptr, "UInt", 32, "UInt", 0)
        DllCall("bcrypt\BCryptDestroyHash", "Ptr", hHash)
        DllCall("bcrypt\BCryptCloseAlgorithmProvider", "Ptr", hAlg, "UInt", 0)

        hex := ""
        loop 32 {
            hex .= Format("{:02x}", NumGet(digest, A_Index - 1, "UChar"))
        }
        return hex
    }

    static UnwrapMasterKey(wrappedStr, secret) {
        s := Trim(wrappedStr)
        if (SubStr(s, 1, 4) != "MK1:")
            throw Error("Định dạng Master Key không hợp lệ (thiếu MK1:)")
        raw := Base64DecodeToBuf(SubStr(s, 5))
        if (raw.Size < 16 + 12 + 16)
            throw Error("Dữ liệu Master Key bị thiếu hoặc hỏng")

        salt := Buffer(16)
        nonce := Buffer(12)
        ctLen := raw.Size - 28
        ct := Buffer(ctLen)

        DllCall("RtlMoveMemory", "Ptr", salt.Ptr, "Ptr", raw.Ptr, "UPtr", 16)
        DllCall("RtlMoveMemory", "Ptr", nonce.Ptr, "Ptr", raw.Ptr + 16, "UPtr", 12)
        DllCall("RtlMoveMemory", "Ptr", ct.Ptr, "Ptr", raw.Ptr + 28, "UPtr", ctLen)

        kek := this.DeriveKek(Trim(secret), salt)
        mk := this.Aes256GcmDecrypt(kek, nonce, ct)
        if (mk.Size != 32)
            throw Error("Độ dài Master Key giải mã không phải 32 bytes")
        return mk
    }

    static EncryptField(mkBuf, plaintext) {
        if !IsObject(mkBuf)
            return plaintext
        ptBuf := StrToUtf8Buf(plaintext)
        nonce := this.RandomBytes(12)
        ctWithTag := this.Aes256GcmEncrypt(mkBuf, nonce, ptBuf)

        combined := Buffer(12 + ctWithTag.Size)
        DllCall("RtlMoveMemory", "Ptr", combined.Ptr, "Ptr", nonce.Ptr, "UPtr", 12)
        DllCall("RtlMoveMemory", "Ptr", combined.Ptr + 12, "Ptr", ctWithTag.Ptr, "UPtr", ctWithTag.Size)
        return "ENC1:" BufToBase64(combined)
    }

    static DecryptField(mkBuf, value) {
        if (SubStr(value, 1, 5) != "ENC1:")
            return value
        if !IsObject(mkBuf)
            return ""
        raw := Base64DecodeToBuf(SubStr(value, 6))
        if (raw.Size < 28)
            return ""
        nonce := Buffer(12)
        ct := Buffer(raw.Size - 12)
        DllCall("RtlMoveMemory", "Ptr", nonce.Ptr, "Ptr", raw.Ptr, "UPtr", 12)
        DllCall("RtlMoveMemory", "Ptr", ct.Ptr, "Ptr", raw.Ptr + 12, "UPtr", raw.Size - 12)
        ptBuf := this.Aes256GcmDecrypt(mkBuf, nonce, ct)
        return StrGet(ptBuf.Ptr, ptBuf.Size, "UTF-8")
    }

    static RandomBytes(len) {
        buf := Buffer(len, 0)
        DllCall("bcrypt\BCryptGenRandom", "Ptr", 0, "Ptr", buf.Ptr, "UInt", len, "UInt", 2)
        return buf
    }

    static DeriveKek(secret, saltBuf) {
        DllCall("bcrypt\BCryptOpenAlgorithmProvider", "Ptr*", &hAlg := 0, "WStr", "SHA256", "Ptr", 0, "UInt", 8)
        passBuf := StrToUtf8Buf(secret)
        kek := Buffer(32, 0)
        status := DllCall("bcrypt\BCryptDeriveKeyPBKDF2"
            , "Ptr", hAlg
            , "Ptr", passBuf.Size > 0 ? passBuf.Ptr : 0, "UInt", passBuf.Size
            , "Ptr", saltBuf.Ptr, "UInt", saltBuf.Size
            , "UInt64", this.PBKDF2_ITERS
            , "Ptr", kek.Ptr, "UInt", 32
            , "UInt", 0, "UInt")
        DllCall("bcrypt\BCryptCloseAlgorithmProvider", "Ptr", hAlg, "UInt", 0)
        if (status != 0)
            throw Error("Lỗi PBKDF2: " Format("0x{:X}", status))
        return kek
    }

    static Aes256GcmEncrypt(keyBuf, nonceBuf, ptBuf) {
        DllCall("bcrypt\BCryptOpenAlgorithmProvider", "Ptr*", &hAlg := 0, "WStr", "AES", "Ptr", 0, "UInt", 0)
        modeW := Buffer(32, 0), StrPut("ChainingModeGCM", modeW, "UTF-16")
        DllCall("bcrypt\BCryptSetProperty", "Ptr", hAlg, "WStr", "ChainingMode", "Ptr", modeW.Ptr, "UInt", 32, "UInt", 0)
        DllCall("bcrypt\BCryptGenerateSymmetricKey", "Ptr", hAlg, "Ptr*", &hKey := 0, "Ptr", 0, "UInt", 0, "Ptr", keyBuf.Ptr, "UInt", keyBuf.Size, "UInt", 0)

        tag := Buffer(16, 0)
        authInfo := Buffer(88, 0)
        NumPut("UInt", 88, authInfo, 0)
        NumPut("UInt", 1, authInfo, 4)
        NumPut("Ptr", nonceBuf.Ptr, authInfo, 8)
        NumPut("UInt", 12, authInfo, 16)
        NumPut("Ptr", tag.Ptr, authInfo, 40)
        NumPut("UInt", 16, authInfo, 48)

        ctBuf := Buffer(ptBuf.Size > 0 ? ptBuf.Size : 1, 0)
        status := DllCall("bcrypt\BCryptEncrypt"
            , "Ptr", hKey
            , "Ptr", ptBuf.Size > 0 ? ptBuf.Ptr : 0, "UInt", ptBuf.Size
            , "Ptr", authInfo.Ptr
            , "Ptr", 0, "UInt", 0
            , "Ptr", ptBuf.Size > 0 ? ctBuf.Ptr : 0, "UInt", ptBuf.Size
            , "UInt*", &cbResult := 0
            , "UInt", 0, "UInt")

        DllCall("bcrypt\BCryptDestroyKey", "Ptr", hKey)
        DllCall("bcrypt\BCryptCloseAlgorithmProvider", "Ptr", hAlg, "UInt", 0)
        if (status != 0)
            throw Error("Lỗi AES-GCM Encrypt: " Format("0x{:X}", status))

        out := Buffer(cbResult + 16)
        if (cbResult > 0)
            DllCall("RtlMoveMemory", "Ptr", out.Ptr, "Ptr", ctBuf.Ptr, "UPtr", cbResult)
        DllCall("RtlMoveMemory", "Ptr", out.Ptr + cbResult, "Ptr", tag.Ptr, "UPtr", 16)
        return out
    }

    static Aes256GcmDecrypt(keyBuf, nonceBuf, ctWithTagBuf) {
        ctLen := ctWithTagBuf.Size - 16
        ct := Buffer(ctLen > 0 ? ctLen : 1, 0)
        tag := Buffer(16, 0)
        if (ctLen > 0)
            DllCall("RtlMoveMemory", "Ptr", ct.Ptr, "Ptr", ctWithTagBuf.Ptr, "UPtr", ctLen)
        DllCall("RtlMoveMemory", "Ptr", tag.Ptr, "Ptr", ctWithTagBuf.Ptr + ctLen, "UPtr", 16)

        DllCall("bcrypt\BCryptOpenAlgorithmProvider", "Ptr*", &hAlg := 0, "WStr", "AES", "Ptr", 0, "UInt", 0)
        modeW := Buffer(32, 0), StrPut("ChainingModeGCM", modeW, "UTF-16")
        DllCall("bcrypt\BCryptSetProperty", "Ptr", hAlg, "WStr", "ChainingMode", "Ptr", modeW.Ptr, "UInt", 32, "UInt", 0)
        DllCall("bcrypt\BCryptGenerateSymmetricKey", "Ptr", hAlg, "Ptr*", &hKey := 0, "Ptr", 0, "UInt", 0, "Ptr", keyBuf.Ptr, "UInt", keyBuf.Size, "UInt", 0)

        authInfo := Buffer(88, 0)
        NumPut("UInt", 88, authInfo, 0)
        NumPut("UInt", 1, authInfo, 4)
        NumPut("Ptr", nonceBuf.Ptr, authInfo, 8)
        NumPut("UInt", 12, authInfo, 16)
        NumPut("Ptr", tag.Ptr, authInfo, 40)
        NumPut("UInt", 16, authInfo, 48)

        ptBuf := Buffer(ctLen > 0 ? ctLen : 1, 0)
        status := DllCall("bcrypt\BCryptDecrypt"
            , "Ptr", hKey
            , "Ptr", ctLen > 0 ? ct.Ptr : 0, "UInt", ctLen
            , "Ptr", authInfo.Ptr
            , "Ptr", 0, "UInt", 0
            , "Ptr", ctLen > 0 ? ptBuf.Ptr : 0, "UInt", ctLen
            , "UInt*", &cbResult := 0
            , "UInt", 0, "UInt")

        DllCall("bcrypt\BCryptDestroyKey", "Ptr", hKey)
        DllCall("bcrypt\BCryptCloseAlgorithmProvider", "Ptr", hAlg, "UInt", 0)
        if (status != 0)
            throw Error("Không thể giải mã Master Key bằng mật khẩu này")

        out := Buffer(cbResult > 0 ? cbResult : 1, 0)
        if (cbResult > 0)
            DllCall("RtlMoveMemory", "Ptr", out.Ptr, "Ptr", ptBuf.Ptr, "UPtr", cbResult)
        out.Size := cbResult
        return out
    }
}

; ============================================================================
; 5. SQLITE3 NATIVE WRAPPER (AN TOÀN CHO MẠNG LAN / SMB)
; ============================================================================
class SqliteDb {
    __New(dbPath) {
        global g_SqliteDll
        dllPath := FileExist(g_SqliteDll) ? g_SqliteDll : "sqlite3.dll"
        if !DllCall("LoadLibrary", "Str", dllPath, "Ptr")
            throw Error("Không tìm thấy sqlite3.dll! Hãy đặt sqlite3.dll (64-bit) vào cùng thư mục script.")

        pathBuf := StrToUtf8Buf(dbPath)
        rc := DllCall("sqlite3\sqlite3_open_v2", "Ptr", pathBuf.Ptr, "Ptr*", &hDb := 0, "Int", 0x10006, "Ptr", 0, "Int")
        if (rc != 0 || !hDb)
            throw Error("Không mở được SQLite DB (mã lỗi " rc "): " dbPath)
        this.hDb := hDb

        DllCall("sqlite3\sqlite3_busy_timeout", "Ptr", this.hDb, "Int", 15000)

        try {
            this.QueryScalar("PRAGMA journal_mode = DELETE;")
            this.QueryScalar("PRAGMA synchronous = FULL;")
        }

        tbl := this.QueryScalar("SELECT name FROM sqlite_master WHERE type='table' AND name='ksk_records' LIMIT 1;")
        if (tbl = "") {
            this.Close()
            throw Error("File DB này chưa có bảng 'ksk_records'! Hãy chọn đúng file database của ứng dụng KSK.")
        }
    }

    Exec(sql) {
        sqlBuf := StrToUtf8Buf(sql)
        rc := DllCall("sqlite3\sqlite3_exec", "Ptr", this.hDb, "Ptr", sqlBuf.Ptr, "Ptr", 0, "Ptr", 0, "Ptr*", &pErr := 0, "Int")
        if (rc != 0) {
            errMsg := pErr ? StrGet(pErr, "UTF-8") : "SQLite error code " rc
            if pErr
                DllCall("sqlite3\sqlite3_free", "Ptr", pErr)
            throw Error(errMsg "`n`nSQL: " SubStr(sql, 1, 300))
        }
    }

    QueryScalar(sql) {
        sqlBuf := StrToUtf8Buf(sql)
        rc := DllCall("sqlite3\sqlite3_prepare_v2", "Ptr", this.hDb, "Ptr", sqlBuf.Ptr, "Int", -1, "Ptr*", &pStmt := 0, "Ptr", 0, "Int")
        if (rc != 0 || !pStmt)
            return ""
        val := ""
        if (DllCall("sqlite3\sqlite3_step", "Ptr", pStmt, "Int") = 100) {
            pTxt := DllCall("sqlite3\sqlite3_column_text", "Ptr", pStmt, "Int", 0, "Ptr")
            if pTxt
                val := StrGet(pTxt, "UTF-8")
        }
        DllCall("sqlite3\sqlite3_finalize", "Ptr", pStmt)
        return val
    }

    QueryRow(sql, colCount) {
        sqlBuf := StrToUtf8Buf(sql)
        rc := DllCall("sqlite3\sqlite3_prepare_v2", "Ptr", this.hDb, "Ptr", sqlBuf.Ptr, "Int", -1, "Ptr*", &pStmt := 0, "Ptr", 0, "Int")
        row := []
        if (rc != 0 || !pStmt)
            return row
        if (DllCall("sqlite3\sqlite3_step", "Ptr", pStmt, "Int") = 100) {
            loop colCount {
                pTxt := DllCall("sqlite3\sqlite3_column_text", "Ptr", pStmt, "Int", A_Index - 1, "Ptr")
                row.Push(pTxt ? StrGet(pTxt, "UTF-8") : "")
            }
        }
        DllCall("sqlite3\sqlite3_finalize", "Ptr", pStmt)
        return row
    }

    QueryPairs(sql) {
        sqlBuf := StrToUtf8Buf(sql)
        rc := DllCall("sqlite3\sqlite3_prepare_v2", "Ptr", this.hDb, "Ptr", sqlBuf.Ptr, "Int", -1, "Ptr*", &pStmt := 0, "Ptr", 0, "Int")
        rows := []
        if (rc != 0 || !pStmt)
            return rows
        while (DllCall("sqlite3\sqlite3_step", "Ptr", pStmt, "Int") = 100) {
            p0 := DllCall("sqlite3\sqlite3_column_text", "Ptr", pStmt, "Int", 0, "Ptr")
            p1 := DllCall("sqlite3\sqlite3_column_text", "Ptr", pStmt, "Int", 1, "Ptr")
            c0 := p0 ? StrGet(p0, "UTF-8") : ""
            c1 := p1 ? StrGet(p1, "UTF-8") : ""
            rows.Push([c0, c1])
        }
        DllCall("sqlite3\sqlite3_finalize", "Ptr", pStmt)
        return rows
    }

    Close() {
        if this.hDb {
            DllCall("sqlite3\sqlite3_close_v2", "Ptr", this.hDb)
            this.hDb := 0
        }
    }
}

; ============================================================================
; 6. ĐỌC UIA TỪ HIS & TẠO TIẾP NHẬN MỚI TRỰC TIẾP VÀO SQLITE
; ============================================================================
PushCurrentHisPatient(showErrors := true) {
    global g_Config, g_Session, g_LastPushedMaTN

    if (g_Config.DbPath = "" || !FileExist(g_Config.DbPath)) {
        if showErrors {
            TrayTip("KSK HIS Bridge", "Chưa cấu hình file Database KSK!", 2)
            ShowDbSettingsGui(true)
        }
        return
    }

    if !g_Session.LoggedIn {
        if showErrors {
            TrayTip("KSK HIS Bridge", "Vui lòng đăng nhập tài khoản KSK trước!", 2)
            ShowLoginGui()
        }
        return
    }

    hwnd := WinExist("HIS: Quản Lý Tổng Thể Bệnh Viện Bình Định")
    if !hwnd {
        if showErrors
            TrayTip("KSK HIS Bridge", "Không tìm thấy cửa sổ HIS Bình Định!", 2)
        return
    }

    rootEl := UiaHelper.ElementFromHandle(hwnd)
    grp := UiaHelper.FindByAutomationId(rootEl, "grpHanhChinh")
    if !grp
        grp := UiaHelper.FindByAutomationId(rootEl, "TiepNhan")
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

    if (!showErrors && maTN = g_LastPushedMaTN)
        return

    maYTe := ReadControlText(grp, "txtMaYTe")
    hoTen := StrUpper(Trim(ReadControlText(grp, "txtHoTen")))
    if (hoTen = "")
        return

    ngaySinh := SubStr(ReadControlText(grp, "txtNgaySinh"), 1, 10)
    soCccd := ReadControlText(grp, "txtCCCD")
    ngayCap := SubStr(ReadControlText(grp, "dtpNgayCap"), 1, 10)
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

    try {
        db := SqliteDb(g_Config.DbPath)

        maDanToc := ResolveCatalogCode(db, "DAN_TOC", tenDanToc, "", "01")
        maNgheNghiep := ResolveCatalogCode(db, "NGHE_NGHIEP", tenNghe, "", "00")

        maTinh := "11221"
        maXa := ""
        if (tenXaTinh != "") {
            parts := StrSplit(tenXaTinh, ",")
            if (parts.Length >= 2) {
                xaPart := Trim(parts[1])
                tinhPart := Trim(parts[parts.Length])
                maTinh := ResolveCatalogCode(db, "TINH_TP", tinhPart, "", "11221")
                maXa := ResolveCatalogCode(db, "XA_PHUONG", xaPart, maTinh, "")
                if (maXa = "")
                    maXa := ResolveCatalogCode(db, "XA_PHUONG", xaPart, "", "")
            } else {
                maXa := ResolveCatalogCode(db, "XA_PHUONG", tenXaTinh, maTinh, "")
            }
        }

        diaChi := diaChiLienHe != "" ? diaChiLienHe : (soNha != "" && tenXaTinh != "" ? soNha ", " tenXaTinh : soNha)

        existingRaw := db.QueryScalar("SELECT payload_json FROM ksk_records WHERE ma_tiep_nhan = '" SqlEsc(maTN) "' LIMIT 1;")
        existingPlain := existingRaw != "" ? RustCrypto.DecryptField(g_Session.MasterKey, existingRaw) : ""

        payloadJson := BuildOrMergePayloadJson(existingPlain, {
            MA_TIEP_NHAN: maTN,
            MA_LK: maTN,
            MA_Y_TE: maYTe,
            HO_TEN: hoTen,
            GIOI_TINH: gioiTinh,
            NGAY_SINH: ngaySinh,
            MA_DAN_TOC: maDanToc,
            SO_CCCD: soCccd,
            NGAYCAP_CCCD: ngayCap,
            NOICAP_CCCD: noiCap,
            DIA_CHI: diaChi,
            MATINH_CU_TRU: maTinh,
            MAXA_CU_TRU: maXa,
            MA_NGHE_NGHIEP: maNgheNghiep,
            DIEN_THOAI: dienThoai,
            NGAY_VAO: ngayVao,
            MA_CSKCB: g_Config.MaCskcb,
            MA_GTIN_CSKCB: g_Config.MaGtinCskcb,
            LY_DO_VV: lyDo,
            HIS_TIEN_SU_RAW: tienSuRaw
        })

        encHoTen := RustCrypto.EncryptField(g_Session.MasterKey, hoTen)
        encNgaySinh := RustCrypto.EncryptField(g_Session.MasterKey, ngaySinh)
        encSoCccd := RustCrypto.EncryptField(g_Session.MasterKey, soCccd)
        encPayload := RustCrypto.EncryptField(g_Session.MasterKey, payloadJson)

        sql := "INSERT INTO ksk_records ("
            . "ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, "
            . "status, exam_mode, exam_templates, payload_json, updated_by, updated_at"
            . ") VALUES ('"
            . SqlEsc(maTN) "', '" SqlEsc(maYTe) "', '" SqlEsc(encHoTen) "', '" SqlEsc(gioiTinh) "', '"
            . SqlEsc(encNgaySinh) "', '" SqlEsc(encSoCccd) "', '" SqlEsc(ngayVao) "', "
            . "0, 'le', '" SqlEsc(g_Config.DefaultTemplates) "', '" SqlEsc(encPayload) "', '"
            . SqlEsc(g_Session.Username) "', datetime('now', 'localtime')) "
            . "ON CONFLICT(ma_tiep_nhan) DO UPDATE SET "
            . "ma_y_te = excluded.ma_y_te, "
            . "ho_ten = excluded.ho_ten, "
            . "gioi_tinh = excluded.gioi_tinh, "
            . "ngay_sinh = excluded.ngay_sinh, "
            . "so_cccd = excluded.so_cccd, "
            . "ngay_vao = excluded.ngay_vao, "
            . "payload_json = excluded.payload_json, "
            . "updated_by = excluded.updated_by, "
            . "updated_at = datetime('now', 'localtime');"

        db.Exec(sql)
        db.Close()

        g_LastPushedMaTN := maTN
        TrayTip("KSK HIS Bridge", "Đã tạo tiếp nhận mới (" g_Session.Username "):`n" hoTen " (" maTN ")", 1)
    } catch as err {
        if showErrors
            TrayTip("KSK HIS Bridge", "Lỗi ghi SQLite: " err.Message, 3)
    }
}

ResolveCatalogCode(db, catType, nameQuery, parentCode, defaultCode) {
    q := Trim(nameQuery)
    if (q = "")
        return defaultCode
    sql := parentCode != ""
        ? "SELECT code, name FROM catalogs WHERE cat_type = '" SqlEsc(catType) "' AND parent_code = '" SqlEsc(parentCode) "' AND is_active = 1;"
        : "SELECT code, name FROM catalogs WHERE cat_type = '" SqlEsc(catType) "' AND is_active = 1;"
    rows := db.QueryPairs(sql)
    qLow := StrLower(q)
    for r in rows {
        if (StrLower(Trim(r[2])) = qLow)
            return r[1]
    }
    qNorm := NormalizeViMatch(q)
    for r in rows {
        if (NormalizeViMatch(r[2]) = qNorm)
            return r[1]
    }
    for r in rows {
        nNorm := NormalizeViMatch(r[2])
        if (qNorm != "" && (InStr(nNorm, qNorm) || InStr(qNorm, nNorm)))
            return r[1]
    }
    return defaultCode
}

NormalizeViMatch(s) {
    s := StrLower(Trim(s))
    for prefix in ["tỉnh ", "thành phố ", "tp. ", "tp ", "phường ", "xã ", "thị trấn "] {
        s := StrReplace(s, prefix, "")
    }
    return Trim(s)
}

BuildOrMergePayloadJson(existingJson, fieldsObj) {
    base := Trim(existingJson)
    if (SubStr(base, 1, 1) != "{" || SubStr(base, -1) != "}") {
        base := '{"DOI_TUONG":"13","NGUON_CHI_TRA":"9","MA_LOAI_KCB":"01"}'
    }
    for k, v in fieldsObj.OwnProps() {
        if (v = "" && (k = "MAXA_CU_TRU" || k = "LY_DO_VV" || k = "HIS_TIEN_SU_RAW"))
            continue
        escapedVal := '"' JsonEscape(v) '"'
        regexSafeVal := StrReplace(escapedVal, "$", "$$")
        pattern := '"' k '"\s*:\s*("(?:\\.|[^"\\])*"|null|true|false|-?\d+(?:\.\d+)?)'
        if RegExMatch(base, pattern) {
            base := RegExReplace(base, pattern, '"' k '":' regexSafeVal, , 1)
        } else {
            base := SubStr(base, 1, -1) . (StrLen(base) > 2 ? "," : "") . '"' k '":' escapedVal . "}"
        }
    }
    return base
}

; ============================================================================
; 7. UIA COM & BASE64 / UTF-8 HELPERS
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

SqlEsc(str) => StrReplace(str, "'", "''")

JsonEscape(str) {
    str := StrReplace(str, "\", "\\")
    str := StrReplace(str, '"', '\"')
    str := StrReplace(str, "`r", "\r")
    str := StrReplace(str, "`n", "\n")
    str := StrReplace(str, "`t", "\t")
    return str
}

StrToUtf8Buf(str) {
    len := StrPut(str, "UTF-8") - 1
    buf := Buffer(len > 0 ? len : 1, 0)
    if (len > 0)
        StrPut(str, buf, len, "UTF-8")
    buf.Size := len
    return buf
}

BufToBase64(buf) {
    DllCall("crypt32\CryptBinaryToStringW", "Ptr", buf.Ptr, "UInt", buf.Size, "UInt", 0x40000001, "Ptr", 0, "UInt*", &cch := 0)
    out := Buffer(cch * 2, 0)
    DllCall("crypt32\CryptBinaryToStringW", "Ptr", buf.Ptr, "UInt", buf.Size, "UInt", 0x40000001, "Ptr", out.Ptr, "UInt*", &cch)
    return StrGet(out.Ptr, "UTF-16")
}

Base64DecodeToBuf(b64) {
    DllCall("crypt32\CryptStringToBinaryW", "WStr", b64, "UInt", 0, "UInt", 1, "Ptr", 0, "UInt*", &cb := 0, "Ptr", 0, "Ptr", 0)
    buf := Buffer(cb > 0 ? cb : 1, 0)
    if (cb > 0)
        DllCall("crypt32\CryptStringToBinaryW", "WStr", b64, "UInt", 0, "UInt", 1, "Ptr", buf.Ptr, "UInt*", &cb, "Ptr", 0, "Ptr", 0)
    buf.Size := cb
    return buf
}
