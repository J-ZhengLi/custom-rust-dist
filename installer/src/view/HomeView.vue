<script lang="ts" setup>
import { useCustomRouter } from '../router';
import { invokeCommand } from '../utils';

const { routerPush } = useCustomRouter();

function handleBackClick() {
  invokeCommand('close_window');
}
</script>

<template>
  <div flex="~ col items-center" w="full">
    <div grow="2">
      <a href="https://xuanwu.beta.atomgit.com/" target="_blank">
        <img
          src="/logo.svg"
          class="logo xuanwu"
          alt="Xuanwu logo"
          w="200px"
          mt="50%"
        />
      </a>
    </div>
    <div grow="2">
      <h1>欢迎使用玄武 Rust 一键式安装工具</h1>
    </div>
    <div basis="80px" w="full" flex="~ items-center justify-end">
      <base-button mr="12px" @click="handleBackClick"> 关闭</base-button>
      <base-button mr="12px" @click="() => routerPush('/explain')"
        >开始安装</base-button
      >
    </div>
  </div>
</template>
