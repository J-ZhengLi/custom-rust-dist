<script setup lang="ts">
import { computed } from 'vue';

const { label, oldVer, newVer } = defineProps<{
  label: string;
  oldVer?: string;
  newVer?: string;
}>();
const showVersionChange = computed(() => oldVer && newVer && oldVer !== newVer);
const showSameVersion = computed(() => oldVer && newVer && oldVer === newVer);
const showNewVersion = computed(
  () => (oldVer && !newVer) || (!oldVer && newVer)
);
const isNewerVersion = computed(() => {
  const oldVers = oldVer?.split('.');
  const newVers = newVer?.split('.');
  if (!oldVers || !newVers) return false;
  for (let i = 0; i < 3; i++) {
    if (parseInt(oldVers[i]) < parseInt(newVers[i])) return true;
    if (parseInt(oldVers[i]) > parseInt(newVers[i])) return false;
  }
  return false;
});
</script>

<template>
  <slot>
    <span mr="1rem">{{ label }}</span>
    <span v-if="showVersionChange">
      <base-tag size="small">{{ oldVer }}</base-tag>
      <i
        class="i-mdi:arrow-right-thin w-[1.5em] h-[1.5em]"
        mx="4px"
        align="middle"
      />
      <base-tag v-if="isNewerVersion" type="success" size="small">{{
        newVer
      }}</base-tag>
      <base-tag v-else type="warning" size="small">{{ newVer }}</base-tag>
    </span>
    <span v-else-if="showSameVersion">
      <base-tag size="small">{{ oldVer }}</base-tag>
      <i
        class="i-mdi:arrow-right-thin w-[1.5em] h-[1.5em]"
        mx="4px"
        align="middle"
      />
      <base-tag size="small">{{ newVer }}</base-tag>
    </span>
    <span v-else-if="showNewVersion">
      <base-tag type="success" size="small">new {{ newVer }}</base-tag>
    </span>
  </slot>
</template>
