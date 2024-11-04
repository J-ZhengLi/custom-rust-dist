export interface Component {
  id: number;
  name: string;
  version?: string;
  required: boolean;
  optional: boolean;
  installed: boolean;
  desc: string;
  groupName: string | null;
  isToolchainComponent: boolean;
  toolInstaller?: {
    required: boolean;
    optional: boolean;
    path?: string;
  };
}
