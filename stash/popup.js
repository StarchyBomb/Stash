const $ = (id) => document.getElementById(id);

// เก็บเฉพาะแท็บที่เปิดคืนได้จริง (ไม่เอา chrome://, edge://, หน้า extension)
const STASHABLE = /^(https?|file):/;

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
  const date = d.toLocaleDateString("th-TH", { day: "numeric", month: "short" });
  const time = d.toLocaleTimeString("th-TH", { hour: "2-digit", minute: "2-digit" });
  return `เก็บเมื่อ ${date} ${time}`;
}

function timeAgo(ts) {
  const mins = Math.floor((Date.now() - ts) / 60000);
  if (mins < 1) return "เมื่อครู่";
  if (mins < 60) return `${mins} นาทีที่แล้ว`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs} ชม.ที่แล้ว`;
  return `${Math.floor(hrs / 24)} วันที่แล้ว`;
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
  const tabs = await getStashableTabs();
  $("tab-count").textContent = `หน้าต่างนี้มี ${tabs.length} แท็บที่เก็บได้`;
  $("stash-btn").disabled = tabs.length === 0;

  const sessions = await loadSessions();
  const box = $("sessions");
  box.innerHTML = "";

  if (sessions.length === 0) {
    box.innerHTML = '<div class="empty">ยังไม่มีเซสชันที่เก็บไว้</div>';
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
    meta.textContent = `${s.tabs.length} แท็บ · ${timeAgo(s.createdAt)}`;
    info.append(title, meta);

    const openBtn = document.createElement("button");
    openBtn.textContent = "เปิดคืน";
    openBtn.addEventListener("click", () => restore(s.id));

    const delBtn = document.createElement("button");
    delBtn.className = "del";
    delBtn.textContent = "✕";
    delBtn.addEventListener("click", () => {
      if (delBtn.classList.contains("confirm")) {
        remove(s.id);
      } else {
        delBtn.classList.add("confirm");
        delBtn.textContent = "ลบ?";
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

$("stash-btn").addEventListener("click", stash);
$("name").addEventListener("keydown", (e) => {
  if (e.key === "Enter") stash();
});

render();
