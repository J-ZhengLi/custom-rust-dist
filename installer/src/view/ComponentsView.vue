<script setup lang="ts">
import { computed, ref } from 'vue';
import ScrollBox from '../components/ScrollBox.vue';
import { installConf } from '../utils';
import type { Component } from '../utils';
import { useCustomRouter } from '../router';

const { routerPush, routerBack } = useCustomRouter();
const focusIndex = ref(0);
const components = installConf.components;

const curDescriptions = computed(() => {
  if (
    focusIndex.value !== null &&
    focusIndex.value >= 0 &&
    focusIndex.value < components.value.length
  ) {
    return components.value[focusIndex.value].desc;
  }
  return '';
});

function handleComponentsClick(component: Component) {
  focusIndex.value = components.value.findIndex(
    (item: Component) => item.name === component.name
  );
}
</script>

<template>
  <div flex="~ col" w="full" h="full">
    <h4 ml="12px">安装选项</h4>
    <div flex="1 ~" p="12px" overflow="auto">
      <scroll-box grow="1" basis="100px">
        <div>组件</div>
        <div
          v-for="(item, index) of components"
          :key="item.name"
          mt="8px"
          h="1.5rem"
        >
          <base-check-box
            v-model="item.checked"
            :title="`${item.name}${item.required ? ' (required)' : ''}`"
            :disabled="item.required"
            decoration="hover:underline"
            :class="{ 'decoration-underline': index === focusIndex }"
            @titleClick="() => handleComponentsClick(item)"
          />
        </div>
      </scroll-box>
      <scroll-box basis="200px" grow="4" ml="12px">
        <div>组件详细信息</div>
        <p v-for="item in curDescriptions">{{ item }}</p>
      </scroll-box>
    </div>

    <div basis="60px" flex="~ justify-end items-center">
      <base-button mr="12px" @click="routerBack">上一步</base-button>
      <base-button mr="12px" @click="() => routerPush('/confirm')"
        >下一步</base-button
      >
    </div>
  </div>
</template>
