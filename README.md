# Literal Eureka Home Systems

## Run with Docker

Build and start the backend service from the repository root:

```bash
docker compose up --build
```

The backend container listens on `0.0.0.0:3000` and publishes port `3000` to the host, so you can visit the app at `http://localhost:3000`.

## Bill PDF download

Download a bill PDF by id:

```http
GET /bills/:id/pdf
```

Example:

```bash
curl -OJ http://localhost:3000/bills/1/pdf
```

Successful responses return `200 OK` with `Content-Type: application/pdf` and `Content-Disposition: attachment; filename="bill-<id>.pdf"`.

If the bill id does not exist, the API returns `404 Not Found` with a JSON error body from the standard error mapper.
