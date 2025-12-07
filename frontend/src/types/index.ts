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

// 资源类型：link(链接)、note(笔记)、snippet(代码片段)、file(文件)
export type ResourceType = 'link' | 'note' | 'snippet' | 'file';

export interface Resource {
  id: number;
  title: string;
  type: ResourceType; // 资源类型
  url?: string; // 链接类型必填，其他可选
  content?: string; // 笔记/代码片段内容
  source?: string; // 来源
  mime_type?: string; // MIME类型（文件类型）
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
  favicon_url?: string;
  screenshot_url?: string;
  thumbnail_url?: string;
  visit_count: number;
}

/**
 * @deprecated Use Resource instead. This type alias is maintained for backward compatibility only.
 * Will be removed in a future version.
 */
export type Bookmark = Resource;

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

export interface ResourceTag {
  resource_id: number;
  tag_id: number;
}

/**
 * @deprecated Use ResourceTag instead. This type alias is maintained for backward compatibility only.
 * Will be removed in a future version.
 */
export type BookmarkTag = ResourceTag;

export interface CreateResourceRequest {
  title: string;
  type: ResourceType; // 资源类型
  url?: string; // 链接类型必填，其他可选
  content?: string; // 笔记/代码片段内容
  source?: string; // 来源
  mime_type?: string; // MIME类型（文件类型）
  description?: string;
  collection_id?: number;
  tags?: string[];
  is_favorite?: boolean;
  is_private?: boolean;
}

/**
 * @deprecated Use CreateResourceRequest instead. This type alias is maintained for backward compatibility only.
 * Will be removed in a future version.
 */
export type CreateBookmarkRequest = CreateResourceRequest;

export interface UpdateResourceRequest {
  title?: string;
  type?: ResourceType;
  url?: string;
  content?: string;
  source?: string;
  mime_type?: string;
  description?: string;
  collection_id?: number;
  clear_collection_id?: boolean;
  tags?: string[];
  is_favorite?: boolean;
  is_archived?: boolean;
  is_private?: boolean;
  is_read?: boolean;
}

/**
 * @deprecated Use UpdateResourceRequest instead. This type alias is maintained for backward compatibility only.
 * Will be removed in a future version.
 */
export type UpdateBookmarkRequest = UpdateResourceRequest;

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

// 搜索过滤器接口（用于前端UI）
export interface SearchFilters {
  collectionId: string;
  tagId: string;
  sortBy: 'relevance' | 'created_at' | 'updated_at' | 'visit_count';
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

// Simplified response types for new API format
export interface SimpleApiResponse<T> {
  data?: T;
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
  total_resources: number;
  total_bookmarks: number; // 向下兼容
  total_collections: number;
  total_tags: number;
  favorite_resources: number;
  favorite_bookmarks: number; // 向下兼容
  archived_resources: number;
  archived_bookmarks: number; // 向下兼容
  total_visits: number;
  recent_resources: Resource[];
  recent_bookmarks: Bookmark[]; // 向下兼容
  recent_activity: RecentActivityEntry[];
  top_tags: TopTagEntry[];
  top_domains: TopDomainEntry[];
}

export interface RecentActivityEntry {
  date: number;
  resources_added: number;
  bookmarks_added: number; // 向下兼容
  resources_visited: number;
  bookmarks_visited: number; // 向下兼容
}

export interface TopTagEntry {
  name: string;
  count: number;
}

export interface TopDomainEntry {
  domain: string;
  count: number;
}

// 资源引用相关类型
export type ReferenceType = 'related' | 'depends_on' | 'references';

export interface ResourceReference {
  id: number;
  source_id: number;
  target_id: number;
  type: ReferenceType;
  created_at: number;
}

export interface CreateResourceReferenceRequest {
  target_id: number;
  type?: ReferenceType; // 默认 "related"
}

export interface ResourceReferenceQuery {
  limit?: number;
  offset?: number;
  type?: ReferenceType; // 过滤引用类型
  direction?: 'source' | 'target' | 'both'; // 查询方向
}

export interface ResourceReferenceList {
  items: Resource[];
  limit: number;
  offset: number;
  has_more: boolean;
}