<script setup lang="ts">
import { computed, onBeforeMount } from 'vue';
import { managerConf } from '@/utils';
import { useCustomRouter } from '@/router';

const { isBack } = useCustomRouter();

const transitionName = computed(() => {
  if (isBack.value === true) return 'back';
  if (isBack.value === false) return 'push';
  return '';
});

onBeforeMount(() => managerConf.loadConf());
</script>

<template>
  <main absolute top="0" bottom="0" left="0" right="0" overflow-hidden>
    <router-view v-slot="{ Component }">
      <transition :name="transitionName">
        <keep-alive>
          <component
            :is="Component"
            absolute
            w="full"
            style="height: calc(100% - 2rem)"
          />
        </keep-alive>
      </transition>
    </router-view>
    <footer absolute bottom="0" right="2rem" c="regular">
      管理工具版本 0.0.0
    </footer>
  </main>
</template>
