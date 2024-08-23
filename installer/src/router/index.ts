import {
  createWebHashHistory,
  createRouter,
  useRouter,
  RouteLocationRaw,
} from 'vue-router';
import { ref } from 'vue';
import HomeView from '../view/HomeView.vue';
import FolderView from '../view/FolderView.vue';
import ComponentsView from '../view/ComponentsView.vue';
import ConfirmView from '../view/ConfirmView.vue';
import InstallView from '../view/InstallView.vue';
import FinishView from '../view/FinishView.vue';

const routes = [
  {
    name: 'Home',
    path: '/',
    component: HomeView,
    meta: { title: '开始', order: 0, required: true },
  },
  {
    name: 'Folder',
    path: '/folder',
    component: FolderView,
    meta: { title: '安装位置', order: 1, required: false },
  },
  {
    name: 'Components',
    path: '/components',
    component: ComponentsView,
    meta: { title: '组件选项', order: 2, required: false },
  },
  {
    name: 'Confirm',
    path: '/confirm',
    component: ConfirmView,
    meta: { title: '信息确认', order: 3, required: true },
  },
  {
    name: 'Install',
    path: '/install',
    component: InstallView,
    meta: { title: '进行安装', order: 4, required: true },
  },
  {
    name: 'Finish',
    path: '/finish',
    component: FinishView,
    meta: { title: '安装完成', order: 5, required: true },
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
