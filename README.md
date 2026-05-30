# AgentBox

AgentBox - 基于 Rust + Docker 的 AI Agent 运行沙箱平台。为 Claude Agent SDK 等 AI Agent 提供隔离、安全、可管理的容器化运行环境。

## 用途

- **代码审查**：为每个 PR 创建独立容器，运行 Claude Agent 进行自动化代码审查
- **代码生成**：隔离执行 AI 生成的代码，防止对主环境造成影响
- **多租户 Agent 服务**：不同业务团队使用各自的 Skill 和配置，互不干扰
- **CI/CD 集成**：在流水线中动态创建 Agent 容器执行任务

## 架构

```
┌──────────────────────────────────────────────────────────────┐
│                    Control Plane (主服务)                     │
│  REST API ─ Auth ─ Docker Manager ─ Lifecycle ─ SQLite       │
│                       │           │                           │
│                       │           └─ WebSocket 日志流(脱敏)    │
└───────────────────────┼───────────────────────────────────────┘
                        │ Docker Socket
                        ▼
┌──────────────────────────────────────────────────────────────┐
│                    Agent Container                            │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ Sidecar (Axum :9000)                                     │ │
│  │  ├─ POST /query   → SSE 流式返回 cc_sdk::query 消息       │ │
│  │  ├─ GET  /health                                         │ │
│  │  └─ 心跳上报 → control-plane /api/containers/{id}/status  │ │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### 核心组件

| 组件 | 语言 | 职责 |
|------|------|------|
| **Control Plane** | Rust | 容器生命周期管理、REST/WebSocket API、鉴权、CORS、日志脱敏、Docker 实际状态校验、空闲销毁 |
| **Sidecar** | Rust | 容器内 HTTP server（`:9000`）；封装 `cc-sdk::query`，SSE 流式返回 Claude 消息；状态/心跳回传 |
| **Agent Image** | Docker | 包含 Sidecar 二进制 + Claude Code CLI（`@anthropic-ai/claude-code`）的容器镜像 |
| **Admin UI** | React + Vite | 前端管理界面；容器 CRUD、实时 WebSocket 日志流、状态监控 |

### 技术栈

- **Web 框架**：Axum 0.8（HTTP / WebSocket / SSE）
- **Claude SDK**：cc-sdk 0.8（sidecar 内调用 Claude Code）
- **Docker 客户端**：Bollard 0.21
- **数据库**：SQLite (sqlx 0.8)
- **异步运行时**：Tokio 1.x
- **日志**：Tracing
- **前端**：React 19 + Vite + shadcn/ui + Tailwind CSS v4

## 快速开始

### 前置要求

- Rust 1.70+
- Docker Desktop / OrbStack
- Node.js 20+（仅开发 admin-ui 时需要）

### 一键启动（推荐）

```bash
git clone <repo-url>
cd agentbox
make setup    # 编译 Rust + 构建所有 Docker 镜像 + 启动服务
```

启动后：
- Control Plane API: `http://localhost:8080`
- Admin UI: `http://localhost:3000`

### 本地开发

```bash
# 安装前端依赖
cd admin-ui && npm install && cd ..

# 启动后端 + 前端开发服务器（热更新）
make dev
```

- Control Plane: `http://localhost:8080`
- Admin UI (dev): `http://localhost:5173`

### Docker 运行

```bash
# 构建所有镜像（含 agent-sandbox）
make build

# 启动全部（control-plane + admin-ui）
docker compose up -d

# 仅启动 control-plane
docker compose up control-plane -d
```

## API 文档

### 健康检查

```http
GET /health
```

**响应**:
```json
{
  "status": "ok",
  "timestamp": "2026-05-28T10:23:49.544829+00:00"
}
```

### 创建容器

```http
POST /api/containers
Content-Type: application/json
```

