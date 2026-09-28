# Todo App — Actix Web 4 API + Next.js 14 Web + Supabase Postgres

Production-ready Todo uygulaması: JWT (Supabase uyumlu), RBAC, audit logları,
bildirimler + cron, OpenAPI/Swagger, Docker Compose.

## Mimari (özet)

```text
Client → CORS → RateLimit → Tracing → AuditMw → Auth(RBAC) → Handler → Service → Repository → Postgres
                                                     ↓
                                              NotificationService + cron worker
```

Detay: [`ARCHITECTURE.md`](./ARCHITECTURE.md)

## Tip güvenli API istemcisi

Frontend, backend'in OpenAPI spec'inden üretilen tiplerle çalışır:

```sh
cd frontend
npm run api:gen     # canlı API'den spec çek (api:spec) + tipleri üret (api:types)
npm run typecheck   # tsc --noEmit
```

- `lib/api-spec.json` — canlı API'den alınan OpenAPI snapshot'ı (commit edilir)
- `lib/api-types.d.ts` — openapi-typescript çıktısı (commit edilir)
- `lib/client.ts` — openapi-fetch tabanlı `apiClient` (Bearer + otomatik refresh)

Spec değişirse CI bunu yakalar (drift kontrolü): backend'i ayağa kaldırıp
`npm run api:gen` tekrar üretir; fark varsa PR kırmızıdır. Spec değiştiğinde
`npm run api:gen` çalıştırıp üretilen dosyaları commit edin.

## Portlar (çakışmasız)

Host'ta dolu portlar tespit edildi (`3000, 8000, 8080, 9000, 5432...`), bu yüzden:

| Servis   | Container | Host (localhost) | Env                  | Adres |
|----------|-----------|------------------|----------------------|-------|
| Web (Next.js) | 3000 | **3005**        | `HOST_WEB_PORT`      | http://127.0.0.1:3005 |
| API      | 8080      | **18081**        | `HOST_APP_PORT`      | http://127.0.0.1:18081 |
| Postgres | 5432      | **25433**        | `HOST_POSTGRES_PORT` | `psql -h 127.0.0.1 -p 25433 -U todo` |

Tüm publish'ler `127.0.0.1`'e bağlıdır. Değiştirmek için `.env` dosyasındaki
`HOST_APP_PORT` / `HOST_POSTGRES_PORT` değerlerini güncelleyin.

## Dış IP'den erişim

Portlar varsayılan olarak **tüm arayüzlere** açıktır (`.env` → `BIND_IP=0.0.0.0`).
Sunucunun IP'siyle `http://<SUNUCU_IP>:3005` üzerinden web uygulamasına erişebilirsiniz:

- Tarayıcı API isteklerini **aynı origin** üzerinden `/api/...` diye gönderir; Next.js
  sunucu tarafı reverse-proxy (`next.config.mjs` rewrites) bunları compose ağındaki
  API container'ına iletir. Böylece hangi IP/host'tan açarsanız açın çalışır,
  CORS sorunu doğmaz.
- Yalnızca yerel kullanım için `.env` içinde `BIND_IP=127.0.0.1` yapın.
- Prod ortamında güvenlik duvarı üzerinden yalnızca 3005'e (web) izin verip
  18081/25433'i dışarıya kapatmanız önerilir.

## Demo hesaplar

Uygulama ilk açılışta (idempotent) şu hesapları oluşturur — login sayfasında
tek tıkla doldurulabilir:

| E-posta             | Şifre      | Rol        |
|---------------------|------------|------------|
| admin@demo.local    | demo1234   | admin      |
| mod@demo.local      | demo1234   | moderator  |
| user@demo.local     | demo1234   | user       |

Üretimde `.env` içinde `SEED_DEMO_USERS=false` yapın ve/veya `DEMO_PASSWORD`'ü değiştirin.

## Hızlı başlangıç

```sh
cp .env.example todo-backend/.env
# .env içindeki JWT secret'ları üretimde değiştir!
docker compose up --build -d
# Web (herhangi bir IP'den):
xdg-open http://127.0.0.1:3005
# Demo hesapla giriş: admin@demo.local / demo1234
# API sağlık:
curl http://127.0.0.1:18081/health
curl http://127.0.0.1:18081/health/db
# Swagger:
xdg-open http://127.0.0.1:18081/swagger-ui/
```

