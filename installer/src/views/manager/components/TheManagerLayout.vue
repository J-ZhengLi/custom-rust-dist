<script setup lang="ts">
import { computed, onBeforeMount, ref } from 'vue';
import { invokeCommand, managerConf } from '@/utils';
import { useCustomRouter } from '@/router';

const { isBack } = useCustomRouter();

const transitionName = computed(() => {
  if (isBack.value === true) return 'back';
  if (isBack.value === false) return 'push';
  return '';
});

const footerLabel = ref('');
invokeCommand('footer_label').then((f) => {
  if (typeof f === 'string') {
    footerLabel.value = f;
  }
});

onBeforeMount(() => managerConf.loadConf());
</script>

<template>
  <main absolute top="0" bottom="0" left="0" right="0" overflow-hidden>
    <router-view v-slot="{ Component }">
      <transition :name="transitionName">
        <keep-alive>
          <component :is="Component" absolute w="full" style="height: calc(100% - 2rem)" />
        </keep-alive>
      </transition>
    </router-view>
    <footer absolute bottom="0" right="2rem" c="regular">
      {{ footerLabel }}
    </footer>
  </main>
</template>
