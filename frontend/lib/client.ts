// openapi-typescript tarafından üretilen tipler (lib/api-spec.json kaynağından).
import type { paths, components } from "./api-types";
import createClient, { type Middleware } from "openapi-fetch";

// Şema tipleri — eski el yazımı tiplerle uyumlu isimler.
export type Todo = components["schemas"]["Todo"];
export type TodoStatus = components["schemas"]["TodoStatus"];
export type TodoPriority = components["schemas"]["TodoPriority"];
export type Profile = components["schemas"]["Profile"];
export type Role = components["schemas"]["Role"];
export type Notification = components["schemas"]["Notification"];
export type NotificationType = components["schemas"]["NotificationType"];
export type AuditLog = components["schemas"]["AuditLog"];
export type PaginatedTodo = components["schemas"]["PaginatedTodo"];
export type TokenResponse = components["schemas"]["TokenResponse"];
export type StatsResponse = components["schemas"]["StatsResponse"];

// Endpoint bazlı request/response tipleri.
export type CreateTodoBody =
  | paths["/api/todos"]["post"]["requestBody"]["content"]["application/json"];
export type UpdateTodoBody =
  | paths["/api/todos/{id}"]["patch"]["requestBody"]["content"]["application/json"];
export type LoginBody =
  | paths["/api/auth/login"]["post"]["requestBody"]["content"]["application/json"];
export type ListTodosResponse =
  | paths["/api/todos"]["get"]["responses"]["200"]["content"]["application/json"];

function getAccess(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem("access_token");
}

function getRefresh(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem("refresh_token");
}

/** api.ts (localStorage tabanlı) ile aynı token havuzunu paylaşmak için. */
export const getAccessToken = getAccess;

/** Bearer header ekler; 401'de refresh ile bir kez yeniden dener. */
const authMiddleware: Middleware = {
  async onRequest({ request }) {
    const at = getAccess();
    if (at) request.headers.set("Authorization", `Bearer ${at}`);
    return request;
  },
  async onResponse({ request, response }) {
    if (response.status !== 401 || !getRefresh()) return response;
    const r = await fetch("/api/auth/refresh", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ refresh_token: getRefresh() }),
    });
    if (!r.ok) {
      localStorage.removeItem("access_token");
      localStorage.removeItem("refresh_token");
      if (typeof window !== "undefined") window.location.href = "/login";
      return response;
    }
    const tokens = (await r.json()) as TokenResponse;
    localStorage.setItem("access_token", tokens.access_token);
    localStorage.setItem("refresh_token", tokens.refresh_token);
    const retry = request.clone();
    retry.headers.set("Authorization", `Bearer ${tokens.access_token}`);
    return fetch(retry);
  },
};

/**
 * Tip güvenli API istemcisi (openapi-fetch).
 *
 * Kullanım:
 *   import { apiClient } from "@/lib/client";
 *   const { data, error } = await apiClient.GET("/api/todos", { params: { query: { limit: 20 } } });
 *   const { data } = await apiClient.POST("/api/todos", { body: { title: "Yeni" } });
 *
 * - Yol/metod adları ve request/response tipleri spec'ten derleme zamanında doğrulanır.
 * - Tarayıcı aynı origin'deki /api reverse-proxy'sini kullanır (baseUrl boş).
 */
export const apiClient = createClient<paths>({ baseUrl: "" });
apiClient.use(authMiddleware);

export type ApiClient = typeof apiClient;
