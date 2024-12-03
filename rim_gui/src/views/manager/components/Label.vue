<script setup lang="ts">
import { computed } from 'vue';

const { label, oldVer, newVer } = defineProps<{
  label: string;
  oldVer?: string;
  newVer?: string;
}>();
const isSameVersion = computed(() => oldVer && newVer && oldVer === newVer);
const isNewerVersion = computed(() => {
  const oldVers = oldVer?.split('.');
  const newVers = newVer?.split('.');

  if (!oldVers && newVers) return true;

  if (!oldVers || !newVers) return false;

  for (let i = 0; i < newVers.length; i++) {
    if (parseInt(oldVers[i]) < parseInt(newVers[i])) return true;
    if (parseInt(oldVers[i]) > parseInt(newVers[i])) return false;
  }
  return false;
});

const showSingleVer = computed(() => oldVer && !newVer);
</script>

<template>
  <slot>
    <span v-if="showSingleVer" v-bind="$attrs">
      <base-tag min-w="5em" text="center" size="small">{{
        oldVer ? oldVer : '--'
      }}</base-tag>
    </span>
    <span v-else v-bind="$attrs">
      <base-tag min-w="5em" text="center" size="small">{{
        oldVer ? oldVer : '--'
      }}</base-tag>
      <i class="i-mdi:arrow-right-thin w-[1.5em] h-[1.5em]" align="middle" />
      <base-tag
        min-w="5em"
        text="center"
        v-if="isNewerVersion"
        type="success"
        size="small"
        >{{ newVer }}</base-tag
      >
      <base-tag
        v-else-if="isSameVersion"
        min-w="5em"
        text="center"
        size="small"
        >{{ newVer }}</base-tag
      >
      <base-tag v-else min-w="5em" text="center" type="warning" size="small">{{
        newVer
      }}</base-tag>
    </span>
    <span ml="0.5rem">{{ label }}</span>
  </slot>
</template>
