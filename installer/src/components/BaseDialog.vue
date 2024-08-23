<script setup lang="ts">
// base-dialog
import { ref } from 'vue';
import MdiClose from './icons/MdiClose.vue';

const { title, width, height } = defineProps({
  title: {
    type: String,
    default: '',
  },
  width: {
    type: String,
    default: '50%',
  },
  height: {
    type: String,
    default: 'auto',
  },
  closeButton: {
    type: Boolean,
    default: false,
  },
});
const visible = defineModel();

const dialogStyle = ref({
  width: width,
  height: height,
});

const emit = defineEmits(['update:modelValue']);

const close = () => {
  visible.value = false;
  emit('update:modelValue', false);
};
</script>
<template>
  <div v-if="visible" fixed w="full" h="full" bg="black op-30" @click="close">
    <div
      v-if="visible"
      :style="{ ...dialogStyle }"
      z="1"
      bg="back"
      absolute
      top="50%"
      left="50%"
      p="12px"
      rounded="12px"
      transform="translate--50%"
      overflow="y-auto"
      flex="~ col"
      max-h="90%"
    >
      <div>
        <h3 mt="0">{{ title }}</h3>
        <mdi-close
          v-if="closeButton"
          h="1.2rem"
          w="1.2rem"
          absolute
          top="8px"
          right="8px"
          @click="close"
          cursor-pointer
          c="regular hover:active"
        />
      </div>
      <slot flex="1" overflow="y-auto"></slot>
      <slot name="footer"></slot>
    </div>
  </div>
</template>
