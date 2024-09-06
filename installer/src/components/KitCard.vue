<script setup lang="ts">
import { KitItem } from '@/utils/index';
import { useCustomRouter } from '@/router/index';

const { routerPush } = useCustomRouter();

const props = defineProps<{
  kit: KitItem;
  installed: boolean;
}>();

const handleUpdate = () => {
  console.log('update');
  routerPush('/manager/change');
};

const handleUninstall = () => {
  console.log('uninstall');
  routerPush('/manager/uninstall');
};

const handleInstall = () => {
  console.log('install');
};
</script>
<template>
  <div
    shadow
    flex="~ justify-between"
    p="8px"
    mx="12px"
    rounded="4px"
    b="1px solid light"
  >
    <div>
      <div>
        <p flex="~ items-center">
          <img src="/favicon.ico" h="2rem" />
          <span ml="1rem">玄武 Rust 安装工具</span>
        </p>
        <p ml="3rem">{{ props.kit.version }}</p>
        <p ml="3rem">{{ props.kit.desc }}</p>
        <a m="l-3rem t-0.5rem">{{ props.kit.notes }}</a>
      </div>
    </div>
    <div v-if="props.installed" flex="~ col justify-around">
      <base-button p="y-2px x-24px" theme="primary" @click="handleUpdate"
        >更改</base-button
      >
      <base-button p="y-2px x-24px" @click="handleUninstall">卸载</base-button>
    </div>
    <div v-else flex="~ col justify-around">
      <base-button p="y-2px x-24px" theme="primary" @click="handleInstall"
        >安装</base-button
      >
    </div>
  </div>
</template>

<style scoped>
p {
  margin-top: 0.5rem;
  margin-bottom: 0;
}
</style>
