import { ManagerComponent } from './Component';

export interface KitItem {
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
   * The version release date
   */
  date: string;

  /**
   * The version release type
   */
  type: string;

  /**
   * The version release notes
   */
  notes: string;

  manifestURL: string;

  components: ManagerComponent[];
}
