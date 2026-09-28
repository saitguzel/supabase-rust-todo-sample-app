-- 0001: extensions + enum types
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

DO $$ BEGIN CREATE TYPE user_role AS ENUM ('user','moderator','admin','super_admin'); EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN CREATE TYPE todo_status AS ENUM ('pending','in_progress','completed','archived'); EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN CREATE TYPE todo_priority AS ENUM ('low','medium','high','urgent'); EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN CREATE TYPE share_permission AS ENUM ('read','write'); EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN CREATE TYPE notification_type AS ENUM ('todo_assigned','todo_due_soon','todo_overdue','todo_completed','system_announcement','role_changed'); EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN CREATE TYPE audit_severity AS ENUM ('debug','info','warn','error','critical'); EXCEPTION WHEN duplicate_object THEN NULL; END $$;
DO $$ BEGIN CREATE TYPE audit_status AS ENUM ('success','failure'); EXCEPTION WHEN duplicate_object THEN NULL; END $$;

-- Supabase uyumluluğu için minimal auth şeması (yerel/docker ortamında).
-- Gerçek Supabase'te auth.users zaten vardır; burada IF NOT EXISTS ile idempotent.
CREATE SCHEMA IF NOT EXISTS auth;

CREATE TABLE IF NOT EXISTS auth.users (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  email text UNIQUE NOT NULL,
  encrypted_password text NOT NULL DEFAULT '',
  created_at timestamptz NOT NULL DEFAULT now()
);

-- Supabase'in auth.uid() fonksiyonunun yerel stub'ı.
-- Gerçek Supabase'te JWT'den gelir; backend JWT'yi kendisi doğruladığı için
-- RLS ikinci savunma katmanı olarak kalır, birincil yetki backend'dedir.
CREATE OR REPLACE FUNCTION auth.uid() RETURNS uuid
LANGUAGE sql STABLE AS $$ SELECT NULL::uuid $$;
