export interface SearchResult {
  id: string;
  type: 'bookmark' | 'file';
  title: string;
  subtitle: string;
  icon?: string;
  url?: string;
  path?: string;
}

export interface SearchState {
  query: string;
  results: SearchResult[];
  loading: boolean;
  selectedIndex: number;
}
