import { ref, Ref, shallowRef } from 'vue';
import { KitItem, OriginKitItem } from './types/KitItem';
import { ManagerComponent } from './types/Component';
import { CheckGroup, CheckGroupItem } from './types/CheckBoxGroup';
import LabelComponent from '@/views/manager/components/Label.vue';
import { invokeCommand } from './invokeCommand';
import { ask } from '@tauri-apps/api/dialog';


type Target = {
  operation: 'update' | 'uninstall';
  components: ManagerComponent[];
};

class ManagerConf {
  path: Ref<string> = ref('');
  private _availableKits: Ref<KitItem[]> = ref([]);
  private _installedKit: Ref<KitItem | null> = ref(null);
  private _current: Ref<KitItem | null> = ref(null);
  private _target: Ref<Target> = ref({ operation: 'update', components: [] });
  // TODO: change to `false` after implementing toolkit installation
  private _isUninstallManager: Ref<boolean> = ref(true);

  constructor() { }

  public getUninstallManager() {
    return this._isUninstallManager.value;
  }

  public getKits(): KitItem[] {
    return this._availableKits.value;
  }

  public getInstalled(): KitItem | null {
    return this._installedKit.value;
  }

  public getInstalledComponents(): ManagerComponent[] | undefined {
    return this._installedKit.value?.components;
  }

  public getGroups(): CheckGroup<ManagerComponent>[] {
    const checkItems: CheckGroupItem<ManagerComponent>[] =
      this._current.value?.components.map((item) => {
        const installedItem = this._installedKit.value?.components.find(
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
    this._availableKits.value.splice(0, this._availableKits.value.length, ...kits);
  }

  public setInstalled(installed: KitItem): void {
    this._installedKit.value = installed;
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
    // TODO: Complete update detection
    // await this.checkUpdate();

    let dir = await invokeCommand('get_install_dir');
    if (typeof dir === 'string' && dir.trim() !== '') {
      this.path.value = dir;
    }

    await this.loadInstalledKit();
    await this.loadAvailableKit();
  }

  async checkUpdate() {
    let update = await invokeCommand('check_manager_version') as boolean;

    if (update) {
      // TODO: It is up to the user to decide whether to upgrade
      await ask("检测到新的可用版本，是否现在更新？");
    }
  }

  async upgradeManager() {
    await invokeCommand('upgrade_manager');
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
    const availableKits = (await invokeCommand(
      'get_available_kits'
    )) as KitItem[];

    // if (availableKits.length > 0) {
      // const installed = {
      //   ...tauriInstalled, components: tauriInstalled.components.filter((c) => c.installed).map((item) => {

      //     const {
      //       group_name,
      //       is_toolchain_component,
      //       tool_installer,
      //       desc,
      //       ...rest
      //     } = item;
      //     return {
      //       ...rest,
      //       desc: desc.split('\n'),
      //       groupName: group_name,
      //       isToolchainComponent: is_toolchain_component,
      //       toolInstaller: tool_installer,
      //       version: tool_installer?.version || 'no version'

      //     } as ManagerComponent;
      //   })
      // };
      
    // }
    this.setKits(availableKits);
  }
}

export const managerConf = new ManagerConf();
