# Microsserviço Encoder

Microsserviço de encoding de vídeo que consome mensagens de uma fila RabbitMQ, faz download do vídeo do Google Cloud Storage, fragmenta e converte para MPEG-DASH usando Bento4, faz upload do resultado e notifica via RabbitMQ.

Implementado em **Rust** e **Go** — duas versões independentes do mesmo serviço.

## Arquitetura

```
RabbitMQ (videos queue)
    │
    ▼
JobManager ──► Semaphore (N workers)
    │
    ▼
JobWorker
    │
    ├─ 1. Download do GCS (input bucket)
    ├─ 2. Fragment (mp4fragment)
    ├─ 3. Encode MPEG-DASH (mp4dash)
    ├─ 4. Upload paralelo para GCS (output bucket)
    └─ 5. Cleanup + notificação
    │
    ▼
RabbitMQ (jobs notification)
```

## Stack

| Componente | Tecnologia |
|---|---|
| Linguagem (Rust) | Rust 2024 edition, Tokio |
| Linguagem (Go) | Go |
| Banco de dados | PostgreSQL 16 |
| Fila | RabbitMQ |
| Storage | Google Cloud Storage |
| Encoding | FFmpeg + Bento4 (mp4fragment, mp4dash) |
| Container | Docker (Alpine multi-stage) |

## Estrutura

### Rust (`encoder-rust/`)

```
src/
  main.rs              # Entrypoint: DB + RabbitMQ + JobManager + graceful shutdown
  lib.rs               # Re-exports
  config.rs            # Config via env vars
  db.rs                # Database<Postgres|Sqlite> + macro impl_with_db!
  domain/
    video.rs           # Entidade Video
    job.rs             # Entidade Job + JobStatus
    error.rs           # DomainError
  repositories/
    repository.rs      # Trait Repository<T>
    video.rs           # VideoRepository (LEFT JOIN com jobs)
    job.rs             # JobRepository
  services/
    job_manager.rs     # Consome fila, dispatch workers com semaphore
    job_worker.rs      # Processa mensagem individual
    job.rs             # Pipeline: download → fragment → encode → upload → finish
    video.rs           # Download GCS, mp4fragment, mp4dash
    upload.rs          # Upload paralelo para GCS
  queue/
    config.rs          # QueueConfig com defaults
    rabbitmq.rs        # Connect, consume (mpsc), notify
    error.rs           # QueueError
migrations/
  20251228154112_add-job-and-video-migrate.sql
```

### Go (`encoder-go/`)

```
cmd/
  server/main.go       # Entrypoint
internal/
  handler/             # HTTP/gRPC handlers
  service/             # Lógica de negócio
  repository/          # Acesso a dados
  domain/              # Entidades e value objects
```

## Setup

### Pré-requisitos

- Docker e Docker Compose
- Credenciais GCS (`bucket-credential.json` na raiz)

### Subir infraestrutura

```bash
docker compose up -d db rabbit
```

### Desenvolvimento (Rust)

```bash
docker compose up -d encoder-rust
docker compose exec encoder-rust bash

# Dentro do container
cargo run
```

### Desenvolvimento (Go)

```bash
docker compose up -d encoder-go
docker compose exec encoder-go bash

# Dentro do container
go run cmd/server/main.go
```

## Variáveis de Ambiente

Copie `.env.example` para `.env` dentro de `encoder-rust/`:

```bash
cp encoder-rust/.env.example encoder-rust/.env
```

| Variável | Descrição | Default |
|---|---|---|
| `DATABASE_URL` | URL de conexão PostgreSQL | — (obrigatório) |
| `DATABASE_URL_TEST` | URL para testes | `sqlite::memory:` |
| `AUTO_MIGRATE_DB` | Rodar migrations no startup | `false` |
| `localStoragePath` | Diretório temporário local | `/tmp` |
| `inputBucketName` | Bucket GCS de entrada | — (obrigatório) |
| `outputBucketName` | Bucket GCS de saída | — (obrigatório) |
| `CONCURRENCY_UPLOAD` | Workers paralelos de upload | `4` |
| `CONCURRENCY_WORKERS` | Workers paralelos de encoding | `2` |
| `RABBITMQ_DEFAULT_USER` | Usuário RabbitMQ | — (obrigatório) |
| `RABBITMQ_DEFAULT_PASS` | Senha RabbitMQ | — (obrigatório) |
| `RABBITMQ_DEFAULT_HOST` | Host RabbitMQ | — (obrigatório) |
| `RABBITMQ_DEFAULT_PORT` | Porta RabbitMQ | `5672` |
| `RABBITMQ_DEFAULT_VHOST` | Virtual host | `/` |
| `RABBITMQ_CONSUMER_QUEUE_NAME` | Fila de consumo | `videos` |
| `RABBITMQ_CONSUMER_NAME` | Nome do consumer | `encoder-consumer` |
| `RABBITMQ_DLX` | Dead letter exchange | `dlx` |
| `RABBITMQ_NOTIFICATION_EX` | Exchange de notificação | `amq.direct` |
| `RABBITMQ_NOTIFICATION_ROUTING_KEY` | Routing key de notificação | `jobs` |
| `GOOGLE_APPLICATION_CREDENTIALS` | Path para credenciais GCS | — (obrigatório) |

## Mensagem da Fila

O serviço consome mensagens JSON da fila `videos`:

```json
{
  "resource_id": "video-uuid",
  "file_path": "path/to/video.mp4"
}
```

## Testes

### Rust

```bash
cd encoder-rust

cargo test                    # Unit tests (SQLite in-memory)
cargo test -- --ignored       # Integration tests (requer RabbitMQ + GCS)
cargo clippy -- -D warnings   # Linter
cargo fmt -- --check          # Formatação
```

### Go

```bash
cd encoder-go

go test ./...
go vet ./...
```

## Docker

### Build (Rust)

```bash
docker build -t encoder-rust ./encoder-rust
```

O Dockerfile usa multi-stage build:
1. **builder**: Rust nightly Alpine + FFmpeg + Bento4 → compila release
2. **runtime**: Alpine mínimo + FFmpeg + Bento4 + binário

## Banco de Dados

Schema criado via migration automática (`AUTO_MIGRATE_DB=true`):

- **videos**: `id`, `resource_id`, `file_path`, `created_at`
- **jobs**: `id`, `output_bucket_path`, `status`, `video_id`, `error`, `created_at`, `updated_at`

Status do job: `Pending` → `Processing` → `Completed` | `Failed`
