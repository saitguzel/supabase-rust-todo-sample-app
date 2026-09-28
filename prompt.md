Sen kıdemli bir full-stack yazılım mimarısın. Aşağıdaki gereksinimlere göre PRODUCTION-READY, 
ölçeklenebilir ve güvenli bir Todo uygulaması tasarla ve kodla.

## 🧱 TEKNOLOJİ YIĞINI

### Backend
- **Rust** (stable, edition 2021+)
- **Actix Web 4.x** — REST API
- **SQLx veya Supabase Rust SDK** — Supabase Postgres bağlantısı
- **JWT (jsonwebtoken)** — Supabase JWT doğrulaması
- **Tower / actix-web middleware** — auth, logging, rate limiting
- **tracing + tracing-subscriber** — yapılandırılmış log
- **validator** — input validation
- **dotenvy** — env yönetimi
- **utoipa + utoipa-swagger-ui** — OpenAPI/Swagger dokümantasyonu
- **thiserror / anyhow** — hata yönetimi

### Veritabanı & Servisler
- **Supabase (PostgreSQL)** — ana veri deposu
- **Supabase Auth** — kimlik doğrulama (email/password + OAuth)
- **Supabase Storage** — dosya ekleri (opsiyonel)
- **Supabase Realtime** — bildirim ve canlı güncelleme
- **Supabase Row Level Security (RLS)** — ikinci savunma katmanı

### Frontend (öneri)
- Next.js 14 (App Router) + TypeScript + TailwindCSS + shadcn/ui
- Veya Leptos / Yew (Rust full-stack tercih edilirse)

### DevOps
- Docker + docker-compose
- GitHub Actions (CI/CD)
- Fly.io / Railway / Render (deploy)

---

## 📊 VERİTABANI ŞEMASI (Supabase / PostgreSQL)

Aşağıdaki tabloları migration dosyalarıyla oluştur:

### 1. `profiles` (Supabase auth.users'a 1-1 bağlı)
- id (uuid, PK, references auth.users)
- email (text, unique)
- full_name (text)
- avatar_url (text)
- role (enum: 'user', 'moderator', 'admin', 'super_admin')
- is_active (bool, default true)
- last_login_at (timestamptz)
- created_at, updated_at (timestamptz)

### 2. `todos`
- id (uuid, PK)
- user_id (uuid, FK → profiles.id)
- title (text, not null, max 255)
- description (text)
- status (enum: 'pending', 'in_progress', 'completed', 'archived')
- priority (enum: 'low', 'medium', 'high', 'urgent')
- due_date (timestamptz)
- completed_at (timestamptz)
- tags (text[])
- is_deleted (bool, default false) — soft delete
- created_at, updated_at

### 3. `todo_shares` (paylaşılan todo'lar)
- id, todo_id, shared_with_user_id, permission ('read' | 'write')
- created_at

### 4. `categories` / `tags`
- id, user_id, name, color, created_at

### 5. `notifications`
- id (uuid, PK)
- user_id (uuid, FK)
- type (enum: 'todo_assigned', 'todo_due_soon', 'todo_overdue', 
  'todo_completed', 'system_announcement', 'role_changed')
- title (text)
- message (text)
- payload (jsonb) — ek veri
- is_read (bool, default false)
- read_at (timestamptz)
- created_at

### 6. `audit_logs` — TÜM LOGLAR BURADA
- id (bigserial, PK)
- user_id (uuid, nullable — anonim işlemler için)
- actor_role (text) — işlemi yapan rol
- action (text) — 'CREATE', 'UPDATE', 'DELETE', 'LOGIN', 'LOGOUT', 
  'ROLE_CHANGE', 'PERMISSION_DENIED', 'EXPORT' vb.
- entity_type (text) — 'todo', 'user', 'notification' vb.
- entity_id (uuid/text)
- old_values (jsonb)
- new_values (jsonb)
- ip_address (inet)
- user_agent (text)
- request_id (uuid) — trace için
- severity (enum: 'debug','info','warn','error','critical')
- status ('success' | 'failure')
- error_message (text)
- duration_ms (int)
- created_at (timestamptz, indexed)

### 7. `sessions` (opsiyonel — aktif oturum takibi)
- id, user_id, refresh_token_hash, ip, user_agent, expires_at, created_at

### 8. `role_permissions` (dinamik RBAC için)
- id, role, resource ('todo','user','log','notification'), 
  action ('read','create','update','delete','manage'), allowed (bool)

### 9. `system_settings` (admin tarafından yönetilir)
- key (PK), value (jsonb), updated_by, updated_at

