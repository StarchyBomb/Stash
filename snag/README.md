# ⚡ Snag

**เห็นอะไรน่าเก็บ กดปุ่มเดียว เข้ากล่องเรา** — ไฮไลต์ข้อความ/รูปที่ไหนก็ได้ กด `Ctrl+Alt+S` แล้วมันเด้งเข้ากล่อง inbox ส่วนตัวทันที ไม่ต้องสลับแอป ไม่เสียโฟกัส

## ฟีเจอร์เดียวที่มี

- `Ctrl+Alt+S` — เก็บสิ่งที่ไฮไลต์อยู่ (ข้อความหรือรูป) เข้ากล่อง พร้อมชื่อแอปต้นทาง + เวลา
- `Ctrl+Alt+O` — เปิด/ปิดกล่อง
- ในกล่อง: ค้นหา · คัดลอกกลับ · เปิดรูปเต็ม · ลบ

ปิดหน้าต่างแล้วยังทำงานต่อใน system tray · ข้อมูลอยู่ใน SQLite ในเครื่องล้วน ไม่มี cloud

## Dev

```powershell
npm install
npm run dev        # รันโหมดพัฒนา
npm run build      # สร้าง .exe + installer (src-tauri/target/release/)
```

## Stack

Tauri v2 · Rust (arboard + enigo + rusqlite + image) · static HTML — ไม่มี framework, ไม่มี bundler, ไม่มี server
