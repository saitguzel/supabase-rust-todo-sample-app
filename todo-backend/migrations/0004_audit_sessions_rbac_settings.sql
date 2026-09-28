-- 0004: audit_logs + sessions + rbac + settings
CREATE TABLE IF NOT EXISTS audit_logs (
  id bigserial PRIMARY KEY,
  user_id uuid REFERENCES profiles(id) ON DELETE SET NULL,
  actor_role text NOT NULL DEFAULT 'anonymous',
  action text NOT NULL,
  entity_type text NOT NULL DEFAULT '',
  entity_id text NOT NULL DEFAULT '',
  old_values jsonb,
  new_values jsonb,
  ip_address inet,
  user_agent text,
  request_id uuid,
  severity audit_severity NOT NULL DEFAULT 'info',
  status audit_status NOT NULL DEFAULT 'success',
  error_message text,
  duration_ms integer NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_severity ON audit_logs(severity);
CREATE INDEX IF NOT EXISTS idx_audit_request ON audit_logs(request_id);

CREATE TABLE IF NOT EXISTS sessions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  refresh_token_hash text NOT NULL,
  ip inet,
  user_agent text,
  expires_at timestamptz NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);

CREATE TABLE IF NOT EXISTS role_permissions (
  id bigserial PRIMARY KEY,
  role user_role NOT NULL,
  resource text NOT NULL,
  action text NOT NULL,
  allowed boolean NOT NULL DEFAULT false,
  UNIQUE(role, resource, action)
);

CREATE TABLE IF NOT EXISTS system_settings (
  key text PRIMARY KEY,
  value jsonb NOT NULL DEFAULT '{}',
  updated_by uuid REFERENCES profiles(id) ON DELETE SET NULL,
  updated_at timestamptz NOT NULL DEFAULT now()
);

-- Dinamik RBAC varsayılan matris (prompt'taki yetki matrisine uygun)
INSERT INTO role_permissions (role, resource, action, allowed) VALUES
  ('user','todo','read',true), ('user','todo','create',true),
  ('user','todo','update',true), ('user','todo','delete',true),
  ('user','notification','read',true), ('user','notification','update',true),
  ('user','user','read',false), ('user','log','read',false),
  ('moderator','todo','read',true), ('moderator','todo','create',true),
  ('moderator','todo','update',true), ('moderator','todo','delete',true),
  ('moderator','notification','read',true),
  ('moderator','user','read',true), ('moderator','log','read',true),
  ('moderator','user','update',false), ('moderator','log','manage',false),
  ('admin','todo','read',true), ('admin','todo','create',true),
  ('admin','todo','update',true), ('admin','todo','delete',true),
  ('admin','todo','manage',true),
  ('admin','user','read',true), ('admin','user','create',true),
  ('admin','user','update',true), ('admin','user','delete',true),
  ('admin','user','manage',true),
  ('admin','log','read',true), ('admin','log','manage',true),
  ('admin','notification','read',true), ('admin','notification','create',true),
  ('admin','notification','manage',true),
  ('super_admin','todo','manage',true), ('super_admin','user','manage',true),
  ('super_admin','log','manage',true), ('super_admin','notification','manage',true)
ON CONFLICT (role, resource, action) DO NOTHING;

INSERT INTO system_settings (key, value) VALUES
  ('maintenance_mode', '{"enabled": false}'),
  ('registration_enabled', '{"enabled": true}'),
  ('default_page_size', '{"value": 20}')
ON CONFLICT (key) DO NOTHING;
