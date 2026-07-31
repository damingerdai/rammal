# rammal

`rammal` 是一个使用 Rust 和 [Actix Web](https://actix.rs/) 编写的轻量级
Docker Registry Proxy。它将 Docker Registry API 请求转发到 Docker Hub，并
重写 Docker Hub 返回的认证挑战，使 Docker 客户端可以继续通过代理完成认证。

项目适合用于需要通过自建服务访问 Docker Hub 的网络环境。

> 当前上游地址和监听地址均写死在程序中：`https://registry-1.docker.io`
> 和 `0.0.0.0:8000`。

## 工作原理

```text
Docker 客户端
    |
    |  /v2/*
    v
rammal :8000  ------------------>  registry-1.docker.io
    |
    |  /auth/token（认证挑战重写后）
    v
auth.docker.io
```

对于 Registry API 请求，rammal 会：

1. 接收 `/v2/{path}` 请求；
2. 将请求方法、请求头、查询参数和请求体转发到
   `https://registry-1.docker.io/v2/{path}`；
3. 以流式方式返回上游响应；
4. 将响应头 `WWW-Authenticate` 中的 `https://auth.docker.io` 替换为
   `/auth`，让认证请求继续经过代理。

`/auth/token` 会将请求转发到 `https://auth.docker.io/token`，并以流式方式
返回认证响应。

## 功能

- 转发 `/v2/*` 下的 Docker Registry HTTP API 请求；
- 通过 `/auth/token` 转发 Docker Hub Token 认证请求；
- 请求体和响应体采用流式处理，避免完整缓冲大文件；
- 使用 `awc` 和 Rustls 连接 Docker Hub HTTPS 服务；
- 提供 `/ping` 健康检查接口；
- 提供 `/version` 构建信息接口；
- 支持多阶段 Docker 镜像构建。

## 环境要求

本地开发需要：

- 支持 Edition 2024 的 Rust toolchain；
- Cargo。

容器化运行需要：

- Docker。

代理所在环境还必须能够访问以下 HTTPS 地址：

- `registry-1.docker.io`；
- `auth.docker.io`。

## 本地运行

```bash
cargo run
```

服务默认监听 `0.0.0.0:8000`。

检查服务是否正常运行：

```bash
curl http://127.0.0.1:8000/ping
# pong

curl http://127.0.0.1:8000/version
```

构建并运行 Release 版本：

```bash
cargo build --release
./target/release/rammal
```

## 使用 Docker 运行

直接构建镜像：

```bash
docker build -t rammal:latest .
```

启动容器并映射端口：

```bash
docker run --rm --name rammal -p 8000:8000 rammal:latest
```

项目还提供了 `build.sh`，会在构建镜像时注入 Git 信息，并默认使用
`damingerdai/rammal` 作为镜像名：

```bash
./build.sh                 # damingerdai/rammal:latest
./build.sh 0.0.1-alpha.0   # damingerdai/rammal:0.0.1-alpha.0
```

## 配置 Docker 客户端

如果代理使用普通 HTTP，仅建议在本地测试时将代理地址加入 Docker 的
`insecure-registries` 配置，然后重启 Docker。生产环境建议在 rammal 前面
部署支持 HTTPS 的反向代理，并使用 HTTPS 地址访问。

例如：

```text
https://docker-proxy.example.com/v2/
```

rammal 当前不负责 TLS 终止、访问控制、限流、持久化缓存或上游地址配置；
生产部署时应由反向代理或网络层补充这些能力。

## HTTP 接口

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/ping` | 返回 `pong`，可用于健康检查。 |
| `GET` | `/version` | 以 JSON 返回构建、Git、Rust 和目标平台信息。 |
| `GET` | `/auth/token` | 转发 Docker Hub Token 接口。 |
| 任意 | `/v2/{path}` | 转发 Docker Registry API 请求。 |

示例：

```bash
curl -i http://127.0.0.1:8000/v2/
```

## 项目结构

```text
src/
├── main.rs              # 应用启动和服务配置
├── routes.rs            # 路由模块声明
├── routes/
│   ├── auth.rs          # Docker Hub Token 代理
│   ├── ping.rs          # 健康检查接口
│   ├── proxy.rs         # Registry API 转发
│   └── version.rs       # 构建信息接口
└── state/
    └── app_state.rs     # 共享 HTTP 客户端状态
```

## 开发检查

提交修改前可以运行：

```bash
cargo fmt --check
cargo check
cargo test
```

## License

当前仓库尚未声明 License。如需发布或再分发，请先补充 License 文件。

英文文档请参阅 [README.md](README.md)。
