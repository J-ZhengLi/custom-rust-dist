<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

const menuFirstItem = computed(() => {
  return router.options.routes.find(
    (item) => (item.meta?.order as number) === 0
  );
});
const menu = ref(
  router.options.routes.filter((item) => (item.meta?.order as number) > 0)
);

const beforeTop = computed(() => {
  return {
    '--beforeTop': `${(router.currentRoute.value.meta.order as number) * 150}%`,
  };
});

function menuItemActive(order: number) {
  const isActive = order <= (router.currentRoute.value.meta.order as number);
  const isFocus = order === (router.currentRoute.value.meta.order as number);
  return {
    'c-active': isActive,
    'text-1.2em': isFocus,
  };
}
</script>

<template>
  <div text="end secondary">
    <div
      :style="beforeTop"
      class="activeItem"
      :class="{ ...menuItemActive(menuFirstItem?.meta?.order as number) }"
      relative
      h="24px"
      mt="12px"
      pr="1em"
    >
      {{ menuFirstItem?.meta?.title }}
    </div>
    <div
      v-for="item in menu"
      :key="item.path"
      :class="{ ...menuItemActive(item.meta?.order as number) }"
      h="24px"
      mt="12px"
      pr="1em"
      transition="all 0.3s"
    >
      {{ item.meta?.title }}
    </div>
  </div>
</template>

<style scoped>
.activeItem::before {
  content: '';
  position: absolute;
  right: 0;
  top: 0;
  transform: translateY(var(--beforeTop));
  width: 4px;
  height: 100%;
  transition: all 0.3s;
  --uno: bg-primary;
}
</style>
