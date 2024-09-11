<script setup lang="ts">
import { useCustomRouter } from '@/router/index';
import { managerConf } from '@/utils';
import { ref, watch } from 'vue';
const { routerBack, routerPush } = useCustomRouter();

const isUninstallManger = ref(false);

watch(isUninstallManger, (val: boolean) => {
  managerConf.setUninstallManager(val);
});

function handleUninstall() {
  // TODO: uninstall
  routerPush('/manager/progress');
}
</script>
<template>
  <div h="full" flex="~ col">
    <div mx="12px">
      <h1>卸载</h1>
      <p>即将卸载以下产品</p>
    </div>
    <scroll-box mx="12px" flex="1">
      <div mb="12px">
        <p m="0">Visual Studio 生成工具 2022</p>
        <p m="0">
          C:\Program Files(x86)\Microsoft Visual Studio\2022\BuildTools
        </p>
      </div>
    </scroll-box>
    <div mx="16px">
      <base-check-box
        v-model="isUninstallManger"
        block
        title="同时卸载此管理工具"
      />
    </div>

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
