import { ref, Ref, shallowRef } from 'vue';
import { KitItem } from './types/KitItem';
import { ManagerComponent } from './types/Component';
import { CheckGroup, CheckGroupItem } from './types/CheckBoxGroup';
import LabelComponent from '@/views/manager/components/Label.vue';

const kits: KitItem[] = [
  {
    name: '玄武 Rust 安装工具',
    version: '1.70.0',
    desc: '稳定版适用于大多数用户，具有稳定的性能和兼容性。',
    date: '2023-01-01',
    type: 'stable',
    notes: '更新日志',
    manifestURL: 'https://example.com/manifest.json',
    components: [
      {
        desc: ['基础组件，包含核心功能。'],
        groupName: 'Rust',
        id: 1,
        installed: false,
        isToolchainComponent: true,
        name: 'Basic',
        optional: false,
        required: true,
        version: '1.70.0',
        toolInstaller: null,
      },
      {
        desc: ['(windows-msvc only) Requirement for Windows'],
        groupName: 'Prerequisites',
        id: 587,
        installed: true,
        isToolchainComponent: false,
        name: 'buildtools',
        optional: false,
        required: true,
        version: '1.70.0',
        toolInstaller: null,
      },
      {
        desc: [
          'Prints out the result of macro expansion and #[derive] expansion applied to the current crate.',
        ],
        groupName: 'Misc',
        id: 623,
        installed: false,
        isToolchainComponent: false,
        name: 'cargo-expand',
        optional: true,
        required: false,
        version: '1.70.0',
        toolInstaller: null,
      },
    ],
  },
  {
    name: '玄武 Rust 安装工具',
    version: '1.71.0',
    desc: '测试版适用于喜欢尝鲜的用户，可能包含新功能和改进。',
    date: '2023-01-02',
    type: 'beta',
    notes: '更新日志',
    manifestURL: 'https://example.com/manifest.json',
    components: [],
  },
  {
    name: '玄武 Rust 安装工具',
    version: '1.72.0',
    desc: '开发版适用于开发者，可能包含不稳定的功能和改进。',
    date: '2023-01-03',
    type: 'alpha',
    notes: '更新日志',
    manifestURL: 'https://example.com/manifest.json',
    components: [],
  },
  {
    name: '玄武 Rust 安装工具',
    version: '1.73.0',
    desc: '开发版适用于开发者，可能包含不稳定的功能和改进。',
    date: '2023-01-03',
    type: 'alpha',
    notes: '更新日志',
    manifestURL: 'https://example.com/manifest.json',
    components: [],
  },
];

type Target = {
  operation: 'update' | 'uninstall';
  components: ManagerComponent[];
};

class ManagerConf {
  private _kits: Ref<KitItem[]> = ref([]);
  private _installed: Ref<KitItem | null> = ref(null);
  private _current: Ref<KitItem | null> = ref(null);
  private _target: Ref<Target> = ref({ operation: 'update', components: [] });
  private _isUninstallManager: Ref<boolean> = ref(false);

  constructor() {}

  public getUninstallManager() {
    return this._isUninstallManager.value;
  }

  public getKits(): KitItem[] {
    return this._kits.value;
  }

  public getInstalled() {
    return this._installed;
  }

  public getInstalledComponents(): ManagerComponent[] | undefined {
    return this._installed.value?.components;
  }

  public getGroups(): CheckGroup<ManagerComponent>[] {
    const checkItems: CheckGroupItem<ManagerComponent>[] =
      this._current.value?.components.map((item) => {
        const installedItem = this._current.value?.components.find(
          (c) => c.name === item.name
        );

        let versionStr =
          installedItem?.version && installedItem?.version === item.version
            ? `(${installedItem?.version} -> ${item.version})`
            : ` (${item.version})`;

        return {
          label: `${item.name}${versionStr}`,
          checked: item.required || !item.optional,
          required: item.required,
          disabled: item.required,

          focused: false,
          value: item,
          labelComponent: shallowRef(LabelComponent),
          labelComponentProps: {
            label: item.name,
            ver1: installedItem?.version,
            ver2: item.version,
          },
        };
      }) || [];

    const groups = checkItems.reduce(
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

        acc[groupName].items.push({ ...item });

        return acc;
      },
      {} as Record<string, CheckGroup<ManagerComponent>>
    );

    return Object.values(groups);
  }

  public getCurrent() {
    return this._current;
  }

  public getCurrentComponents(): ManagerComponent[] | undefined {
    return this._current.value?.components;
  }

  public getOperation() {
    return this._target.value.operation;
  }

  public getTargetComponents() {
    return this._target.value.components;
  }

  public setUninstallManager(value: boolean) {
    this._isUninstallManager.value = value;
  }

  public setKits(kits: KitItem[]): void {
    this._kits.value.splice(0, this._kits.value.length, ...kits);
  }

  public setInstalled(installed: KitItem): void {
    this._installed.value = installed;
  }

  public setCurrent(current: KitItem): void {
    this._current.value = current;
  }

  public setOperation(operation: Target['operation']): void {
    this._target.value.operation = operation;
  }
  public setComponents(components: Target['components']): void {
    this._target.value.components.splice(
      0,
      this._target.value.components.length,
      ...components
    );
  }
  public loadConf(): any {
    this.setCurrent(kits[0]);
    this.setInstalled(kits[0]);
    this.setKits(kits);
    this.setComponents(
      this.getInstalled().value?.components.filter(
        (i) => i.installed || i.required
      ) || []
    );
  }
}

export const managerConf = new ManagerConf();
