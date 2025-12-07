# 部署指南

本文档提供了书签管理系统的完整部署指南，包括本地开发、Docker 部署和 CI/CD 流程。

## 目录

- [本地开发](#本地开发)
- [Docker 部署](#docker-部署)
- [CI/CD 流程](#cicd-流程)
- [环境配置](#环境配置)
- [监控和日志](#监控和日志)
- [故障排除](#故障排除)

## 本地开发

### 前置要求

- Rust 1.75+
- Node.js 20+
- SQLite 3+
- Docker (可选，用于容器化部署)

### 快速启动

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

3. **访问应用**
   - 前端: http://localhost:5173
   - 后端 API: http://localhost:3000

### 环境变量配置

#### 后端环境变量 (.env)

```bash
# 数据库配置
DATABASE_URL=sqlite:./data/bookmarks.db

# 服务器配置
SERVER_PORT=3000
SERVER_HOST=0.0.0.0

# 认证配置
JWT_SECRET=your-super-secret-jwt-key-change-in-production
JWT_EXPIRES_IN=604800  # 7 days in seconds

# 日志配置
RUST_LOG=info  # debug, info, warn, error

# 前端 URL（CORS）
FRONTEND_URL=http://localhost:5173
```

#### 前端环境变量 (.env)

```bash
# API 地址
VITE_API_BASE_URL=http://localhost:3000/api

# 应用标题
VITE_APP_TITLE=Resources Manager

# 功能开关
VITE_ENABLE_ANALYTICS=false
VITE_ENABLE_PWA=true
```

### 使用 Docker Compose 进行本地开发

```bash
# 启动后端服务
docker-compose up backend

# 启动前端开发服务
docker-compose --profile dev up

# 启动开发工具（数据库管理）
docker-compose --profile tools up

# 启动监控服务
docker-compose --profile monitoring up
```

## Docker 部署

### 构建镜像

```bash
# 构建应用镜像
docker build -t bookmarks:latest .

# 使用多平台构建
docker buildx build --platform linux/amd64,linux/arm64 -t bookmarks:latest .
```

### 运行容器

```bash
# 基本运行
docker run -d \
  --name bookmarks \
  -p 3000:3000 \
  -e DATABASE_URL=sqlite:///app/data/bookmarks.db \
  -e JWT_SECRET=your-secret-key \
  bookmarks:latest

# 带数据持久化
docker run -d \
  --name bookmarks \
  -p 3000:3000 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/logs:/app/logs \
  -e DATABASE_URL=sqlite:///app/data/bookmarks.db \
  -e JWT_SECRET=your-secret-key \
  bookmarks:latest
```

### 使用 Docker Compose 生产部署

```bash
# 创建生产环境配置
cp docker-compose.yml docker-compose.prod.yml

# 编辑生产配置
# 修改环境变量、卷挂载等

# 启动生产服务
docker-compose -f docker-compose.prod.yml up -d
```

## CI/CD 流程

### GitHub Actions 工作流

项目包含两个主要工作流：

#### 1. 构建所有平台 (`.github/workflows/build-all-platforms.yml`)

**触发条件：**
- 推送到 `main` 或 `develop` 分支
- 创建 Pull Request
- 发布 Release

**工作流程：**
1. **代码质量检查** - 运行 Rust 和前端的代码检查
2. **后端多平台构建** - 构建 5 个目标平台：
   - Linux x86_64 (glibc)
   - Linux x86_64 (musl)
   - Windows x86_64
   - macOS x86_64
   - macOS ARM64
3. **前端构建** - 构建生产版本
4. **Docker 镜像构建** - 构建多架构 Docker 镜像
5. **发布资产** - 上传构建产物到 Release
6. **部署** - 自动部署到测试/生产环境

#### 2. 测试 (`.github/workflows/test.yml`)

**触发条件：**
- 推送到 `main` 或 `develop` 分支
- 创建 Pull Request

**工作流程：**
1. **后端测试** - 运行单元测试和集成测试
2. **前端测试** - 运行单元测试和组件测试
3. **代码覆盖率** - 生成并上传覆盖率报告

### 环境变量配置

在 GitHub 仓库设置中配置以下 Secrets：

```bash
# Docker Hub 认证
DOCKER_USERNAME=your-docker-username
DOCKER_PASSWORD=your-docker-password

# 部署相关
DEPLOY_HOST=your-server.com
DEPLOY_USER=deploy
DEPLOY_KEY=your-ssh-key

# 通知（可选）
SLACK_WEBHOOK_URL=your-slack-webhook
```

## 环境配置

### 后端环境变量

```bash
# 数据库
DATABASE_URL=sqlite:///app/data/bookmarks.db

# 服务器
SERVER_PORT=3000
SERVER_HOST=0.0.0.0

# 认证
JWT_SECRET=your-super-secret-jwt-key
JWT_EXPIRES_IN=7d

# 日志
RUST_LOG=info  # debug, info, warn, error

# 前端 URL（CORS）
FRONTEND_URL=http://localhost:5173
```

### 前端环境变量

```bash
# API 地址
VITE_API_BASE_URL=http://localhost:3000/api

# 应用标题
VITE_APP_TITLE=Resources Manager

# 功能开关
VITE_ENABLE_ANALYTICS=false
VITE_ENABLE_PWA=true
```

## 监控和日志

### 日志配置

```bash
# 查看应用日志
docker-compose logs -f backend

# 查看特定时间段的日志
docker-compose logs --since="2023-01-01T00:00:00" backend

# 日志级别控制
docker run -e RUST_LOG=debug bookmarks:latest
```

### 监控配置

使用 Prometheus + Grafana 进行监控：

```bash
# 启动监控服务
docker-compose --profile monitoring up

# 访问监控界面
# Prometheus: http://localhost:9090
# Grafana: http://localhost:3001 (admin/admin)
```

### 健康检查

```bash
# 检查应用健康状态
curl http://localhost:3000/health

# 检查 Docker 容器状态
docker ps
docker-compose ps
```

## 故障排除

### 常见问题

#### 1. 后端编译失败

```bash
# 清理 Rust 缓存
cd backend
cargo clean
cargo build

# 检查 Rust 版本
rustc --version  # 需要 1.75+
```

#### 2. 前端构建失败

```bash
# 清理 node_modules
cd frontend
rm -rf node_modules package-lock.json
npm install

# 检查 Node.js 版本
node --version  # 需要 20+
npm --version
```

#### 3. Docker 构建问题

```bash
# 清理 Docker 缓存
docker system prune -a

# 重新构建镜像
docker build --no-cache -t bookmarks:latest .
```

#### 4. 数据库连接问题

```bash
# 检查数据库文件权限
ls -la data/bookmarks.db

# 重新初始化数据库
rm data/bookmarks.db
docker-compose restart backend
```

### 性能优化

#### 1. 后端优化

```bash
# 使用 release 模式
cargo build --release

# 启用 Rust 优化
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

#### 2. 前端优化

```bash
# 构建分析
npm run build -- --analyze

# 启用压缩
npm run build -- --minify
```

#### 3. Docker 优化

```bash
# 使用多阶段构建
# 已在 Dockerfile 中配置

# 使用 .dockerignore
# 已创建 .dockerignore 文件
```

## 安全建议

1. **更改默认密钥**：生产环境中必须更改 JWT_SECRET
2. **使用 HTTPS**：配置反向代理（Nginx/Traefik）启用 HTTPS
3. **定期更新**：保持依赖项和基础镜像的最新版本
4. **限制访问**：使用防火墙限制不必要的端口访问
5. **备份策略**：定期备份数据库和配置文件

## 扩展部署

### Kubernetes 部署

可以使用提供的 Docker 镜像在 Kubernetes 中部署：

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bookmarks
spec:
  replicas: 3
  selector:
    matchLabels:
      app: bookmarks
  template:
    metadata:
      labels:
        app: bookmarks
    spec:
      containers:
      - name: bookmarks
        image: bookmarks:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          value: "sqlite:///app/data/bookmarks.db"
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: bookmarks-secrets
              key: jwt-secret
```

### 云平台部署

支持部署到以下云平台：
- AWS ECS/Fargate
- Google Cloud Run
- Azure Container Instances
- DigitalOcean App Platform

详细配置请参考各平台的官方文档。
