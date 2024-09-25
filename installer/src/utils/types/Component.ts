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
  toolInstaller?: {
    required: boolean;
    optional: boolean;
    ver: string;
    path?: string;
  };
}

export interface TauriComponent extends OriginComponent {
  desc: string;
  group_name: string | null;
  is_toolchain_component: boolean;
  tool_installer?: {
    required: boolean;
    optional: boolean;
    version: string;
    path?: string;
  };
}

export interface ManagerComponent extends Component {
  version?: string;
}