**请求体**:
```json
{
  "task": "Review code PR #42",
  "skill_repos": ["https://github.com/company/skills.git"],
  "skill_branch": "main",
  "cpu_limit": "2",
  "memory_limit": "4Gi",
  "idle_timeout": 300,
  "max_lifetime": 3600,
  "env": {
    "ANTHROPIC_API_KEY": "your-key"
  }
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `task` | string | ✅ | Agent 执行的任务描述 |
| `skill_repos` | string[] | ❌ | Skill 仓库地址列表（可选） |
| `skill_branch` | string | ❌ | Skill 仓库分支，默认 `main` |
| `cpu_limit` | string | ❌ | CPU 限制，默认 `2` (核) |
| `memory_limit` | string | ❌ | 内存限制，默认 `4Gi` |
| `idle_timeout` | integer | ❌ | 空闲超时(秒)，默认 300 |
| `max_lifetime` | integer | ❌ | 最大生命周期(秒)，默认 3600 |
| `env` | object | ❌ | 额外环境变量 |

**响应** (201 Created):
```json
{
  "id": "9c52cbc6-20a4-48a9-a7fb-cc8e8d64319b",
  "status": "Running",
  "created_at": "2026-05-28T10:23:51.155912+00:00",
  "docker_id": "agent-9c52cbc6-20a4-48a9-a7fb-cc8e8d64319b"
}
```

### 查询容器状态

```http
GET /api/containers/{id}
```

**响应** (200 OK):
```json
{
  "id": "9c52cbc6-20a4-48a9-a7fb-cc8e8d64319b",
  "task": "Review code PR #42",
  "status": "Running",
  "docker_id": "agent-9c52cbc6-20a4-48a9-a7fb-cc8e8d64319b",
  "skill_repos": "[\"https://github.com/company/skills.git\"]",
  "cpu_limit": "2",
  "memory_limit": "4Gi",
  "idle_timeout": 300,
  "max_lifetime": 3600,
  "created_at": "2026-05-28T10:23:51.155912+00:00",
  "last_activity": "2026-05-28T10:23:53.459101+00:00"
}
```

### 删除容器

```http
DELETE /api/containers/{id}
```

**响应** (204 No Content)

### 容器日志流 (WebSocket)

```http
GET /api/containers/{id}/logs
```

WebSocket 端点，实时推送容器 stdout/stderr。已对启动时收集到的密钥值（`ANTHROPIC_API_KEY`、`API_KEY`、`OPENAI_API_KEY`、`GITHUB_TOKEN`、`GH_TOKEN`）做 `***REDACTED***` 替换。

```bash
wscat -H "Authorization: Bearer $API_KEY" \
      -c ws://localhost:8080/api/containers/<id>/logs
```

### 状态回传 (Sidecar → Control Plane)

```http
POST /api/containers/{id}/status
Content-Type: application/json
```

**请求体**:
```json
{
  "status": "running",
  "progress": 0.5,
  "current_step": "analyzing code",
  "logs": ["Reading src/main.rs", "Checking imports"],
  "timestamp": "2026-05-28T10:24:00Z"
}
```

## Sidecar API（容器内 `:9000`）

由 control-plane 透传或在同一 Docker 网络下直连 `agent-{id}:9000`。

### 健康检查

```http
GET /health        → 200 "ok"
```

### Query (SSE)

```http
POST /query
Content-Type: application/json
Accept: text/event-stream
```

**请求体**:
```json
{
  "prompt": "重构这段代码",
  "options": {
    "model": "claude-opus-4-7",
    "system_prompt": "你是 Rust 专家",
    "max_turns": 5,
    "allowed_tools": ["Read", "Edit"]
  }
}
```

`options` 字段（任意可选；未识别字段会被忽略）：

| 字段 | 类型 | 说明 |
|------|------|------|
| `model` | string | 主模型 |
| `fallback_model` | string | 兜底模型 |
| `system_prompt` | string | 系统提示词 |
| `append_system_prompt` | string | 追加到默认系统提示词后 |
| `max_turns` | i32 | 最大对话轮次 |
| `max_output_tokens` | u32 | 输出 token 上限 |
| `max_thinking_tokens` | i32 | thinking token 上限 |
| `allowed_tools` | string[] | 允许的工具白名单 |
| `disallowed_tools` | string[] | 禁用工具黑名单 |
| `cwd` | string | 工作目录 |
| `session_id` | string | 复用 session ID |
| `resume` | string | 从指定 session 续跑 |
| `continue_conversation` | bool | 继续上一次会话 |
| `include_partial_messages` | bool | 是否推送增量消息 |
| `max_budget_usd` | f64 | 单次预算上限 |

需要更多字段在 `sidecar/src/query.rs::build_options` 加一行 builder 调用即可。

**响应**: `Content-Type: text/event-stream`，每条 `cc_sdk::Message` 一个事件，类型即 event 名：

```
event: assistant
data: {"type":"assistant","message":{...}}

