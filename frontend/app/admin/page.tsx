"use client";

import { useRouter } from "next/navigation";
import { useCallback, useEffect, useState } from "react";
import { apiClient, getAccessToken, type Profile, type AuditLog } from "@/lib/client";
import { canAccess, useAuth } from "@/lib/auth";

const inputCls =
  "rounded-lg border border-zinc-300 bg-transparent px-3 py-2 text-sm dark:border-zinc-700";

function Users() {
  const [rows, setRows] = useState<Profile[]>([]);
  const [search, setSearch] = useState("");
  const load = useCallback(async () => {
    const { data, error } = await apiClient.GET("/api/admin/users", {
      params: { query: { limit: 50, ...(search ? { search } : {}) } },
    });
    if (!error) setRows(data?.data ?? []);
  }, [search]);
  useEffect(() => {
    void load();
  }, [load]);
  return (
    <div className="space-y-2">
      <input className={inputCls} placeholder="Kullanıcı ara…" value={search} onChange={(e) => setSearch(e.target.value)} />
      {rows.map((u) => (
        <div key={u.id} className="flex flex-wrap items-center gap-2 rounded-xl border border-zinc-200 p-3 text-sm dark:border-zinc-800">
          <span className="font-medium">{u.full_name}</span>
          <span className="text-zinc-500">{u.email}</span>
          <span className="rounded-full bg-zinc-200 px-2 py-0.5 text-xs dark:bg-zinc-800">{u.role}</span>
          <span className="text-xs">{u.is_active ? "aktif" : "pasif"}</span>
          <select
            className={`${inputCls} ml-auto`}
            value={u.role}
            onChange={async (e) => {
              await apiClient.PATCH("/api/admin/users/{id}/role", {
                params: { path: { id: u.id } },
                body: { role: e.target.value as Profile["role"] },
              });
              await load();
            }}
          >
            <option value="user">user</option>
            <option value="moderator">moderator</option>
            <option value="admin">admin</option>
            <option value="super_admin">super_admin</option>
          </select>
          <button
            className="text-xs text-red-600"
            onClick={async () => {
              await apiClient.PATCH("/api/admin/users/{id}/status", {
                params: { path: { id: u.id } },
                body: { is_active: !u.is_active },
              });
              await load();
            }}
          >
            {u.is_active ? "Pasifleştir" : "Aktifleştir"}
          </button>
        </div>
      ))}
    </div>
  );
}

