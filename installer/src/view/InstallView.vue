<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue';
import { useCustomRouter } from '../router';
import { event } from '@tauri-apps/api';

const { routerPush } = useCustomRouter();
const progress = ref(0);
const output = ref(['']);
const scrollBox = ref(null);

const title = computed(
  () => `安装${progress.value >= 100 ? '已完成' : '进行中...'}`
);

function toBottom() {
  nextTick(() => {
    if (scrollBox?.value) {
      (scrollBox.value as HTMLElement).scrollTop = (
        scrollBox.value as HTMLElement
      ).scrollHeight;
    }
  });
}

function progressFormat(value: number) {
  return value.toFixed(2).toString().padStart(5, '0') + '%';
}

onMounted(() => {
  event.listen('install-progress', (event) => {
    if (typeof event.payload === 'number') {
      progress.value = event.payload;
    }
  });

  event.listen('install-details', (event) => {
    console.log(event.payload);
    if (typeof event.payload === 'string') {
      output.value.push(event.payload);
      toBottom();
    }
  });

  event.listen('install-complete', () => {
    output.value.push('安装完成');
    toBottom();
    setTimeout(() => {
      routerPush('/finish');
    }, 1000);
  });
});
</script>

<template>
  <div flex="~ col">
    <h4 ml="12px">{{ title }}</h4>
    <div px="12px">
      <base-progress
        w="full"
        :percentage="progress"
        striped
        stripedFlow
        :format="progressFormat"
      />
    </div>
    <div
      ref="scrollBox"
      flex="1"
      m="12px"
      p="12px"
      overflow-y="auto"
      b="1px solid light hover:active"
      rounded="4px"
    >
      <p my="8px" v-for="item in output" :key="item">{{ item }}</p>
    </div>
    <div basis="60px" flex="~ justify-end items-center">
      <base-button v-show="progress === 100" @click="() => routerPush('/finish')" mr="12px"
        >下一步</base-button
      >
    </div>
  </div>
</template>
