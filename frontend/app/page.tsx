"use client";

import { useRouter } from "next/navigation";
import { useCallback, useEffect, useState } from "react";
import { apiClient, type Todo } from "@/lib/client";
import { useAuth } from "@/lib/auth";

const inputCls =
  "rounded-lg border border-zinc-300 bg-transparent px-3 py-2 text-sm dark:border-zinc-700";

export default function Dashboard() {
  const { user, loading } = useAuth();
  const router = useRouter();
  const [todos, setTodos] = useState<Todo[]>([]);
  const [status, setStatus] = useState("");
  const [priority, setPriority] = useState("");
  const [search, setSearch] = useState("");
  const [title, setTitle] = useState("");
  const [detail, setDetail] = useState<Todo | null>(null);
  const [shareEmail, setShareEmail] = useState("");
  const [err, setErr] = useState("");
  const [dragId, setDragId] = useState<string | null>(null);
  const [dragging, setDragging] = useState(false);

  const load = useCallback(async () => {
    const { data, error } = await apiClient.GET("/api/todos", {
      params: {
        query: {
          limit: 50,
          ...(status ? { status: status as Todo["status"] } : {}),
          ...(priority ? { priority: priority as Todo["priority"] } : {}),
          ...(search ? { search } : {}),
        },
      },
    });
    if (error) throw new Error("Görevler yüklenemedi");
    setTodos(data?.data ?? []);
  }, [status, priority, search]);

  useEffect(() => {
    if (!loading && !user) router.push("/login");
  }, [loading, user, router]);

  useEffect(() => {
    if (user) void load().catch((e: unknown) => setErr(e instanceof Error ? e.message : "Yüklenemedi"));
  }, [user, load]);

  // Sürükle-bırak: bırakılan konuma göre yerel sırayı uygula, sonra sunucuya kaydet.
  const onDrop = useCallback(
    async (targetId: string) => {
      if (!dragId || dragId === targetId) return;
      const current = [...todos];
      const from = current.findIndex((t) => t.id === dragId);
      const to = current.findIndex((t) => t.id === targetId);
      if (from < 0 || to < 0) return;
      const [moved] = current.splice(from, 1);
      current.splice(to, 0, moved);
      setTodos(current);
      setDragId(null);
      const { error } = await apiClient.POST("/api/todos/reorder", {
        body: { ids: current.map((t) => t.id) },
      });
      if (error) await load().catch(() => undefined); // hata → sırayı geri yükle
    },
    [dragId, todos, load],
  );

  if (loading) return <p>Yükleniyor…</p>;
  if (!user) return null;

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">Görevlerim</h1>

      <form
        className="flex gap-2"
        onSubmit={async (e) => {
          e.preventDefault();
          if (!title.trim()) return;
          const { error } = await apiClient.POST("/api/todos", { body: { title } });
          if (error) setErr("Eklenemedi");
          setTitle("");
          await load();
        }}
      >
        <input className={`${inputCls} flex-1`} placeholder="Yeni görev…" value={title} onChange={(e) => setTitle(e.target.value)} />
        <button className="rounded-lg bg-zinc-900 px-4 py-2 text-sm text-white dark:bg-zinc-100 dark:text-zinc-900">
          Ekle
        </button>
      </form>

      <div className="flex flex-wrap gap-2">
        <select className={inputCls} value={status} onChange={(e) => setStatus(e.target.value)}>
          <option value="">Tüm durumlar</option>
          <option value="pending">Bekliyor</option>
          <option value="in_progress">Sürüyor</option>
          <option value="completed">Tamamlandı</option>
          <option value="archived">Arşiv</option>
        </select>
        <select className={inputCls} value={priority} onChange={(e) => setPriority(e.target.value)}>
          <option value="">Tüm öncelikler</option>
          <option value="low">Düşük</option>
          <option value="medium">Orta</option>
          <option value="high">Yüksek</option>
          <option value="urgent">Acil</option>
        </select>
        <input className={`${inputCls} flex-1`} placeholder="Ara…" value={search} onChange={(e) => setSearch(e.target.value)} />
      </div>

      {err && <p className="text-sm text-red-600">{err}</p>}

      <ul className={`space-y-2 ${dragging ? "opacity-80" : ""}`}>
        {todos.map((t) => (
          <li
            key={t.id}
            draggable
            onDragStart={(e) => {
              setDragId(t.id);
              setDragging(true);
              e.dataTransfer.effectAllowed = "move";
            }}
            onDragEnd={() => {
              setDragId(null);
              setDragging(false);
            }}
            onDragOver={(e) => e.preventDefault()}
            onDrop={(e) => {
              e.preventDefault();
              void onDrop(t.id);
            }}
            className={`flex cursor-grab items-center gap-3 rounded-xl border p-3 active:cursor-grabbing ${
              dragId === t.id
                ? "border-blue-400 bg-blue-50 opacity-60 dark:border-blue-600 dark:bg-blue-950"
                : "border-zinc-200 dark:border-zinc-800"
            }`}
          >
            <span title="Sürükleyerek sırala" className="select-none text-zinc-400">
              ⠿
            </span>
            <button
              title="Tamamla"
              onClick={async () => {
                await apiClient.POST("/api/todos/{id}/complete", { params: { path: { id: t.id } } });
                await load();
              }}
              className="flex h-6 w-6 items-center justify-center rounded-full border border-zinc-400 text-sm"
            >
              {t.status === "completed" ? "✓" : ""}
            </button>
            <button className="flex-1 text-left" onClick={() => setDetail(t)}>
              <span className={`font-medium ${t.status === "completed" ? "line-through opacity-60" : ""}`}>
                {t.title}
              </span>
              <span className="ml-2 rounded-full bg-zinc-200 px-2 py-0.5 text-xs dark:bg-zinc-800">{t.priority}</span>
              <span className="ml-1 rounded-full bg-zinc-200 px-2 py-0.5 text-xs dark:bg-zinc-800">{t.status}</span>
            </button>
            <button
              className="text-sm text-red-600"
              onClick={async () => {
                if (!confirm("Silinsin mi?")) return;
                await apiClient.DELETE("/api/todos/{id}", { params: { path: { id: t.id } } });
                await load();
              }}
            >
              Sil
            </button>
          </li>
        ))}
      </ul>
      {todos.length === 0 && <p className="text-sm text-zinc-500">Görev yok.</p>}

      {detail && (
        <div className="fixed inset-0 z-30 flex items-center justify-center bg-black/50 p-4" onClick={() => setDetail(null)}>
          <div
            className="w-full max-w-md space-y-3 rounded-2xl bg-white p-5 dark:bg-zinc-900"
            onClick={(e) => e.stopPropagation()}
          >
            <h2 className="text-lg font-bold">{detail.title}</h2>
            <p className="text-sm text-zinc-600 dark:text-zinc-400">{detail.description || "Açıklama yok."}</p>
            <p className="text-xs text-zinc-500">
              {detail.status} • {detail.priority} • {new Date(detail.created_at).toLocaleString("tr-TR")}
            </p>
            <div className="flex gap-2">
              <input
                className={`${inputCls} flex-1`}
                placeholder="Paylaşılacak e-posta"
                value={shareEmail}
                onChange={(e) => setShareEmail(e.target.value)}
              />
              <button
                className="rounded-lg bg-zinc-900 px-3 py-2 text-sm text-white dark:bg-zinc-100 dark:text-zinc-900"
                onClick={async () => {
                  const { error } = await apiClient.POST("/api/todos/{id}/share", {
                    params: { path: { id: detail.id } },
                    body: { email: shareEmail, permission: "read" },
                  });
                  alert(error ? "Paylaşılamadı" : "Paylaşıldı");
                  setShareEmail("");
                }}
              >
                Paylaş
              </button>
            </div>
            <button className="w-full rounded-lg border border-zinc-300 py-2 text-sm dark:border-zinc-700" onClick={() => setDetail(null)}>
              Kapat
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
