<script setup lang="ts">
import { ref } from 'vue';
import { useCustomRouter } from '../router';
import { invoke } from '@tauri-apps/api';

const runApp = ref(true);

function closeWindow() {
  if (runApp.value) {
    // run app
    // TODO: Pass install_dir to the `run_app` function
    invoke('run_app').catch((e) => {
      console.error(e);
    })
    .catch((e) => {
      console.error(e);
    });
  }
  invoke('finish').catch((e) => {
    console.error(e);
  });
}
</script>

<template>
  <div flex="~ col" w="full">
    <div flex="1" px="12px">
      <h4>安装完成</h4>
      <p>安装程序已经将Rust安装到您的电脑中，</p>
      <p>单击“完成”退出安装程序</p>
      <base-check-box
        v-model="runApp"
        title="安装完成后打开示例项目"
        mt="12px"
      ></base-check-box>
    </div>
    <div basis="60px" flex="~ items-center justify-end">
      <base-button mr="12px" @click="closeWindow">完成并关闭</base-button>
    </div>
  </div>
</template>
