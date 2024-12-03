<script setup lang="ts">
import { useCustomRouter } from '@/router';
import { invokeCommand, managerConf } from '@/utils';
import { computed } from 'vue';

const { routerPush } = useCustomRouter();
const isUninstallManger = computed(() => managerConf.getUninstallManager());

function closeOrReturn() {
  if (isUninstallManger.value) {
    invokeCommand('close_window');
  } else {
    // FIXME: refresh `kit` list, since the `installed` kit should no longer exist after uninstallation.
    routerPush('/manager');
  }
}
</script>

<template>
  <section flex="~ col">
    <h4 ml="12px">卸载完成</h4>
    <div flex="1" p="12px">
      <p>所选产品已经从您的电脑移除。</p>
    </div>
    <div basis="60px" flex="~ justify-end items-center">
      <base-button theme="primary" mr="12px" @click="closeOrReturn"
        >{{ isUninstallManger ? "关闭" : "返回" }}</base-button
      >
    </div>
  </section>
</template>
