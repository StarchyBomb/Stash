# Stash → AI Buddy — แผนพัฒนา (ร่าง)

> เอกสารภายในสำหรับวางแผนทิศทางของ Stash จาก "ตัวเซฟเซสชัน" ไปสู่ "AI work buddy บนเดสก์ท็อป"
> เจ้าของ: **Tawin** ([@StarchyBomb](https://github.com/StarchyBomb)) · repo: `StarchyBomb/Stash` · License: AGPL-3.0
> อัปเดต: 2026-07-08

---

## 1. วิสัยทัศน์ (Vision)

เปลี่ยน Stash จาก **"ตัวเซฟแท็บ/แอป"** → **"Jarvis ประจำเดสก์ท็อปที่ทำงานบนเครื่องคุณเอง"**:
ผู้ช่วย AI ที่อยู่เบื้องหลัง รู้ว่าคุณกำลังทำอะไร ช่วยจัดการ context การทำงาน เรียกคืน workspace ด้วยเสียง/คำสั่ง และช่วยงานประจำวันแบบ on-demand — โดย **จับสัญญาณบนเครื่อง ประมวลผลบนเครื่องเป็นหลัก** ไม่สอดส่อง

**หนึ่งประโยค:** *"เสียงพาเข้า (Jarvis) — ความจำรั้งไว้ (Work Memory)"*

---

## 2. หลักการออกแบบ / เส้นแดง (Guardrails)

ทุกฟีเจอร์ต้องผ่าน 2 ตัวกรองนี้:

1. **การรับรู้ต้องเป็น on-device metadata + on-demand capture — ห้าม scan พิกเซลจอตลอดเวลา**
   ใช้ชื่อหน้าต่าง/title/URL แท็บ (ต้นทุน ~0, เป็นส่วนตัว) เป็นสัญญาณหลัก
   เก็บการจับภาพจริง (screenshot/vision) ไว้เฉพาะตอนผู้ใช้สั่ง ("ชี้แล้วถาม") เท่านั้น
   → กัน creepy / เปลืองแบต / ต้นทุน และรักษาแบรนด์ local-first

2. **อย่าให้ generic AI feature มากลืน identity**
   แชตทั่วไป/สร้างข้อสอบ/สอนพิมพ์ = ทำได้ง่ายแต่ไม่มี moat (ชนกับ ChatGPT ตรงๆ) → เป็นของแถม ไม่ใช่พระเอก
   moat ที่แท้จริง = **context/focus/actions + สะพาน desktop↔extension**

3. **Privacy-first, local-first, action-oriented** — จุดต่างจาก Rewind.ai / Limitless (ที่อัดทุกอย่าง)

---

## 3. สถาปัตยกรรม 3 ชั้น

| ชั้น | ทำอะไร | รันที่ไหน |
|---|---|---|
| **On-device (เสมอ)** | wake word, STT, จับ active window/tab + เวลา, fuzzy match trigger phrase, ปุ่ม action | 🔒 เครื่องผู้ใช้ |
| **On-demand (ตอนสั่ง)** | crop screenshot → Claude vision, research, สรุป/สร้างเอกสาร | ☁️ Claude API (ส่งเฉพาะสิ่งที่สั่ง) |
| **Cloud (จ่ายเงิน)** | sync ข้ามเครื่อง, semantic search, resume brief | ☁️ backend + Claude |

### สะพาน desktop ↔ extension (infra แกนกลาง — ทำก่อน)
- Stash มีทั้ง **desktop app** (เห็นแอป+เวลาโฟกัส) และ **extension** (เห็นแท็บ active) → รวมกันได้ context ระดับแอป *และ* แท็บ ที่ Rewind (เห็นแค่จอ) และ Arc (เห็นแค่เบราว์เซอร์) ทำไม่ได้
- เชื่อมด้วย **native messaging** หรือ **local websocket** (localhost)
- ปลดล็อก: tab-time tracking, focus-nudge, Jarvis ที่ฉลาดขึ้น

---

## 4. คัดกรองฟีเจอร์ (Feature Triage)

✅ = ทำได้/คุ้ม · ⚠️ = ทำได้แต่มีเงื่อนไข/นอกแกน · ❌ = อย่าทำ

| ฟีเจอร์ | สถานะ | หมายเหตุ |
|---|---|---|
| Jarvis สั่งงานด้วยเสียง (wake word → เปิด session) | ✅ | ต่อกับ `restore_session_cmd` ที่มีอยู่ — ทำ 80% เสร็จแล้ว |
| Buddy "Jibi" บน sidebar คุยสั้นๆ | ✅ | หน้าต่าง Tauri โปร่งใส always-on-top + Lottie/Live2D/sprite · **ต้องพูดน้อย ปิดง่าย** (กัน Clippy) |
| รู้ว่าใช้แอปไหนนานสุด | ✅ | `GetForegroundWindow` + จับเวลา (มีโครง EnumWindows แล้ว) |
| รู้ว่าเปิดแท็บไหนนานสุด | ⚠️ | ต้องมีสะพาน desktop↔extension ก่อน |
| Focus nudge (สลับแอปบ่อย → เตือน) | ✅ | ต้องให้ผู้ใช้ตั้ง "งานที่โฟกัส" + snooze ง่าย |
| `/savesession` เซฟแต่ไม่ปิดแอป | ✅ | เล็ก ทำก่อน — แยก save ออกจาก close (ตอนนี้รวมกัน) |
| แชต/command panel (พิมพ์ + slash) | ✅ | agent/tool-use pattern — คือ "สมอง" UI |
| Co-working: กดพูด → วง/ลากกรอบบนจอ → หาข้อมูล → เอกสาร preview | ✅ | overlay ลากกรอบ → crop → **Claude vision** → เสิร์ช/สรุป → render ในแอป · จับภาพเฉพาะตรงนี้ (on-demand) |
| Resume Brief ("เมื่อวานทำอะไรค้าง?") | ✅ | Claude สรุปจาก session ตอน restore — ตัว retention/monetization |
| แชตทั่วไป / สอนพิมพ์ | ⚠️ | ทำง่ายแต่ไม่มี moat → ของแถม |
| สร้างข้อสอบ interactive ติ๊กได้ + สรุปผล | ⚠️ | artifact pattern ทำได้ แต่ scope creep → เลื่อนไปท้าย |
| scan จอตลอดเวลาด้วย vision | ❌ | แพง/เปลืองแบต/creepy — ใช้ metadata แทน |

---

## 5. Roadmap เป็นเฟส

- **เฟส 0 — Quick wins:** แชต/command panel + `/savesession` (ไม่ปิดแอป) → ได้ "สมอง" UI ทันที
- **เฟส 1 — Infra:** สะพาน desktop↔extension + app/tab time tracking (ในเครื่องล้วน)
- **เฟส 2 — Buddy & Focus:** Jibi บน sidebar (พูดน้อย) + focus nudge
- **เฟส 3 — Jarvis เสียง:** wake word + STT ในเครื่อง + fuzzy match trigger phrase → action
- **เฟส 4 — Resume Brief + Cloud:** สรุป context ตอน restore + cloud sync + semantic search (ตัวเก็บเงิน)
- **เฟส 5 — Co-working "ชี้แล้วถาม":** overlay + crop + Claude vision + เอกสาร preview ← ฟีเจอร์ไวรัล
- **เฟส 6 — ของแถม:** quiz maker, agentic resume ฯลฯ

---

## 6. Tech Stack

### AI (Claude API — ดูรายละเอียดต่อใน 8)
| งาน | โมเดลแนะนำ | เหตุผล |
|---|---|---|
| Reasoning หลัก / agent loop / Resume Brief | **Opus 4.8** (`claude-opus-4-8`) — $5/$25 ต่อ 1M | ฉลาดสุดในสาย Opus, agentic/long-horizon |
| "ชี้แล้วถาม" (อ่าน crop รูป) | **Opus 4.8** (multimodal, high-res vision) | รองรับรูปละเอียดถึง 2576px |
| งานถี่/ถูก: auto-name session, จัด intent, buddy คุยเล่น | **Haiku 4.5** (`claude-haiku-4-5`) — $1/$5 | เร็ว ถูก พอสำหรับงานสั้น |
| งาน volume สูง สมดุล | **Sonnet 5** (`claude-sonnet-5`) — $3/$15 | ใกล้ Opus ราคาถูกกว่า |
| ต้องการ latency ต่ำ (Jarvis ตอบไว) | Opus 4.8 **fast mode** (`speed:"fast"`) | เร็วขึ้น ~2.5x |

### เสียง (on-device)
- **Wake word:** Porcupine (Picovoice) / openWakeWord
- **STT:** Whisper.cpp / Vosk (รองรับไทย-อังกฤษปน — ต้องเทสเร็ว จุดเสี่ยงสุด)

### แอป
- Desktop: **Tauri v2 + Rust** (มีอยู่) · overlay/crop, tray, autostart
- Extension: **Chrome MV3 + Vanilla JS** (มีอยู่)
- สะพาน: native messaging / local websocket
- Buddy render: Lottie / Live2D / sprite (transparent always-on-top window)

---

## 7. โมเดลธุรกิจ

- **Free (local, open-source/AGPL-3.0)** — ตัวปัจจุบัน = adoption + ความน่าเชื่อถือ
- **Pro (~$8/เดือน)** — cloud sync, Resume Brief, semantic memory, Jarvis เต็มรูปแบบ
- **Team** — ส่งต่อ context กันในทีม ("รับงานต่อจากเพื่อน เห็นเลยว่าเขาค้างตรงไหน")

**Wedge:** ตัวฟรี (utility) ดึงคนเข้า → AI/cloud เป็นตัวเก็บเงิน · จุดขาย privacy = "AI memory ที่ไม่สอดส่อง (on-device)"

---

## 8. หมายเหตุการต่อ Claude API (สำหรับตอน implement)

- Default model: **`claude-opus-4-8`** · thinking แบบ **adaptive** เท่านั้น (`thinking:{type:"adaptive"}`) — `budget_tokens`/`temperature`/`top_p` โดน 400
- คุมความลึก/ต้นทุนด้วย `output_config.effort` (`low`→`max`)
- คำสั่งธรรมชาติ (แชต/Jarvis) = **tool use / agent loop** → map เจตนาเป็น action (restore/save/close ฯลฯ)
- "ชี้แล้วถาม" = ส่ง crop เป็น image block (base64) เข้า messages
- เอกสาร interactive (quiz/รายงาน) = ให้ Claude gen HTML แล้ว render ในแอป (artifact pattern)
- **Hybrid ประหยัด:** trigger phrase ที่ตั้งไว้ → จับคู่ในเครื่อง (ฟรี/ทันที); เรียก Claude เฉพาะตอนกำกวม/เป็นธรรมชาติ
- ต่อ API ผ่าน official Anthropic SDK ของภาษาที่ใช้ (ถ้าเป็น backend TS/Python) — ดู skill `/claude-api` ตอนลงมือ

---

## 9. ความเสี่ยง + วิธีรับมือ

| ความเสี่ยง | รับมือ |
|---|---|
| Always-listening = privacy/แบต | on-device wake word เท่านั้น + ไฟไมค์ชัด + ประมวลผลในเครื่อง |
| เสียงในโลกจริงพัง (ไทยปนอังกฤษ, noise) | เทส STT ไทยเร็วมากตั้งแต่ต้น + fast-path ในเครื่อง |
| Latency > การคลิกเอง | fast-path ในเครื่อง + Opus fast mode |
| Voice เป็น party trick ไม่ใช้จริง | มี hotkey + dashboard + เสียง ยิง action เดียวกัน — เสียงคือ "ว้าว" ไม่ใช่ทางเดียว |
| Buddy กวน (Clippy) | เริ่มเงียบ พูดน้อย ปิดง่าย |
| Scope creep (generic AI) | ยึดตัวกรอง #2 — ของแถมไว้ท้ายสุด |

---

## 10. ก้าวถัดไป

1. เริ่ม **เฟส 0**: prototype แชต/command panel + `/savesession` (ไม่ปิดแอป) ต่อกับ desktop app ที่มีอยู่
2. วางสเปก **สะพาน desktop↔extension** (เฟส 1)
3. เทส **STT ไทย-อังกฤษ** ให้เร็ว (ตัดสินความเป็นไปได้ของ Jarvis)
