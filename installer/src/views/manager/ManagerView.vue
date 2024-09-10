<script setup lang="ts">
import { managerConf } from '@/utils';
import KitCard from './components/KitCard.vue';
import { computed } from 'vue';

const kits = computed(() => managerConf.getKits());
</script>

<template>
  <section overflow-auto>
    <h1 mx="12px">更新和卸载</h1>
    <kit-card
      v-for="(kit, index) in kits"
      :key="kit.name"
      :kit="kit"
      :installed="index === 0"
      mt="1rem"
    ></kit-card>
  </section>
</template>
