# 安装部署指南

本指南详细说明如何在不同环境中安装和部署书签管理系统。

## 📋 目录

- [环境要求](#环境要求)
- [开发环境搭建](#开发环境搭建)
- [生产环境部署](#生产环境部署)
- [Docker 部署](#docker-部署)
- [云平台部署](#云平台部署)
- [配置说明](#配置说明)
- [故障排除](#故障排除)

## 🔧 环境要求

### 基础要求

- **操作系统**: Linux, macOS, Windows
- **内存**: 最少 2GB RAM
- **存储**: 最少 1GB 可用空间
- **网络**: 稳定的互联网连接

### 软件依赖

#### 后端依赖

- **Rust**: 1.75.0 或更高版本
- **SQLite**: 3.0 或更高版本
- **OpenSSL**: 用于加密功能

#### 前端依赖

- **Node.js**: 18.0.0 或更高版本
- **npm**: 9.0.0 或更高版本

### 可选工具

- **Git**: 版本控制
- **Docker**: 容器化部署
- **Make**: 构建自动化

## 🛠️ 开发环境搭建

### 1. 克隆项目

```bash
git clone <repository-url>
cd bookmarks
```

### 2. 安装 Rust

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 3. 安装 Node.js

#### 使用 nvm (推荐)

```bash
# 安装 nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc

# 安装 Node.js
nvm install 18
nvm use 18
```

#### 直接下载

从 [Node.js 官网](https://nodejs.org/) 下载并安装。

### 4. 后端设置

```bash
cd backend

# 创建环境配置文件
cp .env.example .env

# 编辑配置文件
nano .env
```

环境配置示例：

```env
# 数据库配置
DATABASE_URL=sqlite:bookmarks.db

# JWT 配置
JWT_SECRET=your-super-secret-jwt-key-here
JWT_EXPIRES_IN=15m

# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# 日志配置
RUST_LOG=info
```

### 5. 数据库初始化

```bash
# 安装 SQLx CLI
cargo install sqlx-cli

# 运行数据库迁移
sqlx migrate run

# 验证数据库
sqlite3 bookmarks.db ".tables"
```

### 6. 启动后端服务

```bash
# 开发模式运行
cargo run

# 或者使用 watch 模式（需要安装 cargo-watch）
cargo install cargo-watch
cargo watch -x run
```

### 7. 前端设置

```bash
cd frontend

# 安装依赖
npm install

# 创建环境配置文件
cp .env.example .env.local

# 编辑配置文件
nano .env.local
```

前端配置示例：

```env
# API 配置
VITE_API_BASE_URL=http://localhost:3000/api

# 应用配置
VITE_APP_NAME=Bookmarks
VITE_APP_VERSION=1.0.0
```

### 8. 启动前端服务

```bash
# 开发模式运行
npm run dev

# 或者使用 TypeScript 检查
npm run type-check
```

### 9. 验证安装

访问以下地址验证服务运行状态：

- 前端应用: <http://localhost:5173>
- 后端 API: <http://localhost:3000/api/auth/me>
- API 健康检查: <http://localhost:3000/health>

## 🚀 生产环境部署

### 1. 服务器准备

#### 系统要求

- **CPU**: 2 核心或更多
- **内存**: 4GB RAM 或更多
- **存储**: 20GB SSD 或更多
- **操作系统**: Ubuntu 20.04+ / CentOS 8+ / Debian 11+

#### 系统更新

```bash
# Ubuntu/Debian
sudo apt update && sudo apt upgrade -y

# CentOS/RHEL
sudo yum update -y
```

### 2. 安装依赖

```bash
# Ubuntu/Debian
sudo apt install -y build-essential pkg-config libssl-dev sqlite3 nginx

# CentOS/RHEL
sudo yum groupinstall -y "Development Tools"
sudo yum install -y openssl-devel sqlite nginx
```

### 3. 部署用户设置

```bash
# 创建部署用户
sudo useradd -m -s /bin/bash bookmarks
sudo usermod -aG sudo bookmarks

# 切换到部署用户
sudo su - bookmarks
```

### 4. 应用部署

#### 克隆代码

```bash
cd /home/bookmarks
git clone <repository-url> app
cd app
```

#### 后端构建

```bash
cd backend

# 生产构建
cargo build --release

# 创建服务目录
sudo mkdir -p /opt/bookmarks
sudo cp target/release/bookmarks /opt/bookmarks/
sudo cp -r migrations /opt/bookmarks/
```

#### 前端构建

```bash
cd frontend

# 安装依赖
npm ci --only=production

# 构建生产版本
npm run build

# 部署静态文件
sudo mkdir -p /var/www/bookmarks
sudo cp -r dist/* /var/www/bookmarks/
```

### 5. 配置生产环境

#### 后端配置

```bash
# 创建生产配置
sudo mkdir -p /etc/bookmarks
sudo nano /etc/bookmarks/.env
```

生产环境配置：

```env
# 数据库配置
DATABASE_URL=sqlite:/opt/bookmarks/data/bookmarks.db

# JWT 配置
JWT_SECRET=your-production-jwt-secret-key
JWT_EXPIRES_IN=15m

# 服务器配置
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# 日志配置
RUST_LOG=warn

# 生产环境标识
ENVIRONMENT=production
```

#### 创建数据库目录

```bash
sudo mkdir -p /opt/bookmarks/data
sudo chown -R bookmarks:bookmarks /opt/bookmarks
```

#### 运行数据库迁移

```bash
cd /opt/bookmarks
sudo -u bookmarks sqlx migrate run --database-url "sqlite:/opt/bookmarks/data/bookmarks.db"
```

### 6. 系统服务配置

#### 创建 systemd 服务

```bash
sudo nano /etc/systemd/system/bookmarks.service
```

服务配置文件：

```ini
[Unit]
Description=Bookmarks Management System
After=network.target

[Service]
Type=simple
User=bookmarks
Group=bookmarks
WorkingDirectory=/opt/bookmarks
Environment=DATABASE_URL=sqlite:/opt/bookmarks/data/bookmarks.db
Environment=JWT_SECRET=your-production-jwt-secret-key
Environment=RUST_LOG=warn
ExecStart=/opt/bookmarks/bookmarks
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

#### 启用和启动服务

```bash
# 重新加载 systemd
sudo systemctl daemon-reload

# 启用服务
sudo systemctl enable bookmarks

# 启动服务
sudo systemctl start bookmarks

# 检查状态
sudo systemctl status bookmarks
```

### 7. Nginx 配置

#### 创建 Nginx 配置

```bash
sudo nano /etc/nginx/sites-available/bookmarks
```

Nginx 配置文件：

```nginx
server {
    listen 80;
    server_name your-domain.com;

    # 前端静态文件
    location / {
        root /var/www/bookmarks;
        index index.html;
        try_files $uri $uri/ /index.html;
    }

    # API 代理
    location /api {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }

    # 静态资源缓存
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

#### 启用站点

```bash
# 启用站点
sudo ln -s /etc/nginx/sites-available/bookmarks /etc/nginx/sites-enabled/

# 测试配置
sudo nginx -t

# 重启 Nginx
sudo systemctl restart nginx
```

### 8. SSL 证书配置

#### 使用 Let's Encrypt

```bash
# 安装 Certbot
sudo apt install certbot python3-certbot-nginx

# 获取证书
sudo certbot --nginx -d your-domain.com

# 自动续期
sudo crontab -e
```

添加自动续期任务：

```crontab
0 12 * * * /usr/bin/certbot renew --quiet
```

## 🐳 Docker 部署

### 1. 创建 Dockerfile

#### 后端 Dockerfile

```dockerfile
# backend/Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# 构建应用
RUN cargo build --release

# 运行时镜像
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制构建产物
COPY --from=builder /app/target/release/bookmarks /usr/local/bin/
COPY --from=builder /app/migrations ./migrations

# 创建数据目录
RUN mkdir -p /data

# 设置用户
RUN useradd -r -s /bin/false bookmarks
USER bookmarks

EXPOSE 3000

CMD ["bookmarks"]
```

#### 前端 Dockerfile

```dockerfile
# frontend/Dockerfile
FROM node:18-alpine as builder

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

# Nginx 服务镜像
FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

### 2. Docker Compose

创建 `docker-compose.yml`：

```yaml
version: '3.8'

services:
  backend:
    build:
      context: ./backend
      dockerfile: Dockerfile
    environment:
      - DATABASE_URL=sqlite:/data/bookmarks.db
      - JWT_SECRET=your-docker-jwt-secret
      - RUST_LOG=info
    volumes:
      - ./data:/data
    ports:
      - "3000:3000"
    restart: unless-stopped

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    ports:
      - "80:80"
    depends_on:
      - backend
    restart: unless-stopped

volumes:
  data:
```

### 3. 部署命令

```bash
# 构建和启动
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down

# 重新构建
docker-compose up -d --build
```

## ☁️ 云平台部署

### Vercel 部署（前端）

```bash
# 安装 Vercel CLI
npm install -g vercel

# 部署
cd frontend
vercel --prod
```

### Railway 部署（后端）

```bash
# 安装 Railway CLI
npm install -g @railway/cli

# 登录
railway login

# 部署
cd backend
railway up
```

### Docker Cloud 部署

```bash
# 构建镜像
docker build -t your-username/bookmarks .

# 推送到 Docker Hub
docker push your-username/bookmarks

# 部署到云平台
# 根据具体平台操作
```

## ⚙️ 配置说明

### 环境变量

#### 后端环境变量

| 变量名 | 必需 | 默认值 | 说明 |
|--------|------|--------|------|
| `DATABASE_URL` | 是 | - | SQLite 数据库路径 |
| `JWT_SECRET` | 是 | - | JWT 签名密钥 |
| `JWT_EXPIRES_IN` | 否 | 15m | Token 过期时间 |
| `SERVER_HOST` | 否 | 0.0.0.0 | 服务器监听地址 |
| `SERVER_PORT` | 否 | 3000 | 服务器端口 |
| `RUST_LOG` | 否 | info | 日志级别 |
| `ENVIRONMENT` | 否 | development | 运行环境 |

#### 前端环境变量

| 变量名 | 必需 | 默认值 | 说明 |
|--------|------|--------|------|
| `VITE_API_BASE_URL` | 是 | <http://localhost:3000/api> | API 基础地址 |
| `VITE_APP_NAME` | 否 | Bookmarks | 应用名称 |
| `VITE_APP_VERSION` | 否 | 1.0.0 | 应用版本 |

### 数据库配置

#### SQLite 优化

```sql
-- 性能优化设置
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = 10000;
PRAGMA foreign_keys = ON;
```

### 安全配置

#### JWT 安全

- 使用强密钥（至少 32 字符）
- 定期轮换密钥
- 设置合理的过期时间

#### 网络安全

- 使用 HTTPS
- 配置防火墙
- 限制数据库访问

## 🔧 故障排除

### 常见问题

#### 1. 后端启动失败

**问题**: 服务无法启动

**解决方案**:

```bash
# 检查日志
sudo journalctl -u bookmarks -f

# 检查端口占用
sudo netstat -tlnp | grep 3000

# 检查配置文件
cat /etc/bookmarks/.env
```

#### 2. 数据库连接失败

**问题**: 无法连接到数据库

**解决方案**:

```bash
# 检查数据库文件权限
ls -la /opt/bookmarks/data/

# 检查 SQLite 版本
sqlite3 --version

# 手动测试数据库
sqlite3 /opt/bookmarks/data/bookmarks.db ".tables"
```

#### 3. 前端构建失败

**问题**: npm 构建错误

**解决方案**:

```bash
# 清理缓存
npm cache clean --force

# 删除 node_modules
rm -rf node_modules package-lock.json

# 重新安装
npm install
```

#### 4. Nginx 配置错误

**问题**: 502 Bad Gateway

**解决方案**:

```bash
# 检查 Nginx 配置
sudo nginx -t

# 检查后端服务状态
sudo systemctl status bookmarks

# 查看 Nginx 日志
sudo tail -f /var/log/nginx/error.log
```

### 日志分析

#### 后端日志

```bash
# 实时日志
sudo journalctl -u bookmarks -f

# 历史日志
sudo journalctl -u bookmarks --since "1 hour ago"
```

#### Nginx 日志

```bash
# 访问日志
sudo tail -f /var/log/nginx/access.log

# 错误日志
sudo tail -f /var/log/nginx/error.log
```

### 性能监控

#### 系统监控

```bash
# CPU 和内存使用
top
htop

# 磁盘使用
df -h

# 网络连接
netstat -tlnp
```

#### 应用监控

```bash
# 进程状态
ps aux | grep bookmarks

# 端口监听
ss -tlnp | grep 3000
```

## 📞 支持

如果遇到问题，请：

1. 查看本文档的故障排除部分
2. 检查项目的 GitHub Issues
3. 提交新的 Issue 并包含详细的错误信息
4. 联系技术支持团队

---

**更新时间**: 2025-12-02
**版本**: 1.0.0
