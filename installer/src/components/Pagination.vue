<script setup lang="ts">
import { computed } from 'vue';

interface PageItem {
  type: 'page' | 'more';
  value: number;
}
// Prop：总记录数，页大小，当前页码
const props = defineProps({
  total: {
    type: Number,
    required: true,
  },
  size: {
    type: Number,
    default: 10,
  },
  hideOnOnePage: {
    type: Boolean,
    default: true,
  },
  showJumper: {
    type: Boolean,
    default: false,
  },
});
const current = defineModel({ default: 1 });

const totalPages = computed(() => Math.ceil(props.total / props.size));
const showPagination = computed(() => {
  return !(props.hideOnOnePage && totalPages.value <= 1);
});
const pageList = computed(() => {
  const list: PageItem[] = [];

  // 显示页码范围的辅助函数
  const createPageList = (start: number, count: number) => {
    return Array.from({ length: count }, (_, i) => ({
      type: 'page',
      value: start + i,
    })) as PageItem[];
  };

  if (totalPages.value <= 7 || (totalPages.value === 7 && current.value <= 4)) {
    // 显示所有页码
    list.push(...createPageList(1, totalPages.value));
  } else if (current.value <= 4) {
    // 省略右边页码
    list.push(...createPageList(1, 6));
    list.push(
      { type: 'more', value: Math.min(current.value + 5, totalPages.value) },
      { type: 'page', value: totalPages.value }
    );
  } else if (totalPages.value - current.value < 4) {
    // 省略左边页码
    list.push(
      { type: 'page', value: 1 },
      { type: 'more', value: Math.max(current.value - 5, 1) }
    );
    list.push(...createPageList(totalPages.value - 5, 6));
  } else {
    // 显示当前页码前后各2个页码
    list.push(
      { type: 'page', value: 1 },
      { type: 'more', value: Math.max(current.value - 2, 1) }
    );
    list.push(...createPageList(current.value - 2, 5));
    list.push({
      type: 'more',
      value: Math.min(current.value + 2, totalPages.value),
    });
    list.push({ type: 'page', value: totalPages.value });
  }

  return list;
});

function toPage(target: number) {
  if (target < 1 || target > totalPages.value) return;
  current.value = target;
}
</script>

<template>
  <div v-if="showPagination" class="pagination">
    <i
      class="i-mdi:chevron-left"
      w="1.5em"
      h="1.5em"
      c="secondary hover:active"
      @click="toPage(current - 1)"
    />
    <span
      v-for="(item, index) in pageList"
      :key="index"
      :class="{ 'c-active': item.value === current }"
      px="4px"
      mx="4px"
      c="hover:active"
      cursor="pointer"
      @click="toPage(item.value)"
      >{{ item.type === 'page' ? item.value : '...' }}</span
    >
    <i
      class="i-mdi:chevron-right"
      w="1.5em"
      h="1.5em"
      c="secondary hover:active"
      @click="toPage(current + 1)"
    />
    <div v-if="props.showJumper" ml="12px">
      跳转至
      <base-input
        :min="1"
        :max="totalPages"
        w="4em"
        px="8px"
        py="4px"
        type="number"
        text="center"
        :value="current"
        @change="
          (event: Event) =>
            toPage(parseInt((event.target as HTMLInputElement).value))
        "
      />
    </div>
  </div>
</template>

<style scoped>
.pagination {
  display: flex;
  justify-content: center;
  align-items: center;
}

button {
  margin: 0 5px;
}
</style>
