<script setup lang="ts">
import { onMounted, ref } from 'vue';
// import { installConf } from './utils';
import { RouterView } from 'vue-router';
import Titlebar from './components/Titlebar.vue';
import { invokeCommand } from './utils';

const appTitle = ref('');

onMounted(() => {
  invokeCommand("window_title").then((res) => {
    if (typeof res === 'string') {
      appTitle.value = res
    }
  });
});
</script>

<template>
  <div>
    <Titlebar :title="appTitle" />
    <RouterView style="margin-top: 41px;" />
  </div>
</template>

<style>
:root {
  margin: 0;
  padding: 0;
  font-family:
    -apple-system,
    BlinkMacSystemFont,
    Segoe UI,
    Helvetica,
    Arial,
    sans-serif,
    Apple Color Emoji,
    Segoe UI Emoji,
    Segoe UI Symbol;
  font-size: 14px;
  line-height: 24px;
  letter-spacing: normal;
  font-weight: 400;
  --uno: bg-back c-header;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}
</style>
