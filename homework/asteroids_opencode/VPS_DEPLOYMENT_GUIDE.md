# VPS 部署指南

本指南将帮助你将 Asteroids 游戏部署到 VPS 服务器，提供 Web 端访问和在线多人游戏功能。

---

## 📋 目录

1. [准备工作](#准备工作)
2. [方案 A：仅部署单机版（最快）](#方案-a仅部署单机版)
3. [方案 B：单机版 + WebSocket 服务器](#方案-b单机版--websocket-服务器)
4. [方案 C：完整在线多人（未完成）](#方案-c完整在线多人)
5. [HTTPS 配置](#https-配置)
6. [故障排查](#故障排查)

---

## 准备工作

### 1. VPS 要求
- **操作系统**: Ubuntu 20.04 / 22.04 或 Debian 11+
- **内存**: 最少 1GB（推荐 2GB+）
- **CPU**: 1核心（推荐 2核心+）
- **存储**: 最少 2GB 可用空间
- **网络**: 公网 IP + 域名（可选，HTTPS 需要）

### 2. 安装基础工具

```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装 Nginx
sudo apt install nginx -y

# 安装 Rust（如需编译服务器）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 安装 wasm-pack（如需重新构建 WASM）
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

---

## 方案 A：仅部署单机版

这是**最快的方案**，适合快速上线体验单机游戏。

### 步骤 1：构建 WASM

在**本地**或 VPS 上执行：

```bash
# 克隆仓库（如果还没有）
git clone <your-repo-url>
cd asteroids_opencode

# 构建 Web 版本
./build_web.sh
```

### 步骤 2：上传文件到 VPS

```bash
# 本地打包
cd web
tar -czf asteroids_web.tar.gz *

# 上传到 VPS（替换 your_vps_ip）
scp asteroids_web.tar.gz user@your_vps_ip:~/
```

### 步骤 3：配置 Nginx

```bash
# SSH 到 VPS
ssh user@your_vps_ip

# 解压文件
mkdir -p /var/www/asteroids
cd /var/www/asteroids
tar -xzf ~/asteroids_web.tar.gz

# 创建 Nginx 配置
sudo nano /etc/nginx/sites-available/asteroids
```

粘贴以下内容：

```nginx
server {
    listen 80;
    server_name your_domain.com;  # 替换为你的域名或 IP

    root /var/www/asteroids;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # WASM MIME 类型
    location ~* \.wasm$ {
        add_header Content-Type application/wasm;
        add_header Cache-Control "public, max-age=31536000";
    }

    # 静态资源缓存
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|ttf|woff2?)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

### 步骤 4：启用站点

```bash
# 创建软链接
sudo ln -s /etc/nginx/sites-available/asteroids /etc/nginx/sites-enabled/

# 测试配置
sudo nginx -t

# 重启 Nginx
sudo systemctl restart nginx

# 设置开机自启
sudo systemctl enable nginx
```

### 步骤 5：访问游戏

打开浏览器访问：`http://your_domain.com` 或 `http://your_vps_ip`

✅ **完成！单机版已部署**

---

## 方案 B：单机版 + WebSocket 服务器

添加 WebSocket 服务器以支持在线多人游戏（**前端尚未完全集成**）。

### 步骤 1：构建服务器

```bash
cd server

# 构建 Release 版本
cargo build --release

# 二进制文件位于 target/release/asteroids-server
```

### 步骤 2：配置 systemd 服务

```bash
# 创建服务文件
sudo nano /etc/systemd/system/asteroids-server.service
```

粘贴以下内容：

```ini
[Unit]
Description=Asteroids WebSocket Server
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/asteroids-server
ExecStart=/opt/asteroids-server/asteroids-server
Restart=always
RestartSec=5

# 环境变量（可选）
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
```

### 步骤 3：部署服务器

```bash
# 创建目录
sudo mkdir -p /opt/asteroids-server

# 复制二进制文件
sudo cp target/release/asteroids-server /opt/asteroids-server/

# 设置权限
sudo chown -R www-data:www-data /opt/asteroids-server

# 启动服务
sudo systemctl start asteroids-server
sudo systemctl enable asteroids-server

# 检查状态
sudo systemctl status asteroids-server
```

### 步骤 4：配置 Nginx 反向代理

修改 Nginx 配置：

```nginx
server {
    listen 80;
    server_name your_domain.com;

    # 静态文件
    root /var/www/asteroids;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }

    # WebSocket 代理
    location /ws {
        proxy_pass http://127.0.0.1:9001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket 超时设置
        proxy_read_timeout 3600s;
        proxy_send_timeout 3600s;
    }

    # WASM MIME 类型
    location ~* \.wasm$ {
        add_header Content-Type application/wasm;
    }
}
```

重启 Nginx：

```bash
sudo nginx -t
sudo systemctl restart nginx
```

### 步骤 5：测试 WebSocket

```bash
# 安装 wscat（可选）
npm install -g wscat

# 测试连接
wscat -c ws://your_domain.com/ws
```

✅ **完成！服务器已部署**（前端需要额外开发才能使用）

---

## 方案 C：完整在线多人

**当前状态**：
- ✅ WebSocket 服务器已实现（`server/src/main.rs`）
- ✅ 网络模块框架已创建（`src/network.rs`）
- ❌ 前端 WebSocket 集成未完成
- ❌ 在线模式 UI 未实现
- ❌ 游戏状态同步未实现

**待完成工作**：
1. 在 `src/network.rs` 中实现完整的 WASM WebSocket 客户端
2. 创建在线大厅 UI（昵称输入、模式选择）
3. 实现房间等待界面
4. 实现客户端预测 + 服务器权威的游戏状态同步
5. 处理网络延迟和断线重连

**估计开发时间**：6-8 小时

---

## HTTPS 配置

使用 Let's Encrypt 免费证书（**强烈推荐**）：

```bash
# 安装 Certbot
sudo apt install certbot python3-certbot-nginx -y

# 自动配置 HTTPS
sudo certbot --nginx -d your_domain.com

# 自动续期
sudo certbot renew --dry-run
```

Certbot 会自动修改 Nginx 配置，添加 SSL 并设置 HTTP 到 HTTPS 的重定向。

### 手动 HTTPS 配置（高级）

如果需要自定义：

```nginx
server {
    listen 443 ssl http2;
    server_name your_domain.com;

    ssl_certificate /etc/letsencrypt/live/your_domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your_domain.com/privkey.pem;
    
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # ... 其他配置同上
}

# HTTP 重定向到 HTTPS
server {
    listen 80;
    server_name your_domain.com;
    return 301 https://$server_name$request_uri;
}
```

---

## 故障排查

### 1. 502 Bad Gateway

**原因**：服务器未启动或端口不正确

```bash
# 检查服务器状态
sudo systemctl status asteroids-server

# 查看日志
sudo journalctl -u asteroids-server -f

# 检查端口占用
sudo netstat -tlnp | grep 9001
```

### 2. WASM 无法加载

**原因**：MIME 类型不正确

```bash
# 检查 Nginx 配置
sudo nginx -t

# 确认 MIME 类型
curl -I http://your_domain.com/asteroids_opencode.wasm | grep Content-Type
# 应该返回: Content-Type: application/wasm
```

### 3. WebSocket 连接失败

**原因**：Nginx 未正确配置代理

```bash
# 检查 Nginx 错误日志
sudo tail -f /var/log/nginx/error.log

# 检查防火墙
sudo ufw status
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
```

### 4. 性能问题

```bash
# 增加工作进程
sudo nano /etc/nginx/nginx.conf
# 设置 worker_processes auto;

# 启用 gzip 压缩
http {
    gzip on;
    gzip_types application/javascript application/wasm text/css;
}
```

---

## 监控和维护

### 日志查看

```bash
# Nginx 访问日志
sudo tail -f /var/log/nginx/access.log

# Nginx 错误日志
sudo tail -f /var/log/nginx/error.log

# 服务器日志
sudo journalctl -u asteroids-server -f
```

### 自动更新

```bash
# 创建更新脚本
nano ~/update_asteroids.sh
```

```bash
#!/bin/bash
cd /path/to/asteroids_opencode

# 拉取最新代码
git pull

# 重新构建 Web
./build_web.sh

# 复制文件
sudo cp -r web/* /var/www/asteroids/

# 重新构建服务器（如果有更改）
cd server
cargo build --release
sudo systemctl stop asteroids-server
sudo cp target/release/asteroids-server /opt/asteroids-server/
sudo systemctl start asteroids-server

echo "部署完成！"
```

```bash
chmod +x ~/update_asteroids.sh
```

---

## 备份策略

```bash
# 备份脚本
sudo nano /usr/local/bin/backup_asteroids.sh
```

```bash
#!/bin/bash
BACKUP_DIR="/backup/asteroids"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR

# 备份 Web 文件
tar -czf $BACKUP_DIR/web_$DATE.tar.gz -C /var/www asteroids

# 备份服务器二进制
tar -czf $BACKUP_DIR/server_$DATE.tar.gz -C /opt asteroids-server

# 删除 7 天前的备份
find $BACKUP_DIR -name "*.tar.gz" -mtime +7 -delete

echo "备份完成：$DATE"
```

```bash
sudo chmod +x /usr/local/bin/backup_asteroids.sh

# 添加定时任务（每天凌晨 2 点）
sudo crontab -e
# 添加: 0 2 * * * /usr/local/bin/backup_asteroids.sh
```

---

## 性能优化

### 1. 启用 HTTP/2

```nginx
listen 443 ssl http2;
```

### 2. 启用 Brotli 压缩（可选）

```bash
# 安装 Brotli 模块
sudo apt install nginx-module-brotli -y

# 在 Nginx 配置中启用
load_module modules/ngx_http_brotli_filter_module.so;
load_module modules/ngx_http_brotli_static_module.so;

http {
    brotli on;
    brotli_types application/javascript application/wasm text/css;
}
```

### 3. CDN 加速（高级）

使用 Cloudflare 或其他 CDN 服务加速全球访问。

---

## 总结

- **方案 A**：最快部署（1 小时），仅单机游戏 ✅
- **方案 B**：增加服务器基础设施（2-3 小时）✅
- **方案 C**：完整在线多人（需要额外 6-8 小时开发）⏳

### 推荐路径

1. **立即部署**：使用方案 A 快速上线单机版
2. **逐步升级**：后续添加服务器（方案 B）
3. **完整功能**：完成前端集成（方案 C）

---

## 支持与反馈

- 📖 完整文档：查看项目 `README.md`
- 🐛 问题反馈：在 GitHub 提交 Issue
- 💬 讨论交流：加入项目 Discussions

**祝你部署顺利！** 🚀
