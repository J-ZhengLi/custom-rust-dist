<script setup lang="ts">
import { computed } from 'vue';
import { installConf, invokeCommand } from '../utils';
import type { Component } from '../utils';
import { useCustomRouter } from '../router';
import ScrollBox from '../components/ScrollBox.vue';

const { routerPush, routerBack } = useCustomRouter();
const path = computed(() => installConf.value.path);
const components = computed(() =>
  installConf.value.components
    .filter((i) => i.checked) // 筛选选中组件
    .map((item: Component) => {
      return { ...item, desc: item.desc.join(''), checked: undefined };
    })
);

function handleNextClick() {
  invokeCommand('install_toolchain', {
    components_list: components.value,
    install_dir: path.value,
  }).then(() => routerPush('/install'));
}
</script>

<template>
  <div flex="~ col">
    <div ml="12px">
      <h4 mb="4px">准备安装</h4>
      <p mt="4px">开始安装之前，请确认安装信息无误。</p>
      <p mb="4px">单击“安装”以继续。如果需要修改配置请点击“上一步”。</p>
    </div>
    <scroll-box flex="1" mx="12px" overflow="auto">
      <p mt="0" mb="8px">安装位置：</p>
      <base-input
        :value="installConf.path"
        border-color="focus:base"
        ml="12px"
        w="90%"
        readonly
      />
      <p mb="8px">组件：</p>
      <div ml="12px">
        <p my="4px" v-for="component in components" :key="component.name">
          {{ component.name }}
        </p>
      </div>
    </scroll-box>
    <div h="60px" flex="~ justify-end items-center">
      <base-button mr="12px" @click="routerBack">上一步</base-button>
      <base-button mr="12px" @click="handleNextClick">开始安装</base-button>
    </div>
  </div>
</template>
