import { KitItem } from './types/KitItem';

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

class ManagerConf {
  private _kits: KitItem[] = [];
  private _installed: KitItem | null = null;
  private _current: KitItem | null = null;

  constructor() {}

  public getConf(): KitItem[] {
    return this._kits;
  }
  public getInstalledGroups() {}

  public setConf(conf: KitItem[]): void {
    this._kits = conf;
  }
  public loadConf(): any {
    this._current = kits[0];
    this._installed = kits[0];
    this._kits = kits;
  }
}

export const managerConf = new ManagerConf();
