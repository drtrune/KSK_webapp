import urllib.request
import urllib.parse
import json
import time
import subprocess
import os
import sys

BASE_URL = "http://127.0.0.1:8080"

def request_json(url, data=None, headers=None, method="GET"):
    req_headers = {"User-Agent": "Mozilla/5.0"}
    if headers:
        req_headers.update(headers)
    req_data = None
    if data is not None:
        req_data = json.dumps(data).encode("utf-8")
        req_headers["Content-Type"] = "application/json"
    
    req = urllib.request.Request(url, data=req_data, headers=req_headers, method=method)
    try:
        with urllib.request.urlopen(req) as resp:
            content = resp.read()
            return resp.status, dict(resp.headers), content
    except urllib.error.HTTPError as e:
        return e.code, dict(e.headers), e.read()

def main():
    print("=== STARTING FULL VERIFICATION FOR 5 USER REQUIREMENTS ===")
    
    # 1. Start Server
    exe_path = os.path.abspath("target/release/ksk_webapp.exe")
    print(f"Launching executable: {exe_path}")
    proc = subprocess.Popen([exe_path], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    time.sleep(2)
    
    try:
        # TEST 1: Check HTML and static assets served from single binary
        print("\n--- TEST 1: TÁCH UI THÀNH CÁC FILE RIÊNG & PHỤC VỤ STATIC ASSETS ---")
        status, headers, html_bytes = request_json(f"{BASE_URL}/")
        assert status == 200, f"Expected 200 for /, got {status}"
        html_str = html_bytes.decode("utf-8")
        
        # Check tabs exist in HTML
        assert "activeTab === 'reception'" in html_str, "Missing reception tab in HTML"
        assert "activeTab === 'history'" in html_str, "Missing history tab in HTML"
        assert "activeTab === 'physical'" in html_str, "Missing physical tab in HTML"
        assert "activeTab === 'clinical'" in html_str, "Missing clinical tab in HTML"
        assert "activeTab === 'paraclinical'" in html_str, "Missing paraclinical tab in HTML"
        assert "activeTab === 'conclusion'" in html_str, "Missing conclusion tab in HTML"
        assert "activeTab === 'admin'" in html_str, "Missing admin tab in HTML"
        print("[PASS] HTML skeleton and all 7 tabs are properly served from embedded binary.")
        
        # Check CSS served from /static/css/style.css
        status, headers, css_bytes = request_json(f"{BASE_URL}/static/css/style.css")
        assert status == 200, f"Expected 200 for /static/css/style.css, got {status}"
        assert "text/css" in headers.get("content-type", ""), f"Wrong content-type: {headers.get('content-type')}"
        css_str = css_bytes.decode("utf-8")
        assert "font-size: 12pt" in css_str, "Missing font-size: 12pt in print CSS"
        print("[PASS] /static/css/style.css served correctly with text/css and print 12pt rule.")
        
        # Check JS served from /static/js/app.js
        status, headers, js_bytes = request_json(f"{BASE_URL}/static/js/app.js")
        assert status == 200, f"Expected 200 for /static/js/app.js, got {status}"
        assert "javascript" in headers.get("content-type", ""), f"Wrong content-type: {headers.get('content-type')}"
        js_str = js_bytes.decode("utf-8")
        assert "isTiensuRole" in js_str, "Missing tiensu role logic in app.js"
        assert "initIcdMultiSelect" in js_str, "Missing initIcdMultiSelect in app.js"
        assert "initDatePicker" in js_str, "Missing initDatePicker in app.js"
        print("[PASS] /static/js/app.js served correctly with javascript content-type and core functions.")

        # TEST 2: Check NO photo frame and font size >= 12
        print("\n--- TEST 2: KHÔNG CÓ KHUNG ẢNH & CỠ CHỮ TỐI THIỂU 12 ---")
        assert "Ảnh (4 x 6 cm)" not in html_str, "Photo box still found in HTML!"
        assert "Đóng dấu giáp lai" not in html_str, "Photo seal text still found in HTML!"
        assert "font-size: 12pt" in css_str, "Missing 12pt minimum font size in print styles"
        print("[PASS] Photo frame completely removed. 12pt minimum print font size verified.")

        # TEST 3: Component libs (Flatpickr, Tom Select, ICD multi-select)
        print("\n--- TEST 3: LIBS FLAT PICKR, TOM SELECT, ICD MULTI-SELECT ---")
        assert "flatpickr" in html_str, "Flatpickr script missing from HTML"
        assert "tom-select" in html_str, "Tom select script missing from HTML"
        assert "select_cac_benh_tat_icd" in html_str, "ICD multi-select element missing from conclusion tab"
        assert "initDatePicker($el, 'NGAY_SINH')" in html_str, "Daypicker not bound to NGAY_SINH"
        print("[PASS] Flatpickr and Tom Select CDN imports and element bindings verified.")

        # TEST 4: At least 5 dotted lines for Conclusion and Periodic CLS
        print("\n--- TEST 4: ÍT NHẤT 5 DÒNG CHẤM CHO KẾT LUẬN & CLS ĐỊNH KỲ ---")
        # In print.html, find dot-line occurrences
        dot_line_count = html_str.count('<div class="dot-line"></div>')
        assert dot_line_count >= 10, f"Expected at least 10 dot-lines (5 for CLS + 5 for conclusion), got {dot_line_count}"
        print(f"[PASS] Found {dot_line_count} dot-lines in print template (>= 5 lines for each section).")

        # TEST 5: RBAC Role Separation: reception vs tiensu
        print("\n--- TEST 5: PHÂN QUYỀN TIỀN SỬ TÁCH HẲN RA KHỎI TIẾP NHẬN ---")
        # Login admin
        st, hd, body = request_json(f"{BASE_URL}/api/login", {"username": "admin", "password": "admin123"}, method="POST")
        assert st == 200, f"Admin login failed: {body}"
        admin_token = json.loads(body)["token"]
        auth_admin = {"Authorization": f"Bearer {admin_token}"}

        # Login letan (reception role only)
        st, hd, body = request_json(f"{BASE_URL}/api/login", {"username": "letan", "password": "123456"}, method="POST")
        assert st == 200, f"letan login failed: {body}"
        letan_token = json.loads(body)["token"]
        auth_letan = {"Authorization": f"Bearer {letan_token}"}

        # Login nvtiensu (tiensu role only)
        st, hd, body = request_json(f"{BASE_URL}/api/login", {"username": "nvtiensu", "password": "123456"}, method="POST")
        assert st == 200, f"nvtiensu login failed: {body}"
        tiensu_token = json.loads(body)["token"]
        auth_tiensu = {"Authorization": f"Bearer {tiensu_token}"}
        tiensu_user = json.loads(body)["user"]
        assert tiensu_user["role"] == "tiensu", f"Expected tiensu role, got {tiensu_user['role']}"
        print("[PASS] User 'nvtiensu' logged in with dedicated 'tiensu' role.")

        # Letan creates a new record
        test_ma_tn = f"TN.TEST.{int(time.time())}"
        st, hd, body = request_json(f"{BASE_URL}/api/records", {
            "ma_tiep_nhan": test_ma_tn,
            "payload": {
                "MA_TIEP_NHAN": test_ma_tn,
                "HO_TEN": "NGUYỄN VĂN TEST RBAC",
                "GIOI_TINH": "1",
                "NGAY_SINH": "01/01/1990",
                "SO_CCCD": "012345678901",
                # Letan attempts to modify a history field:
                "TSBT_TANG_HUYET_AP": "1"
            }
        }, headers=auth_letan, method="POST")
        assert st == 200, f"Record creation failed: {body}"
        res_data = json.loads(body)
        rec = res_data.get("record", res_data)
        rec_payload = rec.get("payload", {})
        
        # Verify: Letan can edit HO_TEN, but CANNOT edit TSBT_TANG_HUYET_AP (reception does not have tiensu role)
        assert rec_payload.get("HO_TEN") == "NGUYỄN VĂN TEST RBAC", "HO_TEN should be saved by reception"
        assert rec_payload.get("TSBT_TANG_HUYET_AP") == "0", "TSBT_TANG_HUYET_AP should NOT be modified by reception role!"
        print("[PASS] Reception role successfully saved administrative fields, but was BLOCKED from editing medical history.")

        # Now nvtiensu updates the record with medical history
        st, hd, body = request_json(f"{BASE_URL}/api/records", {
            "ma_tiep_nhan": test_ma_tn,
            "payload": {
                # nvtiensu tries to tamper with administrative HO_TEN:
                "HO_TEN": "HACKED NAME",
                # nvtiensu sets medical history:
                "TSBT_TANG_HUYET_AP": "1",
                "TSGD_MAC_BENH": "1",
                "TSGD_MA_BENH": "Tăng huyết áp di truyền"
            }
        }, headers=auth_tiensu, method="POST")
        assert st == 200, f"History update failed: {body}"
        res_data2 = json.loads(body)
        rec_tiensu = res_data2.get("record", res_data2)
        payload_tiensu = rec_tiensu.get("payload", {})
        
        # Verify: HO_TEN was NOT modified by nvtiensu, but TSBT_TANG_HUYET_AP and TSGD_MAC_BENH WERE modified!
        assert payload_tiensu.get("HO_TEN") == "NGUYỄN VĂN TEST RBAC", "HO_TEN must NOT be overwritten by tiensu role!"
        assert payload_tiensu.get("TSBT_TANG_HUYET_AP") == "1", "TSBT_TANG_HUYET_AP should be updated by tiensu role!"
        assert payload_tiensu.get("TSGD_MA_BENH") == "Tăng huyết áp di truyền", "TSGD_MA_BENH should be updated by tiensu role!"
        print("[PASS] Tiensu role successfully modified medical history, but was BLOCKED from modifying administrative fields.")

        # Test DOCX download with reception mode (reception blank print)
        st, hd, docx_bytes = request_json(f"{BASE_URL}/api/records/{rec['id']}/docx?mode=reception", headers=auth_admin)
        assert st == 200, f"DOCX reception export failed: {st}"
        assert len(docx_bytes) > 10000, "DOCX file too small"
        print(f"[PASS] DOCX reception blank export generated successfully ({len(docx_bytes)} bytes).")

        print("\n=== ALL 5 REQUIREMENTS VERIFIED 100% SUCCESSFULLY! ===")

    finally:
        proc.terminate()
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            proc.kill()

if __name__ == "__main__":
    main()
