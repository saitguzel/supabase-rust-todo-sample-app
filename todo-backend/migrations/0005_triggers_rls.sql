-- 0005: updated_at trigger + RLS (ikinci savunma katmanı)
CREATE OR REPLACE FUNCTION set_updated_at() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN NEW.updated_at = now(); RETURN NEW; END $$;

DROP TRIGGER IF EXISTS trg_profiles_updated ON profiles;
CREATE TRIGGER trg_profiles_updated BEFORE UPDATE ON profiles
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

DROP TRIGGER IF EXISTS trg_todos_updated ON todos;
CREATE TRIGGER trg_todos_updated BEFORE UPDATE ON todos
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

DROP TRIGGER IF EXISTS trg_settings_updated ON system_settings;
CREATE TRIGGER trg_settings_updated BEFORE UPDATE ON system_settings
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- RLS: backend service_role doğrudan bağlanır (bypass), ancak defence-in-depth
-- için tüm tablolarda RLS açık ve kullanıcı-kapsamlı politikalar tanımlı.
ALTER TABLE profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE todos ENABLE ROW LEVEL SECURITY;
ALTER TABLE todo_shares ENABLE ROW LEVEL SECURITY;
ALTER TABLE categories ENABLE ROW LEVEL SECURITY;
ALTER TABLE notifications ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS p_profiles_self ON profiles;
CREATE POLICY p_profiles_self ON profiles FOR ALL USING (id = auth.uid());

DROP POLICY IF EXISTS p_todos_owner ON todos;
CREATE POLICY p_todos_owner ON todos FOR ALL USING (user_id = auth.uid());

DROP POLICY IF EXISTS p_shares_involved ON todo_shares;
CREATE POLICY p_shares_involved ON todo_shares FOR ALL USING (
  shared_with_user_id = auth.uid()
  OR EXISTS (SELECT 1 FROM todos t WHERE t.id = todo_shares.todo_id AND t.user_id = auth.uid())
);

DROP POLICY IF EXISTS p_categories_owner ON categories;
CREATE POLICY p_categories_owner ON categories FOR ALL USING (user_id = auth.uid());

DROP POLICY IF EXISTS p_notifications_owner ON notifications;
CREATE POLICY p_notifications_owner ON notifications FOR ALL USING (user_id = auth.uid());

DROP POLICY IF EXISTS p_audit_self ON audit_logs;
CREATE POLICY p_audit_self ON audit_logs FOR SELECT USING (user_id = auth.uid());
