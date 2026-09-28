"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";
import { login } from "@/lib/api";
import { useAuth } from "@/lib/auth";

const DEMO_ACCOUNTS = [
  { email: "admin@demo.local", label: "Admin", desc: "Tüm yönetim yetkileri" },
  { email: "mod@demo.local", label: "Moderatör", desc: "Kullanıcı/log görüntüleme" },
  { email: "user@demo.local", label: "Kullanıcı", desc: "Sadece kendi görevleri" },
];

export default function LoginPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [err, setErr] = useState("");
  const router = useRouter();
  const { reload } = useAuth();

  const fillDemo = (demoEmail: string) => {
    setEmail(demoEmail);
    setPassword("demo1234");
    setErr("");
  };

  return (
    <div className="mx-auto max-w-sm">
      <h1 className="mb-4 text-2xl font-bold">Giriş</h1>
      <form
        className="space-y-3"
        onSubmit={async (e) => {
          e.preventDefault();
          setErr("");
          try {
            await login(email, password);
            await reload();
            router.push("/");
          } catch (ex) {
            setErr(ex instanceof Error ? ex.message : "Giriş başarısız");
          }
        }}
      >
        <input
          className="w-full rounded-lg border border-zinc-300 bg-transparent px-3 py-2 dark:border-zinc-700"
          placeholder="E-posta"
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          required
        />
        <input
          className="w-full rounded-lg border border-zinc-300 bg-transparent px-3 py-2 dark:border-zinc-700"
          placeholder="Şifre"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          required
        />
        {err && <p className="text-sm text-red-600">{err}</p>}
        <button className="w-full rounded-lg bg-zinc-900 py-2 text-white dark:bg-zinc-100 dark:text-zinc-900">
          Giriş yap
        </button>
      </form>

      <div className="mt-6 rounded-xl border border-dashed border-zinc-300 p-3 dark:border-zinc-700">
        <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-zinc-500">
          Demo hesaplar (şifre: demo1234)
        </p>
        <div className="space-y-2">
          {DEMO_ACCOUNTS.map((d) => (
            <button
              key={d.email}
              type="button"
              onClick={() => fillDemo(d.email)}
              className="flex w-full items-center justify-between rounded-lg border border-zinc-200 px-3 py-2 text-left text-sm transition hover:bg-zinc-100 dark:border-zinc-800 dark:hover:bg-zinc-800"
            >
              <span>
                <span className="font-medium">{d.label}</span>
                <span className="ml-2 text-xs text-zinc-500">{d.email}</span>
                <span className="block text-xs text-zinc-400">{d.desc}</span>
              </span>
              <span className="text-xs text-blue-600">Doldur</span>
            </button>
          ))}
        </div>
        <p className="mt-2 text-xs text-zinc-400">
          Bir hesaba tıklayınca form otomatik dolar; tek tuşla giriş yapabilirsiniz.
        </p>
      </div>
    </div>
  );
}
