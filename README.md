<div align="center">

<img src="assets/logo.png" width="116" alt="Stash logo" />

# Stash

**Save everything on your screen. Restore it all in one click.**

เก็บทุกอย่างตรงหน้า กลับมาทำต่อในคลิกเดียว

[![Version](https://img.shields.io/badge/version-0.3.0-7c3aed?style=flat-square)](https://github.com/StarchyBomb/Stash/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS-0d1117?style=flat-square)](https://github.com/StarchyBomb/Stash)
[![License](https://img.shields.io/badge/license-AGPL--3.0-30a46c?style=flat-square)](LICENSE)
[![Language](https://img.shields.io/badge/language-EN%20%7C%20TH-a78bfa?style=flat-square)](https://github.com/StarchyBomb/Stash)
[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat-square&logo=buymeacoffee&logoColor=black)](https://buymeacoffee.com/tawin)

[Browser Extension](#-browser-extension) •
[Desktop App](#%EF%B8%8F-desktop-app) •
[Use Cases](#-use-cases) •
[How to Update](#-how-to-update) •
[Support](#-support-the-project) •
[License](#-license--ownership)

</div>

---

## 🌐 Browser Extension

Stash all your browser tabs from the current window into a named session, then restore them later — right back in the same window.

เก็บแท็บเบราว์เซอร์ทั้งหมดในหน้าต่างปัจจุบันเป็นเซสชัน แล้วเปิดคืนทีหลังในหน้าต่างเดิม

### Features

| Feature | Description |
|---------|-------------|
| **One-click stash** | Save all tabs and close them instantly. Restore them all at once later. |
| **Restore in same window** | Tabs reopen in your current browser window — not a new one. |
| **Safe delete** | Two-step confirmation prevents accidental session deletion. |
| **100% Local** | All data stays in your browser's local storage. No cloud, no accounts. |
| **Bilingual** | Switch between English and Thai with a single dropdown. |

### Installation

> Works on **Chrome**, **Edge**, **Brave**, and any Chromium-based browser.

1. Open your browser's extension page:
   - Chrome: `chrome://extensions`
   - Edge: `edge://extensions`
2. Enable **Developer mode** (toggle in the top-right corner)
3. Click **Load unpacked** → select the `stash/` folder from this repository

---

## 🖥️ Desktop App

Take control of your entire desktop. Select which applications to stash, close them, and bring them all back with a single click.

ควบคุมโปรแกรมทั้งหมดในระบบเดสก์ท็อป เลือกเก็บ จัดหมวดหมู่ และเปิดกลับคืนได้อย่างแม่นยำ

### Features

| Feature | Description |
|---------|-------------|
| **Selective stash** | Checkbox system lets you pick exactly which apps to save and close. |
| **Real app icons** | Extracts and displays actual application icons from your system. |
| **Restore / Re-close** | Restore a session, then "close back" to re-stash those same apps instantly. |
| **Minimalist dark theme** | Dark gray and off-white interface — clean, modern, premium feel. |
| **Autostart** | Launch Stash automatically when your computer starts up. |
| **Bilingual** | Switch between English and Thai in the Settings panel. |

### Download

Download the latest installer from the [Releases](https://github.com/StarchyBomb/Stash/releases) page:

| Platform | File | Note |
|----------|------|------|
| Windows | `Stash_x.x.x_x64-setup.exe` | NSIS installer (recommended) |
| macOS | Coming soon | — |

> ⚠️ **First launch on Windows**: You may see a SmartScreen warning. See [SmartScreen Bypass](#%EF%B8%8F-smartscreen-warning-windows) below.

---

## 💡 Use Cases

### 1. Context Switching — สลับบริบทการทำงาน
You're deep in a coding session with VS Code, Docker, Chrome DevTools, and Slack open. An urgent meeting pops up. Hit **Stash** to freeze everything, clear your screen, and focus. When the meeting ends, hit **Restore** — everything comes back exactly as you left it.

### 2. Daily Routine Workspaces — กิจวัตรเช้าวันใหม่
Create a "Morning Check" session with your email, Slack, Notion, Spotify, and news tabs. One click every morning to open your entire workspace — no more launching apps one by one.

### 3. RAM Optimization — ประหยัดทรัพยากรแรม
Browsers and background apps consume massive amounts of RAM. Stash lets you safely close everything you don't need right now, free up memory, and restore them when you're ready.

---

## 🔄 How to Update

### Desktop App

When a new version is available on the [Releases](https://github.com/StarchyBomb/Stash/releases) page:

1. **Download** the latest `.exe` installer from the Releases page
2. **Run the installer** — it will automatically replace the old version
3. **Launch Stash** — your saved sessions and settings are preserved

> 💡 **Your data is safe.** Sessions are stored in a separate config file (`stash-config.json` in your user data directory), not inside the application folder. Updating or reinstalling the app will **not** delete your saved sessions.

เมื่อมีเวอร์ชันใหม่ในหน้า Releases:
1. ดาวน์โหลดไฟล์ `.exe` ตัวใหม่
2. รันตัวติดตั้ง — จะลงทับเวอร์ชันเดิมอัตโนมัติ
3. เปิดแอป Stash — เซสชันและการตั้งค่าเดิมทั้งหมดยังอยู่ครบ

---

### Browser Extension

When the extension code is updated on this repository:

**Option A — Re-download the ZIP** (easiest)
1. Download the latest source code ZIP from the [Releases](https://github.com/StarchyBomb/Stash/releases) page or the repository
2. Extract and replace the old `stash/` folder
3. Go to `chrome://extensions` → click the **reload** button (🔄) on the Stash extension card

**Option B — Git pull** (if you cloned the repo)
```bash
cd path/to/Stash
git pull origin main
```
Then go to `chrome://extensions` → click the **reload** button (🔄) on the Stash extension card.

> 💡 **Your data is safe.** Saved sessions are stored in Chrome's local storage, not in the extension files. Reloading or updating the extension will **not** delete your saved sessions.

เมื่อโค้ดของ Extension ได้รับการอัพเดทในรีโพ:
- **วิธี A**: ดาวน์โหลด ZIP ใหม่ → แตกไฟล์ทับ `stash/` เดิม → ไปที่ `chrome://extensions` กดปุ่มรีโหลด 🔄
- **วิธี B**: ถ้า clone repo ไว้ ให้ `git pull` แล้วกดปุ่มรีโหลดที่หน้า extensions

---

## 🛡️ SmartScreen Warning (Windows)

Since the installer is not code-signed, Windows SmartScreen may show a security warning on first launch. **The app is completely safe** — all source code is open and available in this repository.

To bypass the warning:

1. Double-click the `.exe` installer
2. When the blue SmartScreen window appears, click **"More info"**
3. Click **"Run anyway"** to proceed with the installation

เนื่องจากตัวติดตั้งยังไม่ได้ลงนามดิจิทัล Windows SmartScreen อาจขึ้นแจ้งเตือน:
1. คลิก **"ข้อมูลเพิ่มเติม (More Info)"**
2. คลิก **"รันต่อไป (Run anyway)"**

---

## 🛠️ Development

To build and develop Stash locally:

```bash
# Clone the repository
git clone https://github.com/StarchyBomb/Stash.git
cd Stash

# Desktop App
cd stash-desktop
npm install
npm run dev          # Start development mode
npm run build        # Build production installer (.exe)

# Browser Extension
# Load the stash/ folder as an unpacked extension in your browser
```

### Tech Stack

| Component | Technology |
|-----------|-----------|
| Desktop App | [Tauri v2](https://tauri.app/) + Rust + HTML/JS |
| Browser Extension | Chrome Manifest V3 + Vanilla JS |
| Landing Page | HTML + CSS (hosted on Vercel) |

---

## ☕ Support the Project

Stash is **free and open source**, built and maintained by one developer in their spare time. If it saves you time or makes your day a little smoother, please consider buying me a coffee — it directly funds new features and ongoing maintenance.

<div align="center">

[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-Support%20Stash-FFDD00?style=for-the-badge&logo=buymeacoffee&logoColor=black)](https://buymeacoffee.com/tawin)

**→ [buymeacoffee.com/tawin](https://buymeacoffee.com/tawin)**

</div>

Every contribution, no matter how small, is genuinely appreciated. 🙏

---

## 📜 License & Ownership

**Copyright © 2026 Tawin. All rights reserved.**

Stash is the original work of **Tawin** ([@StarchyBomb](https://github.com/StarchyBomb)) and is licensed under the **[GNU Affero General Public License v3.0 (AGPL-3.0)](LICENSE)**.

What this means:

- ✅ You are free to **use, study, modify, and share** Stash.
- ✅ You **must** keep this copyright notice and give appropriate credit to the original author.
- ✅ Any modified or derivative version — including software run over a network — **must remain open source under AGPL-3.0** and disclose its source.
- ❌ You **may not** relicense it, remove attribution, or claim this project (or a derivative) as your own original work.

The authorship and creation history of this project are permanently recorded in this repository's Git commit history under the account [@StarchyBomb](https://github.com/StarchyBomb). This public, timestamped record establishes original ownership.

For commercial licensing or other arrangements outside the terms of AGPL-3.0, please contact the author.

---

<div align="center">

**Made with ❤️ by [Tawin (@StarchyBomb)](https://github.com/StarchyBomb)**

If you find Stash useful, [buy me a coffee ☕](https://buymeacoffee.com/tawin)

</div>
