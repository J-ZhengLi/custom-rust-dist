interface OriginComponent {
  id: number;
  name: string;
  required: boolean;
  optional: boolean;
  installed: boolean;
}
export interface Component extends OriginComponent {
  desc: string[];
  groupName: string | null;
  isToolchainComponent: boolean;
}

export interface TauriComponent extends OriginComponent {
  desc: string;
  group_name: string | null;
  is_toolchain_component: boolean;
}
