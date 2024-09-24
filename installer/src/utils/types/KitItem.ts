import { ManagerComponent, TauriComponent } from './Component';


interface BaseKitItem {
    /**
   * The version number
   */
  version: string;

  /**
   * The Kit name
   */
  name: string;

  /**
   * The version description
   */
  desc: string;

  /**
   * The version release notes
   */
  info: string;

  manifestURL: string;
}

export interface KitItem extends BaseKitItem {
  components: ManagerComponent[];
}

export interface OriginKitItem extends BaseKitItem {
  components: TauriComponent[];
}
