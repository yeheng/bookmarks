import type {
  Action,
  Collection,
  CommandRequest,
  CommandResponse,
  CreateCollectionRequest,
  CreateResourceRequest,
  CreateTagRequest,
  PaginatedApiResponse,
  Resource,
  SearchQuery,
  Stats,
  Tag,
  UpdateResourceRequest,
} from '@/types';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000/api';

class CommandApiError extends Error {
  constructor(
    message: string,
    public code?: string,
    public details?: any
  ) {
    super(message);
    this.name = 'CommandApiError';
  }
}

class CommandApiService {
  private token: string | null = null;

  constructor() {
    if (typeof window !== 'undefined') {
      this.token = localStorage.getItem('auth_token');
    }
  }

  private async request(commandRequest: CommandRequest): Promise<CommandResponse> {
    const url = `${API_BASE_URL}/command/execute`;

    // 生成请求ID（如果没有提供）
    if (!commandRequest.request_id) {
      commandRequest.request_id = this.generateRequestId();
    }

    const headers = new Headers({
      'Content-Type': 'application/json',
    });

    if (this.token) {
      headers.set('Authorization', `Bearer ${this.token}`);
    }

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers,
        body: JSON.stringify(commandRequest),
      });

      if (!response.ok) {
        throw new CommandApiError(
          `HTTP ${response.status}: ${response.statusText}`,
          'HTTP_ERROR',
          { status: response.status }
        );
      }

      const commandResponse: CommandResponse = await response.json();

      // 检查命令执行结果
      if (!commandResponse.success) {
        throw new CommandApiError(
          commandResponse.error?.message || '命令执行失败',
          commandResponse.error?.code,
          commandResponse.error?.details
        );
      }

      return commandResponse;
    } catch (error) {
      if (error instanceof CommandApiError) {
        throw error;
      }

      // 网络错误处理
      if (error instanceof Error) {
        if (error.message.includes('fetch')) {
          throw new CommandApiError('网络连接失败，请检查网络连接');
        } else if (error.message.includes('timeout')) {
          throw new CommandApiError('请求超时，请稍后重试');
        }
      }

      throw new CommandApiError('发生未知错误，请稍后重试');
    }
  }

  private generateRequestId(): string {
    return `cmd_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
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

  // ==================== 资源管理命令 ====================

  /**
   * 获取资源列表
   */
  async getResources(params?: SearchQuery): Promise<PaginatedApiResponse<Resource>> {
    const commandRequest: CommandRequest = {
      action: Action.GetResources,
      params: params || {},
    };

    const response = await this.request(commandRequest);
    return response.data as PaginatedApiResponse<Resource>;
  }

  /**
   * 根据ID获取单个资源
   */
  async getResource(id: number): Promise<Resource> {
    const commandRequest: CommandRequest = {
      action: Action.GetResource,
      params: { id },
    };

    const response = await this.request(commandRequest);
    return response.data as Resource;
  }

  /**
   * 创建新资源
   */
  async createResource(resourceData: CreateResourceRequest): Promise<Resource> {
    const commandRequest: CommandRequest = {
      action: Action.CreateResource,
      params: resourceData,
    };

    const response = await this.request(commandRequest);
    return response.data as Resource;
  }

  /**
   * 更新资源
   */
  async updateResource(id: number, updateData: UpdateResourceRequest): Promise<Resource> {
    const commandRequest: CommandRequest = {
      action: Action.UpdateResource,
      params: {
        id,
        data: updateData,
      },
    };

    const response = await this.request(commandRequest);
    return response.data as Resource;
  }

  /**
   * 删除资源
   */
  async deleteResource(id: number): Promise<void> {
    const commandRequest: CommandRequest = {
      action: Action.DeleteResource,
      params: { id },
    };

    await this.request(commandRequest);
  }

  /**
   * 批量更新资源
   */
  async batchUpdateResources(params: {
    resource_ids: number[];
    action: string;
    data?: any;
  }): Promise<any> {
    const commandRequest: CommandRequest = {
      action: Action.BatchUpdateResources,
      params,
    };

    const response = await this.request(commandRequest);
    return response.data;
  }

  // ==================== 搜索命令 ====================

  /**
   * 搜索资源
   */
  async searchResources(params: SearchQuery): Promise<PaginatedApiResponse<Resource>> {
    const commandRequest: CommandRequest = {
      action: Action.SearchResources,
      params,
    };

    const response = await this.request(commandRequest);
    return response.data as PaginatedApiResponse<Resource>;
  }

  // ==================== 收藏夹管理命令 ====================

  /**
   * 获取收藏夹列表
   */
  async getCollections(params?: {
    limit?: number;
    offset?: number;
    parent_id?: number;
    is_public?: boolean;
  }): Promise<Collection[]> {
    const commandRequest: CommandRequest = {
      action: Action.GetCollections,
      params: params || {},
    };

    const response = await this.request(commandRequest);
    return response.data as Collection[];
  }

  /**
   * 创建收藏夹
   */
  async createCollection(collectionData: CreateCollectionRequest): Promise<Collection> {
    const commandRequest: CommandRequest = {
      action: Action.CreateCollection,
      params: collectionData,
    };

    const response = await this.request(commandRequest);
    return response.data as Collection;
  }

  // ==================== 标签管理命令 ====================

  /**
   * 获取标签列表
   */
  async getTags(params?: {
    limit?: number;
    offset?: number;
    search?: string;
  }): Promise<Tag[]> {
    const commandRequest: CommandRequest = {
      action: Action.GetTags,
      params: params || {},
    };

    const response = await this.request(commandRequest);
    return response.data as Tag[];
  }

  /**
   * 创建标签
   */
  async createTag(tagData: CreateTagRequest): Promise<Tag> {
    const commandRequest: CommandRequest = {
      action: Action.CreateTag,
      params: tagData,
    };

    const response = await this.request(commandRequest);
    return response.data as Tag;
  }

  // ==================== 统计命令 ====================

  /**
   * 获取用户统计信息
   */
  async getUserStats(): Promise<Stats> {
    const commandRequest: CommandRequest = {
      action: Action.GetUserStats,
      params: {},
    };

    const response = await this.request(commandRequest);
    return response.data as Stats;
  }

  // ==================== 高级命令方法 ====================

  /**
   * 执行自定义命令
   */
  async executeCommand(action: Action, params: Record<string, any>): Promise<any> {
    const commandRequest: CommandRequest = {
      action,
      params,
    };

    const response = await this.request(commandRequest);
    return response.data;
  }

  /**
   * 批量执行命令（并行）
   */
  async executeBatchCommands(commands: {
    action: Action;
    params: Record<string, any>;
  }[]): Promise<CommandResponse[]> {
    const promises = commands.map(cmd => this.request(cmd));
    return Promise.all(promises);
  }
}

export const commandApiService = new CommandApiService();

// 导出一个兼容的 useCommandApi 函数
export const useCommandApi = () => commandApiService;