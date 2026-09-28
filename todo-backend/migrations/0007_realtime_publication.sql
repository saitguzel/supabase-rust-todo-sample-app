-- 0007: Supabase Realtime — notifications tablosunu postgres_changes akışına ekle.
-- Not: Bu migration idempotenttir; publication yoksa (düşük Supabase sürümü) sessizce atlanır.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_publication WHERE pubname = 'supabase_realtime') THEN
    IF NOT EXISTS (
      SELECT 1 FROM pg_publication_tables
      WHERE pubname = 'supabase_realtime' AND schemaname = 'public' AND tablename = 'notifications'
    ) THEN
      EXECUTE 'ALTER PUBLICATION supabase_realtime ADD TABLE public.notifications';
    END IF;
  END IF;
END
$$;

-- Realtime, REPLICA IDENTITY FULL ile eski kayıt değerlerini de taşır (opsiyonel ama önerilir).
ALTER TABLE notifications REPLICA IDENTITY FULL;
