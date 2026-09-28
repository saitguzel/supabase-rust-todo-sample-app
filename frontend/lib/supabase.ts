"use client";

// Supabase browser client — Realtime (postgres_changes) için.
// Not: Uygulama auth'u kendi backend JWT'lerini kullanır; buradaki client yalnızca
// notifications tablosuna gelen INSERT event'lerini dinler (anon key yeterli;
// satır görünürlüğü RLS'i backend service bağlantısı üzerinden aşar — event
// payload'ları WAL'dan gelir). Supabase URL/key build-time env'den gelir.
import { createClient, type RealtimeChannel } from "@supabase/supabase-js";

const SUPABASE_URL = process.env.NEXT_PUBLIC_SUPABASE_URL ?? "http://127.0.0.1:54321";
const SUPABASE_ANON_KEY =
  process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY ?? "sb_publishable_ACJWlzQHlZjBrEguHvfOxg_3BJgxAaH";

let client: ReturnType<typeof createClient> | null = null;

export function getSupabaseClient() {
  if (!client) {
    client = createClient(SUPABASE_URL, SUPABASE_ANON_KEY, {
      realtime: { params: { eventsPerSecond: 5 } },
    });
  }
  return client;
}

/** notifications INSERT akışına abone olur; cleanup fonksiyonu döner. */
export function subscribeNotifications(
  onInsert: (row: { id: string; title: string; message: string }) => void,
): () => void {
  const supabase = getSupabaseClient();
  let channel: RealtimeChannel | null = supabase
    .channel("notifications-feed")
    .on(
      "postgres_changes",
      { event: "INSERT", schema: "public", table: "notifications" },
      (payload) => {
        const row = payload.new as {
          id: string;
          title: string;
          message: string;
        };
        onInsert(row);
      },
    )
    .subscribe();

  return () => {
    if (channel) {
      void supabase.removeChannel(channel);
      channel = null;
    }
  };
}
