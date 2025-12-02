# Bookmarks Manager Frontend

一个现代化的书签管理应用前端，使用 Svelte、TypeScript 和 Tailwind CSS 构建。

## 功能特性

### ✅ 已完成功能

1. **用户认证系统**
   - 用户注册和登录页面
   - JWT token 认证
   - 自动路由保护
   - 登出功能

2. **基础UI组件库**
   - Button, Input, Label, Textarea, Select 组件
   - Card 系列组件
   - Toast 通知组件
   - 响应式设计支持

3. **状态管理**
   - 用户认证状态管理
   - 书签数据状态管理
   - 收藏夹状态管理
   - 标签状态管理

4. **布局和导航**
   - 响应式导航栏
   - 路由守卫
   - 统一的页面布局

5. **书签管理**
   - 创建、编辑、删除书签
   - 书签列表展示
   - 支持标题、URL、描述
   - 收藏夹和标签关联

6. **收藏夹管理**
   - 创建、编辑、删除收藏夹
   - 收藏夹列表展示
   - 书签计数显示

7. **标签管理**
   - 创建、编辑、删除标签
   - 标签列表展示
   - 书签计数显示

8. **搜索功能**
   - 全文搜索
   - 收藏夹筛选
   - 标签筛选
   - 组合搜索

9. **环境配置**
   - TypeScript 类型定义
   - 环境变量配置
   - API 服务封装

10. **响应式设计和错误处理**
    - 移动端适配
    - 错误提示
    - 加载状态
    - Toast 通知

## 技术栈

- **前端框架**: Svelte 5
- **类型系统**: TypeScript
- **样式**: Tailwind CSS
- **构建工具**: Vite
- **状态管理**: Svelte Stores
- **HTTP 客户端**: Fetch API
- **UI 组件**: 自定义组件库 (shadcn/ui 风格)

## 项目结构

```
src/
├── lib/
│   ├── components/          # 可复用组件
│   │   └── ui/              # 基础UI组件
│   ├── services/            # API 服务
│   ├── stores/              # 状态管理
│   ├── types/               # TypeScript 类型
│   └── utils/               # 工具函数
├── routes/                  # 页面路由
│   ├── auth/               # 认证页面
│   ├── bookmarks/          # 书签管理
│   ├── collections/        # 收藏夹管理
│   ├── tags/               # 标签管理
│   └── search/             # 搜索页面
└── app.html                # HTML 模板
```

## 开发指南

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run dev
```

应用将在 http://localhost:5173 启动

### 构建生产版本

```bash
npm run build
```

### 预览生产版本

```bash
npm run preview
```

### 类型检查

```bash
npm run check
```

## 环境配置

创建 `.env.local` 文件：

```
VITE_API_BASE_URL=http://localhost:8080/api/v1
VITE_APP_NAME=Bookmarks Manager
```

## API 集成

前端通过 `src/lib/services/api.ts` 与后端 API 通信，支持：

- 认证相关 API
- 书签 CRUD 操作
- 收藏夹 CRUD 操作
- 标签 CRUD 操作
- 搜索功能
- 统计数据

## 页面路由

- `/` - 仪表板（需要认证）
- `/auth/login` - 登录页面
- `/auth/register` - 注册页面
- `/bookmarks` - 书签管理
- `/collections` - 收藏夹管理
- `/tags` - 标签管理
- `/search` - 搜索页面

## 特性亮点

1. **类型安全**: 完整的 TypeScript 类型定义
2. **响应式设计**: 支持桌面和移动设备
3. **用户体验**: 加载状态、错误处理、Toast 通知
4. **代码组织**: 清晰的文件结构和组件复用
5. **性能优化**: 懒加载和代码分割

## 开发注意事项

- 使用 Svelte 4+ 语法（已禁用 runes 模式）
- 遵循 TypeScript 严格模式
- 使用 Tailwind CSS 进行样式开发
- 所有 API 调用都有错误处理
- 支持客户端路由守卫

## 下一步计划

- 添加更多书签导入/导出功能
- 实现书签预览功能
- 添加用户设置页面
- 实现书签分享功能
- 添加更多搜索过滤器