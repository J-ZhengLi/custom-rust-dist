import { ref, Ref } from 'vue';
import type { Component, TauriComponent } from './types/Component';
import { invokeCommand } from './invokeCommand';
import { CheckGroup, CheckItem } from './types/CheckBoxGroup';

class InstallConf {
  path: Ref<string>;
  checkComponents: Ref<CheckItem<Component>[]>;
  isCustomInstall: boolean;

  constructor(path: string, components: CheckItem<Component>[]) {
    this.path = ref(path);
    this.checkComponents = ref(components);
    this.isCustomInstall = true;
  }

  setPath(newPath: string) {
    if (newPath === this.path.value || newPath === '') {
      return;
    }
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

  getCheckedComponents(): TauriComponent[] {
    return this.checkComponents.value
      .filter((i) => i.checked) // 筛选选中组件
      .map((item: CheckItem<Component>) => {
        const {
          groupName,
          isToolchainComponent,
          toolInstaller,
          desc,
          ...rest
        } = item.value;
        return {
          ...rest,
          desc: desc.join(''),
          group_name: groupName,
          is_toolchain_component: isToolchainComponent,
          tool_installer: toolInstaller,
        };
      });
  }

  async loadPath() {
    const defaultPath = await invokeCommand('default_install_dir');
    if (typeof defaultPath === 'string' && defaultPath.trim() !== '') {
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
        // 名称排序
        return a.name.localeCompare(b.name);
      });

      const newComponents: CheckItem<Component>[] = componentList.map(
        (item) => {
          const {
            group_name,
            is_toolchain_component,
            tool_installer,
            desc,
            ...rest
          } = item;
          return {
            label: `${item.name}${item.installed ? ' (installed)' : item.required ? ' (required)' : ''}`,
            checked: !item.installed && (item.required || !item.optional),
            required: item.required,
            disabled: item.installed ? false : item.required,
            value: {
              ...rest,
              desc: item.desc.split('\n'),
              groupName: group_name,
              isToolchainComponent: is_toolchain_component,
              toolInstaller: tool_installer,
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
