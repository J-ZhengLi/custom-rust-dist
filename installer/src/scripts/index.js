const { invoke } = window.__TAURI__.tauri;

let nextEl;
let closeEl;

async function next() {
  window.location.href = "explain.html";
}

async function close() {
  await invoke("close_window");
}

window.addEventListener("DOMContentLoaded", () => {
  nextEl = document.getElementById("next");
  closeEl = document.getElementById("close");
  nextEl.addEventListener("click", (e) => {
    e.preventDefault();
    next();
  });
  closeEl.addEventListener("click", (e) => {
    e.preventDefault();
    close();
  });
});
