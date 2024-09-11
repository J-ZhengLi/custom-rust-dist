<script setup lang="ts">
import { computed, defineProps } from 'vue';

// Define props for the component
const props = defineProps({
  theme: {
    type: String,
    default: 'default', // Default theme
  },
  disabled: {
    type: Boolean,
    default: false, // Button is enabled by default
  },
});

// Computed class for dynamic theme application
const themeClasses = computed(() => {
  switch (props.theme) {
    case 'primary':
      return 'bg-primary text-white border-primary active:bg-deep-primary';
    case 'default':
      return 'bg-gray-200 text-header border-gray-400 active:bg-gray-300';
    // Add more themes as needed
    default:
      return '';
  }
});
</script>

<template>
  <button
    p="x-16px y-8px"
    :class="[
      themeClasses,
      ' rounded-[4px] b b-solid hover:op-80', // Common classes
      { 'opacity-50 cursor-not-allowed': disabled }, // Disabled styles
    ]"
    :disabled="disabled"
  >
    <slot></slot>
  </button>
</template>

<style scoped>
button {
  transition:
    background-color 0.3s,
    border-color 0.3s;
}
</style>