event: stream_event
data: {"type":"stream_event","uuid":"...","event":{...}}

event: result
data: {"type":"result","duration_ms":1234,"total_cost_usd":0.0012,...}
```

| Event | 说明 |
|-------|------|
| `assistant` / `user` / `system` | 普通对话消息 |
| `stream_event` | 流式增量（启用 `include_partial_messages` 时） |
| `rate_limit` | 速率受限通知 |
| `result` | 终止事件，附 duration / cost / usage |
| `error` | 流中错误，随后流自然结束 |

Keep-alive 间隔 15s。客户端按 SSE 标准断线重连即可。

**curl 示例**:
```bash
curl -N -X POST http://agent-<id>:9000/query \
  -H 'Content-Type: application/json' \
  -d '{"prompt":"hello"}'
```

## 容器状态

| 状态 | 说明 |
|------|------|
| `Creating` | 正在创建 |
| `Running` | 运行中 |
| `Idle` | 空闲 |
| `Stopping` | 正在停止 |
| `Stopped` | 已停止 |
| `Failed` | 失败 |

## 配置

### 环境变量 (Control Plane)

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `DATABASE_URL` | `sqlite:agent_sandbox.db?mode=rwc` | SQLite 数据库路径 |
| `SERVER_ADDR` | `0.0.0.0:8080` | 监听地址 |
| `AGENT_IMAGE` | `agent-sandbox:latest` | Agent 容器镜像 |
| `API_KEY` | *(unset)* | 设置后所有非 `/health` 路由要求 `Authorization: Bearer <key>`；未设置则不鉴权（开发模式） |
| `ALLOWED_ORIGINS` | localhost only | CORS 允许来源；逗号分隔；`*` 表示通配（会打 warning） |
| `ANTHROPIC_API_KEY` | *(unset)* | 注入到所有 agent 容器；同时被收集用于日志流脱敏 |
| `RUST_LOG` | `info` | 日志级别 |

### 环境变量 (Sidecar 容器)

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `CONTAINER_ID` | *(必填)* | 容器 ID（由 Control Plane 注入） |
| `CONTROL_PLANE_URL` | `http://localhost:8080` | 心跳上报地址 |
| `SIDECAR_ADDR` | `0.0.0.0:9000` | sidecar HTTP server 监听地址 |
| `SKILL_REPOS` | *(可选)* | 启动时克隆到 `/workspace/skills/` 的 Git 仓库（逗号分隔） |
| `ANTHROPIC_API_KEY` | *(可选)* | 由 cc-sdk 内部使用 |

## Skill 加载

Skill 文件从 Git 仓库自动克隆到容器内 `/workspace/skills/` 目录。

**仓库结构**:
```
skills-repo/
├── code-review/
│   └── SKILL.md
├── test-generator/
│   └── SKILL.md
└── custom-tool/
    └── SKILL.md
```

**SKILL.md 示例**:
```markdown
---
name: code-review
description: 自动化代码审查工具
---

# Code Review Skill

你是一个代码审查专家，负责审查 Pull Request。

## 审查要点
- 代码风格一致性
- 潜在 bug 和安全问题
- 性能优化建议
```

## 开发指南

### 项目结构

