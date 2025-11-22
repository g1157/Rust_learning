# Asteroids 游戏 - Web 部署指南

## 🎯 快速开始

### 本地测试

使用内置脚本启动本地服务器：

```bash
./serve.sh
```

然后访问：http://localhost:8000

### 生产构建

使用自动化构建脚本：

```bash
./build_web.sh
```

这会：
1. 编译优化的 WASM 文件
2. 复制所有资源到 `web/` 目录
3. 验证所有文件完整性

## 📦 部署文件结构

```
web/
├── index.html                    # 主页面
├── mq_js_bundle.js               # Macroquad JS 引擎 (36 KB)
├── asteroids_opencode.wasm       # 游戏主程序 (623 KB)
└── assets/
    ├── sounds/                   # 音效文件
    │   ├── shoot.wav            # 射击音效
    │   ├── powerup.wav          # 拾取道具音效
    │   ├── explosion.wav → powerup.wav  # 符号链接
    │   ├── thrust.wav → shoot.wav
    │   └── hit.wav → powerup.wav
    └── fonts/                    # 字体文件
        ├── DejaVuSans.ttf       # 主字体
        ├── ubuntu.ttf           # 备用字体
        ├── wqy-microhei.ttc     # 中文字体
        └── font.ttf → DejaVuSans.ttf  # 符号链接
```

**总大小**: ~7.3 MB（未压缩）

## 🚀 VPS 部署步骤

### 方案 A: Nginx 静态托管

1. **上传文件到服务器**

```bash
# 打包 web 目录
tar -czf asteroids-web.tar.gz -C web .

# 上传到服务器
scp asteroids-web.tar.gz user@your-server:/var/www/asteroids/

# SSH 到服务器并解压
ssh user@your-server
cd /var/www/asteroids
tar -xzf asteroids-web.tar.gz
```

2. **配置 Nginx**

创建 `/etc/nginx/sites-available/asteroids`:

```nginx
server {
    listen 80;
    server_name your-domain.com;
    
    root /var/www/asteroids;
    index index.html;
    
    # WASM MIME 类型
    types {
        application/wasm wasm;
    }
    
    # 启用 gzip 压缩
    gzip on;
    gzip_types application/wasm application/javascript text/css text/html;
    gzip_min_length 1000;
    
    # 缓存策略
    location ~* \.(wasm|js)$ {
        expires 1d;
        add_header Cache-Control "public, immutable";
    }
    
    location ~* \.(ttf|ttc|wav)$ {
        expires 7d;
        add_header Cache-Control "public, immutable";
    }
    
    # 处理符号链接
    location / {
        try_files $uri $uri/ =404;
    }
}
```

3. **启用站点**

```bash
sudo ln -s /etc/nginx/sites-available/asteroids /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

4. **（可选）配置 HTTPS**

```bash
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d your-domain.com
```

### 方案 B: Caddy 服务器（更简单）

1. **安装 Caddy**

```bash
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy
```

2. **配置 Caddyfile**

创建 `/etc/caddy/Caddyfile`:

```
your-domain.com {
    root * /var/www/asteroids
    file_server
    
    encode gzip
    
    header {
        Cache-Control "public, max-age=3600"
    }
}
```

3. **重启 Caddy**

```bash
sudo systemctl reload caddy
```

Caddy 会自动处理 HTTPS！

### 方案 C: GitHub Pages（免费托管）

1. **创建 GitHub 仓库**

```bash
cd web/
git init
git add .
git commit -m "Initial commit"
git branch -M main
git remote add origin https://github.com/your-username/asteroids-game.git
git push -u origin main
```

2. **启用 GitHub Pages**

- 前往仓库设置 → Pages
- Source: Deploy from branch
- Branch: main, /root

3. **访问游戏**

https://your-username.github.io/asteroids-game/

**注意**: GitHub Pages 不支持符号链接，需要复制实际文件：

```bash
cd web/assets/sounds/
cp powerup.wav explosion.wav
cp powerup.wav hit.wav
cp shoot.wav thrust.wav

cd ../fonts/
cp DejaVuSans.ttf font.ttf
```

## 🔧 性能优化

### 1. WASM 文件优化

当前配置已经很优化：

```toml
[profile.release]
opt-level = 'z'     # 最小化体积
lto = true          # 链接时优化
```

进一步优化（可选）：

```bash
# 安装 wasm-opt（来自 Binaryen）
sudo apt install binaryen

# 优化 WASM
wasm-opt -Oz -o asteroids_optimized.wasm asteroids_opencode.wasm
```

### 2. 启用 Brotli 压缩

Nginx 配置：

```nginx
brotli on;
brotli_types application/wasm application/javascript;
```

预期压缩率：
- WASM: 623 KB → ~200 KB (gzip) → ~180 KB (brotli)
- JS: 36 KB → ~12 KB

### 3. CDN 加速

将静态资源上传到 CDN：
- Cloudflare (免费)
- AWS CloudFront
- 阿里云 OSS

## 📊 监控和分析

### 添加 Google Analytics（可选）

在 `index.html` 的 `<head>` 中添加：

```html
<!-- Google Analytics -->
<script async src="https://www.googletagmanager.com/gtag/js?id=GA_MEASUREMENT_ID"></script>
<script>
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());
  gtag('config', 'GA_MEASUREMENT_ID');
</script>
```

## 🐛 故障排查

### 问题 1: WASM 加载失败

**症状**: 控制台显示 `Failed to load WASM`

**解决**:
1. 检查 MIME 类型：`curl -I https://your-domain.com/asteroids_opencode.wasm | grep Content-Type`
2. 应该返回 `application/wasm`

### 问题 2: 画布尺寸错误

**症状**: 游戏窗口太小

**解决**: 确保使用最新的 `asteroids_opencode.wasm`（包含 `window_conf()` 配置）

### 问题 3: 成就不保存

**症状**: 刷新后成就重置

**解决**:
1. 打开浏览器控制台
2. 检查 LocalStorage: `localStorage.getItem('achievements')`
3. 如果为空，检查是否使用了 quad-storage 版本的 WASM

### 问题 4: 音效不播放

**症状**: 游戏静音

**解决**:
1. 检查浏览器自动播放策略（需要用户交互）
2. 检查音频文件路径：`web/assets/sounds/*.wav`
3. 查看控制台错误

## 🔒 安全注意事项

1. **CORS 头部**（如果需要跨域）

```nginx
add_header Access-Control-Allow-Origin "https://your-domain.com";
```

2. **CSP 策略**（内容安全策略）

```nginx
add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline';";
```

3. **禁用目录列表**

```nginx
autoindex off;
```

## 📱 移动端支持

当前游戏使用键盘控制，不支持触屏。

未来改进：
- 添加虚拟摇杆
- 触屏按钮
- 陀螺仪控制

## 🎮 功能特性

### 已实现
- ✅ 双人本地多人游戏
- ✅ 生存模式和对战模式
- ✅ 成就系统（LocalStorage 持久化）
- ✅ 粒子效果
- ✅ 音效系统
- ✅ 响应式画布

### 未来计划
- ⏳ WebSocket 在线多人游戏
- ⏳ 排行榜系统
- ⏳ 触屏控制
- ⏳ PWA 支持（离线游玩）

## 📄 许可证

本游戏基于原始 Asteroids 概念重制，供学习和娱乐使用。

## 🆘 获取帮助

- GitHub Issues: [报告 bug]
- 邮件联系: your-email@example.com

---

**构建时间**: `date`  
**WASM 版本**: 0.3.0  
**引擎**: Macroquad 0.4.14
