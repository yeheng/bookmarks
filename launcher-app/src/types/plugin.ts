/**
 * TypeScript types for the plugin system.
 * Maps 1:1 to Rust executor/registry structs.
 */

// ── Plugin Metadata ──

export interface PluginInfo {
  id: string;
  title: string;
  description: string | null;
  version: string;
  author: string | null;
  enabled: boolean;
  install_path: string;
  installed_at: string;
  updated_at: string | null;
  icon: string | null;
}

// ── Plugin Execution Results ──

export interface PluginResponse {
  items: PluginResultItem[];
  cache?: CacheDirective | null;
}

export interface PluginResultItem {
  uid: string;
  title: string;
  subtitle?: string | null;
  arg?: string | null;
  icon?: PluginIcon | null;
  actions: PluginAction[];
  badge?: string | null;
}

export interface PluginIcon {
  path?: string | null;
  url?: string | null;
  emoji?: string | null;
}

export interface CacheDirective {
  ttl_seconds: number;
}

// ── Plugin Actions (tagged union matching Rust serde) ──

export type PluginAction =
  | { type: 'open-url'; url: string; title?: string | null }
  | { type: 'copy'; text: string; title?: string | null }
  | { type: 'paste'; text: string; title?: string | null }
  | { type: 'open-file'; path: string; title?: string | null }
  | { type: 'run-command'; command: string; arg?: string | null; title?: string | null };

// ── Plugin Preferences Schema (from manifest) ──

export interface PluginPreferenceSchema {
  name: string;
  title: string;
  description?: string;
  type: 'text' | 'password' | 'checkbox' | 'select' | 'number';
  required?: boolean;
  default?: string;
  placeholder?: string;
  options?: { title: string; value: string }[];
}
