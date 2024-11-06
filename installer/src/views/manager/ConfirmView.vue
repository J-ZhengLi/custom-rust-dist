<script setup lang="ts">
import { useCustomRouter } from '@/router';
import { invokeCommand, managerConf, Component } from '@/utils';
import { computed } from 'vue';
import ComponentLabel from './components/Label.vue';

const { routerPush, routerBack } = useCustomRouter();
const components = computed(() => managerConf.getTargetComponents());

const labels = computed(() => {
  const installed = managerConf.getInstalled();
  return components.value.map((item) => {
    const installedComponent = installed?.components.find((i) => i.name === item.name);
    let installedVersion = item.isToolchainComponent ? installed?.version : installedComponent?.version;
    return {
      label: item.name,
      originVer: installedVersion,
      targetVer: item.version,
    };
  });
});

function handleNextClick() {
  invokeCommand('install_toolkit', {
    components_list: components.value as Component[],
  }).then(() => routerPush('/manager/progress'));
}
</script>

<template>
  <section flex="~ col" w="full" h="full">
    <div mx="12px">
      <h1>确认信息</h1>
      <p>即将安装以下产品</p>
    </div>
    
    <scroll-box mx="12px" flex="1">
      <div v-for="item in labels" :key="item.label" mb="24px">
        <component-label :label="item.label" :oldVer="item.originVer" :newVer="item.targetVer" />
      </div>
    </scroll-box>
    <div basis="60px" flex="~ justify-end items-center">
      <base-button theme="primary" mr="12px" @click="routerBack()">上一步</base-button>
      <base-button theme="primary" mr="12px" @click="handleNextClick">开始安装</base-button>
    </div>
  </section>
</template>
