export interface SearchResult {
  id: string;
  type: 'bookmark' | 'file' | 'plugin';
  title: string;
  subtitle: string;
  icon?: string;
  url?: string;
  path?: string;
  frecency_score?: number;
  match_score?: number;
  metadata?: {
    size?: string;
    modified?: string;
    domain?: string;
  };
  /** Plugin-specific: attached actions for the result item */
  pluginActions?: import('./plugin').PluginAction[];
  /** Plugin-specific: badge text (e.g., "v1.0") */
  pluginBadge?: string;
  /** Plugin-specific: the keyword that triggered this result */
  pluginKeyword?: string;
}

export interface SearchState {
  query: string;
  results: SearchResult[];
  loading: boolean;
  selectedIndex: number;
}

// Backend response types (matching Rust structs)
export interface BookmarkSearchResult {
  id: number;
  title: string;
  url: string;
  description: string | null;
  favicon_url: string | null;
  score: number;
  frecency_score: number;
}

export interface FileSearchResult {
  id: number;
  path: string;
  name: string;
  extension: string | null;
  size: number;
  modified_at: number;
  score: number;
  frecency_score: number;
}

export interface OpenResult {
  success: boolean;
  resource_type: string;
  resource_id: number;
  error: string | null;
}

// Unified search result from the backend (matching Rust UnifiedSearchResult)
export interface UnifiedSearchResult {
  id: string;
  title: string;
  subtitle: string;
  source_type: 'bookmark' | 'file' | 'plugin';
  source_id: string;
  score: number;
  frecency_score: number;
  icon: string | null;
  url: string | null;
  path: string | null;
  favicon_url: string | null;
  description: string | null;
  extension: string | null;
  size: number | null;
  modified_at: number | null;
  plugin_actions: any[] | null;
  plugin_badge: string | null;
  plugin_keyword: string | null;
}
