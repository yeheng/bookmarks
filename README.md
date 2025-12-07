# 多资源聚合系统

一个现代化的多资源聚合系统，采用 Rust 后端和 Vue.js 3 前端构建，支持链接、文件、笔记等多种类型资源的统一管理、组织和搜索。

## 🚀 特性

- **现代化技术栈**: Rust + Axum 后端，Vue.js 3 + TypeScript 前端
- **多资源支持**: 统一管理链接、文件、笔记等多种类型资源
- **极简设计**: 专注核心功能，减少学习成本
- **快速搜索**: 全局搜索 + 快捷键支持，支持中英文混合搜索
- **灵活组织**: 收藏夹和标签系统，支持资源分类管理
- **响应式设计**: 支持桌面和移动设备
- **类型安全**: 全栈 TypeScript 支持
- **轻量级部署**: SQLite 数据库，零配置

## 🛠️ 技术栈

### 后端

- **语言**: Rust 1.75+
- **框架**: Axum 0.7+
- **数据库**: SQLite + SQLx
- **认证**: JWT + bcrypt
- **异步运行时**: Tokio

### 前端

- **框架**: Vue.js 3.4+
- **语言**: TypeScript 5.5+
- **UI库**: shadcn-vue + Radix Vue
- **状态管理**: Pinia
- **路由**: Vue Router 4.4+
- **构建工具**: Vite 5.4+
- **样式**: Tailwind CSS 4+

## 📁 项目结构

```
bookmarks/
├── backend/                 # Rust 后端
│   ├── src/
│   │   ├── handlers/        # API 处理器
│   │   ├── models/          # 数据模型
│   │   ├── services/        # 业务逻辑
│   │   ├── middleware/      # 中间件
│   │   ├── routes/          # 路由配置
│   │   └── utils/           # 工具函数
│   ├── migrations/          # 数据库迁移
│   └── Cargo.toml
├── frontend/                # Vue.js 前端
│   ├── src/
│   │   ├── components/      # Vue 组件
│   │   ├── stores/          # Pinia 状态管理
│   │   ├── router/          # 路由配置
│   │   ├── services/        # API 服务
│   │   ├── types/           # TypeScript 类型
│   │   ├── utils/           # 工具函数
│   │   └── views/           # 页面组件
│   └── package.json
└── docs/                    # 项目文档
```

## 🚀 快速开始

### 环境要求

- Rust 1.75+
- Node.js 18+
- SQLite 3+

### 安装和运行

1. **克隆仓库**

```bash
git clone <repository-url>
cd bookmarks
```

2. **启动后端**

```bash
cd backend
cargo run
```

3. **启动前端**

```bash
cd frontend
npm install
npm run dev
```

4. **访问应用**

- 前端: <http://localhost:5173>
- 后端 API: <http://localhost:3000>

### 环境配置

创建 `.env` 文件：

```env
# 后端配置
DATABASE_URL=sqlite:bookmarks.db
JWT_SECRET=your-secret-key
SERVER_PORT=3000

# 前端配置
VITE_API_BASE_URL=http://localhost:3000/api
```

## 📖 API 文档

API 接口文档位于 [docs/api-interface-specification.md](docs/api-interface-specification.md)

主要端点：

- `POST /api/auth/login` - 用户登录
- `GET /api/resources` - 获取资源列表
- `POST /api/resources` - 创建资源
- `GET /api/collections` - 获取收藏夹
- `GET /api/tags` - 获取标签

## 🎯 核心功能

### 资源管理

- ✅ 创建、编辑、删除多种类型资源（链接、文件、笔记）
- ✅ 添加描述和标签
- ✅ 收藏夹组织
- ✅ 快速搜索（⌘K），支持全文搜索
- ✅ 访问统计和使用分析

### 用户界面

- ✅ 响应式设计
- ✅ 深色/浅色主题
- ✅ 键盘快捷键
- ✅ 极简设计理念
- ✅ 无障碍支持

### 数据管理

- ✅ SQLite 数据库
- ✅ 数据迁移
- ✅ 备份和恢复

## 🔧 开发

### 后端开发

```bash
cd backend

# 运行测试
cargo test

# 代码检查
cargo clippy

# 格式化代码
cargo fmt

# 数据库迁移
sqlx migrate run
```

### 前端开发

```bash
cd frontend

# 安装依赖
npm install

# 开发服务器
npm run dev

# 类型检查
npm run type-check

# 代码检查
npm run lint

# 构建生产版本
npm run build
```

### 代码规范

- **Rust**: 使用 `rustfmt` 和 `clippy`
- **TypeScript**: ESLint + Prettier
- **提交信息**: 遵循 Conventional Commits

## 🧪 测试

### 后端测试

```bash
cd backend
cargo test
```

### 前端测试

```bash
cd frontend
npm run test
```

### E2E 测试

```bash
cd frontend
npm run test:e2e
```

## 📦 部署

### Docker 部署

```bash
# 构建镜像
docker build -t bookmarks .

# 运行容器
docker run -p 3000:3000 bookmarks
```

### 生产部署

1. **后端部署**

```bash
cd backend
cargo build --release
./target/release/bookmarks
```

2. **前端部署**

```bash
cd frontend
npm run build
# 部署 dist/ 目录到 Web 服务器
```

## 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 📞 支持

- 📧 邮箱: <support@example.com>
- 🐛 问题反馈: [GitHub Issues](https://github.com/your-username/bookmarks/issues)
- 📖 文档: [项目文档](docs/)

## 🙏 致谢

感谢以下开源项目：

- [Axum](https://github.com/tokio-rs/axum) - Rust Web 框架
- [Vue.js](https://github.com/vuejs/vue) - 前端框架
- [shadcn-vue](https://github.com/radix-vue/shadcn-vue) - UI 组件库
- [Tailwind CSS](https://github.com/tailwindlabs/tailwindcss) - CSS 框架

---

**Built with ❤️ by the Resources Team**
