import type {
  ApiErrorResponse,
  AuthResponse,
  Bookmark,
  Collection,
  CreateBookmarkRequest,
  CreateCollectionRequest,
  CreateTagRequest,
  LoginRequest,
  RegisterRequest,
  SearchQuery,
  Stats,
  Tag,
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

    // Debug: Log request details for auth endpoints
    if (endpoint.includes('/auth/')) {
      console.log(`Request to ${endpoint}:`, {
        url,
        token: this.token ? `${this.token.substring(0, 20)}...` : 'none',
        authorization: headers.get('Authorization') ? `${headers.get('Authorization')?.substring(0, 30)}...` : 'none'
      });
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
      
      // Handle the new API response format where data is wrapped in a "data" field
      if (jsonResponse && typeof jsonResponse === 'object' && 'data' in jsonResponse && 'success' in jsonResponse) {
        return jsonResponse.data;
      }
      
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
    console.log('setToken called with:', token ? `${token.substring(0, 20)}...` : 'null');
    this.token = token;
    if (typeof window !== 'undefined') {
      if (token) {
        localStorage.setItem('auth_token', token);
        console.log('Token saved to localStorage');
      } else {
        localStorage.removeItem('auth_token');
        console.log('Token removed from localStorage');
      }
    }
  }

  // Auth endpoints
  async login(data: LoginRequest): Promise<AuthResponse> {
    console.log('Login request data:', data);
    
    const apiResponse: any = await this.request('/auth/login', {
      method: 'POST',
      body: JSON.stringify(data),
    });

    console.log('Login API response:', apiResponse);

    // Handle the actual API response format
    const authResponse: AuthResponse = {
      user: apiResponse.user,
      token: apiResponse.access_token
    };

    console.log('Processed auth response:', authResponse);
    this.setToken(authResponse.token);
    return authResponse;
  }

  async register(data: RegisterRequest): Promise<AuthResponse> {
    const apiResponse: any = await this.request('/auth/register', {
      method: 'POST',
      body: JSON.stringify(data),
    });

    // Handle the actual API response format
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
    console.log('getCurrentUser called, token:', this.token);
    return this.request('/auth/me');
  }

  // Bookmark endpoints
  async getBookmarks(params?: SearchQuery): Promise<Bookmark[]> {
    const searchParams = new URLSearchParams();
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          if (key === 'tags' && Array.isArray(value)) {
            searchParams.append(key, value.join(','));
          } else {
            searchParams.append(key, value.toString());
          }
        }
      });
    }
    const query = searchParams.toString();
    return this.request(`/bookmarks${query ? `?${query}` : ''}`);
  }

  async getBookmark(id: number): Promise<Bookmark> {
    return this.request(`/bookmarks/${id}`);
  }

  async createBookmark(data: CreateBookmarkRequest): Promise<Bookmark> {
    return this.request('/bookmarks', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateBookmark(id: number, data: UpdateBookmarkRequest): Promise<Bookmark> {
    return this.request(`/bookmarks/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async deleteBookmark(id: number): Promise<void> {
    return this.request(`/bookmarks/${id}`, { method: 'DELETE' });
  }

  // Collection endpoints
  async getCollections(params?: { limit?: number; offset?: number; parent_id?: number; is_public?: boolean }): Promise<Collection[]> {
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

  async createCollection(data: CreateCollectionRequest): Promise<Collection> {
    return this.request('/collections', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateCollection(id: number, data: UpdateCollectionRequest): Promise<Collection> {
    return this.request(`/collections/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async deleteCollection(id: number): Promise<void> {
    return this.request(`/collections/${id}`, { method: 'DELETE' });
  }

  // Tag endpoints
  async getTags(params?: { limit?: number; offset?: number; search?: string }): Promise<Tag[]> {
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

  async createTag(data: CreateTagRequest): Promise<Tag> {
    return this.request('/tags', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async updateTag(id: number, data: UpdateTagRequest): Promise<Tag> {
    return this.request(`/tags/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async deleteTag(id: number): Promise<void> {
    return this.request(`/tags/${id}`, { method: 'DELETE' });
  }

  // Search endpoints
  async search(params: SearchQuery): Promise<Bookmark[]> {
    const searchParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        if (key === 'tags' && Array.isArray(value)) {
          searchParams.append(key, value.join(','));
        } else {
          searchParams.append(key, value.toString());
        }
      }
    });
    return this.request(`/search/bookmarks?${searchParams.toString()}`);
  }

  // Stats endpoints
  async getStats(): Promise<Stats> {
    return this.request('/stats/user');
  }
}

export const apiService = new ApiService();