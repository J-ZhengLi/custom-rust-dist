<script setup lang="ts">
import { event } from '@tauri-apps/api';
import { computed, nextTick, onMounted, Ref, ref } from 'vue';
import { managerConf, progressFormat } from '@/utils';
import { useCustomRouter } from '@/router';
import { message } from '@tauri-apps/api/dialog';

const { routerPush, routerBack } = useCustomRouter();

const progress = ref(0);
const output: Ref<string[]> = ref([]);
const isUninstall = computed(() => managerConf.getOperation() === 'uninstall');
const scrollBox = ref(null);

function complete() {
  if (isUninstall.value) {
    routerPush('/manager/complete');
  } else {
    message('完成更改', { title: '提示' }).then(() => routerBack(-3));
  }
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
    setTimeout(() => {
      routerPush('/manager/complete');
    }, 1000);
  });

  event.listen('on-failed', (event) => {
    if (typeof event.payload === 'string') {
      message(event.payload, { title: '错误', type: 'error' }).then(() =>
        routerPush('/manager')
      );
    }
  });
});

function toBottom() {
  nextTick(() => {
    if (scrollBox?.value) {
      (scrollBox.value as HTMLElement).scrollTop = (
        scrollBox.value as HTMLElement
      ).scrollHeight;
    }
  });
}
</script>
<template>
  <section flex="~ col">
    <h4 ml="12px">正在{{ isUninstall ? '卸载' : '安装' }}，请稍候...</h4>
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
        theme="primary"
        v-show="progress === 100"
        @click="complete"
        mr="12px"
        >下一步</base-button
      >
    </div>
  </section>
</template>
