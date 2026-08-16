# Zero To Production — Chapter 5 Deployment on AWS

This guide documents the AWS/EC2 equivalent of the DigitalOcean deployment workflow used in Chapter 5 of *Zero To Production in Rust*. It reflects the deployment completed for `zero2prod` using Amazon ECR, an ARM64 EC2 instance, Docker, PostgreSQL, persistent storage, environment-variable secrets, and PostgreSQL TLS.

> **Public-repository note:** This version is sanitized. Replace placeholders such as `<AWS_ACCOUNT_ID>`, `<EC2_PUBLIC_IP>`, and `<DB_PASSWORD>` only in your local/runtime environment. Do not commit real credentials, private keys, `.env` files, client IP addresses, or other account-specific identifiers.

## 1. Architecture

- **AWS Region:** `us-east-1`
- **Amazon ECR:** stores the `zero2prod` Docker image
- **Amazon EC2:** Amazon Linux 2023, `t4g.nano` (`aarch64`)
- **Docker network:** `zero2prod-net`
- **Application container:** `zero2prod`
- **Database container:** PostgreSQL 18
- **Persistent Docker volume:** `zero2prod-postgresql-data`
- **Application port:** TCP 8000
- **Database port:** 5432, exposed only inside the Docker network
- **Database TLS:** enabled with a self-signed certificate for this learning deployment

The application and PostgreSQL containers communicate by Docker DNS using the hostname `postgres`.

## 2. Create the ECR Repository

```bash
aws ecr create-repository \
  --repository-name zero2prod \
  --region us-east-1
```

Authenticate Docker with ECR:

```bash
aws ecr get-login-password --region us-east-1 \
  | docker login \
      --username AWS \
      --password-stdin \
      <AWS_ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com
```

## 3. Build the Production Docker Image

The project uses a multi-stage Docker build with `cargo-chef`: dependencies are cached separately, the Rust toolchain stays out of the runtime image, the runtime is based on `debian:bookworm-slim`, and `SQLX_OFFLINE=true` permits compilation without a live database.

```bash
docker build --tag zero2prod .
```

Check architecture:

```bash
docker image inspect zero2prod:latest \
  --format '{{.Os}}/{{.Architecture}}'
```

For a `t4g` EC2 instance, the image must be compatible with `linux/arm64`.

## 4. Push the Image to ECR

```bash
docker tag zero2prod:latest \
  <AWS_ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/zero2prod:latest

docker push \
  <AWS_ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/zero2prod:latest
```

Verify:

```bash
aws ecr describe-images \
  --repository-name zero2prod \
  --region us-east-1
```

## 5. EC2 Instance

Deployment environment:

```text
Amazon Linux 2023
t4g.nano
ARM64 / aarch64
```

Verify:

```bash
uname -m
cat /etc/os-release
```

The EC2 security group must permit TCP 8000 for direct application testing. Do **not** expose PostgreSQL port 5432 publicly in the final configuration.

## 6. Pull the Application Image on EC2

```bash
aws ecr get-login-password --region us-east-1 \
  | sudo docker login \
      --username AWS \
      --password-stdin \
      <AWS_ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com

sudo docker pull \
  <AWS_ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/zero2prod:latest
```

## 7. Create the Docker Network

```bash
sudo docker network create zero2prod-net
```

Both containers join this network. The application can therefore reach PostgreSQL at `postgres:5432` without publishing the database port through EC2.

## 8. Persistent PostgreSQL Storage

```bash
sudo docker volume create zero2prod-postgresql-data
```

PostgreSQL 18 is mounted as:

```bash
--volume zero2prod-postgresql-data:/var/lib/postgresql
```

The database therefore survives container recreation.

## 9. Start PostgreSQL

Initial non-TLS setup:

```bash
sudo docker run -d \
  --name postgres \
  --network zero2prod-net \
  --volume zero2prod-postgresql-data:/var/lib/postgresql \
  -e POSTGRES_USER=app \
  -e POSTGRES_PASSWORD=<DB_PASSWORD> \
  -e POSTGRES_DB=newsletter \
  postgres:18
```

There is deliberately no `--publish 5432:5432` in the final configuration.

## 10. Database Migration

The migration creates the `subscriptions` table:

```sql
CREATE TABLE subscriptions(
    id uuid NOT NULL,
    PRIMARY KEY (id),
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subscribed_at timestamptz NOT NULL
);
```

During initial deployment the migration was run with `sqlx migrate run`. Verify with:

```bash
sqlx migrate info
```

## 11. Production Configuration

The application must listen on all container interfaces:

```yaml
application:
  host: 0.0.0.0
```

For Docker deployment the database host is `postgres` and the production database user is `app`. Secrets should not be committed to YAML.

## 12. Inject Secrets with Environment Variables

Nested configuration uses a **double underscore** separator:

```text
APP_DATABASE__PASSWORD
```

not:

```text
APP_DATABASE_PASSWORD
```

For example:

```bash
--env APP_DATABASE__PASSWORD=<DB_PASSWORD>
```

maps to `database.password`, while:

```bash
--env APP_DATABASE__REQUIRE_SSL=true
```

maps to `database.require_ssl`.

> Use a strong generated value for `<DB_PASSWORD>`. Do not commit the real password to source control; for production, prefer AWS Secrets Manager or Systems Manager Parameter Store.

## 13. Test the Application

Health check:

