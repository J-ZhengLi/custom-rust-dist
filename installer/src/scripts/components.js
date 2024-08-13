const { invoke } = window.__TAURI__.tauri;

let selectedComponents = [];

let previousEl;
let nextEl;
let closeEl;

async function previous() {
  window.location.href = "explain.html";
}

async function install() {
  await invoke('install_toolchain', { components_list: selectedComponents, install_dir: localStorage.getItem("installPath") });
  window.location.href = "progress.html";
}

async function close() {
  await invoke("close_window");
}

async function show_install_dir() {
  let installPath = localStorage.getItem('installPath');
  if (installPath !== null) {
    document.getElementById('default_home').value = installPath;
  } else {
    let path = await invoke('default_install_dir');
    localStorage.setItem('installPath', path);
    document.getElementById('default_home').value = path;
  }
}

// 定义一个异步函数来调用 select_folder 命令并存储结果到 localStorage 中
async function select_folder() {
  await invoke('select_folder');
}

async function loadComponents() {
  const componentList = await invoke('get_component_list');
  const componentListElement = document.getElementById('component-list');

  componentList.forEach(component => {
      const componentElement = document.createElement('div');
      componentElement.classList.add('component');
      componentElement.dataset.id = component.id;

      const checkbox = document.createElement('input');
      checkbox.type = 'checkbox';
      checkbox.id = `component-${component.id}`;
      checkbox.checked = component.required;
      checkbox.disabled = component.required;

      if (component.required) {
        selectedComponents.push(component);
      }

      const nameElement = document.createElement('span');
      nameElement.classList.add('component-name');
      nameElement.textContent = component.name;

      componentElement.appendChild(checkbox);
      componentElement.appendChild(nameElement);
      componentListElement.appendChild(componentElement);

      componentElement.addEventListener('click', () => {
          displayComponentDetail(component);
      });

      checkbox.addEventListener('change', (event) => {
          if (event.target.checked) {
              displayComponentDetail(component);
              selectedComponents.push(component);
          } else {
              clearComponentDetail();
              selectedComponents = selectedComponents.filter(c => c.id !== component.id);
            }
      });
  });
}

function displayComponentDetail(component) {
  const componentDetailElement = document.getElementById('component-detail');
  componentDetailElement.innerHTML = `
      <p><strong>名称：</strong>${component.name}</p>
      <p><strong>描述：</strong>${component.desc}</p>
  `;
}

function clearComponentDetail() {
  const componentDetailElement = document.getElementById('component-detail');
  componentDetailElement.innerHTML = '';
}

window.addEventListener("DOMContentLoaded", () => {
  // init window
  loadComponents();
  show_install_dir();

  document.getElementById("selectButton").addEventListener("click", select_folder);

  previousEl = document.getElementById("previous");
  nextEl = document.getElementById("install");
  closeEl = document.getElementById("close");
  previousEl.addEventListener("click", (e) => {
    e.preventDefault();
    previous();
  });
  nextEl.addEventListener("click", (e) => {
    e.preventDefault();
    install();
  });
  closeEl.addEventListener("click", (e) => {
    e.preventDefault();
    close();
  });
});

// 监听文件夹选择事件
window.__TAURI__.event.listen('folder-selected', (event) => {
  const installPath = event.payload;
  if (installPath && installPath.trim() !== '') {
    localStorage.setItem('installPath', installPath);
    document.getElementById('default_home').value = `${installPath}`;
    localStorage.setItem('installPath', installPath);
  } else {
      const currentPath = localStorage.getItem('installPath');
      document.getElementById('default_home').value = `${currentPath}`;
  }
});
