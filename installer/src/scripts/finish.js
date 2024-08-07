const { invoke } = window.__TAURI__.tauri;

let finishEl;
let closeEl;

async function finish() {
  await invoke("finish");
}

async function close() {
  await invoke("close_window");
}

window.addEventListener("DOMContentLoaded", () => {
  finishEl = document.getElementById("finish");
  closeEl = document.getElementById("close");
  finishEl.addEventListener("click", (e) => {
    e.preventDefault();
    finish();
  });
  closeEl.addEventListener("click", (e) => {
    e.preventDefault();
    close();
  });
});
