<script setup lang="ts">
import { computed, onMounted, Ref, ref } from 'vue';
import { useCustomRouter } from '../router';
import { Component } from '../utils';
import { invoke } from '@tauri-apps/api';
import ScrollBox from '../components/ScrollBox.vue';
import { useRoute } from 'vue-router';
const route = useRoute();

const { routerPush, routerBack } = useCustomRouter();

const focusIndex = ref();
const components: Ref<Component[]> = ref([]);

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

function loadComponents() {
  invoke('get_component_list')
    .then((componentList: unknown) => {
      if (Array.isArray(componentList)) {
        components.value = componentList.map((item) => {
          return {
            ...item,
            desc: item.desc.split('\n'),
            checked: item.required,
          };
        });
        components.value.sort(
          (a, b) => Number(b.required) - Number(a.required)
        );
      }
    })
    .catch((e) => {
      console.error(e);
    });
}

function handleComponentsClick(component: Component) {
  focusIndex.value = components.value.findIndex(
    (item: Component) => item.name === component.name
  );
}

function handleInstallClick() {
  const components_list = components.value
    .filter((item: Component) => item.checked) // 选中的组件
    .map((item: Component) => {
      // 去掉checked属性
      return { ...item, desc: item.desc.join(''), checked: undefined };
    });
  routerPush({
    path: '/confirm',
    query: {
      path: route.query.path,
      components: JSON.stringify(components_list),
    },
  });
}

onMounted(() => {
  loadComponents();
});
</script>

<template>
  <div flex="~ col" w="full" h="full">
    <h4 ml="12px">安装选项</h4>
    <div flex="1 ~" p="12px" overflow="auto">
      <scroll-box grow="1" basis="100px">
        <div>组件</div>
        <div v-for="(item, index) of components" :key="item.name" mt="12px">
          <base-check-box
            v-model="item.checked"
            :title="`${item.name}${item.required ? '(必需)' : ''}`"
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
      <base-button mr="12px" @click="handleInstallClick">下一步</base-button>
    </div>
  </div>
</template>