Yerel (Docker'sız) geliştirme:

```sh
cd todo-backend
cp ../.env.example .env
# Postgres'i ayağa kaldır (sadece DB):
docker compose up -d postgres
cargo run
```

## Endpoint listesi

### Auth (`/api/auth`)
- `POST /api/auth/register` — kayıt
- `POST /api/auth/login` — giriş
- `POST /api/auth/refresh` — token yenileme (rotasyonlu)
- `POST /api/auth/logout` — çıkış
- `GET  /api/auth/me` — profil (Bearer gerekli)

### Todos (`/api/todos`, Bearer gerekli)
- `GET    /api/todos?status=&priority=&search=&page=&limit=&cursor=`
- `POST   /api/todos`
- `GET    /api/todos/:id`
- `PATCH  /api/todos/:id`
- `DELETE /api/todos/:id` (soft delete)
- `POST   /api/todos/:id/complete`
- `POST   /api/todos/:id/share` (`{email, permission}`)
- `POST   /api/todos/bulk` (`{ids, action: complete|archive|delete}`)
- `POST   /api/todos/reorder` (`{ids: [hedef sıra]}` — sürükle-bırak sıralaması)

### Bildirimler (`/api/notifications`)
- `GET    /api/notifications?unread=true`
- `PATCH  /api/notifications/:id/read`
- `POST   /api/notifications/read-all`
- `DELETE /api/notifications/:id`

### Admin (rol korumalı)
- `GET   /api/admin/users?role=&active=&search=` (moderator+)
- `PATCH /api/admin/users/:id/role` (admin+)
- `PATCH /api/admin/users/:id/status` (admin+)
- `GET   /api/admin/logs?...` (moderator+)
- `GET   /api/admin/logs/export?format=csv|json` (admin)
- `GET   /api/admin/stats` (moderator+)
- `POST  /api/admin/notifications/broadcast` (admin+)
- `GET   /api/admin/settings` / `PATCH /api/admin/settings/:key` (admin+)

### Sistem
- `GET /health`, `GET /health/db`, `GET /swagger-ui/`, `GET /api-docs/openapi.json`

## Örnek akış

```sh
B=http://127.0.0.1:18081
curl -s -X POST $B/api/auth/register -H 'Content-Type: application/json' \
  -d '{"email":"a@ex.com","password":"secret123","full_name":"A"}'
TOKEN=$(curl -s -X POST $B/api/auth/login -H 'Content-Type: application/json' \
  -d '{"email":"a@ex.com","password":"secret123"}' | python3 -c "import sys,json;print(json.load(sys.stdin)['tokens']['access_token'])")
curl -s -X POST $B/api/todos -H "Authorization: Bearer $TOKEN" -H 'Content-Type: application/json' \
  -d '{"title":"İlk iş","priority":"high"}'
curl -s "$B/api/todos?limit=5" -H "Authorization: Bearer $TOKEN"
```

## Supabase ile kullanım

- **Dış Supabase:** `.env` içinde `SUPABASE_JWT_SECRET` + `DATABASE_URL` (Supabase Postgres DSN)
  değerlerini doldurun; JWT'ler Supabase Auth tarafından üretilmiş olsa bile doğrulanır.
- **Yerel (compose):** Compose'daki Postgres, Supabase şemasıyla uyumlu tabloları migration ile kurar
  (`auth.users` stub'ı dahil). RLS politikaları ikinci savunma katmanıdır; birincil yetki backend'dedir.
- **Yerel (tam Supabase stack + Studio):** `npx supabase start` ile Studio/Auth/Realtime/REST dahil
  gerçek Supabase'i çalıştırıp API'yi o DB'ye bağlayabilirsiniz:

  ```sh
  npx supabase start          # Studio: http://127.0.0.1:54323, DB: 54322
  # İlk kurulumda auth şeması supabase_admin'e ait olduğu için owner devri gerekir:
  docker exec supabase_db_$(basename $PWD) psql -U supabase_admin -d postgres \
    -c "ALTER SCHEMA auth OWNER TO postgres; ALTER SCHEMA public OWNER TO postgres;"
  # API'yi Supabase DB'sine bağla:
  docker compose -f docker-compose.yml -f docker-compose.supabase.yml up -d --force-recreate api
  ```

  Migration'lar + demo seed otomatik koşar; tabloları Studio'daki Table Editor'de görürsünüz.
- **Realtime:** `notifications` tablosuna yazılan satırlar Supabase Realtime ile dinlenebilir
  (yerel stack'te `ws://127.0.0.1:54321/realtime/v1`).

## Güvenlik notları

- Secret'lar yalnızca env'den; repo'da secret yok.
- Parametrik sorgular, validator, bcrypt, JWT exp kontrolü.
- Rate limiting (`actix-governor`), CORS whitelist, güvenlik başlıkları.
- Hata gövdeleri RFC 7807; içsel detay sızdırılmaz.
