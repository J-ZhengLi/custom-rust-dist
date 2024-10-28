<script setup lang="ts">
import { managerConf } from '@/utils';
import KitCard from './components/KitCard.vue';
import { computed } from 'vue';
import Pagination from '@/components/Pagination.vue';
import { usePagination } from '@/utils/pagination';

const installedKit = computed(() => managerConf.getInstalled())
const kits = computed(() => managerConf.getKits());
const { current, size, total, list } = usePagination({
  data: kits.value,
  size: 6,
});
</script>

<template>
  <h2 mx="12px">更新和卸载</h2>
  <h3 mx="12px">已安装</h3>
  <kit-card
    v-if="installedKit"
    :key="installedKit.name"
    :kit="installedKit"
    :installed="true"
    mt="1rem"
  ></kit-card>
  <section overflow-auto flex="~ col">
    <h3 mx="12px" v-if="kits.length > 0">可用版本</h3>
    <kit-card
      v-for="kit in list"
      :key="kit.name"
      :kit="kit"
      :installed="false"
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
