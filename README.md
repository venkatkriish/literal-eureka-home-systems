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
curl -OJ -H "X-User-Id: 1" http://localhost:3000/bills/1/pdf
```

A bill's PDF is for its own customer or an admin. Until the API has real sign-in, the caller identifies themselves with the `X-User-Id` header:

- `200 OK` with `Content-Type: application/pdf` and `Content-Disposition: attachment; filename="bill-<id>.pdf"`;
- `401 Unauthorized` without the header, or when it names no user;
- `403 Forbidden` when the bill belongs to another customer;
- `404 Not Found` when the bill doesn't exist.
