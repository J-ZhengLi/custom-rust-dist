import { ref, Ref } from 'vue';
import type { Component, TauriComponent } from './types/Component';
import { invokeCommand } from './invokeCommand';

class InstallConf {
  path: Ref<string>;
  checkComponents: Ref<CheckItem<Component>[]>;
  isCustomInstall: boolean;

  constructor(path: string, components: CheckItem<Component>[]) {
    this.path = ref(path);
    this.checkComponents = ref(components);
    this.isCustomInstall = false;
  }

  setPath(newPath: string) {
    if (newPath === this.path.value || newPath === '') {
      return;
    }
    this.path.value = newPath;
    localStorage.setItem('installPath', newPath);
  }

  setComponents(newComponents: CheckItem<Component>[]) {
    const length = this.checkComponents.value.length;
    this.checkComponents.value.splice(0, length, ...newComponents);
  }

  setCustomInstall(isCustomInstall: boolean) {
    this.isCustomInstall = isCustomInstall;
  }

  getGroups(): CheckGroup<Component>[] {
    const groups = this.checkComponents.value.reduce(
      (acc, item) => {
        const groupName = item.value.groupName
          ? item.value.groupName
          : 'Others'; // 确保 groupName 不为 null

        if (!acc[groupName]) {
          acc[groupName] = {
            label: groupName,
            items: [],
          };
        }

        acc[groupName].items.push({ ...item, selected: false });

        return acc;
      },
      {} as Record<string, CheckGroup<Component>>
    );
    return Object.values(groups);
  }

  getCheckedComponents(): TauriComponent[] {
    return this.checkComponents.value
      .filter((i) => i.checked) // 筛选选中组件
      .map((item: CheckItem<Component>) => {
        const { groupName, isToolchainComponent, desc, ...rest } = item.value;
        return {
          ...rest,
          desc: desc.join(''),
          group_name: groupName,
          is_toolchain_component: isToolchainComponent,
        };
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
    const componentList = (await invokeCommand(
      'get_component_list'
    )) as TauriComponent[];
    if (Array.isArray(componentList)) {
      componentList.sort((a, b) => {
        if (a.required && !b.required) {
          return -1;
        }
        if (!a.required && b.required) {
          return 1;
        }

        if (a.group_name === null && b.group_name !== null) {
          return 1;
        }
        if (a.group_name !== null && b.group_name === null) {
          return -1;
        }
        return 0;
      });

      const newComponents: CheckItem<Component>[] = componentList.map(
        (item) => {
          const { group_name, is_toolchain_component, desc, ...rest } = item;
          return {
            label: item.name,
            checked: item.required,
            value: {
              ...rest,
              desc: item.desc.split('\n'),
              groupName: group_name,
              isToolchainComponent: is_toolchain_component,
            },
          };
        }
      );

      this.setComponents(newComponents);
    }
  }

  async loadAll() {
    await this.loadPath();
    await this.loadComponents();
  }
}

export const installConf = new InstallConf('', []);
