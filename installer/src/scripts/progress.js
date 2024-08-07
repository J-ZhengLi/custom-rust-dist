const { invoke } = window.__TAURI__.tauri;

// 监听进度事件
window.__TAURI__.event.listen("install-progress", (event) => {
  const progress = event.payload;
  document.getElementById("progress").style.width = progress + "%";
});

// 监听详细信息事件
window.__TAURI__.event.listen("install-details", (event) => {
  const detail = event.payload;
  document.getElementById("details").innerText = detail;
});

// 监听安装完成事件
window.__TAURI__.event.listen("install-complete", () => {
  window.location.href = "finish.html";
});