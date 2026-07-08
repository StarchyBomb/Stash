const $ = (id) => document.getElementById(id);

// เก็บเฉพาะแท็บที่เปิดคืนได้จริง (ไม่เอา chrome://, edge://, หน้า extension)
const STASHABLE = /^(https?|file):/;

let currentLang = "th";

const TRANSLATIONS = {
  th: {
    placeholder: "ตั้งชื่อเซสชัน (ไม่บังคับ)",
    stashBtn: "📥 เก็บแท็บทั้งหมด แล้วปิด",
    stashBtnEmpty: "📥 ไม่มีแท็บที่เก็บได้",
    tabCount: (n) => `หน้าต่างนี้มี ${n} แท็บที่เก็บได้`,
    savedSessions: "เซสชันที่เก็บไว้",
    emptySessions: "ยังไม่มีเซสชันที่เก็บไว้",
    timeJustNow: "เมื่อครู่",
    timeMinAgo: (m) => `${m} นาทีที่แล้ว`,
    timeHourAgo: (h) => `${h} ชม.ที่แล้ว`,
    timeDayAgo: (d) => `${d} วันที่แล้ว`,
    defaultName: (d, t) => `เก็บเมื่อ ${d} ${t}`,
    restoreBtn: "เปิดคืน",
    delConfirm: "ลบ?",
    tabsUnit: "แท็บ"
  },
  en: {
    placeholder: "Session name (optional)",
    stashBtn: "📥 Stash all tabs and close",
    stashBtnEmpty: "📥 No stashable tabs",
    tabCount: (n) => `This window has ${n} stashable tabs`,
    savedSessions: "Saved Sessions",
    emptySessions: "No saved sessions",
    timeJustNow: "just now",
    timeMinAgo: (m) => `${m}m ago`,
    timeHourAgo: (h) => `${h}h ago`,
    timeDayAgo: (d) => `${d}d ago`,
    defaultName: (d, t) => `Stashed on ${d} ${t}`,
    restoreBtn: "Restore",
    delConfirm: "Delete?",
    tabsUnit: "tabs"
  }
};

async function getStashableTabs() {
  const tabs = await chrome.tabs.query({ currentWindow: true });
  return tabs.filter((t) => t.url && STASHABLE.test(t.url));
}

async function loadSessions() {
  const { sessions = [] } = await chrome.storage.local.get("sessions");
  return sessions;
}

async function saveSessions(sessions) {
  await chrome.storage.local.set({ sessions });
}

function defaultName() {
  const d = new Date();
  if (currentLang === "th") {
    const date = d.toLocaleDateString("th-TH", { day: "numeric", month: "short" });
    const time = d.toLocaleTimeString("th-TH", { hour: "2-digit", minute: "2-digit" });
    return TRANSLATIONS.th.defaultName(date, time);
  } else {
    const date = d.toLocaleDateString("en-US", { day: "numeric", month: "short" });
    const time = d.toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit", hour12: false });
    return TRANSLATIONS.en.defaultName(date, time);
  }
}

function timeAgo(ts) {
  const t = TRANSLATIONS[currentLang];
  const mins = Math.floor((Date.now() - ts) / 60000);
  if (mins < 1) return t.timeJustNow;
  if (mins < 60) return t.timeMinAgo(mins);
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return t.timeHourAgo(hrs);
  return t.timeDayAgo(Math.floor(hrs / 24));
}

async function stash() {
  const tabs = await getStashableTabs();
  if (tabs.length === 0) return;

  const sessions = await loadSessions();
  sessions.unshift({
    id: Date.now().toString(36) + Math.random().toString(36).slice(2, 6),
    name: $("name").value.trim() || defaultName(),
    createdAt: Date.now(),
    tabs: tabs.map((t) => ({ url: t.url, title: t.title || t.url })),
  });
  await saveSessions(sessions);

  // เปิดแท็บใหม่กันหน้าต่างปิดตัวเอง แล้วค่อยปิดแท็บที่เก็บแล้ว
  await chrome.tabs.create({});
  await chrome.tabs.remove(tabs.map((t) => t.id));

  $("name").value = "";
  render();
}

async function restore(id) {
  const sessions = await loadSessions();
  const s = sessions.find((x) => x.id === id);
  if (!s) return;
  for (const t of s.tabs) {
    await chrome.tabs.create({ url: t.url });
  }
}

async function remove(id) {
  const sessions = (await loadSessions()).filter((x) => x.id !== id);
  await saveSessions(sessions);
  render();
}

async function render() {
  const t = TRANSLATIONS[currentLang];

  // Update static UI texts
  $("name").placeholder = t.placeholder;
  document.querySelector("h2").textContent = t.savedSessions;

  const tabs = await getStashableTabs();
  $("tab-count").textContent = t.tabCount(tabs.length);
  
  if (tabs.length === 0) {
    $("stash-btn").textContent = t.stashBtnEmpty;
    $("stash-btn").disabled = true;
  } else {
    $("stash-btn").textContent = t.stashBtn;
    $("stash-btn").disabled = false;
  }

  const sessions = await loadSessions();
  const box = $("sessions");
  box.innerHTML = "";

  if (sessions.length === 0) {
    box.innerHTML = `<div class="empty">${t.emptySessions}</div>`;
    return;
  }

  for (const s of sessions) {
    const el = document.createElement("div");
    el.className = "session";

    const info = document.createElement("div");
    info.className = "info";
    const title = document.createElement("b");
    title.textContent = s.name;
    const meta = document.createElement("span");
    meta.textContent = `${s.tabs.length} ${t.tabsUnit} · ${timeAgo(s.createdAt)}`;
    info.append(title, meta);

    const openBtn = document.createElement("button");
    openBtn.textContent = t.restoreBtn;
    openBtn.addEventListener("click", () => restore(s.id));

    const delBtn = document.createElement("button");
    delBtn.className = "del";
    delBtn.textContent = "✕";
    delBtn.addEventListener("click", () => {
      if (delBtn.classList.contains("confirm")) {
        remove(s.id);
      } else {
        delBtn.classList.add("confirm");
        delBtn.textContent = t.delConfirm;
        setTimeout(() => {
          delBtn.classList.remove("confirm");
          delBtn.textContent = "✕";
        }, 2000);
      }
    });

    el.append(info, openBtn, delBtn);
    box.appendChild(el);
  }
}

// Language handler
async function initLang() {
  const res = await chrome.storage.local.get("lang");
  currentLang = res.lang || "th";
  $("lang-select").value = currentLang;

  $("lang-select").addEventListener("change", async (e) => {
    currentLang = e.target.value;
    await chrome.storage.local.set({ lang: currentLang });
    render();
  });
}

$("stash-btn").addEventListener("click", stash);
$("name").addEventListener("keydown", (e) => {
  if (e.key === "Enter") stash();
});

(async () => {
  await initLang();
  render();
})();
