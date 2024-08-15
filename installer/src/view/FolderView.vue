<script setup lang="ts">
import { invoke, event } from '@tauri-apps/api';
import { message } from '@tauri-apps/api/dialog';
import { onMounted, ref } from 'vue';
import { useCustomRouter } from '../router';

const { routerPush, routerBack } = useCustomRouter();
const installDir = ref('');
const diskRequire = ref(33);

function handleNextClick() {
  routerPush({
    path: '/components',
    query: { path: installDir.value },
  });
}

// show_install_dir
function showInstallDir() {
  const installPath = localStorage.getItem('installPath');
  if (installPath !== null) {
    installDir.value = installPath;
    return;
  }

  invoke('default_install_dir')
    .then((path: unknown) => {
      if (typeof path === 'string' && path.trim() !== '') {
        localStorage.setItem('installPath', path);
        installDir.value = path;
      }
    })
    .catch((e) => {
      message(e.message, { title: '错误', type: 'error' });
    });
}

function openFolder() {
  invoke('select_folder').catch((e) => {
    message(e.message, { title: '错误', type: 'error' });
  });
}
onMounted(() => {
  showInstallDir();

  // 监听文件夹选择事件
  event.listen('folder-selected', (event) => {
    const path = event.payload;
    if (typeof path === 'string' && path.trim() !== '') {
      installDir.value = path;
      localStorage.setItem('installPath', path);
    } else {
      installDir.value = localStorage.getItem('installPath') || '';
    }
  });
});
</script>

<template>
  <div flex="~ col">
    <div flex="1" mx="12px">
      <h4>安装目录</h4>
      <p>Rust的本体和组件将会一起安装到该路径中。</p>
      <div flex="~ items-center">
        <base-input
          v-bind:value="installDir"
          flex="1"
          type="text"
          placeholder="选择一个文件夹"
        />
        <base-button ml="12px" @click="openFolder">选择文件夹</base-button>
      </div>
    </div>
    <div mx="12px">
      <p>至少需要{{ diskRequire.toFixed(1) }}M的磁盘空间</p>
    </div>
    <div h="60px" flex="~ justify-end items-center">
      <base-button mr="12px" @click="routerBack">上一步</base-button>
      <base-button mr="12px" @click="handleNextClick">下一步</base-button>
    </div>
  </div>
</template>
