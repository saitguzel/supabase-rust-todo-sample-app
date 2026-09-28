-- 0006: todo sıralaması (position) + dosya ekleri + push abonelikleri
ALTER TABLE todos ADD COLUMN IF NOT EXISTS position bigint NOT NULL DEFAULT 0;

-- Mevcut satırları kullanıcı bazında oluşturulma sırasına göre numarala.
WITH ranked AS (
  SELECT id, ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at, id) - 1 AS rn
  FROM todos
)
UPDATE todos t SET position = r.rn FROM ranked r WHERE r.id = t.id;

CREATE INDEX IF NOT EXISTS idx_todos_position ON todos(user_id, position);

-- Dosya ekleri (Supabase Storage yoksa yerel dizine yazılır; bkz. ATTACHMENTS_DIR).
CREATE TABLE IF NOT EXISTS todo_attachments (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  todo_id uuid NOT NULL REFERENCES todos(id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  file_name text NOT NULL,
  content_type text NOT NULL DEFAULT 'application/octet-stream',
  size_bytes bigint NOT NULL DEFAULT 0,
  storage_path text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_attachments_todo ON todo_attachments(todo_id);

-- Web Push abonelikleri (tarayıcı PushManager).
CREATE TABLE IF NOT EXISTS push_subscriptions (
  endpoint text PRIMARY KEY,
  user_id uuid NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  p256dh text NOT NULL,
  auth text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_push_user ON push_subscriptions(user_id);

ALTER TABLE todo_attachments ENABLE ROW LEVEL SECURITY;
ALTER TABLE push_subscriptions ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS p_attachments_owner ON todo_attachments;
CREATE POLICY p_attachments_owner ON todo_attachments FOR ALL USING (user_id = auth.uid());

DROP POLICY IF EXISTS p_push_owner ON push_subscriptions;
CREATE POLICY p_push_owner ON push_subscriptions FOR ALL USING (user_id = auth.uid());
