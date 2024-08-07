const { invoke } = window.__TAURI__.tauri;


let previousEl;
let nextEl;
let closeEl;

async function previous() {
  window.location.href = "index.html";
}

async function next() {
  window.location.href = "components.html";
}

async function close() {
  await invoke("close_window");
}

window.addEventListener("DOMContentLoaded", async () => {
  previousEl = document.getElementById("previous");
  nextEl = document.getElementById("next");
  closeEl = document.getElementById("close");
  previousEl.addEventListener("click", (e) => {
    e.preventDefault();
    previous();
  })
  nextEl.addEventListener("click", (e) => {
    e.preventDefault();
    next();
  });
  closeEl.addEventListener("click", (e) => {
    e.preventDefault();
    close();
  });
});
