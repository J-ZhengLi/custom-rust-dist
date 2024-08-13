import { createWebHashHistory, createRouter, useRouter } from 'vue-router';

import HomeView from '../view/HomeView.vue';
import ExplainView from '../view/ExplainView.vue';
import ComponentsView from '../view/ComponentsView.vue';
import InstallView from '../view/InstallView.vue';
import FinishView from '../view/FinishView.vue';
import { ref } from 'vue';

const routes = [
  {
    path: '/',
    component: HomeView,
    meta: { title: '开始', order: 0 },
  },
  {
    path: '/explain',
    component: ExplainView,
    meta: { title: '说明', order: 1 },
  },
  {
    path: '/components',
    component: ComponentsView,
    meta: { title: '配置', order: 2 },
  },
  {
    path: '/install',
    component: InstallView,
    meta: { title: '安装', order: 3 },
  },
  {
    path: '/finish',
    component: FinishView,
    meta: { title: '完成', order: 4 },
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

  function routerPush(path: string) {
    isBack.value = false;
    newRouter.push(path);
  }
  function routerBack() {
    isBack.value = true;
    newRouter.back();
  }

  return { isBack, routerPush, routerBack };
}
