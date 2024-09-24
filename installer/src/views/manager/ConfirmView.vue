<script setup lang="ts">
import { useCustomRouter } from '@/router';
import { managerConf } from '@/utils';
import { computed } from 'vue';
import ComponentLabel from './components/Label.vue';

const { routerPush, routerBack } = useCustomRouter();
const components = computed(() => managerConf.getTargetComponents());

const labels = computed(() => {
  const installed = managerConf.getInstalledComponents();
  return components.value.map((item) => {
    const installedComponent = installed?.find((i) => i.name === item.name);
    return {
      label: item.name,
      originVer: installedComponent?.version,
      targetVer: item.version,
    };
  });
});
</script>

<template>
  <section flex="~ col" w="full" h="full">
    <h4 ml="12px">确认信息</h4>
    <scroll-box mx="12px" flex="1">
      <div v-for="item in labels" :key="item.label" mb="24px">
        <component-label
          :label="item.label"
          :oldVer="item.originVer"
          :newVer="item.targetVer"
        />
      </div>
    </scroll-box>
    <div basis="60px" flex="~ justify-end items-center">
      <base-button theme="primary" mr="12px" @click="routerBack()"
        >上一步</base-button
      >
      <base-button
        theme="primary"
        mr="12px"
        @click="routerPush('/manager/progress')"
        >下一步</base-button
      >
    </div>
  </section>
</template>
