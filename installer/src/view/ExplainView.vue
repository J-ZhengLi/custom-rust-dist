<script setup lang="ts">
import { ref } from 'vue';
import { message } from '@tauri-apps/api/dialog';
import { useCustomRouter } from '../router';
import ScrollBox from '../components/ScrollBox.vue';

const { routerPush, routerBack } = useCustomRouter();

const agree = ref();

const explainText: string[] = `通过安装 Rustup，您同意以下条款：
1. 软件许可：Rustup 由 Rust 社区开发，并遵循 MIT 和 Apache 2.0 许可协议。您可以自由使用、修改和分发 Rustup，但必须遵循相应的许可证条款。
2. 责任限制：Rustup 是按“现状”提供的，不提供任何形式的明示或暗示保证。对于因使用 Rustup 导致的任何直接或间接损失，Rust 社区不承担责任。
3. 更新与维护：Rustup 将定期发布更新，您同意在安装后接受这些更新。您可以选择不更新，但这可能会影响 Rustup 的功能和安全性。
4. 隐私政策：Rustup 可能会收集和使用一些匿名数据以改善用户体验。具体的隐私政策请参见官方文档。
5. 适用法律：本条款受适用法律管辖，任何争议将提交至相关法律机构解决。`.split(
  '\n'
);

function handleNextClick() {
  if (agree.value) {
    routerPush('/folder');
  } else {
    message('请先同意许可协议', { title: '提示' });
  }
}
</script>

<template>
  <div flex="~ col" w="full">
    <div ml="12px">
      <h4 mb="4px">许可协议</h4>
      <p mt="0">继续安装前请阅读以下重要信息。</p>
      <p mb="4px">
        请仔细阅读下列许可协议，您必须在继续安装前同意这些协议条款。
      </p>
    </div>
    <scroll-box flex="1" mx="12px" overflow="auto">
      <p v-for="txt in explainText" :key="txt">
        {{ txt }}
      </p>
    </scroll-box>
    <div ml="12px">
      <base-radio
        v-model="agree"
        :value="true"
        name="agree"
        label="我同意此协议"
      />
      <base-radio
        v-model="agree"
        :value="false"
        name="agree"
        label="我不同意此协议"
      />
    </div>
    <div basis="60px" flex="~ items-center justify-end">
      <base-button @click="routerBack" mr="12px">上一步</base-button>
      <base-button @click="handleNextClick" mr="12px">下一步</base-button>
    </div>
  </div>
</template>