function Logs() {
  const [rows, setRows] = useState<AuditLog[]>([]);
  const [action, setAction] = useState("");
  const [severity, setSeverity] = useState("");
  const load = useCallback(async () => {
    const { data, error } = await apiClient.GET("/api/admin/logs", {
      params: {
        query: {
          limit: 50,
          ...(action ? { action } : {}),
          ...(severity ? { severity: severity as AuditLog["severity"] } : {}),
        },
      },
    });
    if (!error) setRows(data?.data ?? []);
  }, [action, severity]);
  useEffect(() => {
    void load();
    const t = setInterval(load, 10000);
    return () => clearInterval(t);
  }, [load]);
  return (
    <div className="space-y-2">
      <div className="flex gap-2">
        <input className={inputCls} placeholder="action filtresi" value={action} onChange={(e) => setAction(e.target.value)} />
        <select className={inputCls} value={severity} onChange={(e) => setSeverity(e.target.value)}>
          <option value="">Tümü</option>
          <option value="info">info</option>
          <option value="warn">warn</option>
          <option value="error">error</option>
        </select>
        <button
          className="rounded-lg border border-zinc-300 px-3 py-2 text-sm dark:border-zinc-700"
          onClick={async () => {
            const at = getAccessToken();
            const r = await fetch(`/api/admin/logs/export?format=csv`, {
              headers: at ? { Authorization: `Bearer ${at}` } : {},
            });
            const blob = await r.blob();
            const url = URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = "audit-logs.csv";
            a.click();
            URL.revokeObjectURL(url);
          }}
        >
          CSV indir
        </button>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full text-left text-sm">
          <thead>
            <tr className="text-zinc-500">
              <th className="py-1">Zaman</th>
              <th>Aksiyon</th>
              <th>Rol</th>
              <th>Varlık</th>
              <th>Önem</th>
              <th>Durum</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.id} className="border-t border-zinc-200 dark:border-zinc-800">
                <td className="py-1">{new Date(r.created_at).toLocaleString("tr-TR")}</td>
                <td>{r.action}</td>
                <td>{r.actor_role}</td>
                <td>{r.entity_type}</td>
                <td>{r.severity}</td>
                <td>{r.status}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function Stats() {
  const [s, setS] = useState<Record<string, number> | null>(null);
  useEffect(() => {
    void apiClient
      .GET("/api/admin/stats")
      .then(({ data }) => {
        if (data) setS({ ...data });
      })
      .catch(() => undefined);
  }, []);
  if (!s) return <p>Yükleniyor…</p>;
  const max = Math.max(1, ...Object.values(s));
  return (
    <div className="space-y-2">
      {Object.entries(s).map(([k, v]) => (
        <div key={k} className="text-sm">
          <div className="flex justify-between">
            <span>{k}</span>
            <span className="font-semibold">{v}</span>
          </div>
          <div className="h-2 rounded bg-zinc-200 dark:bg-zinc-800">
            <div className="h-2 rounded bg-zinc-900 dark:bg-zinc-100" style={{ width: `${(v / max) * 100}%` }} />
          </div>
        </div>
      ))}
    </div>
  );
}

function Broadcast() {
  const [title, setTitle] = useState("");
  const [message, setMessage] = useState("");
  const [info, setInfo] = useState("");
  return (
    <form
      className="space-y-2"
      onSubmit={async (e) => {
        e.preventDefault();
        const { data, error } = await apiClient.POST("/api/admin/notifications/broadcast", {
          body: { title, message },
        });
        setInfo(error ? "Gönderilemedi" : `${data?.sent} kullanıcıya gönderildi`);
        setTitle("");
        setMessage("");
      }}
    >
      <input className={`${inputCls} w-full`} placeholder="Başlık" value={title} onChange={(e) => setTitle(e.target.value)} required />
      <textarea className={`${inputCls} w-full`} placeholder="Mesaj" value={message} onChange={(e) => setMessage(e.target.value)} required />
      <button className="rounded-lg bg-zinc-900 px-4 py-2 text-sm text-white dark:bg-zinc-100 dark:text-zinc-900">
        Yayınla
      </button>
      {info && <p className="text-sm text-green-600">{info}</p>}
    </form>
  );
}

function Settings() {
  const [rows, setRows] = useState<{ key: string; value: unknown }[]>([]);
  useEffect(() => {
    void apiClient
      .GET("/api/admin/settings")
      .then(({ data }) => {
        if (Array.isArray(data)) setRows(data as { key: string; value: unknown }[]);
      })
      .catch(() => undefined);
  }, []);
  return (
    <ul className="space-y-2 text-sm">
      {rows.map((r) => (
        <li key={r.key} className="rounded-xl border border-zinc-200 p-3 dark:border-zinc-800">
          <span className="font-medium">{r.key}</span>
          <pre className="mt-1 overflow-auto text-xs">{JSON.stringify(r.value, null, 2)}</pre>
        </li>
      ))}
    </ul>
  );
}

const TABS = ["users", "logs", "stats", "broadcast", "settings"] as const;

export default function AdminPage() {
  const { user, loading } = useAuth();
  const router = useRouter();
  const [tab, setTab] = useState<(typeof TABS)[number]>("users");

  useEffect(() => {
    if (!loading) {
      if (!user) router.push("/login");
      else if (!canAccess(user, "moderator")) router.push("/");
    }
  }, [loading, user, router]);

  if (loading || !user) return <p>Yükleniyor…</p>;

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">Yönetim</h1>
      <div className="flex flex-wrap gap-2">
        {TABS.map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`rounded-lg px-3 py-1.5 text-sm ${
              tab === t ? "bg-zinc-900 text-white dark:bg-zinc-100 dark:text-zinc-900" : "border border-zinc-300 dark:border-zinc-700"
            }`}
          >
            {t}
          </button>
        ))}
      </div>
      {tab === "users" && <Users />}
      {tab === "logs" && <Logs />}
      {tab === "stats" && <Stats />}
      {tab === "broadcast" && (canAccess(user, "admin") ? <Broadcast /> : <p>Yetkin yok.</p>)}
      {tab === "settings" && (canAccess(user, "admin") ? <Settings /> : <p>Yetkin yok.</p>)}
    </div>
  );
}
