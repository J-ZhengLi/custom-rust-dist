<script setup lang="ts">
import MdiCheck from './icons/MdiCheck.vue';

const { title, disabled } = defineProps({
  title: String,
  disabled: Boolean,
});

const emit = defineEmits(['titleClick']);

const isChecked = defineModel<boolean>();

const toggleCheck = () => {
  if (disabled) {
    return;
  }

  isChecked.value = !isChecked.value;
};

function titleClick() {
  emit('titleClick');
}
</script>

<template>
  <label
    flex="inline items-center"
    :class="{ 'opacity-60': disabled }"
    :title="title"
  >
    <span
      flex="~ items-center justify-center"
      w="1rem"
      h="1rem"
      b="1px solid base"
      shrink="0"
      rounded="2px"
      bg="white"
      :class="{
        'bg-active border-active': isChecked,
        'bg-disabled-bg': disabled,
        'hover:b-active': !isChecked && !disabled,
      }"
      @click="toggleCheck"
    >
      <slot name="icon">
        <mdi-check v-if="isChecked" c="active" />
      </slot>
    </span>
    <span ml="4px" line-clamp="1" @click="titleClick">
      <slot>{{ title }}</slot>
    </span>
  </label>
</template>
