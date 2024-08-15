import {
  createWebHashHistory,
  createRouter,
  useRouter,
  RouteLocationRaw,
  RouteLocationAsPath,
} from 'vue-router';

import HomeView from '../view/HomeView.vue';
import ExplainView from '../view/ExplainView.vue';
import FolderView from '../view/FolderView.vue';
import ComponentsView from '../view/ComponentsView.vue';
import ConfirmView from '../view/ConfirmView.vue';
import InstallView from '../view/InstallView.vue';
import FinishView from '../view/FinishView.vue';
import { ref } from 'vue';

const routes = [
  {
    name: 'Home',
    path: '/',
    component: HomeView,
    meta: { title: '开始', order: 0 },
  },
  {
    name: 'Explain',
    path: '/explain',
    component: ExplainView,
    meta: { title: '说明', order: 1 },
  },
  {
    name: 'Folder',
    path: '/folder',
    component: FolderView,
    meta: { title: '安装位置', order: 2 },
  },
  {
    name: 'Components',
    path: '/components',
    component: ComponentsView,
    meta: { title: '选择组件', order: 3 },
  },
  {
    name: 'Confirm',
    path: '/confirm',
    component: ConfirmView,
    meta: { title: '确认信息', order: 4 },
  },
  {
    name: 'Install',
    path: '/install',
    component: InstallView,
    meta: { title: '安装开始', order: 5 },
  },
  {
    name: 'Finish',
    path: '/finish',
    component: FinishView,
    meta: { title: '安装完成', order: 6 },
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

const isBack = ref();
// 为路由添加前进后退标识
export function useCustomRouter() {
  const newRouter = useRouter();

  function routerPush(path: RouteLocationRaw) {
    isBack.value = false;
    newRouter.push(path);
  }
  function routerBack() {
    isBack.value = true;
    newRouter.back();
  }

  return { isBack, routerPush, routerBack };
}
