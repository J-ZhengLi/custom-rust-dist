import { ref, Ref, shallowRef } from 'vue';
import { KitItem, OriginKitItem } from './types/KitItem';
import { ManagerComponent } from './types/Component';
import { CheckGroup, CheckGroupItem } from './types/CheckBoxGroup';
import LabelComponent from '@/views/manager/components/Label.vue';
import { invokeCommand } from './invokeCommand';


type Target = {
  operation: 'update' | 'uninstall';
  components: ManagerComponent[];
};

class ManagerConf {
  path: Ref<string> = ref('');
  private _kits: Ref<KitItem[]> = ref([]);
  private _installed: Ref<KitItem | null> = ref(null);
  private _current: Ref<KitItem | null> = ref(null);
  private _target: Ref<Target> = ref({ operation: 'update', components: [] });
  // TODO: change to `false` after implementing toolkit installation
  private _isUninstallManager: Ref<boolean> = ref(true);

  constructor() { }

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
        const installedItem = this._installed.value?.components.find(
          (c) => c.name === item.name
        );

        let versionStr =
          installedItem?.version && installedItem?.version !== item.version
            ? `(${installedItem?.version} -> ${item.version})`
            : ` (${item.version})`;

        return {
          label: `${item.name}${versionStr}`,
          checked: item.installed || item.required,
          required: item.required,
          disabled: item.required,

          focused: false,
          value: item,
          labelComponent: shallowRef(LabelComponent),
          labelComponentProps: {
            label: item.name,
            oldVer: installedItem?.version,
            newVer: item.version,
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
  async loadConf() {
    let dir = await invokeCommand('get_install_dir');
    if (typeof dir === 'string' && dir.trim() !== '') {
      this.path.value = dir;
    }

    await this.loadInstalledKit();
    await this.loadAvailableKit();
  }

  async loadInstalledKit() {
    const tauriInstalled = (await invokeCommand(
      'get_installed_kit'
    )) as OriginKitItem | undefined;
    if (tauriInstalled) {
      const installed = {
        ...tauriInstalled, components: tauriInstalled.components.filter((c) => c.installed).map((item) => {

          const {
            group_name,
            is_toolchain_component,
            tool_installer,
            desc,
            ...rest
          } = item;
          return {
            ...rest,
            desc: desc.split('\n'),
            groupName: group_name,
            isToolchainComponent: is_toolchain_component,
            toolInstaller: tool_installer,
            version: tool_installer?.version || 'no version'

          } as ManagerComponent;
        })
      };
      this.setKits([installed]);
      this.setInstalled(installed);
      this.setCurrent(installed);
    }
  }

  // TODO: Separate `installed` and `available` toolkit list.
  // something like:
  //
  // Installed
  //   - xxx
  // Available
  //   - xxxx
  //   - xxxxx
  //
  // but we'll need to download `DistManifest` from server fot it at first.
  async loadAvailableKit() {
  }
}

export const managerConf = new ManagerConf();
