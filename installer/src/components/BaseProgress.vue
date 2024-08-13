<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps({
  percentage: {
    type: Number,
    required: true,
    validator: (value: number) => value >= 0 && value <= 100,
  },
  format: {
    type: Function,
  },
  striped: {
    type: Boolean,
    default: false,
  },
  stripedFlow: {
    type: Boolean,
    default: false,
  },
  duration: {
    type: Number,
    default: 8000,
  },
});

const progressStyle = computed(() => {
  return {
    width: props.percentage + '%',
    animationDuration: props.duration + 'ms',
    animationPlayState: props.stripedFlow ? 'running' : 'paused',
    backgroundImage: props.striped
      ? `linear-gradient(
        45deg,
        rgba(255, 255, 255, 0.1) 25%,
        transparent 25%,
        transparent 50%,
        rgba(255, 255, 255, 0.1) 50%,
        rgba(255, 255, 255, 0.1) 75%,
        transparent 75%,
        transparent
      )`
      : 'linear-gradient(to right, rgba(0, 0, 0, 0.1), transparent)',
  };
});
</script>

<template>
  <div flex="~ items-center justify-between">
    <div class="progress" bg-disabled>
      <div class="progress-bar" bg-primary :style="{ ...progressStyle }"></div>
    </div>
    <div v-if="format" text-end w="5em">{{ format(percentage) }}</div>
  </div>
</template>

<style scoped>
.progress {
  width: 100%;
  height: 20px;
  border-radius: 10px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  border-radius: 10px;
  transition: width 0.3s ease;
  background-size: 1.25em 1.25em;
  animation: striped 3s linear infinite;
  animation: striped-flow 1s linear infinite;
}

@keyframes striped-flow {
  0% {
    background-position: -100%;
  }

  100% {
    background-position: 100%;
  }
}
</style>
