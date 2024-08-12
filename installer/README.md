# Tauri + Vue + TypeScript

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## 推荐使用IDE和插件

- `VS Code` + `Vue-official` + `Tauri` + `rust-analyzer` + `Prettier-Code formatter` + `UnoCSS/UnoT`

## 开始

**推荐使用pnpm对前端依赖进行管理**
```
npm install pnpm -g
```

**下载前端依赖**
```
cd ./installer
pnpm install
```

**运行**
```
cargo tauri dev
```

**构建**
```
cargo tauri build
```

## 说明
- Vue 和 TypeScript文件使用Prettier进行代码格式化，但依赖中没有包含prettier，可以添加全局包
```
npm install prettier -g
```
或者
```
pnpm add prettier -g
```
嫌麻烦也可以添加到项目依赖中
```
cd ./installer
pnpm add --save-dev --save-exact prettier
```

- `tauri` API 可以从依赖 `@tauri-apps/api` 中引入
```ts
import { invoke } from '@tauri-apps/api';
import { event } from '@tauri-apps/api';
```

- 页面代码在文件夹 `installer/scr/view` 中
```
--install
  --src
    --view
      --HomeView.vue
      ...
```
