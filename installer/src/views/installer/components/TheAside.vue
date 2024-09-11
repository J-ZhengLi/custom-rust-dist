<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { installConf } from '@/utils';

const router = useRouter();

const menuFirstItem = computed(() => {
  const index = router.options.routes.findIndex(
    (route) => route.name === 'Installer'
  );
  return router.options.routes[index].children?.find(
    (item) => (item.meta?.order as number) === 0
  );
});
const menu = computed(() => {
  const index = router.options.routes.findIndex(
    (route) => route.name === 'Installer'
  );
  return (
    router.options.routes[index].children?.filter((item) => {
      const hasValidOrder = (item.meta?.order as number) > 0;

      if (installConf.isCustomInstall) {
        return hasValidOrder;
      }

      return hasValidOrder && item.meta?.required;
    }) || []
  );
});

const beforeTop = computed(() => {
  return {
    '--beforeTop': `${(menu.value.findIndex((i) => `/installer/${i.path}` === router.currentRoute.value.path) + 1) * 150}%`,
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
  <div flex="~ col items-center" h="full">
    <h4 w="full" text="center" m="0" py="12px">安装步骤</h4>
    <div flex="1" text="end secondary" w="full">
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
    <div mb="8px" text="center">
      <img src="/logo.svg" alt="logo" w="90%" />
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
