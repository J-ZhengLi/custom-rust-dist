<script setup lang="ts">
import { event } from '@tauri-apps/api';
import { onMounted, ref } from 'vue';
import { useCustomRouter } from '../router';
import { installConf, invokeCommand } from '../utils';

const { routerPush, routerBack } = useCustomRouter();
const diskRequire = ref(33);

function handleNextClick() {
  routerPush({
    path: '/components',
    query: { path: installConf.value.path },
  });
}

async function showInstallDir() {
  const installPath = localStorage.getItem('installPath');
  if (installPath !== null) {
    installConf.value.path = installPath;
    return;
  }

  const path = await invokeCommand('default_install_dir');
  if (typeof path === 'string' && path.trim() !== '') {
    localStorage.setItem('installPath', path);
    installConf.value.path = path;
  }
}

function openFolder() {
  invokeCommand('select_folder');
}

onMounted(() => {
  showInstallDir();

  // 监听文件夹选择事件
  event.listen('folder-selected', (event) => {
    const path = event.payload;
    if (typeof path === 'string' && path.trim() !== '') {
      installConf.value.path = path;
      localStorage.setItem('installPath', path);
    } else {
      installConf.value.path = localStorage.getItem('installPath') || '';
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
          v-bind:value="installConf.path"
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
