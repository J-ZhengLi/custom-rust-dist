<script setup lang="ts">
import { useCustomRouter } from '@/router/index';
import { invokeCommand, managerConf } from '@/utils';
import { computed, ref, watch } from 'vue';
import Label from './components/Label.vue';
const { routerBack, routerPush } = useCustomRouter();

// TODO: change to `false` after implementing installation via manager
const isUninstallManger = ref(true);
const installDir = computed(() => managerConf.path);

watch(isUninstallManger, (val: boolean) => {
  managerConf.setUninstallManager(val);
});

const installed = computed(() => managerConf.getInstalled());

function handleUninstall() {
  invokeCommand('uninstall_toolkit', {
    remove_self: isUninstallManger.value,
  }).then(() => routerPush('/manager/progress'));
}
</script>
<template>
  <div h="full" flex="~ col">
    <div mx="12px">
      <h1>卸载</h1>
      <p>即将卸载以下产品</p>
    </div>
    <scroll-box mx="12px" flex="1">
      <Label
        m="0"
        :label="installed?.name || ''"
        :old-ver="installed?.version"
      ></Label>
      <div mt="1em"><b>位置</b></div>
      <span m="l-1em">{{ installDir }}</span>
      <div mt="1em"><b>组件</b></div>
      <div
        v-for="item in installed?.components"
        :key="item.id"
        m="b-1em l-1em"
      >
        <Label :label="item.name" :old-ver="item.version"></Label>
      </div>
    </scroll-box>
    <!-- TODO: uncomment after implementing installation via manager -->
    <!-- <div m="l-2em t-0.5em" h="2em">
      <base-check-box v-model="isUninstallManger" title="同时卸载此管理工具" />
    </div> -->
    <div basis="60px" flex="~ justify-end items-center">
      <base-button theme="primary" mr="12px" @click="routerBack"
        >取消</base-button
      >
      <base-button theme="primary" mr="12px" @click="handleUninstall"
        >卸载</base-button
      >
    </div>
  </div>
</template>
