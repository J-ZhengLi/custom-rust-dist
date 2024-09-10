<script setup lang="ts">
import { ref } from 'vue';
import { installConf, invokeCommand } from '@/utils/index';

const runApp = ref(true);

function closeWindow() {
  if (runApp.value) {
    invokeCommand('run_app', { install_dir: installConf.path.value });
  }
  invokeCommand('finish');
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
      <base-button theme="primary" mr="12px" @click="closeWindow"
        >完成并关闭</base-button
      >
    </div>
  </div>
</template>
