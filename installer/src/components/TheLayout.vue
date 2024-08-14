<script setup lang="ts">
import { computed } from 'vue';
import { useCustomRouter } from '../router';
import { useRoute } from 'vue-router';
import TheAside from './TheAside.vue';

const route = useRoute();
const isBack = computed(() => useCustomRouter().isBack);
const isHome = computed(() => route.path === '/');

const transitionName = computed(() => {
  if (isBack.value.value === true) return 'back';
  if (isBack.value.value === false) return 'push';
  return '';
});
</script>

<template>
  <div flex="~ items-stretch" absolute top-0 left-0 bottom-0 right-0>
    <transition name="aside">
      <aside
        v-if="!isHome"
        w="150px"
        h="full"
        b="t-0 r-1 b-0 l-0 solid base"
        absolute
        bg-back
        z-1
      >
        <the-aside />
      </aside>
    </transition>
    <main
      flex="1"
      overflow="hidden"
      absolute
      top="0"
      right="0"
      bottom="0"
      :style="{ left: isHome ? '0' : '150px' }"
    >
      <div h-full relative>
        <router-view v-slot="{ Component }">
          <transition :name="transitionName">
            <keep-alive>
              <component :is="Component" absolute w="full" h="full" />
            </keep-alive>
          </transition>
        </router-view>
      </div>
    </main>
  </div>
</template>

<style scoped>
.aside-enter-active,
.aside-leave-active,
.push-enter-active,
.push-leave-active,
.back-enter-active,
.back-leave-active {
  transition: all 0.5s ease;
}
/* 菜单进出 */
.aside-enter-from {
  left: -150px;
}

.aside-enter-to {
  left: 0;
}

.aside-leave-from {
  left: 0;
}

.aside-leave-to {
  left: -150px;
}
/* 页面前进 */
.push-enter-from {
  right: -100%;
  opacity: 0.5;
}

.push-enter-to {
  right: 0;
  opacity: 1;
}

.push-leave-from {
  left: 0;
  opacity: 1;
}

.push-leave-to {
  left: -100%;
  opacity: 0;
}
/* 页面返回 */
.back-enter-from {
  left: -100%;
  opacity: 0.5;
}

.back-enter-to {
  left: 0;
  opacity: 1;
}

.back-leave-from {
  right: 0;
  opacity: 1;
}

.back-leave-to {
  right: -100%;
  opacity: 0;
}
</style>
