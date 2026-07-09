const $ = (id) => document.getElementById(id);

// เก็บเฉพาะแท็บที่เปิดคืนได้จริง (ไม่เอา chrome://, edge://, หน้า extension)
const STASHABLE = /^(https?|file):/;

let currentLang = "en";
let activeWindows = [];

const TRANSLATIONS = {
  th: {
    placeholder: "ตั้งชื่อเซสชัน (ไม่บังคับ)",
    stashBtn: "📥 เก็บแท็บทั้งหมด แล้วปิด",
    stashBtnEmpty: "📥 ไม่มีแท็บที่เก็บได้",
    tabCount: (n) => `เบราว์เซอร์มี ${n} แท็บที่เก็บได้`,
    savedSessions: "เซสชันที่เก็บไว้",
    emptySessions: "ยังไม่มีเซสชันที่เก็บไว้",
    timeJustNow: "เมื่อครู่",
    timeMinAgo: (m) => `${m} นาทีที่แล้ว`,
    timeHourAgo: (h) => `${h} ชม.ที่แล้ว`,
    timeDayAgo: (d) => `${d} วันที่แล้ว`,
    defaultName: (d, t) => `เก็บเมื่อ ${d} ${t}`,
    restoreBtn: "เปิดคืน",
    delConfirm: "ลบ?",
    tabsUnit: "แท็บ",
    support: "☕ เลี้ยงกาแฟผู้พัฒนา",
    tabsHeaderLabel: "แท็บที่จะเก็บ",
    toggleAllSelect: "เลือกทั้งหมด",
    toggleAllNone: "ไม่เลือกเลย",
    stashBtnActive: (n) => `📥 เก็บ ${n} แท็บที่เลือก แล้วปิด`,
    stashBtnEmptySelection: "📥 เลือกแท็บที่จะเก็บ"
  },
  en: {
    placeholder: "Session name (optional)",
    stashBtn: "📥 Stash all tabs and close",
    stashBtnEmpty: "📥 No stashable tabs",
    tabCount: (n) => `This browser has ${n} stashable tabs`,
    savedSessions: "Saved Sessions",
    emptySessions: "No saved sessions",
    timeJustNow: "just now",
    timeMinAgo: (m) => `${m}m ago`,
    timeHourAgo: (h) => `${h}h ago`,
    timeDayAgo: (d) => `${d}d ago`,
    defaultName: (d, t) => `Stashed on ${d} ${t}`,
    restoreBtn: "Restore",
    delConfirm: "Delete?",
    tabsUnit: "tabs",
    support: "☕ Buy me a coffee",
    tabsHeaderLabel: "Tabs to stash",
    toggleAllSelect: "Select all",
    toggleAllNone: "Deselect all",
    stashBtnActive: (n) => `📥 Stash ${n} selected tabs and close`,
    stashBtnEmptySelection: "📥 Select tabs to stash"
  }
};

function getSessionTabCount(s) {
  if (s.windows) {
    return s.windows.reduce((sum, w) => sum + w.tabs.length, 0);
  }
  return (s.tabs || []).length;
}

async function getStashableTabs() {
  const tabs = await chrome.tabs.query({});
  return tabs.filter((t) => t.url && STASHABLE.test(t.url));
}

async function loadActiveTabs() {
  try {
    const windows = await chrome.windows.getAll({ populate: true, windowTypes: ['normal'] });
    activeWindows = windows.map(w => ({
      id: w.id,
      left: w.left,
      top: w.top,
      width: w.width,
      height: w.height,
      state: w.state,
      tabs: (w.tabs || []).filter(t => t.url && STASHABLE.test(t.url)).map(t => ({
        id: t.id,
        url: t.url,
        title: t.title || t.url,
        favIconUrl: t.favIconUrl,
        selected: true
      }))
    })).filter(w => w.tabs.length > 0);
  } catch (e) {
    console.error(e);
  }
  renderTabsList();
}

function renderTabsList() {
  const listEl = $("tabs-list");
  listEl.innerHTML = "";
  
  let totalStashable = 0;
  let totalSelected = 0;
  const t = TRANSLATIONS[currentLang];
  
  activeWindows.forEach((win, winIdx) => {
    if (activeWindows.length > 1) {
      const winHeader = document.createElement("div");
      winHeader.style.cssText = "font-size: 10px; font-weight: 700; color: var(--muted); padding: 6px 6px 2px; margin-top: 4px; border-bottom: 1px solid var(--border);";
      const isActive = winIdx === 0;
      winHeader.textContent = `${t.tabsUnit === 'แท็บ' ? 'หน้าต่างที่' : 'WINDOW'} ${winIdx + 1}${isActive ? ' (Active)' : ''}`;
      listEl.appendChild(winHeader);
    }
    
    win.tabs.forEach((tab) => {
      totalStashable++;
      if (tab.selected) totalSelected++;
      
      const tabEl = document.createElement("div");
      tabEl.className = "tab-item";
      
      const chk = document.createElement("input");
      chk.type = "checkbox";
      chk.checked = tab.selected;
      chk.addEventListener("change", (e) => {
        tab.selected = e.target.checked;
        updateStashButtonState();
      });
      
      const img = document.createElement("img");
      img.className = "tab-favicon";
      img.src = tab.favIconUrl || "icons/icon16.png";
      img.onerror = () => { img.src = "icons/icon16.png"; };
      
      const info = document.createElement("div");
      info.className = "tab-info";
      
      const title = document.createElement("span");
      title.className = "tab-title";
      title.textContent = tab.title;
      
      const url = document.createElement("span");
      url.className = "tab-url";
      url.textContent = tab.url;
      
      info.append(title, url);
      
      tabEl.addEventListener("click", (e) => {
        if (e.target !== chk) {
          chk.checked = !chk.checked;
          tab.selected = chk.checked;
          updateStashButtonState();
        }
      });
      
      tabEl.append(chk, img, info);
      listEl.appendChild(tabEl);
    });
  });
  
  if (totalStashable === 0) {
    listEl.innerHTML = `<div class="empty">${t.emptySessions}</div>`;
  }
  
  updateToggleAllButton(totalSelected, totalStashable);
  updateStashButtonState();
}

