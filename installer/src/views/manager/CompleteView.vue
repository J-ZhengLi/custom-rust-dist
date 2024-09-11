<script setup lang="ts">
import { useCustomRouter } from '@/router';
import { invokeCommand, managerConf } from '@/utils';
import { computed } from 'vue';

const { routerBack } = useCustomRouter();
const isUninstallManger = computed(() => managerConf.getUninstallManager());

function closeWindow() {
  if (isUninstallManger.value) {
    // TODO uninstall manager
    invokeCommand('');
  }

  invokeCommand('');
}
</script>

<template>
  <section flex="~ col">
    <h4 ml="12px">卸载完成</h4>
    <div flex="1" p="12px">
      <p>所有组件已经从您的电脑移除。</p>
      <p v-if="isUninstallManger">我们将在此窗口关闭后移除此管理工具。</p>
    </div>
    <div basis="60px" flex="~ justify-end items-center">
      <base-button mr="12px" @click="routerBack()">上一步</base-button>
      <base-button theme="primary" mr="12px" @click="closeWindow"
        >关闭</base-button
      >
    </div>
  </section>
</template>
