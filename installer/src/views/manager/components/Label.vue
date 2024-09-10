<script setup lang="ts">
import { computed } from 'vue';

const { label, ver1, ver2 } = defineProps<{
  label: string;
  ver1?: string;
  ver2?: string;
}>();
const showVersionChange = computed(() => ver1 && ver2 && ver1 !== ver2);
const showSameVersion = computed(() => ver1 && ver2 && ver1 === ver2);
const showNewVersion = computed(() => (ver1 && !ver2) || (!ver1 && ver2));
</script>

<template>
  <span>
    <slot
      ><span mr="1rem">{{ label }}</span>
      <span v-if="showVersionChange"> {{ ver1 }} → {{ ver2 }}</span>
      <span v-else-if="showSameVersion"> {{ ver1 }} → {{ ver2 }}</span>
      <span v-else-if="showNewVersion"> {{ ver1 ? ver1 : ver2 }}</span></slot
    >
  </span>
</template>
