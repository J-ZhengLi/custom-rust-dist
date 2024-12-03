<template>
  <div flex="~ items-center">
    <input
      type="radio"
      :name="name"
      :value="value"
      :checked="value === modelValue"
      @input="$emit('update:modelValue', value)"
      b="hover:active"
      cursor="pointer"
    />
    <span
      class="custom-radio"
      @click="$emit('update:modelValue', value)"
    ></span>
    <label>{{ label }}</label>
  </div>
</template>

<script setup lang="ts">
const { modelValue, name, value, label } = defineProps({
  modelValue: {
    type: [Boolean, String, Number],
    required: false,
  },
  name: {
    type: String,
    required: true,
  },
  value: {
    type: [Boolean, String, Number],
    required: true,
  },
  label: {
    type: String,
    required: true,
  },
});
</script>

<style scoped>
.custom-radio {
  display: inline-block;
  width: 1rem;
  height: 1rem;
  border-radius: 50%;
  margin-right: 0.5em;
  position: relative;
  cursor: pointer;
  --uno: b-1 b-base b-solid;
}
.custom-radio:checked {
  --uno: b-active;
}
input[type='radio'] {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

/* 使用伪元素表示选中的状态 */
input[type='radio']:checked + .custom-radio::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  width: 0.7rem;
  height: 0.7rem;
  border-radius: 50%;
  transform: translate(-50%, -50%);
  --uno: bg-active;
}
</style>
