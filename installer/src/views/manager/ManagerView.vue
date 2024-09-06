<script setup lang="ts">
import { KitItem } from '@/utils/index';
import KitCard from '@/components/KitCard.vue';

const kits: KitItem[] = [
  {
    name: '玄武 Rust 安装工具',
    version: '1.70.0',
    desc: '稳定版适用于大多数用户，具有稳定的性能和兼容性。',
    date: '2023-01-01',
    type: 'stable',
    notes: '更新日志',
    manifestURL: 'https://example.com/manifest.json',
    components: [],
  },
  {
    name: '玄武 Rust 安装工具',
    version: '1.71.0',
    desc: '测试版适用于喜欢尝鲜的用户，可能包含新功能和改进。',
    date: '2023-01-02',
    type: 'beta',
    notes: '',
    manifestURL: 'https://example.com/manifest.json',
    components: [],
  },
  {
    name: '玄武 Rust 安装工具',
    version: '1.72.0',
    desc: '开发版适用于开发者，可能包含不稳定的功能和改进。',
    date: '2023-01-03',
    type: 'alpha',
    notes: '',
    manifestURL: 'https://example.com/manifest.json',
    components: [],
  },
  {
    name: '玄武 Rust 安装工具',
    version: '1.73.0',
    desc: '开发版适用于开发者，可能包含不稳定的功能和改进。',
    date: '2023-01-03',
    type: 'alpha',
    notes: '',
    manifestURL: 'https://example.com/manifest.json',
    components: [],
  },
];
</script>

<template>
  <section overflow-auto>
    <h1 mx="12px">更新和卸载</h1>
    <kit-card
      v-for="(kit, index) in kits"
      :key="kit.name"
      :kit="kit"
      :installed="index === 0"
      mt="1rem"
    ></kit-card>
  </section>
</template>
