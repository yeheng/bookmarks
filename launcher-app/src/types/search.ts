export interface SearchResult {
  id: string;
  type: 'bookmark' | 'file';
  title: string;
  subtitle: string;
  icon?: string;
  url?: string;
  path?: string;
  frecency_score?: number;  // Frecency score from backend
  match_score?: number;      // Search match score
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
