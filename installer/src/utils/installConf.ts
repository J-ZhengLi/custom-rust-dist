import { ref, Ref } from 'vue';
import { Component } from './types/Component';

export const installConf: Ref<{ path: string; components: Component[] }> = ref({
  path: '',
  components: [],
});
