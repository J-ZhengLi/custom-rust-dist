<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { progressFormat } from '@/utils';
import { useCustomRouter } from '@/router';

const { routerPush } = useCustomRouter();

const progress = ref(0);
const output = ref([]);
let timer: any;

onMounted(() => {
  timer = setInterval(() => {
    if (progress.value < 100) progress.value += 1;
    else clearInterval(timer);
  }, 100);
});
onUnmounted(() => {
  clearInterval(timer);
});
</script>
<template>
  <section flex="~ col">
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
        @click="() => routerPush('/manager/complete')"
        mr="12px"
        >下一步</base-button
      >
    </div>
  </section>
</template>
