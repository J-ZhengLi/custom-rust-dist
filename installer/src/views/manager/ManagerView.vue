<script setup lang="ts">
import { managerConf } from '@/utils';
import KitCard from './components/KitCard.vue';
import { computed } from 'vue';
import Pagination from '@/components/Pagination.vue';
import { usePagination } from '@/utils/pagination';

const kits = computed(() => managerConf.getKits());
const { current, size, total, list } = usePagination({
  data: kits.value,
  size: 6,
});
</script>

<template>
  <section overflow-auto flex="~ col">
    <h1 mx="12px">更新和卸载</h1>
    <kit-card
      v-for="(kit, index) in list"
      :key="kit.name"
      :kit="kit"
      :installed="index === 0"
      mt="1rem"
    ></kit-card>
    <div flex="1"></div>
    <pagination
      :size="size"
      v-model="current"
      :total="total"
      hide-on-one-page
      show-jumper
      my="12px"
    />
  </section>
</template>
