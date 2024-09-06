<script setup lang="ts">
import { computed, onMounted, Ref, ref, watch } from 'vue';
import ScrollBox from '@/components/ScrollBox.vue';
import { installConf } from '@/utils/index';
import type {
  CheckGroup,
  CheckGroupItem,
  CheckItem,
  Component,
} from '@/utils/index';
import { useCustomRouter } from '@/router/index';
import CheckBoxGroup from '@/components/CheckBoxGroup.vue';

const { routerPush, routerBack } = useCustomRouter();
const selectComponentId = ref(0);

const groupComponents: Ref<CheckGroup<Component>[]> = ref([]);
const checkedAllBundle = ref(false);
const checkedAll = computed(() => {
  return groupComponents.value.every((item) =>
    item.items.every((i) => i.checked)
  );
});

watch(checkedAll, (val) => {
  checkedAllBundle.value = val;
});

const curCheckComponent = computed(() => {
  for (const group of groupComponents.value) {
    for (const item of group.items) {
      if (item.selected) {
        return item;
      }
    }
  }
  return null;
});

function updateInstallConf() {
  installConf.setComponents(
    groupComponents.value.reduce((components, group) => {
      components.push(
        ...group.items.map((item) => {
          return {
            label: item.label,
            checked: item.checked,
            value: { ...item.value },
          };
        })
      );
      return components;
    }, [] as CheckItem<Component>[])
  );
}

function handleComponentsClick(checkItem: CheckGroupItem<Component>) {
  selectComponentId.value = checkItem.value.id;
  groupComponents.value.forEach((group) => {
    group.items.forEach((item) => {
      if (item.value.id === checkItem.value.id) {
        item.selected = true;
      } else {
        item.selected = false;
      }
    });
  });
}
function handleComponentsChange(items: CheckGroupItem<Component>[]) {
  groupComponents.value.forEach((group) => {
    group.items.forEach((item) => {
      const findItem = items.find((i) => i.value.id === item.value.id);
      if (findItem) {
        item.checked = findItem.checked;
      }
    });
  });
  updateInstallConf();
}

function handleSelectAll() {
  const target = !checkedAll.value;
  groupComponents.value.forEach((group) => {
    group.items.forEach((item) => {
      item.checked = item.value.required ? true : target;
    });
  });
}

onMounted(() => {
  groupComponents.value = installConf.getGroups();
});
</script>

<template>
  <div flex="~ col" w="full" h="full">
    <h4 ml="12px">安装选项</h4>
    <div flex="1 ~" p="12px" overflow="auto">
      <scroll-box overflow-auto p="4px" grow="1">
        <div p="t-8px l-8px">组件</div>
        <div ml="1.5rem">
          <base-check-box
            flex="~ items-center"
            v-model="checkedAllBundle"
            title="全选"
            @click="handleSelectAll"
          />
        </div>

        <check-box-group
          v-for="group of groupComponents"
          :key="group.label"
          :group="group"
          expand
          @itemClick="handleComponentsClick"
          @change="handleComponentsChange"
        />
      </scroll-box>
      <scroll-box basis="200px" grow="4" ml="12px">
        <div>组件详细信息</div>
        <p font="b">{{ curCheckComponent?.value.name }}</p>
        <p v-for="item in curCheckComponent?.value.desc">{{ item }}</p>
      </scroll-box>
    </div>

    <div basis="60px" flex="~ justify-end items-center">
      <base-button mr="12px" @click="routerBack">上一步</base-button>
      <base-button mr="12px" @click="routerPush('/installer/confirm')"
        >下一步</base-button
      >
    </div>
  </div>
</template>
