<script setup lang="ts">
import { useCustomRouter } from '@/router/index';
import { invokeCommand, managerConf } from '@/utils';
import { computed, ref, watch } from 'vue';
import Label from './components/Label.vue';
const { routerBack, routerPush } = useCustomRouter();

const isUninstallManger = ref(false);
const installDir = managerConf.path;

watch(isUninstallManger, (val: boolean) => {
  managerConf.setUninstallManager(val);
});

const installed = computed(() => managerConf.getInstalled() || []);

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
        :label="installed.value?.name || ''"
        :old-ver="installed.value?.version"
      ></Label>
      <div mt="1em">组件</div>
      <div
        v-for="item in installed.value?.components"
        :key="item.id"
        m="b-12px l-1em"
      >
        <Label m="0" :label="item.name" :old-ver="item.version"></Label>
        <p v-if="item.toolInstaller?.path" m="0">
          {{ item.toolInstaller?.path }}
        </p>
      </div>
      <div mt="1em">其他选项</div>
      <div mx="1em">
        <base-check-box
          v-model="isUninstallManger"
          block
          title="同时卸载此管理工具"
        />
      </div>
    </scroll-box>

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
