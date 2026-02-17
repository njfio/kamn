# Local TLS Material For Docker Compose

`deploy/docker-compose.yml` requires service API TLS files mounted at:

- `/tls/service-api-cert.pem`
- `/tls/service-api-key.pem`

Generate local self-signed material before running compose:

```bash
mkdir -p deploy/certs
openssl req -x509 -newkey rsa:2048 -nodes -days 365 \
  -keyout deploy/certs/service-api-key.pem \
  -out deploy/certs/service-api-cert.pem \
  -subj "/CN=localhost"
```

The compose healthchecks use `curl --insecure` against `https://127.0.0.1:<port>/healthz`.
