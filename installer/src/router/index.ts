import {
  createWebHashHistory,
  createRouter,
  useRouter,
  RouteLocationRaw,
} from 'vue-router';
import { Ref, ref } from 'vue';

import TheInstallerLayout from '@/views/installer/components/TheInstallerLayout.vue';
import HomeView from '@/views/installer/HomeView.vue';
import FolderView from '@/views/installer/FolderView.vue';
import ComponentsView from '@/views/installer/ComponentsView.vue';
import ConfirmView from '@/views/installer/ConfirmView.vue';
import InstallView from '@/views/installer/InstallView.vue';
import FinishView from '@/views/installer/FinishView.vue';

import TheManagerLayout from '@/views/manager/components/TheManagerLayout.vue';
import ManagerView from '@/views/manager/ManagerView.vue';
import ManagerComponentsView from '@/views/manager/ComponentsView.vue';
import ManagerConfirmView from '@/views/manager/ConfirmView.vue';
import UninstallView from '@/views/manager/UninstallView.vue';
import ProgressView from '@/views/manager/ProgressView.vue';
import CompleteView from '@/views/manager/CompleteView.vue';

const routes = [
  {
    name: 'Installer',
    path: '/installer',
    component: TheInstallerLayout,
    children: [
      {
        name: 'Home',
        path: '',
        component: HomeView,
        meta: { title: '开始', order: 0, required: true },
      },
      {
        name: 'Folder',
        path: 'folder',
        component: FolderView,
        meta: { title: '安装位置', order: 1, required: false },
      },
      {
        name: 'Components',
        path: 'components',
        component: ComponentsView,
        meta: { title: '组件选项', order: 2, required: false },
      },
      {
        name: 'Confirm',
        path: 'confirm',
        component: ConfirmView,
        meta: { title: '信息确认', order: 3, required: true },
      },
      {
        name: 'Install',
        path: 'install',
        component: InstallView,
        meta: { title: '进行安装', order: 4, required: true },
      },
      {
        name: 'Finish',
        path: 'finish',
        component: FinishView,
        meta: { title: '安装完成', order: 5, required: true },
      },
    ],
  },
  {
    name: 'Manager',
    path: '/manager',
    component: TheManagerLayout,
    children: [
      {
        name: 'ManagerHome',
        path: '',
        component: ManagerView,
        meta: { title: '管理', order: 0, required: true },
      },
      {
        name: 'Change',
        path: 'change',
        component: ManagerComponentsView,
        meta: { title: '修改配置', order: 1, required: false },
      },
      {
        name: 'ManagerConfirm',
        path: 'confirm',
        component: ManagerConfirmView,
        meta: { title: '信息确认', order: 2, required: true },
      },
      {
        name: 'Uninstall',
        path: 'uninstall',
        component: UninstallView,
        meta: { title: '卸载', order: 2, required: false },
      },
      {
        name: 'Progress',
        path: 'progress',
        component: ProgressView,
        meta: { title: '处理', order: 3, required: true },
      },
      {
        name: 'Complete',
        path: 'complete',
        component: CompleteView,
        meta: { title: '卸载完成', order: 4, required: true },
      },
    ],
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

const isBack: Ref<boolean | null> = ref(null);
// 为路由添加前进后退标识
export function useCustomRouter() {
  const newRouter = useRouter();

  function routerPush(path: RouteLocationRaw) {
    isBack.value = false;
    newRouter.push(path);
  }
  function routerBack(deep: number = -1) {
    isBack.value = true;
    if (typeof deep === 'number') newRouter.go(deep);
    else newRouter.back();
  }

  return { isBack, routerPush, routerBack };
}
