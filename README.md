# rammal

`rammal` is a lightweight Docker Registry proxy written in Rust with
[Actix Web](https://actix.rs/). It forwards Docker Registry API requests to
Docker Hub and rewrites Docker Hub's authentication challenge so Docker
clients can continue authentication through the proxy.

The project is useful when Docker clients need a small, self-hosted endpoint
for reaching Docker Hub through a controlled network path.

> The upstream registry and listening address are currently built into the
> application: `https://registry-1.docker.io` and `0.0.0.0:8000`.

## How it works

```text
Docker client
    |
    |  /v2/*
    v
rammal :8000  ------------------>  registry-1.docker.io
    |
    |  /auth/token (after challenge rewrite)
    v
auth.docker.io
```

For Registry API requests, rammal:

1. accepts the request under `/v2/{path}`;
2. forwards the method, request headers, query string, and request body to
   `https://registry-1.docker.io/v2/{path}`;
3. streams the upstream response back to the client; and
4. changes `https://auth.docker.io` in the `WWW-Authenticate` response header
   to `/auth`, keeping the authentication flow inside the proxy.

The `/auth/token` endpoint forwards token requests to
`https://auth.docker.io/token` and streams the response back.

## Features

- Docker Registry HTTP API proxying for `/v2/*`.
- Docker Hub token authentication proxying through `/auth/token`.
- Streaming request and response bodies instead of buffering the complete
  payload in memory.
- HTTPS connections to Docker Hub using `awc` and Rustls.
- `/ping` health endpoint.
- `/version` build metadata endpoint.
- Multi-stage Docker image build.

## Requirements

For local development:

- Rust toolchain with Edition 2024 support
- Cargo

For containerized deployment:

- Docker

The proxy also needs outbound HTTPS access to:

- `registry-1.docker.io`
- `auth.docker.io`

## Run locally

Start the development server with:

```bash
cargo run
```

The server listens on `0.0.0.0:8000`.

You can verify that it is running:

```bash
curl http://127.0.0.1:8000/ping
# pong

curl http://127.0.0.1:8000/version
```

To build an optimized binary:

```bash
cargo build --release
./target/release/rammal
```

## Run with Docker

Build the image directly:

```bash
docker build -t rammal:latest .
```

Run it and publish the proxy port:

```bash
docker run --rm --name rammal -p 8000:8000 rammal:latest
```

The repository also includes `build.sh`, which passes Git metadata to the
image build and tags the image as `damingerdai/rammal`:

```bash
./build.sh                 # damingerdai/rammal:latest
./build.sh 0.0.1-alpha.0   # damingerdai/rammal:0.0.1-alpha.0
```

## Configure a Docker client

Point Docker at the proxy as an insecure registry only when the proxy is
served over plain HTTP, for example during local testing. Add the endpoint to
Docker's `insecure-registries` configuration, restart Docker, and use the
proxy host as the registry endpoint.

For production, place rammal behind a TLS-terminating reverse proxy and use
the HTTPS endpoint. A typical client request path will then look like:

```text
https://docker-proxy.example.com/v2/
```

The proxy itself does not currently provide TLS termination, access control,
rate limiting, persistent caching, or configurable upstream registries.

## HTTP endpoints

| Method | Path | Description |
| --- | --- | --- |
| `GET` | `/ping` | Returns `pong`. Suitable for health checks. |
| `GET` | `/version` | Returns build, Git, Rust, and target metadata as JSON. |
| `GET` | `/auth/token` | Proxies the Docker Hub token endpoint. |
| Any | `/v2/{path}` | Proxies the Docker Registry API request. |

Example Registry API request:

```bash
curl -i http://127.0.0.1:8000/v2/
```

An authentication response from Docker Hub may contain a rewritten challenge
similar to:

```text
WWW-Authenticate: Bearer realm="/auth",service="registry.docker.io",scope="..."
```

## Project layout

```text
src/
├── main.rs              # Application bootstrap and server configuration
├── routes.rs            # Route module declarations
├── routes/
│   ├── auth.rs          # Docker Hub token proxy
│   ├── ping.rs          # Health endpoint
│   ├── proxy.rs         # Registry API forwarding
│   └── version.rs       # Build metadata endpoint
└── state/
    └── app_state.rs     # Shared HTTP client state
```

## Development

Format and check the project before submitting changes:

```bash
cargo fmt --check
cargo check
cargo test
```

## License

No license has been declared in the repository yet. Add a license file before
redistributing the project.

For Chinese documentation, see [README_ZH.md](README_ZH.md).
