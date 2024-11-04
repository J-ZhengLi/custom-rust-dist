import { ref, Ref } from 'vue';
import type { Component } from './types/Component';
import { invokeCommand } from './invokeCommand';
import { CheckGroup, CheckItem } from './types/CheckBoxGroup';

class InstallConf {
  path: Ref<string>;
  checkComponents: Ref<CheckItem<Component>[]>;
  isCustomInstall: boolean;
  version: Ref<string>;

  constructor(path: string, components: CheckItem<Component>[]) {
    this.path = ref(path);
    this.checkComponents = ref(components);
    this.isCustomInstall = true;
    this.version = ref('');
  }

  setPath(newPath: string) {
    this.path.value = newPath;
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

        acc[groupName].items.push({ ...item, focused: false });

        return acc;
      },
      {} as Record<string, CheckGroup<Component>>
    );
    return Object.values(groups);
  }

  getCheckedComponents(): Component[] {
    return this.checkComponents.value
      .filter((i) => i.checked) // 筛选选中组件
      .map((item: CheckItem<Component>) => {
        return item.value as Component;
      });
  }

  loadManifest() {
    invokeCommand("load_manifest_and_ret_version").then((ver) => {
      if (typeof ver === 'string') {
        this.version.value = ver;
      }
    });
  }

  async loadDefaultPath() {
    const defaultPath = await invokeCommand('default_install_dir');
    if (typeof defaultPath === 'string' && defaultPath.trim() !== '') {
      this.setPath(defaultPath);
    }
  }

  async loadComponents() {
    const componentList = (await invokeCommand(
      'get_component_list'
    )) as Component[];
    if (Array.isArray(componentList)) {
      componentList.sort((a, b) => {
        if (a.required && !b.required) {
          return -1;
        }
        if (!a.required && b.required) {
          return 1;
        }

        if (a.groupName === null && b.groupName !== null) {
          return 1;
        }
        if (a.groupName !== null && b.groupName === null) {
          return -1;
        }
        // 名称排序
        return a.name.localeCompare(b.name);
      });

      const newComponents: CheckItem<Component>[] = componentList.map(
        (item) => {
          return {
            label: `${item.name}${item.installed ? ' (installed)' : item.required ? ' (required)' : ''}`,
            checked: !item.installed && (item.required || !item.optional),
            required: item.required,
            disabled: item.installed ? false : item.required,
            value: item,
          } as CheckItem<Component>;
        }
      );

      this.setComponents(newComponents);
    }
  }

  async loadAll() {
    await this.loadDefaultPath();
    await this.loadComponents();
  }
}

export const installConf = new InstallConf('', []);
