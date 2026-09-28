"use client";

import Link from "next/link";
import { useEffect, useRef, useState } from "react";
import { useAuth } from "@/lib/auth";
import { api, type Notification } from "@/lib/api";
import { subscribeNotifications } from "@/lib/supabase";

function ThemeToggle() {
  const [dark, setDark] = useState(false);
  useEffect(() => {
    const saved = localStorage.getItem("theme");
    const on = saved ? saved === "dark" : window.matchMedia("(prefers-color-scheme: dark)").matches;
    setDark(on);
    document.documentElement.classList.toggle("dark", on);
  }, []);
  return (
    <button
      onClick={() => {
        const next = !dark;
        setDark(next);
        document.documentElement.classList.toggle("dark", next);
        localStorage.setItem("theme", next ? "dark" : "light");
      }}
      className="rounded-lg border border-zinc-300 px-2 py-1 text-sm dark:border-zinc-700"
      aria-label="Tema değiştir"
    >
      {dark ? "☀️" : "🌙"}
    </button>
  );
}

function Bell() {
  const [open, setOpen] = useState(false);
  const [items, setItems] = useState<Notification[]>([]);
  const [unread, setUnread] = useState(0);
  const { user } = useAuth();

  const load = async () => {
    try {
      const d = await api<{ data: Notification[]; unread: number }>("/api/notifications?unread=false");
      // API yanıtı {data, unread} içerir.
      setItems(d.data ?? []);
      setUnread(d.unread ?? 0);
    } catch {
      /* giriş yoksa sessiz */
    }
  };

  useEffect(() => {
    void load();
    const t = setInterval(load, 15000); // polling fallback (realtime koparsa)
    return () => clearInterval(t);
  }, []);

  // Supabase Realtime: notifications INSERT'lerini anlık dinle (15 sn polling'e gerek kalmadan).
  // Broadcast todo'ya göre değil kullanıcıya göre akar; user değişince yeniden abone ol.
  const subRef = useRef<string | null>(null);
  useEffect(() => {
    if (!user) return;
    subRef.current = user.id;
    const unsub = subscribeNotifications((row) => {
      setItems((prev) => [{ ...row, type: "system_announcement", is_read: false, created_at: new Date().toISOString() } as Notification, ...prev].slice(0, 20));
      setUnread((u) => u + 1);
    });
    return unsub;
  }, [user?.id]);

  return (
    <div className="relative">
      <button
        onClick={() => setOpen((v) => !v)}
        className="relative rounded-lg border border-zinc-300 px-2 py-1 text-sm dark:border-zinc-700"
        aria-label="Bildirimler"
      >
        🔔{unread > 0 && <span className="ml-1 rounded-full bg-red-600 px-1.5 text-xs text-white">{unread}</span>}
      </button>
      {open && (
        <div className="absolute right-0 z-20 mt-2 w-80 rounded-xl border border-zinc-200 bg-white p-2 shadow-lg dark:border-zinc-700 dark:bg-zinc-900">
          <div className="flex items-center justify-between px-2 py-1">
            <span className="text-sm font-semibold">Bildirimler</span>
            <button
              className="text-xs text-blue-600"
              onClick={async () => {
                await api("/api/notifications/read-all", { method: "POST" });
                await load();
              }}
            >
              Tümünü okundu işaretle
            </button>
          </div>
          <div className="max-h-72 overflow-auto">
            {items.length === 0 && <p className="px-2 py-4 text-sm text-zinc-500">Bildirim yok.</p>}
            {items.slice(0, 20).map((n) => (
              <div
                key={n.id}
                role="button"
                tabIndex={0}
                title={n.is_read ? "Okundu" : "Okundu işaretlemek için tıkla"}
                onClick={async () => {
                  if (n.is_read) return;
                  await api(`/api/notifications/${n.id}/read`, { method: "PATCH" }).catch(() => undefined);
                  await load();
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter") void (async () => {
                    if (n.is_read) return;
                    await api(`/api/notifications/${n.id}/read`, { method: "PATCH" }).catch(() => undefined);
                    await load();
                  })();
                }}
                className={`cursor-pointer rounded-lg px-2 py-1.5 text-sm ${n.is_read ? "opacity-60" : "bg-zinc-100 dark:bg-zinc-800"}`}
              >
                <p className="font-medium">{n.title}</p>
                <p className="text-zinc-600 dark:text-zinc-400">{n.message}</p>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

export function Navbar() {
  const { user, signOut } = useAuth();
  const isStaff = user && user.role !== "user";
  return (
    <header className="sticky top-0 z-10 border-b border-zinc-200 bg-white/80 backdrop-blur dark:border-zinc-800 dark:bg-zinc-950/80">
      <div className="mx-auto flex max-w-5xl items-center gap-3 px-4 py-3">
        <Link href="/" className="font-bold">
          ✅ Todo
        </Link>
        <nav className="flex items-center gap-3 text-sm">
          <Link href="/" className="hover:underline">
            Panel
          </Link>
          {isStaff && (
            <Link href="/admin" className="hover:underline">
              Yönetim
            </Link>
          )}
        </nav>
        <div className="ml-auto flex items-center gap-2">
          <ThemeToggle />
          {user && <Bell />}
          {user ? (
            <>
              <span className="hidden text-sm text-zinc-600 sm:inline dark:text-zinc-400">
                {user.full_name} ({user.role})
              </span>
              <button onClick={() => void signOut()} className="rounded-lg bg-zinc-900 px-3 py-1.5 text-sm text-white dark:bg-zinc-100 dark:text-zinc-900">
                Çıkış
              </button>
            </>
          ) : (
            <>
              <Link href="/login" className="text-sm hover:underline">
                Giriş
              </Link>
              <Link href="/register" className="rounded-lg bg-zinc-900 px-3 py-1.5 text-sm text-white dark:bg-zinc-100 dark:text-zinc-900">
                Kayıt
              </Link>
            </>
          )}
        </div>
      </div>
    </header>
  );
}
