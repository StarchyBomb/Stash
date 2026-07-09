# 🚿 Rinse

**วางแบบสะอาดในปุ่มเดียว** — กด `Ctrl+Shift+V` แล้วข้อความในคลิปบอร์ดจะถูกล้างให้สะอาดก่อนวางทันที ทุกแอป

## ฟีเจอร์เดียวที่มี

กด `Ctrl+Shift+V` ที่ไหนก็ได้ → ข้อความถูกล้างแล้ววางเลย:

- **ซ่อมบรรทัดพัง** — รวมบรรทัดที่โดน hard-wrap จาก PDF/อีเมล กลับเป็นย่อหน้า (รองรับไทย ไม่แทรกช่องว่าง)
- **เครื่องหมายมาตรฐาน** — smart quotes → `" "` · em-dash → `-` · `…` → `...`
- **ล้างลิงก์สะกดรอย** — ตัด `utm_*`, `fbclid`, `gclid`, `igshid`, `si` ฯลฯ ออกจาก URL
- **เก็บกวาดช่องว่าง** — ช่องว่างซ้ำ, บรรทัดว่างเกิน, zero-width chars, NBSP

ปิดหน้าต่างแล้วยังทำงานต่อใน system tray · ตั้งเปิดพร้อม Windows ได้ · ข้อมูลอยู่ในเครื่องล้วน ไม่มี cloud

## Dev

```powershell
npm install
npm run dev        # รันโหมดพัฒนา
npm run build      # สร้าง .exe + installer (src-tauri/target/release/)
```

ทดสอบ logic การล้าง: `cd src-tauri; cargo test`

## Stack

Tauri v2 · Rust (arboard + enigo + regex) · static HTML — ไม่มี framework, ไม่มี bundler, ไม่มี server
