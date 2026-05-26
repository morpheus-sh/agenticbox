export interface Session {
  id: string;
  name: string;
  status: "creating" | "running" | "paused" | "destroyed";
  createdAt: string;
  updatedAt: string;
}

export interface ModelConfig {
  provider: string;
  model: string;
  apiKey?: string;
  baseUrl?: string;
}

export interface PermissionSet {
  terminal: boolean;
  filesystem: "deny" | "readOnly" | "readWrite";
  browser: boolean;
  network: "offline" | "allowlist" | "localhostOnly" | "full";
}
