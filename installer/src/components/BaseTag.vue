<script setup lang="ts">
withDefaults(
  defineProps<{
    type?: 'success' | 'warning' | 'danger' | 'info' | 'default' | 'primary';
    size?: 'small' | 'medium' | 'large';
    rounded?: boolean;
  }>(),
  { type: 'default', size: 'medium', rounded: false }
);
</script>

<template>
  <span class="tag" :class="`${type} ${size} ${rounded ? 'rounded' : ''}`">
    <slot />
  </span>
</template>

<style scoped>
.tag {
  display: inline-block;
  font-weight: 500;
  line-height: 1.25rem;
  color: #fff;
  border-radius: 0.25rem;
  vertical-align: middle;
}
.tag.default {
  --uno: bg-regular;
}
.tag.primary {
  --uno: bg-primary;
}
.tag.info {
  --uno: bg-secondary;
}
.tag.success {
  background-color: #10b981;
}
.tag.danger {
  background-color: #ef4444;
}
.tag.warning {
  background-color: #f59e0b;
}
.tag.small {
  font-size: 0.75rem;
  padding: 0.125rem 0.25rem;
}
.tag.large {
  padding: 0.5rem 1rem;
  font-size: 1rem;
}
.tag.medium {
  padding: 0.25rem 0.5rem;
  font-size: 0.875rem;
}
.tag.rounded {
  border-radius: 9999px;
}
</style>