**Tüm tablolar için:**
- Uygun index'ler (user_id, created_at, status, action)
- `updated_at` için trigger
- RLS politikaları
- Foreign key constraint'ler ve ON DELETE stratejileri

---

## 🔐 ROLE-BASED ACCESS CONTROL (RBAC)

Şu rolleri destekle:
- **super_admin**: Her şey + rol atama
- **admin**: Tüm kullanıcı/todo/log yönetimi
- **moderator**: İçerik moderasyonu, log görüntüleme
- **user**: Sadece kendi verisi

### Middleware Katmanları (Actix)
1. `AuthMiddleware` — JWT doğrula, `req.extensions`'a `Claims` ekle
2. `RoleGuard(roles: Vec<Role>)` — belirli endpoint'ler için rol kontrolü
3. `PermissionGuard(resource, action)` — granular yetki kontrolü
4. `AuditMiddleware` — her isteği otomatik logla (opt-out edilebilir)
5. `RateLimitMiddleware` — IP ve user bazlı

### Endpoint Yetki Matrisi (örnek)
| Endpoint | user | moderator | admin |
|---|---|---|---|
| GET /todos | own | all | all |
| DELETE /todos/:id | own | any | any |
| GET /admin/users | ✗ | read | full |
| GET /admin/logs | ✗ | read | read+export |
| PATCH /admin/users/:id/role | ✗ | ✗ | ✓ |
| GET /admin/notifications/broadcast | ✗ | ✗ | ✓ |

---

## 🛣️ API ENDPOINT'LERİ

### Auth
- `POST /api/auth/register`
- `POST /api/auth/login`
- `POST /api/auth/refresh`
- `POST /api/auth/logout`
- `GET  /api/auth/me`

### Todos (user scope)
- `GET    /api/todos?status=&priority=&search=&page=&limit=`
- `POST   /api/todos`
- `GET    /api/todos/:id`
- `PATCH  /api/todos/:id`
- `DELETE /api/todos/:id` (soft delete)
- `POST   /api/todos/:id/complete`
- `POST   /api/todos/:id/share`
- `POST   /api/todos/bulk` (bulk işlemler)

### Notifications
- `GET    /api/notifications?unread=true`
- `PATCH  /api/notifications/:id/read`
- `POST   /api/notifications/read-all`
- `DELETE /api/notifications/:id`

### Admin (rol korumalı)
- `GET    /api/admin/users?role=&active=&search=`
- `PATCH  /api/admin/users/:id/role`
- `PATCH  /api/admin/users/:id/status` (ban/unban)
- `GET    /api/admin/logs?action=&severity=&user_id=&from=&to=`
- `GET    /api/admin/logs/export?format=csv|json`
- `GET    /api/admin/stats` — dashboard metrikleri
- `POST   /api/admin/notifications/broadcast`
- `GET    /api/admin/settings`
- `PATCH  /api/admin/settings`

Her endpoint için:
- Request/Response DTO'ları (serde)
- Validation
- Açık hata mesajları (RFC 7807 Problem Details formatı)
- OpenAPI dokümantasyonu

---

## 📝 LOGGING STRATEJİSİ

### Katmanlar
1. **Application Logs** → `tracing` (stdout + dosya, JSON format)
2. **Audit Logs** → Supabase `audit_logs` tablosu
3. **Access Logs** → Actix `Logger` middleware
4. **Error Logs** → Sentry / Supabase

### Audit Middleware Davranışı
- Her mutasyon isteğini (POST/PATCH/DELETE) otomatik logla
- GET isteklerini sadece hassas endpoint'lerde logla
- `old_values` ve `new_values` diff'ini jsonb olarak sakla
- `request_id` (UUID) üret, response header'a ekle, client'a trace için ver
- Başarısız yetki denemelerini `severity='warn'` olarak logla
- Log yazımı **async** (tokio::spawn veya channel) — request'i bloklamasın

### Admin Log Görüntüleme
- Filtreleme: kullanıcı, tarih aralığı, action, severity, entity_type
- Full-text search (jsonb üzerinde)
- Sayfalama + cursor-based
- CSV/JSON export
- Real-time stream (Supabase Realtime)

---

## 🔔 BİLDİRİM SİSTEMİ

### Tetikleyiciler
- Todo atandığında / paylaşıldığında
- Todo due date yaklaştığında (cron job)
- Todo geciktiğinde
- Rol değiştiğinde
- Admin broadcast gönderdiğinde
- Sistem duyuruları

### Kanallar
1. **In-app** (Supabase Realtime + notifications tablosu)
2. **Email** (Resend / Supabase Edge Function)
3. **Web Push** (opsiyonel — web-push crate)

