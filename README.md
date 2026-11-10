# Daylist — Rust + React todo app

Axum + SQLx/PostgreSQL API and React/TypeScript + Zustand + Axios client. All application secrets come directly from AWS Secrets Manager at server startup. No `.env` files or environment-variable fallback for database/JWT secrets.

## Terraform deployment

See [infra/README.md](infra/README.md) for the complete ALB → private EC2 Auto Scaling Group → RDS stack using the supplied AMI. Terraform generates the runtime config on EC2. It creates a new database and networking; existing console-created resources are not adopted automatically.

## 1. Reuse your existing RDS secret

Set `rds_secret_id` in `server/config.json` to your existing secret's full ARN (or name). The minimum supported JSON is:

```json
{
  "username": "your_existing_database_user",
  "password": "your_existing_database_password"
}
```

Do not copy these values into the repository or Compose. If your existing secret also contains `host`, `port`, `dbname`, or `engine`, the server uses them. Supported engine values are `postgres` and `postgresql`. `port` is a JSON number. Other RDS/rotation metadata is ignored. Keep an RDS-managed secret under RDS management; app settings belong in the separate app secret below.

## 2. Add one application secret

In AWS Secrets Manager, create an **Other type of secret**, named `todo/production/app`, with this JSON SecretString:

```json
{
  "JWT_SECRET": "REPLACE_WITH_OUTPUT_OF_openssl_rand_-hex_32",
  "RDS_HOST": "your-database.xxxxx.us-east-1.rds.amazonaws.com",
  "DB_NAME": "todo",
  "RDS_PORT": 5432
}
```

- `JWT_SECRET`: required. Generate with `openssl rand -hex 32` and put the generated value directly in Secrets Manager. Minimum 32 bytes.
- `RDS_HOST`: required when the RDS secret has no `host`. Use the RDS DNS endpoint, without a URL scheme.
- `DB_NAME`: required when the RDS secret has no `dbname`. Use an existing PostgreSQL database on your RDS instance. The server creates tables through migrations, but does not create the database itself.
- `RDS_PORT`: optional when the RDS secret has no `port`; defaults to 5432. Use a JSON number.

Connection fields in the existing RDS secret take precedence over app-secret connection fields. If the RDS secret already includes the endpoint, port, and database name, the new app secret only needs `JWT_SECRET`. Database credentials are never duplicated in the app secret. Passwords with special characters are supported without URL encoding.

## 3. Set non-secret bootstrap configuration

The public repository contains only `server/config.example.json` with placeholders.
After cloning onto EC2, run from the project root:

```bash
cp server/config.example.json server/config.json
nano server/config.json
```

Fill in `server/config.json` on EC2:

```json
{
  "aws_region": "YOUR_AWS_REGION",
  "rds_secret_id": "YOUR_EXISTING_RDS_SECRET_ARN",
  "app_secret_id": "todo/production/app",
  "port": 8080,
  "client_origin": "http://YOUR_EC2_PUBLIC_IP",
  "rds_ca_bundle": "certs/global-bundle.pem"
}
```

Use these values:

- `aws_region`: the AWS region containing both secrets, for example `us-east-1`.
- `rds_secret_id`: the ARN of the existing secret containing your RDS username and password. Find it in Secrets Manager's secret details.
- `app_secret_id`: the name or ARN of the new application secret containing `JWT_SECRET`, and the connection fields described above. Keep `todo/production/app` if that is its name.
- `port`: leave `8080` for the supplied Docker/Nginx setup.
- `client_origin`: the exact browser URL of your app, without a trailing slash. Use `http://YOUR_EC2_PUBLIC_IP` when exposing client port 80, `https://your-domain.com` when using HTTPS, or `http://localhost:3000` for local Docker.
- `rds_ca_bundle`: leave `certs/global-bundle.pem`.

`server/config.json` is ignored by Git, excluded from the Docker build context, and mounted read-only at runtime. No `.env` file is needed. Do not add AWS access keys, database passwords, or `JWT_SECRET` to this file. The EC2 IAM role supplies AWS credentials, and Secrets Manager supplies application secrets. For the Vite development client, use `http://localhost:5173` as `client_origin`.

Before publishing, `git status --short` should include `server/config.example.json` but not `server/config.json`. Private keys, `.aws/`, and actual `deploy/iam-policy.json` are also ignored. The public RDS CA bundle is intentionally included. Put your real IAM policy ARNs into an ignored `deploy/iam-policy.json` copied from the example, or directly into the AWS console.

The API requires TLS with full server-certificate and hostname verification. The public AWS RDS CA bundle is included at `server/certs/global-bundle.pem`. To refresh it:

```bash
curl --fail --location https://truststore.pki.rds.amazonaws.com/global/global-bundle.pem -o server/certs/global-bundle.pem
```

## 4. Give the server an AWS identity and database access

On AWS, attach an IAM role to the EC2 instance or ECS task running the server. The SDK uses the default credential provider chain. AWS identity is needed before any secret can be fetched; do not store access keys in the app secret or source code.

Grant the server role `secretsmanager:GetSecretValue` on **both** the RDS secret and application secret ARNs. See `deploy/iam-policy.example.json`. If either secret uses a customer-managed KMS key, also grant `kms:Decrypt` on that specific key and allow the role in its key policy. For ECS, these SDK calls use the **task role**.

