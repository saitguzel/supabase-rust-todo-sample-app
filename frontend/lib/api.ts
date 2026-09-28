"use client";

// Merkezi API istemcisi — token localStorage'da, 401'de refresh dener.
// Tarayıcı her zaman aynı origin'deki /api reverse-proxy'sini kullanır
// (next.config.mjs rewrites); böylece hangi IP/host'tan açılırsa açılsın çalışır.
const API = "";

export type Role = "user" | "moderator" | "admin" | "super_admin";

export interface User {
  id: string;
  email: string;
  full_name: string;
  role: Role;
  avatar_url?: string | null;
}

export interface Tokens {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

export interface Todo {
  id: string;
  user_id: string;
  title: string;
  description: string;
  status: "pending" | "in_progress" | "completed" | "archived";
  priority: "low" | "medium" | "high" | "urgent";
  due_date: string | null;
  completed_at: string | null;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface Notification {
  id: string;
  type: string;
  title: string;
  message: string;
  is_read: boolean;
  created_at: string;
}

function getAccess(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem("access_token");
}

export function setTokens(t: Tokens) {
  localStorage.setItem("access_token", t.access_token);
  localStorage.setItem("refresh_token", t.refresh_token);
}

export function clearTokens() {
  localStorage.removeItem("access_token");
  localStorage.removeItem("refresh_token");
}

async function refresh(): Promise<boolean> {
  const rt = typeof window === "undefined" ? null : localStorage.getItem("refresh_token");
  if (!rt) return false;
  const r = await fetch(`${API}/api/auth/refresh`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ refresh_token: rt }),
  });
  if (!r.ok) {
    clearTokens();
    return false;
  }
  const t = (await r.json()) as Tokens;
  setTokens(t);
  return true;
}

export async function api<T>(path: string, init?: RequestInit, retry = true): Promise<T> {
  const headers: Record<string, string> = { "Content-Type": "application/json" };
  const at = getAccess();
  if (at) headers["Authorization"] = `Bearer ${at}`;
  const r = await fetch(`${API}${path}`, { ...init, headers: { ...headers, ...(init?.headers as Record<string, string> | undefined) } });
  if (r.status === 401 && retry && getAccess()) {
    if (await refresh()) return api<T>(path, init, false);
    if (typeof window !== "undefined") window.location.href = "/login";
    throw new Error("unauthorized");
  }
  if (r.status === 204) return undefined as T;
  if (!r.ok) {
    const body = await r.text();
    throw new Error(body || `request failed: ${r.status}`);
  }
  return (await r.json()) as T;
}

// --- Auth (tokensız) ---
export async function register(email: string, password: string, full_name: string) {
  const r = await fetch(`${API}/api/auth/register`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password, full_name }),
  });
  if (!r.ok) throw new Error(await r.text());
  const d = (await r.json()) as { user: User; tokens: Tokens };
  setTokens(d.tokens);
  return d;
}

export async function login(email: string, password: string) {
  const r = await fetch(`${API}/api/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password }),
  });
  if (!r.ok) throw new Error(await r.text());
  const d = (await r.json()) as { user: User; tokens: Tokens };
  setTokens(d.tokens);
  return d;
}

export async function logout() {
  const rt = typeof window === "undefined" ? null : localStorage.getItem("refresh_token");
  if (rt) {
    await fetch(`${API}/api/auth/logout`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ refresh_token: rt }),
    }).catch(() => undefined);
  }
  clearTokens();
}