```bash
curl -v http://<EC2_PUBLIC_IP>:8000/health_check
```

Expected: `HTTP/1.1 200 OK`.

Subscription test:

```bash
curl -v \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "name=James&email=james@example.com" \
  http://<EC2_PUBLIC_IP>:8000/subscriptions
```

Expected: `HTTP/1.1 200 OK`.

## 14. Enable PostgreSQL TLS

Create a certificate directory:

```bash
mkdir -p ~/postgres-certs
cd ~/postgres-certs
```

Generate a self-signed certificate:

```bash
openssl req -new -x509 -days 365 -nodes \
  -text \
  -out server.crt \
  -keyout server.key \
  -subj "/CN=postgres"
```

Find the PostgreSQL user's UID/GID:

```bash
sudo docker run --rm postgres:18 id postgres
```

For the image used here it returned `uid=999(postgres) gid=999(postgres)`.

Set ownership and permissions:

```bash
sudo chown 999:999 server.key server.crt
sudo chmod 600 server.key
sudo chmod 644 server.crt
```

PostgreSQL rejects a private key with inappropriate ownership or permissions.

## 15. Recreate PostgreSQL with TLS

Stop it cleanly:

```bash
sudo docker stop postgres
sudo docker rm postgres
```

Start with TLS enabled:

```bash
sudo docker run -d \
  --name postgres \
  --network zero2prod-net \
  --volume zero2prod-postgresql-data:/var/lib/postgresql \
  --volume "$HOME/postgres-certs:/certs:ro" \
  -e POSTGRES_USER=app \
  -e POSTGRES_PASSWORD=<DB_PASSWORD> \
  -e POSTGRES_DB=newsletter \
  postgres:18 \
  -c ssl=on \
  -c ssl_cert_file=/certs/server.crt \
  -c ssl_key_file=/certs/server.key
```

Verify:

```bash
sudo docker exec postgres \
  psql -U app -d newsletter \
  -c "SHOW ssl;"
```

Expected:

```text
 ssl
-----
 on
```

## 16. Run zero2prod with SSL Required

```bash
sudo docker rm -f zero2prod

sudo docker run -d \
  --name zero2prod \
  --network zero2prod-net \
  --publish 8000:8000 \
  --env APP_DATABASE__PASSWORD=<DB_PASSWORD> \
  --env APP_DATABASE__REQUIRE_SSL=true \
  <AWS_ACCOUNT_ID>.dkr.ecr.us-east-1.amazonaws.com/zero2prod:latest
```

Confirm the injected variables:

```bash
sudo docker exec zero2prod env | grep APP_DATABASE
```

## 17. Final SSL Smoke Test

From the development Mac:

```bash
curl -v \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "name=James3&email=james3@example.com" \
  http://<EC2_PUBLIC_IP>:8000/subscriptions
```

Expected: `HTTP/1.1 200 OK`.

For a server-side check of active SSL sessions:

```bash
sudo docker exec postgres \
  psql -U app -d newsletter \
  -c "
SELECT
    a.usename,
    a.client_addr,
    s.ssl,
    s.version,
    s.cipher
FROM pg_stat_activity a
JOIN pg_stat_ssl s USING (pid)
WHERE a.usename = 'app';
"
```

For the application's connection, `ssl` should be `t`.

## 18. Useful Diagnostics

```bash
sudo docker logs zero2prod
sudo docker logs --tail 30 zero2prod
sudo docker logs postgres
sudo docker ps

sudo docker inspect postgres \
  --format '{{range .Mounts}}{{.Name}} -> {{.Destination}}{{println}}{{end}}'
```

## 19. DigitalOcean-to-AWS Mapping

| Book / DigitalOcean concept | AWS implementation |
| --- | --- |
| Container registry | Amazon ECR |
| DigitalOcean app/server | Amazon EC2 |
| Deployment specification | EC2 + Docker configuration/commands |
| Managed runtime | Docker on Amazon Linux 2023 |
| Application secret injection | Environment variables passed to Docker |
| Application public endpoint | EC2 public endpoint on TCP 8000 for this exercise |
| Database | PostgreSQL 18 Docker container |
| Persistent database storage | Docker named volume |
| Private app/database communication | User-defined Docker network |
| SSL-required database connection | PostgreSQL TLS + `APP_DATABASE__REQUIRE_SSL=true` |

This is intentionally a Chapter 5 learning deployment rather than a production AWS architecture. A production evolution would normally use services such as RDS for PostgreSQL, AWS Secrets Manager or Parameter Store, HTTPS through an ALB, restrictive security groups, IAM roles, automated deployment, and potentially ECS.

## 20. Chapter 5 Completion Checklist

- [x] Production multi-stage Docker image
- [x] `cargo-chef` dependency caching
- [x] Small runtime image
- [x] ARM64-compatible image
- [x] ECR repository and image push
- [x] Amazon Linux 2023 EC2 deployment
- [x] Docker application/database network
- [x] PostgreSQL persistent volume
- [x] Database migration
- [x] Production configuration
- [x] Secrets injected via environment variables
- [x] PostgreSQL TLS enabled
- [x] `require_ssl=true` exercised
- [x] `/health_check` externally reachable
- [x] `/subscriptions` externally tested successfully

At this point the AWS implementation of the Chapter 5 deployment workflow is ready for the Chapter 5 commit and tag.