The server must have network access to Secrets Manager and to the RDS endpoint. Permit port 5432 (or your configured port) from the server's security group to the RDS security group. Private RDS instances require VPC connectivity, such as an EC2/ECS deployment in that VPC or a VPN when developing locally. The DB user needs permission to create the app tables/indexes and read/write them. RDS IAM database authentication is not required; the database password is retrieved from the existing secret.

For local Docker development, authenticate an AWS profile on your laptop and use the optional profile mount:

```bash
docker compose -f compose.yaml -f compose.aws-profile.yaml up --build -d
```

The override mounts your existing `~/.aws` configuration/cache read-only. It uses the default profile; change the non-secret `AWS_PROFILE` value in that override if you use a named profile. Run `aws sso login --profile YOUR_PROFILE` on the host first for SSO. The API image sets HOME for its unprivileged app user so the SDK can discover this mount. No AWS credential files are copied into the image. Your host profile/cache files must be readable by container UID 10001; otherwise run `cargo run` directly with your host's authenticated profile or use an AWS-hosted role.

## 5. Run

On an AWS host with an attached role and RDS connectivity, create `server/config.json` as described above before starting Compose. A missing file fails the mount instead of creating a directory.

For access through the EC2 public IP, change the **client** port mapping in `compose.yaml` from `127.0.0.1:3000:80` to `80:80` and allow inbound HTTP port 80 in the EC2 security group. Keep the API port bound to localhost; Nginx forwards browser API calls internally. Set `client_origin` to the public browser URL. Then run:

```bash
docker compose up --build -d
```

Compose runs only the server and client. PostgreSQL is your existing RDS instance. With the default local port mapping, open http://localhost:3000; with the EC2 port-80 mapping, open `http://YOUR_EC2_PUBLIC_IP`. API: http://localhost:8080 on the host. Database health: http://localhost:8080/health. Migrations run before the server starts; the client starts after the API becomes healthy. Stop with `docker compose down`.

Secrets are retrieved at startup and held in server memory. Restart the server after rotating RDS credentials or the JWT secret. Automatic credential refresh after database password rotation is not implemented. Changing the JWT secret invalidates outstanding tokens.

## Structure

- `server/config.example.json`: public bootstrap template; copy to ignored `server/config.json` on the deployment host.
- `server/src/config.rs`: fetch and validate RDS/app secrets; build TLS PostgreSQL connection options.
- `server/src/routes/`, `middleware/`, `services/`, `repositories/`, `models/`: HTTP handlers, auth, application logic, database queries, and types.
- `server/migrations/`: embedded SQLx schema migrations.
- `client/src/pages/`, `components/`, `stores/`, `lib/`: pages, UI components, Zustand state, and Axios client.
- `deploy/iam-policy.example.json`: example role permissions.
- `scripts/smoke-test.py`: API integration checks, including account isolation.

## API

Public endpoints:

- `GET /health`: 200 when PostgreSQL is available, otherwise 503.
- `POST /api/auth/signup`: `{ "name": "Alex", "email": "alex@example.com", "password": "password123" }`.
- `POST /api/auth/login`: `{ "email": "alex@example.com", "password": "password123" }`.

Signup/login return `{ "token": "...", "user": { "id": "...", "name": "...", "email": "..." } }`.

Authenticated endpoints require `Authorization: Bearer <token>`:

- `GET /api/auth/me`: current user.
- `GET /api/todos`: current user's tasks, newest first.
- `POST /api/todos`: `{ "title": "Buy groceries" }`.
- `PATCH /api/todos/:id`: `{ "title": "Buy milk", "completed": true }` (either field is optional).
- `DELETE /api/todos/:id`: deletes an owned task; returns 204.

Todo titles are trimmed and limited to 200 characters. Passwords are 8–128 characters on signup and hashed with salted Argon2id. Tokens use HS256 with a 24-hour expiry and issuer validation. Database queries scope every todo mutation to the authenticated user. Unknown and other users' todo IDs both return 404.

The client stores tokens in `sessionStorage`, clears auth on protected-request 401s, and resets tasks on account changes. Signing out removes the local token; issued JWTs remain valid until expiry. For an internet-facing deployment, terminate HTTPS and add authentication rate limiting at your ingress. This starter does not implement password reset, email verification, refresh tokens, or token revocation.

## Development and checks

With a current stable Rust toolchain and authenticated AWS profile, run `cargo run` from `server/`. It reads `config.json` by default; pass a different public config file as the first argument if needed (`cargo run -- /path/to/config.json`). AWS region is selected in that file. A non-default host AWS profile can be selected with the SDK's standard `AWS_PROFILE` setting; it is an identity selector, not an application-secret fallback.

In another terminal, run `npm ci` and `npm run dev` from `client/`. Vite serves http://localhost:5173 and proxies `/api` and `/health` to the API. The client has no environment configuration and always uses `/api`.

```bash
cd client && npm run build
cd ../server && cargo test --locked
cd ..
python3 scripts/smoke-test.py
```

The Docker server build runs Rust tests before building the binary. Configuration tests cover minimal RDS-managed secrets, full RDS JSON precedence, missing database fields, weak JWT secrets, and verified TLS. The smoke test requires a running API and creates two test accounts. Live secret retrieval/RDS connectivity requires your AWS configuration and was not tested against your account.

References: [AWS secret JSON fields](https://docs.aws.amazon.com/secretsmanager/latest/userguide/reference_secret_json_structure.html), [RDS-managed passwords](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/rds-secrets-manager.html), and [RDS PostgreSQL TLS verification](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/PostgreSQL.Concepts.General.SSL.html).
