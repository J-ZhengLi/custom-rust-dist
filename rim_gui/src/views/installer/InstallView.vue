<script setup lang="ts">
import type { Ref } from 'vue';
import { event } from '@tauri-apps/api';
import { message } from '@tauri-apps/api/dialog';
import { computed, nextTick, onMounted, ref } from 'vue';
import { useCustomRouter } from '@/router/index';
import { invokeCommand, progressFormat } from '@/utils/index';

const { routerPush } = useCustomRouter();
const progress = ref(0);
const output: Ref<string[]> = ref([]);
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

onMounted(() => {
  event.listen('update-progress', (event) => {
    if (typeof event.payload === 'number') {
      progress.value = event.payload;
    }
  });

  event.listen('update-message', (event) => {
    if (typeof event.payload === 'string') {
      event.payload.split('\n').forEach((line) => {
        output.value.push(line);
      });
      toBottom();
    }
  });

  event.listen('on-complete', () => {
    output.value.push('安装完成');
    toBottom();
    setTimeout(() => {
      routerPush('/installer/finish');
    }, 1000);
  });

  event.listen('on-failed', (event) => {
    if (typeof event.payload === 'string') {
      output.value.push(event.payload);
      toBottom();
      message(event.payload, { title: '错误', type: 'error' }).then(() =>
        invokeCommand('close_window')
      );
    }
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
      <base-button
        v-show="progress === 100"
        theme="primary"
        @click="() => routerPush('/installer/finish')"
        mr="12px"
        >下一步</base-button
      >
    </div>
  </div>
</template>
