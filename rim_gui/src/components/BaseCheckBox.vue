<script setup lang="ts">
const { title, disabled, labelComponent, labelComponentProps } = defineProps<{
  title?: string;
  disabled?: boolean;
  labelComponent?: Object;
  labelComponentProps?: Object;
}>();

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
    :class="{ 'c-secondary': disabled }"
    :title="title"
    cursor-pointer
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
        'cursor-not-allowed': disabled,
      }"
      @click="toggleCheck"
    >
      <slot name="icon">
        <i class="i-mdi:check" v-if="isChecked" c="active" />
      </slot>
    </span>
    <span ml="4px" @click="titleClick" whitespace-nowrap>
      <slot>
        <component
          v-if="labelComponent"
          :is="labelComponent"
          v-bind="labelComponentProps"
        />
        <span v-else>{{ title }}</span>
      </slot>
    </span>
  </label>
</template>