function updateToggleAllButton(totalSelected, totalStashable) {
  const toggleBtn = $("toggle-all-btn");
  const t = TRANSLATIONS[currentLang];
  if (totalSelected === 0) {
    toggleBtn.textContent = t.toggleAllSelect;
  } else {
    toggleBtn.textContent = t.toggleAllNone;
  }
}

function updateStashButtonState() {
  const t = TRANSLATIONS[currentLang];
  let totalSelected = 0;
  activeWindows.forEach(w => w.tabs.forEach(tab => {
    if (tab.selected) totalSelected++;
  }));
  
  const stashBtn = $("stash-btn");
  if (totalSelected === 0) {
    stashBtn.textContent = t.stashBtnEmptySelection;
    stashBtn.disabled = true;
  } else {
    stashBtn.textContent = t.stashBtnActive(totalSelected);
    stashBtn.disabled = false;
  }
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
  const stashWindows = [];
  const tabIdsToRemove = [];
  
  activeWindows.forEach(w => {
    const selectedTabs = w.tabs.filter(t => t.selected);
    if (selectedTabs.length > 0) {
      stashWindows.push({
        left: w.left,
        top: w.top,
        width: w.width,
        height: w.height,
        state: w.state,
        tabs: selectedTabs.map(t => ({ url: t.url, title: t.title }))
      });
      selectedTabs.forEach(t => tabIdsToRemove.push(t.id));
    }
  });
  
  if (tabIdsToRemove.length === 0) return;

  const sessions = await loadSessions();
  sessions.unshift({
    id: Date.now().toString(36) + Math.random().toString(36).slice(2, 6),
    name: $("name").value.trim() || defaultName(),
    createdAt: Date.now(),
    windows: stashWindows,
  });
  await saveSessions(sessions);

  // เปิดแท็บใหม่กันหน้าต่างปิดตัวเอง แล้วค่อยปิดแท็บที่เก็บแล้ว
  await chrome.tabs.create({});
  await chrome.tabs.remove(tabIdsToRemove);

  $("name").value = "";
  await loadActiveTabs();
  render();
}

async function restore(id) {
  const sessions = await loadSessions();
  const s = sessions.find((x) => x.id === id);
  if (!s) return;

  if (s.windows && s.windows.length > 0) {
    for (const w of s.windows) {
      if (w.tabs.length === 0) continue;
      
      const firstTab = w.tabs[0];
      const winOptions = {
        url: firstTab.url,
        left: w.left,
        top: w.top,
        width: w.width,
        height: w.height,
        state: w.state === 'maximized' ? 'maximized' : 'normal'
      };
      
      // Remove positions if they are negative (e.g. minimized)
      if (winOptions.left < 0) delete winOptions.left;
      if (winOptions.top < 0) delete winOptions.top;

      const newWin = await chrome.windows.create(winOptions);
      
      for (let i = 1; i < w.tabs.length; i++) {
        await chrome.tabs.create({ windowId: newWin.id, url: w.tabs[i].url });
      }
    }
  } else if (s.tabs && s.tabs.length > 0) {
    // Fallback for old schema
    for (const t of s.tabs) {
      await chrome.tabs.create({ url: t.url });
    }
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
  $("support-text").textContent = t.support;
  $("tabs-header-label").textContent = t.tabsHeaderLabel;

  const tabs = await getStashableTabs();
  $("tab-count").textContent = t.tabCount(tabs.length);
  
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
    const count = getSessionTabCount(s);
    meta.textContent = `${count} ${t.tabsUnit} · ${timeAgo(s.createdAt)}`;
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
  currentLang = res.lang || "en";
  $("lang-select").value = currentLang;

  $("lang-select").addEventListener("change", async (e) => {
    currentLang = e.target.value;
    await chrome.storage.local.set({ lang: currentLang });
    render();
    renderTabsList();
  });
}

$("stash-btn").addEventListener("click", stash);
$("name").addEventListener("keydown", (e) => {
  if (e.key === "Enter") stash();
});

$("toggle-all-btn").addEventListener("click", () => {
  let anySelected = activeWindows.some(w => w.tabs.some(t => t.selected));
  activeWindows.forEach(w => w.tabs.forEach(t => {
    t.selected = !anySelected;
  }));
  renderTabsList();
  updateStashButtonState();
});

(async () => {
  await initLang();
  await loadActiveTabs();
  render();
})();
