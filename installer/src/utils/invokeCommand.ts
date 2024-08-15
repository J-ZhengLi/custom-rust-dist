import { invoke } from '@tauri-apps/api';
import { message } from '@tauri-apps/api/dialog';

// 使用 message invoke 显示错误信息
export async function invokeCommand(command: string, args = {}) {
  try {
    return await invoke(command, args);
  } catch (error: any) {
    // 捕获错误并显示对话框
    await message(error.message || '发生了一个错误', {
      title: '错误',
      type: 'error',
    });
    throw error; // 重新抛出错误以便外部的 .catch 继续处理
  }
}
