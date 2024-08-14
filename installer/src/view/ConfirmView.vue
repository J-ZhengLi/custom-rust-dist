<script setup lang="ts">
import { useRoute } from 'vue-router';
import { useCustomRouter } from '../router';
import { onMounted, Ref, ref, watch } from 'vue';
import { Component } from '../utils';
import ScrollBox from '../components/ScrollBox.vue';
import { invoke } from '@tauri-apps/api';

const route = useRoute();
const path = ref(route.query.path as string);
const components: Ref<Component[]> = ref(
  JSON.parse(route.query.components as string)
);

watch(
  () => route.query.path,
  (newPath) => {
    if (typeof newPath === 'string') {
      path.value = newPath;
    }
  }
);
watch(
  () => route.query.components,
  (newComponents) => {
    if (typeof newComponents === 'string') {
      components.value = JSON.parse(newComponents);
    }
  }
);

const { routerPush, routerBack } = useCustomRouter();
function handleNextClick() {
  invoke('install_toolchain', {
    components_list: components.value,
    install_dir: path.value,
  })
    .then(() => {
      routerPush('/install');
    })
    .catch((e) => {
      console.error(e);
    });
}

onMounted(() => {
  console.log(route.query);
});
</script>

<template>
  <div flex="~ col">
    <div ml="12px">
      <h4 mb="4px">准备安装</h4>
      <p mt="4px">开始安装之前，请确认安装信息无误。</p>
      <p mb="4px">单击“安装”以继续。如果需要修改配置请点击“上一步”。</p>
    </div>
    <scroll-box flex="1" mx="12px" overflow="auto">
      <p mt="0" mb="8px">安装位置：</p>
      <base-input
        :value="path"
        border-color="focus:base"
        ml="12px"
        w="90%"
        readonly
      />
      <p mb="8px">组件：</p>
      <div ml="12px">
        <p my="4px" v-for="component in components" :key="component.name">
          {{ component.name }}
        </p>
      </div>
    </scroll-box>
    <div h="60px" flex="~ justify-end items-center">
      <base-button mr="12px" @click="routerBack">上一步</base-button>
      <base-button mr="12px" @click="handleNextClick">开始安装</base-button>
    </div>
  </div>
</template>
