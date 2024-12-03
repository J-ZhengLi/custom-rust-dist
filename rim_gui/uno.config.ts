// uno.config.ts
import {
  defineConfig,
  presetAttributify,
  presetIcons,
  presetTypography,
  presetUno,
  presetWebFonts,
  transformerDirectives,
  transformerVariantGroup,
} from 'unocss';

export default defineConfig({
  shortcuts: [
    // ...
  ],

  theme: {
    colors: {
      // 背景色
      back: '#f5f5f5',
      'disabled-bg': '#eeeeee',

      // 主色
      primary: '#526ecc',
      success: '#50d4ab',
      warning: '#fbb175',
      danger: '#f66f6a',
      info: '#909399',
      'light-primary': '#788ed7',
      'deep-primary': '#40569f',

      // 文字色相关
      header: '#252b3a',
      regular: '#575d6c',
      secondary: '#8a8e99',
      placeholder: '#adb0b8',
      disabled: '#c0c4cc',
      reverse: '#ffffff',
      active: '#5e7ce0',

      // 边框色
      base: '#adb0b8',
      light: '#dfe1e6',
      lighter: '#ebeef5',
      'extra-light': '#f2f6fc',
      dark: '#d4d7de',
      darker: '#cdd0d6',
      gold: '#d5bc7B',
    },
  },
  presets: [
    presetUno(),
    presetAttributify(),
    presetIcons({
      extraProperties: { display: 'inline-block', "vertical-align": 'middle' },
    }),
    presetTypography(),
    presetWebFonts({
      fonts: {
        // ...
      },
    }),
  ],
  transformers: [
    transformerDirectives({
      applyVariable: ['--uno'],
    }),
    transformerVariantGroup(),
  ],
});
