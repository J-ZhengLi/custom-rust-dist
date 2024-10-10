<script setup lang="ts">
import { event } from '@tauri-apps/api';
import { onMounted } from 'vue';
import { useCustomRouter } from '@/router/index';
import { installConf, invokeCommand } from '@/utils/index';

const { routerPush, routerBack } = useCustomRouter();
// const diskRequire = ref(33);

function handleNextClick() {
  routerPush('/installer/components');
}

function openFolder() {
  invokeCommand('select_folder');
}

onMounted(() => {
  // 监听文件夹选择事件
  event.listen('folder-selected', (event) => {
    const path = event.payload;
    if (typeof path === 'string') {
      installConf.setPath(path.trim());
    }
  });
});
</script>

<template>
  <div flex="~ col">
    <div flex="1" mx="12px">
      <h4>安装目录</h4>
      <p>Rust 一站式开发套件将会安装到该路径中。</p>
      <div flex="~ items-center">
        <base-input
          v-bind:value="installConf.path.value"
          flex="1"
          type="text"
          placeholder="选择一个文件夹"
          @change="
            (event: Event) =>
              installConf.setPath((event.target as HTMLInputElement).value)
          "
          @keydown.enter="
            (event: Event) =>
              installConf.setPath((event.target as HTMLInputElement).value)
          "
        />
        <base-button theme="primary" ml="12px" @click="openFolder"
          >选择文件夹</base-button
        >
      </div>
    </div>
    <!-- <div mx="12px">
      <p>至少需要{{ diskRequire.toFixed(1) }}M的磁盘空间</p>
    </div> -->
    <div h="60px" flex="~ justify-end items-center">
      <base-button theme="primary" mr="12px" @click="routerBack"
        >上一步</base-button
      >
      <base-button theme="primary" mr="12px" @click="handleNextClick"
        >下一步</base-button
      >
    </div>
  </div>
</template>
