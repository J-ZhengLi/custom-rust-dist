<script setup lang="ts">
import { computed, onMounted, Ref, ref, watch } from 'vue';
import ScrollBox from '@/components/ScrollBox.vue';
import { managerConf } from '@/utils/index';
import type {
  CheckGroup,
  CheckGroupItem,
  ManagerComponent,
} from '@/utils/index';
import { useCustomRouter } from '@/router/index';
import CheckBoxGroup from '@/components/CheckBoxGroup.vue';
import { message } from '@tauri-apps/api/dialog';

const { routerPush, routerBack } = useCustomRouter();
const selectComponentId = ref(0);

const groupComponents: Ref<CheckGroup<ManagerComponent>[]> = ref([]);
const checkedAllBundle = ref(false);

const checkedAll = computed(() => {
  return groupComponents.value.every((item) =>
    item.items.every((i) => i.checked)
  );
});
const checkedEmpty = computed(() => {
  return groupComponents.value.every((item) =>
    item.items.every((i) => !i.checked)
  );
});

watch(checkedAll, (val) => {
  checkedAllBundle.value = val;
});

const curCheckComponent = computed(() => {
  for (const group of groupComponents.value) {
    for (const item of group.items) {
      if (item.focused) {
        return item;
      }
    }
  }
  return null;
});

function updateTargetComponents() {
  managerConf.setComponents(
    groupComponents.value.reduce((components, group) => {
      components.push(
        ...group.items.filter((i) => i.checked).map((item) => item.value)
      );
      return components;
    }, [] as ManagerComponent[])
  );
}

function handleComponentsClick(checkItem: CheckGroupItem<ManagerComponent>) {
  selectComponentId.value = checkItem.value.id;
  groupComponents.value.forEach((group) => {
    group.items.forEach((item) => {
      if (item.value.id === checkItem.value.id) {
        item.focused = true;
      } else {
        item.focused = false;
      }
    });
  });
}
function handleComponentsChange(items: CheckGroupItem<ManagerComponent>[]) {
  groupComponents.value.forEach((group) => {
    group.items.forEach((item) => {
      const findItem = items.find((i) => i.value.id === item.value.id);
      if (findItem) {
        item.checked = findItem.checked;
      }
    });
  });
  updateTargetComponents();
}

function handleSelectAll() {
  const target = checkedEmpty.value;
  groupComponents.value.forEach((group) => {
    group.items.forEach((item) => {
      if (item.disabled) return;
      item.checked = target;
    });
  });
  updateTargetComponents();
}

function handleClickNext() {
  if (managerConf.getTargetComponents().length === 0) {
    message('请选择至少一个组件', { type: 'error' });
    return;
  }
  managerConf.setOperation('update');
  routerPush('/manager/confirm');
}

onMounted(() => {
  groupComponents.value = managerConf.getGroups();
  updateTargetComponents();
});
</script>

<template>
  <div flex="~ col" w="full" h="full">
    <h4 ml="12px">组件更改</h4>
    <div flex="1 ~" p="12px" overflow="auto">
      <scroll-box overflow-auto p="4px" grow="1" relative>
        <div p="l-8px t-8px" flex="~ items-center wrap" gap="3" bg="back">
          <b>组件</b>
          <span>
            <base-tag size="small" w="1em" h="1.5em" m="r-2px b-4px"></base-tag>
            当前版本
          </span>
          <span>
            <base-tag
              type="success"
              size="small"
              w="1em"
              h="1.5em"
              m="r-2px b-4px"
            ></base-tag>
            新版本
          </span>
          <span>
            <base-tag
              type="warning"
              size="small"
              w="1em"
              h="1.5em"
              m="r-2px b-4px"
            ></base-tag>
            旧版本
          </span>
        </div>

        <div ml="1.5rem">
          <base-check-box
            flex="~ items-center"
            v-model="checkedAllBundle"
            title="全选"
          >
            <template #icon>
              <span
                flex="~ items-center justify-center"
                w="full"
                h="full"
                @click="handleSelectAll"
              >
                <i class="i-mdi:check" v-show="checkedAll" c="active" />
                <i
                  class="i-mdi:minus"
                  v-show="!checkedAll && !checkedEmpty"
                  c="active"
                />
              </span>
            </template>
          </base-check-box>
        </div>

        <check-box-group
          v-for="group of groupComponents"
          :key="group.label"
          :group="group"
          expand
          mb="1rem"
          @itemClick="handleComponentsClick"
          @change="handleComponentsChange"
        />
      </scroll-box>
      <scroll-box basis="200px" grow="4" ml="12px">
        <b>组件详细信息</b>
        <p font="b">{{ curCheckComponent?.value.name }}</p>
        <p v-for="item in curCheckComponent?.value.desc" :key="item">
          {{ item }}
        </p>
      </scroll-box>
    </div>

    <div basis="60px" flex="~ justify-end items-center">
      <base-button theme="primary" mr="12px" @click="routerBack()"
        >上一步</base-button
      >
      <base-button theme="primary" mr="12px" @click="handleClickNext"
        >下一步</base-button
      >
    </div>
  </div>
</template>
