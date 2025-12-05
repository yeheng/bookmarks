export interface User {
  id: number;
  username: string;
  email: string;
  avatar_url?: string;
  is_active: boolean;
  email_verified: boolean;
  last_login_at?: number;
  created_at: number;
  updated_at: number;
}

export interface AuthResponse {
  user: User;
  token: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
}

export interface Bookmark {
  id: number;
  title: string;
  url: string;
  description?: string;
  user_id: number;
  collection_id?: number;
  collection_name?: string;
  collection_color?: string;
  created_at: number;
  updated_at: number;
  tags: string[]; // 后端返回的是字符串数组而不是 Tag 对象
  is_archived: boolean;
  is_favorite: boolean;
  is_private: boolean;
  is_read: boolean;
  last_visited?: number;
  metadata: Record<string, any>;
  reading_time?: number;
  favicon_url?: string;
  screenshot_url?: string;
  thumbnail_url?: string;
  visit_count: number;
  difficulty_level?: number;
}

export interface Collection {
  id: number;
  name: string;
  description?: string;
  user_id: number;
  created_at: number;
  updated_at: number;
  color: string;
  icon: string;
  sort_order: number;
  is_default: boolean;
  is_public: boolean;
  parent_id?: number;
  bookmark_count: number;
}

export interface Tag {
  id: number;
  name: string;
  user_id: number;
  created_at: number;
  updated_at: number;
  color: string;
  description?: string;
  usage_count: number;
}

export interface BookmarkTag {
  bookmark_id: number;
  tag_id: number;
}

export interface CreateBookmarkRequest {
  title: string;
  url: string;
  description?: string;
  collection_id?: number;
  tags?: string[];
  is_favorite?: boolean;
  is_private?: boolean;
}

export interface UpdateBookmarkRequest {
  title?: string;
  url?: string;
  description?: string;
  collection_id?: number;
  clear_collection_id?: boolean;
  tags?: string[];
  is_favorite?: boolean;
  is_archived?: boolean;
  is_private?: boolean;
  is_read?: boolean;
  reading_time?: number;
  difficulty_level?: number;
}

export interface CreateCollectionRequest {
  name: string;
  description?: string;
  color?: string;
  icon?: string;
  parent_id?: number;
}

export interface UpdateCollectionRequest {
  name?: string;
  description?: string;
  color?: string;
  icon?: string;
  parent_id?: number;
  clear_parent_id?: boolean;
  sort_order?: number;
}

export interface CreateTagRequest {
  name: string;
  color?: string;
  description?: string;
}

export interface UpdateTagRequest {
  name?: string;
  color?: string;
  description?: string;
}

export interface SearchQuery {
  q?: string; // 搜索关键词，用于全局搜索
  search?: string; // 搜索关键词，用于特定搜索
  collection_id?: number;
  tags?: string[];
  is_favorite?: boolean;
  is_archived?: boolean;
  is_private?: boolean;
  is_read?: boolean;
  limit?: number;
  offset?: number;
  sort_by?: string; // "created_at", "updated_at", "title", "visit_count"
  sort_order?: string; // "asc", "desc"
}

export interface CollectionsQuery {
  limit?: number;
  offset?: number;
  parent_id?: number;
  is_public?: boolean;
}

export interface TagsQuery {
  limit?: number;
  offset?: number;
  search?: string;
}

export interface PaginationInfo {
  page: number;
  limit: number;
  total: number;
  total_pages: number;
  has_next: boolean;
  has_prev: boolean;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T;
  message?: string;
  search_time?: number;
}

export interface PaginatedApiResponse<T> {
  success: boolean;
  data: {
    items: T[];
    pagination: PaginationInfo;
    highlights?: any; // 用于搜索高亮
  };
  message?: string;
  search_time?: number;
}

// 保持向后兼容
export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  limit: number;
  offset: number;
  has_more: boolean;
}

export interface ApiErrorResponse {
  message: string;
  code?: string;
  details?: any;
}

export interface Stats {
  total_bookmarks: number;
  total_collections: number;
  total_tags: number;
  favorite_bookmarks: number;
  archived_bookmarks: number;
  total_visits: number;
  recent_bookmarks: Bookmark[];
  recent_activity: RecentActivityEntry[];
  top_tags: TopTagEntry[];
  top_domains: TopDomainEntry[];
}

export interface RecentActivityEntry {
  date: number;
  bookmarks_added: number;
  bookmarks_visited: number;
}

export interface TopTagEntry {
  name: string;
  count: number;
}

export interface TopDomainEntry {
  domain: string;
  count: number;
}