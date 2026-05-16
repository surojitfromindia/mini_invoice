# Garage Local Development Setup

This guide sets up:

* PostgreSQL 18
* Valkey
* Garage (S3-compatible object storage)
* AWS CLI connection to Garage

---

# Project Structure

```text
project/
├── .env
├── docker-compose.yml
├── garage.toml
├── Cargo.toml
└── src/
```

---

# 1. Create `.env`

```env
DATABASE_URL=postgres://postgres:mypassword@localhost:5433/smart_audit
REDIS_URL=redis://localhost:6380

GARAGE_RPC_SECRET=PUT_64_CHAR_HEX_SECRET_HERE

GARAGE_KEY_ID=
GARAGE_SECRET_KEY=
```

---

# 2. Generate Garage RPC Secret

Garage requires a 64-character hexadecimal secret.

Windows PowerShell:

```powershell
[System.BitConverter]::ToString((1..32 | ForEach-Object {Get-Random -Max 256})).Replace("-", "").ToLower()
```

Copy the generated value into:

```env
GARAGE_RPC_SECRET=...
```

---

# 3. Create `garage.toml`

```toml
metadata_dir = "/var/lib/garage/meta"
data_dir = "/var/lib/garage/data"

db_engine = "sqlite"

replication_factor = 1

rpc_bind_addr = "0.0.0.0:3901"
rpc_public_addr = "127.0.0.1:3901"

rpc_secret = "PUT_64_CHAR_HEX_SECRET_HERE"

[s3_api]
s3_region = "garage"
api_bind_addr = "0.0.0.0:3900"

[s3_web]
bind_addr = "0.0.0.0:3902"
root_domain = ".localhost"
```


---

# 4. Create `docker-compose.yml`

```yaml
services:
  postgres:
    image: postgres:18
    container_name: postgres_db
    restart: unless-stopped
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: mypassword
      POSTGRES_DB: smart_audit
    ports:
      - "5433:5432"
    volumes:
      - postgres_data:/var/lib/postgresql

  valkey:
    image: valkey/valkey:8
    container_name: valkey_cache
    restart: unless-stopped
    ports:
      - "6380:6379"
    volumes:
      - valkey_data:/data

  garage:
    image: dxflrs/garage:v2.3.0
    container_name: garage
    restart: unless-stopped
    ports:
      - "3900:3900"
      - "3901:3901"
      - "3902:3902"
    volumes:
      - garage_meta:/var/lib/garage/meta
      - garage_data:/var/lib/garage/data
      - ./garage.toml:/etc/garage.toml

volumes:
  postgres_data:
  valkey_data:
  garage_meta:
  garage_data:
```

---

# 5. Start Containers

```bash
docker compose up -d
```

Check running containers:

```bash
docker ps
```

Check logs:

```bash
docker compose logs -f
```

---

# 6. Initialize Garage Layout

## Get Garage Node ID

```bash
docker exec -it garage /garage status
```

Example output:

```text
ID: 5739c36a5cb554ed
```

Copy the node ID.

---

## Assign Storage

Replace the node ID below with your own:

```bash
docker exec -it garage /garage layout assign \
  -z dc1 \
  -c 1G \
  5739c36a5cb554ed
```

---

## Apply Layout

```bash
docker exec -it garage /garage layout apply --version 1
```

---

# 7. Create Garage Access Key

```bash
docker exec -it garage /garage key create
```

Example output:

```text
GARAGE_KEY_ID=GKxxxxxxxxxxxxxxxx
GARAGE_SECRET_KEY=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

Save these into `.env`:

```env
GARAGE_KEY_ID=...
GARAGE_SECRET_KEY=...
```

---

# 8. Create Bucket

```bash
docker exec -it garage /garage bucket create uploads
```

---

# 9. Grant Bucket Permissions

Replace the key ID below with your actual key:

```bash
docker exec -it garage /garage bucket allow \
  --read \
  --write \
  uploads \
  --key GKxxxxxxxxxxxxxxxx
```

---

# 10. Install AWS CLI

Download:

[https://aws.amazon.com/cli/](https://aws.amazon.com/cli/)

---

# 11. Configure AWS CLI

```bash
aws configure
```

Enter:

```text
AWS Access Key ID: GKxxxxxxxxxxxxxxxx
AWS Secret Access Key: xxxxxxxxxxxxxxxxxxxxxxxx
Default region name: garage
Default output format: json
```

---

# 12. Test Garage S3 API

## List Buckets

```bash
aws s3 ls --endpoint-url http://localhost:3900
```

---

## Upload File

Create a test file:

```bash
echo hello > test.txt
```

Upload:

```bash
aws s3 cp test.txt s3://uploads \
  --endpoint-url http://localhost:3900
```

---

## List Files

```bash
aws s3 ls s3://uploads \
  --endpoint-url http://localhost:3900
```

---

## Download File

```bash
aws s3 cp s3://uploads/test.txt . \
  --endpoint-url http://localhost:3900
```

---

# 13. Rust Environment Variables

Example:

```env
DATABASE_URL=postgres://postgres:mypassword@localhost:5433/smart_audit
REDIS_URL=redis://localhost:6380

S3_ENDPOINT=http://localhost:3900
S3_REGION=garage
S3_BUCKET=uploads

GARAGE_KEY_ID=...
GARAGE_SECRET_KEY=...
```

---

# Useful Docker Commands

## Start

```bash
docker compose up -d
```

## Stop

```bash
docker compose down
```

## Stop + Delete Volumes/Data

```bash
docker compose down -v
```

## Restart

```bash
docker compose restart
```

## View Logs

```bash
docker compose logs -f
```

---

# Ports

| Service       | Port |
| ------------- | ---- |
| PostgreSQL    | 5433 |
| Valkey        | 6380 |
| Garage S3 API | 3900 |
| Garage RPC    | 3901 |
| Garage Web    | 3902 |

---

# Notes

* Garage is S3-compatible.
* You can use standard AWS SDKs.
* Works with `aws-sdk-s3` in Rust.
* `replication_factor = 1` is suitable for local development.
* Garage currently does not provide a polished built-in admin UI like MinIO.
