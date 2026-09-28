# Mimari Doküman

## 1. Genel bakış

- **Backend:** Rust stable, Actix Web 4.x — async REST API.
- **Veri:** Supabase Postgres (yerelde `postgres:16` container'ı, şema birebir uyumlu).
- **Auth:** Supabase Auth uyumlu JWT (`jsonwebtoken`); local email/password fallback (`auth.users` + `bcrypt`).
- **Yetki:** RBAC — `Role` hiyerarşisi (`user < moderator < admin < super_admin`) + dinamik `role_permissions` tablosu.
- **Log:** 4 katman — `tracing` JSON stdout, `audit_logs` tablosu (async), Actix access log, hata logları.
- **Bildirim:** `NotificationService` trait + `tokio-cron-scheduler` worker (due-date taraması).
- **Doküman:** `utoipa` OpenAPI + Swagger UI.

## 2. Katmanlar

```text
handlers/      → HTTP + validasyon + auth extractor (ince tutulur)
services/      → iş kuralları (sahiplik, paylaşım, RBAC, bildirim tetikleme)
repositories/  → SQL (parametrik, FromRow, soft-delete bilinci)
middleware/    → Auth (JWT→AuthUser), RBAC (rol/izın), Audit (async log + x-request-id)
models/ + dto/ → entity + API sözleşmesi (serde/validator/utoipa)
```

## 3. Veritabanı

Tablolar: `auth.users` (yerel stub; gerçek Supabase'te hazır), `profiles`, `todos`,
`todo_shares`, `categories`, `notifications`, `notification_preferences`, `audit_logs`,
`sessions`, `role_permissions`, `system_settings`.

- Index'ler: `user_id`, `status`, `created_at`, `due_date`, trigram arama (`pg_trgm`).
- `updated_at` trigger'ı (`set_updated_at()`).
- RLS açık; politikalar kullanıcı-kapsamlı. Backend doğrudan bağlanır (service rolü),
  RLS ikinci savunma katmanıdır.
- Migration'lar `sqlx::migrate!` ile gömülü — container açılışta otomatik uygulanır.

## 4. RBAC

- `AuthMiddleware` → `AuthUser { id, email, role }` (`FromRequest`).
- `require_roles` (kesin liste) / `require_min_role` (hiyerarşik) / `require_permission`
  (dinamik `role_permissions` satırı; `super_admin` bypass).
- Yetki matrisi prompt'taki tabloyla birebir: user yalnızca kendi verisi;
  moderator okuma; admin yönetim; super_admin rol atama.

## 5. Logging

- **App log:** `tracing-subscriber` JSON (`RUST_LOG` ile seviye).
- **Audit:** `Audit` middleware — POST/PATCH/PUT/DELETE + hassas GET'leri
  `tokio::spawn` ile yazar; `x-request-id` header'ı ekler. Başarısız 403'ler `warn`.
- **Kritik olaylar** (`ROLE_CHANGE`, `PERMISSION_DENIED`) ayrıca `audit_service` ile yazılır.

## 6. Bildirimler

- Tetik: paylaşım/atama, due-soon/overdue (cron), rol değişimi, broadcast.
- `check_due_todos`: 24 saat içinde aynı todo'ya tekrar göndermez; kullanıcı
  tercihlerine (`notification_preferences`) saygı duyar.
- Email/WebPush soyut (`NotificationChannel` trait) — Resend/Edge Function tak-çıkar.

## 7. API kararları

- Sayfalama: offset + cursor (keyset `created_at,id`, opak base64).
- Hatalar: RFC 7807 `ProblemDetails`; DB iç detayı sızdırılmaz.
- Rate limit: IP bazlı governor (`RATE_LIMIT_*`).
- CORS whitelist env'den; güvenlik başlıkları varsayılan.

## 8. DevOps

- Multi-stage Dockerfile (`rust:1.82` builder → `debian:bookworm-slim` runtime,
  non-root `appuser`, healthcheck).
- `docker-compose.yml`: `postgres` (16-alpine, volume, healthcheck) + `api`
  (depends_on healthy, env_file, çakışmasız portlar).
- CI: fmt + clippy (`-D warnings`) + test + audit + docker build + compose config.

## 9. Frontend + dağıtım notları

- Next.js 14 App Router; tarayıcı API isteklerini **aynı origin** üzerinden
  `/api/*` diye gönderir, sunucu tarafı reverse-proxy (rewrites) compose ağındaki
  API container'ına (`API_PROXY_TARGET`, build arg) iletir. Dış IP'den erişimde
  CORS/host sorunları oluşmaz; portlar `BIND_IP` ile tüm arayüzlere açılabilir.
- Todo listesi `position` alanına göre sıralanır; sürükle-bırak
  `POST /api/todos/reorder` ile kalıcılaşır.
- Demo hesaplar açılışta idempotent seed'lenir (`SEED_DEMO_USERS`),
  login sayfasında tek tıkla doldurulur.
- Rate limit: governor `per_millisecond(1000/rate)` — `per_second(n)`'in
  "n saniyede 1 token" semantiği değil, gerçek "saniyede n istek".

## 10. Bilinen basitleştirmeler

- Supabase Rust SDK yerine doğrudan `sqlx` (resmi SDK yok; protokol Postgres+JWT).
- Session refresh token hash'i basit hex (üretimde Argon2/SHA-256 + KMS önerilir).
- Email gönderimi log'a yazılır (Resend entegrasyonu arayüz hazır).