### Backend
- `NotificationService` trait'i ile soyutla
- Arka plan worker (tokio task + `tokio-cron-scheduler`) — due date kontrolü
- Kullanıcı bazlı bildirim tercihleri (`notification_preferences` tablosu)

---

## 🏗️ PROJE YAPISI (Rust Backend)
todo-backend/
├── Cargo.toml
├── .env.example
├── docker-compose.yml
├── Dockerfile
├── migrations/ # sqlx migrations
├── src/
│ ├── main.rs
│ ├── config.rs
│ ├── state.rs # AppState (db pool, supabase client)
│ ├── errors.rs # AppError → ResponseError
│ ├── models/ # entity structs
│ ├── dto/ # request/response
│ ├── handlers/ # route handlers
│ │ ├── auth.rs
│ │ ├── todos.rs
│ │ ├── notifications.rs
│ │ └── admin/
│ ├── services/ # business logic
│ │ ├── auth_service.rs
│ │ ├── todo_service.rs
│ │ ├── notification_service.rs
│ │ └── audit_service.rs
│ ├── middleware/
│ │ ├── auth.rs
│ │ ├── rbac.rs
│ │ └── audit.rs
│ ├── repositories/ # db access
│ ├── routes.rs # route tanımları
│ └── utils/
└── tests/ # integration tests

text

---

## ✅ KOD BEKLENTİLERİ

1. **Tüm endpoint'ler async** — `async fn` + `.await`
2. **Error handling** — `Result<T, AppError>`, custom `ResponseError` impl
3. **Input validation** — her DTO'da `#[derive(Validate)]`
4. **Type safety** — `sqlx::query_as!` macro (compile-time SQL kontrolü)
5. **Pagination** — cursor-based (keyset), limit 1-100
6. **SQL injection koruması** — parametrik sorgular
7. **Rate limiting** — `actix-governor`
8. **CORS** — whitelist env'den
9. **Security headers** — Helmet benzeri middleware
10. **Secrets** — sadece env, asla hard-coded
11. **Graceful shutdown** — SIGTERM yakala, connection pool'ları kapat
12. **Health check** — `/health` ve `/health/db`
13. **Birim testler + integration testler** — `#[actix_web::test]`
14. **Clippy clean** — `#![deny(warnings)]` ve `cargo clippy`
15. **Dokümantasyon** — her public item'a doc comment

---

## 🎨 FRONTEND BEKLENTİLERİ

- Login/Register sayfaları (Supabase Auth)
- Dashboard: todo listesi (drag&drop sıralama, filtre, arama)
- Todo detay modal
- Bildirim ikonu + dropdown (realtime)
- Profil ayarları
- **Admin Panel**:
  - Kullanıcı yönetimi (rol atama, ban, arama)
  - Log görüntüleyici (filtre + export + realtime)
  - İstatistik dashboard (chart'lar)
  - Broadcast bildirim gönderimi
  - Sistem ayarları
- Dark/light mode
- Responsive (mobile-first)
- Rol bazlı menü (yetkisi olmayan sekmeler gizli)

---

## 🧪 TEST & KALİTE

- Unit test coverage > %80
- E2E testler (Playwright)
- Load test (k6 / vegeta) — p95 < 200ms
- Security: OWASP Top 10 kontrol listesi
- `cargo audit` CI'da çalışsın

---

## 📦 ÇIKTI FORMATI

Bana adım adım şunları üret:

1. **Mimari doküman** (yukarıdaki yapıya uygun, kısa açıklamalı)
2. **SQL migration dosyaları** (tüm tablolar, index'ler, RLS, trigger'lar)
3. **`Cargo.toml`** — tüm bağımlılıklar, versiyonlarıyla
4. **`.env.example`** — tüm gerekli env değişkenleri
5. **`main.rs` + `state.rs` + `config.rs` + `errors.rs`** — tam kod
6. **Auth middleware + RBAC middleware + Audit middleware** — tam kod
7. **Todo handler + service + repository** — tam kod (CRUD)
8. **Admin log endpoint'i** (filtreli, sayfalı, export) — tam kod
9. **Notification service + cron job** — tam kod
10. **Route tanımları (`routes.rs`)**
11. **Dockerfile + docker-compose.yml**
12. **GitHub Actions CI dosyası**
13. **`README.md`** — kurulum, çalıştırma, endpoint listesi

Her adımı ayrı ayrı, açıklamalı ve çalışır durumda ver. 
Eksik varsayım yapma; belirsizlik varsa bana sor.