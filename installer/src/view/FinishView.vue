<script setup lang="ts">
import { ref } from 'vue';
import { useCustomRouter } from '../router';
import ScrollBox from '../components/ScrollBox.vue';
import { invoke } from '@tauri-apps/api';

const { routerBack } = useCustomRouter();
const runApp = ref(true);

function closeWindow() {
  if (runApp.value) {
    // run app
  }
  invoke('close_window').catch((e) => {
    console.error(e);
  });
}
</script>

<template>
  <div flex="~ col" w="full">
    <h4 mx="12px">安装结束</h4>

    <scroll-box flex="1" mx="12px">
      <base-check-box
        v-model="runApp"
        title="安装完成后自动运行"
        mt="12px"
      ></base-check-box>
    </scroll-box>

    <div basis="60px" flex="~ items-center justify-end">
      <base-button @click="routerBack" mr="12px">上一步-暂</base-button>
      <base-button mr="12px" @click="closeWindow">完成并关闭</base-button>
    </div>
  </div>
</template>
