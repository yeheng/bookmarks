import type {
  ApiErrorResponse,
  ApiResponse,
  AuthResponse,
  Resource,
  Bookmark,
  Collection,
  CollectionsQuery,
  CreateResourceRequest,
  CreateBookmarkRequest,
  CreateCollectionRequest,
  CreateTagRequest,
  LoginRequest,
  PaginatedApiResponse,
  RegisterRequest,
  SearchQuery,
  Stats,
  Tag,
  TagsQuery,
  UpdateResourceRequest,
  UpdateBookmarkRequest,
  UpdateCollectionRequest,
  UpdateTagRequest,
  User
} from '@/types';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000/api';

class ApiError extends Error {
  constructor(
    message: string,
    public status?: number,
    public code?: string,
    public details?: any
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

class ApiService {
  private token: string | null = null;

  constructor() {
    if (typeof window !== 'undefined') {
      this.token = localStorage.getItem('auth_token');
    }
  }

  private sanitizeErrorMessage(message: string | undefined, status: number): string {
    if (!message) {
      return this.getDefaultErrorMessage(status);
    }

    // For server errors (5xx), don't expose detailed error messages
    if (status >= 500) {
      return 'Server error occurred. Please try again later.';
    }

    // For client errors (4xx), provide sanitized messages
    if (status >= 400) {
      // Remove potential sensitive information from error messages
      const sanitized = message
        .replace(/database|sql|query|table|column/gi, 'data')
        .replace(/password|token|secret|key/gi, 'credential')
        .replace(/\/[a-zA-Z0-9\/_\-\.]+/g, '/path')
        .replace(/localhost|127\.0\.0\.1|internal|private/gi, 'server');

      // Check if sanitized message is still meaningful
      if (sanitized.length < 10) {
        return this.getDefaultErrorMessage(status);
      }

      return sanitized;
    }

    return message;
  }

  private sanitizeErrorDetails(details: any, status: number): any {
    // For server errors, don't expose any details
    if (status >= 500) {
      return undefined;
    }

    // For client errors, sanitize details if present
    if (details && typeof details === 'object') {
      const sanitized: any = {};
      for (const [key, value] of Object.entries(details)) {
        // Skip sensitive keys
        if (key.toLowerCase().match(/password|token|secret|key|database|sql/)) {
          continue;
        }
        
        // Sanitize string values
        if (typeof value === 'string') {
          sanitized[key] = value
            .replace(/\/[a-zA-Z0-9\/_\-\.]+/g, '/path')
            .replace(/localhost|127\.0\.0\.1/gi, 'server');
        } else {
          sanitized[key] = value;
        }
      }
      return sanitized;
    }

    return details;
  }

  private getDefaultErrorMessage(status: number): string {
    switch (status) {
      case 400:
        return 'Invalid request. Please check your input.';
      case 401:
        return 'Authentication required. Please login.';
      case 403:
        return 'Access denied. You do not have permission to perform this action.';
      case 404:
        return 'The requested resource was not found.';
      case 409:
        return 'Conflict with existing data. Please check and try again.';
      case 422:
        return 'Invalid data provided. Please check your input.';
      case 429:
        return 'Too many requests. Please try again later.';
      case 500:
        return 'Server error occurred. Please try again later.';
      case 502:
        return 'Service temporarily unavailable. Please try again later.';
      case 503:
        return 'Service maintenance in progress. Please try again later.';
      default:
        return 'An error occurred. Please try again.';
    }
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${API_BASE_URL}${endpoint}`;
    const headers = new Headers(options.headers ?? {});
    if (!headers.has('Content-Type')) {
      headers.set('Content-Type', 'application/json');
    }
    if (this.token) {
      headers.set('Authorization', `Bearer ${this.token}`);
    }

    try {
      const response = await fetch(url, {
        ...options,
        headers,
      });

      if (!response.ok) {
        let errorData: ApiErrorResponse | null = null;
        try {
          errorData = await response.json();
        } catch {
          errorData = null;
        }

        // Sanitize error messages to prevent exposing sensitive information
        let sanitizedMessage = this.sanitizeErrorMessage(
          errorData?.message,
          response.status
        );

        throw new ApiError(
          sanitizedMessage,
          response.status,
          errorData?.code,
          this.sanitizeErrorDetails(errorData?.details, response.status)
        );
      }

      if (response.status === 204) {
        return {} as T;
      }

      const jsonResponse = await response.json();
      
      return jsonResponse;
    } catch (error) {
      if (error instanceof ApiError) {
        throw error;
      }
      
      // Handle network errors without exposing sensitive information
      if (error instanceof Error) {
        // Check for common network error patterns
        if (error.message.includes('fetch')) {
          throw new ApiError('Network connection failed. Please check your internet connection.');
        } else if (error.message.includes('timeout')) {
          throw new ApiError('Request timed out. Please try again.');
        } else {
          // Generic error without exposing technical details
          throw new ApiError('Network error occurred. Please try again.');
        }
      }
      
      throw new ApiError('An unexpected error occurred. Please try again.');
    }
  }

  setToken(token: string | null) {
    this.token = token;
    if (typeof window !== 'undefined') {
      if (token) {
        localStorage.setItem('auth_token', token);
      } else {
        localStorage.removeItem('auth_token');
      }
    }
  }

  // Auth endpoints
  async login(data: LoginRequest): Promise<AuthResponse> {
    
    const apiResponse: any = await this.request('/auth/login', {
      method: 'POST',
      body: JSON.stringify(data),
    });

    // Handle the simplified API response format
    const authResponse: AuthResponse = {
      user: apiResponse.user,
      token: apiResponse.access_token
    };

    this.setToken(authResponse.token);
    return authResponse;
  }

  async register(data: RegisterRequest): Promise<AuthResponse> {
    const apiResponse: any = await this.request('/auth/register', {
      method: 'POST',
      body: JSON.stringify(data),
    });

    // Handle the simplified API response format
    const authResponse: AuthResponse = {
      user: apiResponse.user,
      token: apiResponse.access_token
    };

    this.setToken(authResponse.token);
    return authResponse;
  }

  async logout(): Promise<void> {
    try {
      await this.request('/auth/logout', { method: 'POST' });
    } finally {
      this.setToken(null);
    }
  }

  async getCurrentUser(): Promise<User> {
    return this.request('/auth/me');
  }

  // Resource endpoints (资源管理)
  async getResources(params?: SearchQuery): Promise<PaginatedApiResponse<Resource>> {
    const searchParams = new URLSearchParams();
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          if (Array.isArray(value)) {
            value.forEach(v => searchParams.append(key, v.toString()));
          } else {
            searchParams.append(key, value.toString());
          }
        }
      });
    }
    const query = searchParams.toString();
    return this.request(`/resources${query ? `?${query}` : ''}`);
  }

  async getResource(id: number): Promise<Resource> {
    return this.request(`/resources/${id}`);
  }

  async createResource(request: CreateResourceRequest): Promise<Resource> {
    return this.request('/resources', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  async updateResource(id: number, request: UpdateResourceRequest): Promise<Resource> {
    return this.request(`/resources/${id}`, {
      method: 'PUT',
      body: JSON.stringify(request),
    });
  }

  async deleteResource(id: number): Promise<void> {
    return this.request(`/resources/${id}`, { method: 'DELETE' });
  }

  // 资源引用管理（资源之间的关联）
  async createResourceReference(
    resourceId: number,
    targetId: number,
    type?: string
  ): Promise<void> {
    return this.request(`/resources/${resourceId}/references`, {
      method: 'POST',
      body: JSON.stringify({ target_id: targetId, type }),
    });
  }

  async deleteResourceReference(resourceId: number, targetId: number): Promise<void> {
    return this.request(`/resources/${resourceId}/references/${targetId}`, {
      method: 'DELETE',
    });
  }

  async getResourceReferences(
    resourceId: number,
    params?: { type?: string; limit?: number; offset?: number }
  ): Promise<PaginatedApiResponse<Resource>> {
    const searchParams = new URLSearchParams();
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          searchParams.append(key, value.toString());
        }
      });
    }
    const query = searchParams.toString();
    return this.request(`/resources/${resourceId}/references${query ? `?${query}` : ''}`);
  }

  // Bookmark endpoints (向下兼容，调用 Resource API)
  async getBookmarks(params?: SearchQuery): Promise<PaginatedApiResponse<Bookmark>> {
    return this.getResources(params);
  }

  async getBookmark(id: number): Promise<Bookmark> {
    return this.getResource(id);
  }

  async createBookmark(request: CreateBookmarkRequest): Promise<Bookmark> {
    return this.createResource(request);
  }

  async updateBookmark(id: number, request: UpdateBookmarkRequest): Promise<Bookmark> {
    return this.updateResource(id, request);
  }

  async deleteBookmark(id: number): Promise<void> {
    return this.deleteResource(id);
  }

  // Collection endpoints
  async getCollections(params?: CollectionsQuery): Promise<PaginatedApiResponse<Collection>> {
    const searchParams = new URLSearchParams();
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          searchParams.append(key, value.toString());
        }
      });
    }
    const query = searchParams.toString();
    return this.request(`/collections${query ? `?${query}` : ''}`);
  }

  async getCollection(id: number): Promise<Collection> {
    return this.request(`/collections/${id}`);
  }

  async createCollection(request: CreateCollectionRequest): Promise<Collection> {
    return this.request('/collections', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  async updateCollection(id: number, request: UpdateCollectionRequest): Promise<Collection> {
    return this.request(`/collections/${id}`, {
      method: 'PUT',
      body: JSON.stringify(request),
    });
  }

  async deleteCollection(id: number): Promise<void> {
    return this.request(`/collections/${id}`, { method: 'DELETE' });
  }

  // Tag endpoints
  async getTags(params?: TagsQuery): Promise<PaginatedApiResponse<Tag>> {
    const searchParams = new URLSearchParams();
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          searchParams.append(key, value.toString());
        }
      });
    }
    const query = searchParams.toString();
    return this.request(`/tags${query ? `?${query}` : ''}`);
  }

  async getTag(id: number): Promise<Tag> {
    return this.request(`/tags/${id}`);
  }

  async createTag(request: CreateTagRequest): Promise<Tag> {
    return this.request('/tags', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  async updateTag(id: number, request: UpdateTagRequest): Promise<Tag> {
    return this.request(`/tags/${id}`, {
      method: 'PUT',
      body: JSON.stringify(request),
    });
  }

  async deleteTag(id: number): Promise<void> {
    return this.request(`/tags/${id}`, { method: 'DELETE' });
  }

  // Search endpoints (搜索资源)
  async search(params: SearchQuery): Promise<PaginatedApiResponse<Resource>> {
    const searchParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        if (Array.isArray(value)) {
          value.forEach(v => searchParams.append(key, v.toString()));
        } else {
          searchParams.append(key, value.toString());
        }
      }
    });
    return this.request(`/search/resources?${searchParams.toString()}`);
  }

  // Stats endpoints
  async getStats(): Promise<Stats> {
    return this.request('/stats/user');
  }
}

export const apiService = new ApiService();

// 导出一个兼容的 useApi 函数
export const useApi = () => apiService;