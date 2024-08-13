<script setup lang="ts">
import { computed, onMounted, Ref, ref } from 'vue';
import { useCustomRouter } from '../router';
import { Component } from '../utils';
import { invoke, event } from '@tauri-apps/api';
import ScrollBox from '../components/ScrollBox.vue';

const { routerPush, routerBack } = useCustomRouter();

const installDir = ref('');
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

// show_install_dir
function showInstallDir() {
  const installPath = localStorage.getItem('installPath');
  if (installPath !== null) {
    installDir.value = installPath;
    return;
  }

  invoke('default_install_dir')
    .then((path: unknown) => {
      if (typeof path === 'string' && path.trim() !== '') {
        localStorage.setItem('installPath', path);
        installDir.value = path;
      }
    })
    .catch((e) => {
      console.error(e);
    });
}

// select_folder
function openFolder() {
  invoke('select_folder')
    // .then((path: unknown) => {
    //   if (typeof path === 'string' && path.trim() !== '') {
    //     localStorage.setItem('installPath', path);
    //     installDir.value = path;
    //   }
    // })
    .catch((e) => {
      console.error(e);
    });
}

// loadComponents
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
  invoke('install_toolchain', {
    components_list: components.value
      .filter((item: Component) => item.checked) // 选中的组件
      .map((item: Component) => {
        // 去掉checked属性
        return { ...item, desc: item.desc.join(''), checked: undefined };
      }),
    install_dir: installDir.value,
  })
    .then((result: unknown) => {
      console.log(result);
      // 可以在添加一些回调
      // if (result === 'success') {}
      routerPush('/install');
    })
    .catch((e) => {
      console.error(e);
    });
}

onMounted(() => {
  showInstallDir();
  loadComponents();

  // 监听文件夹选择事件
  event.listen('folder-selected', (event) => {
    const path = event.payload;
    if (typeof path === 'string' && path.trim() !== '') {
      installDir.value = path;
      localStorage.setItem('installPath', path);
    } else {
      installDir.value = localStorage.getItem('installPath') || '';
    }
  });
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
            :class="{ 'text-active': index === focusIndex }"
            @click="() => handleComponentsClick(item)"
          />
        </div>
      </scroll-box>
      <scroll-box basis="200px" grow="4" ml="12px">
        <div>组件详细信息</div>
        <p v-for="item in curDescriptions">{{ item }}</p>
      </scroll-box>
    </div>

    <div basis="120px" grow="0">
      <div ml="12px">安装目录:</div>
      <div flex="~ items-center" p="12px">
        <base-input
          v-bind:value="installDir"
          flex="1"
          type="text"
          placeholder="选择一个文件夹"
          readonly
        />
        <base-button ml="12px" @click="openFolder">选择文件夹</base-button>
      </div>
      <div h="60px" flex="~ justify-end items-center">
        <base-button mr="12px" @click="routerBack">上一步</base-button>
        <base-button mr="12px" @click="handleInstallClick"
          >开始安装</base-button
        >
      </div>
    </div>
  </div>
</template>
