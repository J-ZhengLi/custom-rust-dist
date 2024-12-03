<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { CheckGroup, CheckGroupItem } from '@/utils';

type Props<T> = {
  expand: boolean;
  group: CheckGroup<T>;
};

const { expand, group } = defineProps<Props<unknown>>();
const emit = defineEmits(['itemClick', 'change']);

const groupExpand = ref(expand);
const isCheckedAll = computed(() => group.items.every((item) => item.checked));
const isCheckedEmpty = computed(() =>
  group.items.every((item) => !item.checked)
);

function handleExpandClick() {
  groupExpand.value = !groupExpand.value;
}

function handleCheckAllClick() {
  const target = isCheckedAll.value;
  group.items.forEach((checkItem) => {
    if (checkItem.disabled) return;
    checkItem.checked = !target;
  });
}

function handleItemClick<T>(item: CheckGroupItem<T>) {
  emit('itemClick', item);
}

watch(group.items, (newValue) => {
  emit(
    'change',
    newValue.filter((item) => item.checked)
  );
});
</script>

<template>
  <div>
    <div flex="~ items-center">
      <i
        class="i-mdi:menu-right"
        w="1.5rem"
        h="1.5rem"
        transition="all"
        cursor="pointer"
        c="secondary"
        :class="{ 'rotate-90': groupExpand }"
        @click="handleExpandClick"
      />
      <base-check-box @titleClick="handleExpandClick">
        <template #icon>
          <span
            flex="~ items-center justify-center"
            w="full"
            h="full"
            @click="handleCheckAllClick"
          >
            <i class="i-mdi:check" v-show="isCheckedAll" c="active" />
            <i
              class="i-mdi:minus"
              v-show="!isCheckedAll && !isCheckedEmpty"
              c="active"
            />
          </span>
        </template>
        <b c="active">{{ group.label }}</b>
      </base-check-box>
    </div>
    <transition name="group">
      <div v-if="groupExpand" ml="3rem">
        <base-check-box
          flex="~ items-center"
          v-for="item of group.items"
          :key="item.label"
          v-model="item.checked"
          :title="item.label"
          :disabled="item.disabled"
          :label-component="item.labelComponent"
          :label-component-props="item.labelComponentProps"
          decoration="hover:underline"
          :class="{
            'decoration-underline': item.focused,
          }"
          @titleClick="handleItemClick(item)"
        />
      </div>
    </transition>
  </div>
</template>

<style scoped>
.group-enter-active {
  transition: all 150ms ease;
}
/* 菜单进出 */
.group-enter-from {
  transform: scaleY(0.5) translateY(-50%);
}

.group-enter-to {
  transform: scaleY(1) translateY(0);
}
</style>
