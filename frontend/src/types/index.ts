export interface User {
  id: number;
  username: string;
  email: string;
  created_at: string;
  updated_at: string;
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
  created_at: string;
  updated_at: string;
  tags: Tag[];
}

export interface Collection {
  id: number;
  name: string;
  description?: string;
  user_id: number;
  created_at: string;
  updated_at: string;
  _count?: {
    bookmarks: number;
  };
}

export interface Tag {
  id: number;
  name: string;
  user_id: number;
  created_at: string;
  updated_at: string;
  _count?: {
    bookmarks: number;
  };
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
  tag_ids?: number[];
}

export interface UpdateBookmarkRequest {
  title?: string;
  url?: string;
  description?: string;
  collection_id?: number;
  tag_ids?: number[];
}

export interface CreateCollectionRequest {
  name: string;
  description?: string;
}

export interface UpdateCollectionRequest {
  name?: string;
  description?: string;
}

export interface CreateTagRequest {
  name: string;
}

export interface UpdateTagRequest {
  name?: string;
}

export interface SearchQuery {
  q?: string;
  collection_id?: number;
  tag_id?: number;
  limit?: number;
  offset?: number;
}

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
  recent_bookmarks: Bookmark[];
  popular_tags: Tag[];
}