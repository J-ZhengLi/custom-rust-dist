<script setup lang="ts">
import { installConf, invokeCommand, TauriComponent } from '@/utils/index';
import { useCustomRouter } from '@/router/index';
import ScrollBox from '@/components/ScrollBox.vue';
import { computed } from 'vue';

const { routerPush, routerBack } = useCustomRouter();
const path = installConf.path;

const components = computed(() => {
  const list = installConf.getCheckedComponents();
  list.sort((a, b) => a.id - b.id);
  return list;
});

function handleNextClick() {
  invokeCommand('install_toolchain', {
    components_list: components.value as TauriComponent[],
    install_dir: path.value as string,
  }).then(() => routerPush('/installer/install'));
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
      <p m="0">安装位置：</p>
      <p my="4px">{{ path }}</p>
      <p mb="8px">组件：</p>
      <div ml="12px">
        <p my="4px" v-for="component in components" :key="component.name">
          {{
            `${component.name} ${component.installed ? '(installed, re-installing)' : component.required ? '(required)' : ''} `
          }}
        </p>
      </div>
    </scroll-box>
    <div h="60px" flex="~ justify-end items-center">
      <base-button theme="primary" mr="12px" @click="routerBack"
        >上一步</base-button
      >
      <base-button theme="primary" mr="12px" @click="handleNextClick"
        >开始安装</base-button
      >
    </div>
  </div>
</template>
