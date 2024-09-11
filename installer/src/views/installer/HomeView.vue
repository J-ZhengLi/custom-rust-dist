<script lang="ts" setup>
import { ref } from 'vue';
import { useCustomRouter } from '@/router/index';
import { message } from '@tauri-apps/api/dialog';
import { installConf } from '@/utils/index';

const { routerPush } = useCustomRouter();
const isDialogVisible = ref(false);
const explainText: string[] = `通过安装 Rustup，您同意以下条款：
1. 软件许可：Rustup 由 Rust 社区开发，并遵循 MIT 和 Apache 2.0 许可协议。您可以自由使用、修改和分发 Rustup，但必须遵循相应的许可证条款。
2. 责任限制：Rustup 是按“现状”提供的，不提供任何形式的明示或暗示保证。对于因使用 Rustup 导致的任何直接或间接损失，Rust 社区不承担责任。
3. 更新与维护：Rustup 将定期发布更新，您同意在安装后接受这些更新。您可以选择不更新，但这可能会影响 Rustup 的功能和安全性。
4. 隐私政策：Rustup 可能会收集和使用一些匿名数据以改善用户体验。具体的隐私政策请参见官方文档。
5. 适用法律：本条款受适用法律管辖，任何争议将提交至相关法律机构解决。`.split(
  '\n'
);

const isUserAgree = ref(false);

function handleDialogOk() {
  isDialogVisible.value = false;
  isUserAgree.value = true;
}

function handleInstallClick(custom: boolean) {
  if (isUserAgree.value) {
    installConf.setCustomInstall(custom);
    routerPush(custom ? '/installer/folder' : '/installer/confirm');
  } else {
    message('请先同意许可协议', { title: '提示' });
  }
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
      <h1>欢迎使用玄武 Rust 一站式开发套件</h1>
    </div>
    <div basis="120px" w="full" text="center">
      <div flex="~ items-end justify-center">
        <base-button
          theme="primary"
          w="12rem"
          mx="8px"
          text="1.2rem"
          font="bold"
          @click="handleInstallClick(true)"
          >安装</base-button
        >
      </div>
      <base-check-box v-model="isUserAgree" mt="8px"
        >我同意
        <span
          @click="isDialogVisible = !isDialogVisible"
          c="primary"
          cursor-pointer
          decoration="hover:underline"
          >许可协议</span
        >
      </base-check-box>
    </div>
    <base-dialog v-model="isDialogVisible" title="许可协议" width="80%">
      <scroll-box flex="1" overflow="auto">
        <p v-for="txt in explainText" :key="txt">
          {{ txt }}
        </p>
      </scroll-box>
      <template #footer>
        <div flex="~ items-center justify-end" gap="12px" mt="12px">
          <base-button @click="isDialogVisible = !isDialogVisible"
            >关闭</base-button
          >
          <base-button @click="handleDialogOk">我同意</base-button>
        </div>
      </template>
    </base-dialog>
  </div>
</template>
