import { ref, Ref } from 'vue';
import { Component } from './types/Component';
import { invokeCommand } from './invokeCommand';

class InstallConf {
  path: Ref<string>;
  components: Ref<
    {
      id: number;
      name: string;
      desc: string[];
      required: boolean;
      checked: boolean;
    }[]
  >;
  isCustomInstall: boolean;

  constructor(path: string, components: Component[]) {
    this.path = ref(path);
    this.components = ref(components);
    this.isCustomInstall = false;
  }
  setPath(newPath: string) {
    if (newPath === this.path.value || newPath === '') {
      return;
    }
    this.path.value = newPath;
    localStorage.setItem('installPath', newPath);
  }
  setComponents(newComponents: Component[]) {
    const length = this.components.value.length;
    this.components.value.splice(0, length, ...newComponents);
    this.components.value.sort(
      (a, b) => Number(b.required) - Number(a.required)
    );
  }

  setCustomInstall(isCustomInstall: boolean) {
    this.isCustomInstall = isCustomInstall;
  }

  getCheckedComponents() {
    return this.components.value
      .filter((i) => i.checked) // 筛选选中组件
      .map((item: Component) => {
        return { ...item, desc: item.desc.join(''), checked: undefined };
      });
  }

  async loadPath() {
    const localPath = localStorage.getItem('installPath');
    if (localPath !== null) {
      this.setPath(localPath);
      return;
    }

    const defaultPath = await invokeCommand('default_install_dir');
    if (typeof defaultPath === 'string' && defaultPath.trim() !== '') {
      localStorage.setItem('installPath', defaultPath);
      this.setPath(defaultPath);
    }
  }

  async loadComponents() {
    const componentList = await invokeCommand('get_component_list');
    if (Array.isArray(componentList)) {
      const newComponents = componentList.map((item) => {
        return {
          ...item,
          desc: item.desc.split('\n'),
          checked: item.required,
        };
      });
      this.setComponents(newComponents);
    }
  }

  loadAll() {
    this.loadPath();
    this.loadComponents();
  }
}

export const installConf = new InstallConf('', []);