```
agentbox/
├── Cargo.toml                          # Workspace 配置
├── Dockerfile                          # Control Plane 镜像
├── docker-compose.yml                  # Docker 编排
├── control-plane/                      # 主服务
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                     # 入口 + 路由 + CORS 构造
│       ├── config.rs                   # 配置加载（含 ALLOWED_ORIGINS）
│       ├── auth.rs                     # Bearer token 中间件
│       ├── redact.rs                   # 日志流密钥脱敏
│       ├── error.rs                    # 错误类型
│       ├── models/container.rs         # 数据模型
│       ├── docker/
│       │   ├── manager.rs              # Docker 操作（Bollard）
│       │   └── lifecycle.rs            # 生命周期巡检（含 Docker 实际状态校验）
│       ├── db/sqlite.rs                # 数据库操作
│       └── api/
│           ├── containers.rs           # 容器 CRUD + 状态回调
│           ├── ws.rs                   # WebSocket 日志流（含 redact）
│           └── health.rs               # 健康检查
├── sidecar/                            # 容器内服务
│   ├── Cargo.toml                      # 含 cc-sdk = "0.8"
│   └── src/
│       ├── main.rs                     # axum server :9000
│       ├── query.rs                    # POST /query SSE handler（cc_sdk::query 封装）
│       ├── reporter.rs                 # 状态/心跳上报
│       └── health.rs                   # 后台心跳循环
├── agent-image/                        # Agent 镜像
│   ├── Dockerfile                      # sidecar + Claude CLI
│   └── entrypoint.sh                   # 克隆 skills 后 exec sidecar
└── admin-ui/                           # 前端管理界面 (React + Vite + shadcn/ui)
    ├── Dockerfile                      # Nginx 静态部署
    └── src/
        ├── pages/                      # 容器列表、详情、创建
        ├── components/                 # UI 组件（含 LogViewer WebSocket 日志）
        └── hooks/                      # API hooks、WebSocket 日志 hook
```

### 运行测试

```bash
# 单元测试
cargo test

# 指定包测试
cargo test -p control-plane
cargo test -p sidecar

# 前端开发
cd admin-ui && npm run dev
```

### 构建镜像

```bash
# 编译 release
cargo build --release

# 构建 Agent 镜像
docker build -t agent-sandbox:latest -f agent-image/Dockerfile .
```

## 扩展方向

- [x] WebSocket 实时日志流（含密钥脱敏）
- [x] API 认证鉴权（API Key Bearer token）
- [x] 收紧 CORS 默认值（localhost only，可配 `ALLOWED_ORIGINS`）
- [x] Sidecar 接入 Claude SDK（cc-sdk 0.8，SSE 流式 query）
- [x] Admin UI 管理界面（容器 CRUD + 实时日志流）
- [x] 容器列表/分页查询、历史日志（非实时）查询
- [ ] Control-plane 透传 sidecar 的 `/query` SSE（统一外部入口 + 鉴权 + 流量控制）
- [ ] 容器池/预热机制
- [ ] Kubernetes 部署支持
- [ ] Prometheus 监控指标
- [ ] 容器快照与恢复
- [ ] 多节点调度

## Admin UI

React + Vite + TypeScript 管理后台，位于 `admin-ui/` 目录。

### 功能

- API Key 登录认证
- Dashboard 统计面板（容器状态分布 + 最近容器）
- 容器列表（搜索、状态筛选、排序、分页、删除）
- 创建容器表单（任务、Skill Repos、资源限制、环境变量）
- 容器详情 + WebSocket 实时日志流
- 中英双语（自动检测 + 手动切换）
- 暗色/亮色主题切换

### 本地开发

```bash
cd admin-ui
npm install
npm run dev      # → http://localhost:5173
```

Vite dev server 自动代理 `/api` 和 `/health` 到 `localhost:8080`（需同时运行 Control Plane）。

### 生产部署

```bash
docker compose up -d admin-ui   # → http://localhost:3000
```

Nginx 反向代理 SPA + API + WebSocket。

## License

MIT
