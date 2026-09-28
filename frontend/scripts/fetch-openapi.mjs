// Canlı API'den OpenAPI spec'ini çeker (lib/api-spec.json) — tip üretiminin kaynağı.
// Kullanım: node scripts/fetch-openapi.mjs [apiBase]
//   apiBase varsayılan: http://127.0.0.1:18081 (compose API)
import { writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const out = path.join(root, "lib", "api-spec.json");
const apiBase = process.argv[2] ?? process.env.API_BASE ?? "http://127.0.0.1:18081";

const res = await fetch(`${apiBase}/api-docs/openapi.json`);
if (!res.ok) {
  console.error(`✘ openapi.json alınamadı: ${res.status} (${apiBase})`);
  console.error("  API ayakta mı? → docker compose up -d api");
  process.exit(1);
}
const spec = await res.text();
// Geçerli JSON olduğundan emin ol.
JSON.parse(spec);
await writeFile(out, spec + "\n", "utf8");
const paths = Object.keys(JSON.parse(spec).paths ?? {}).length;
console.log(`✓ ${out} güncellendi (${paths} path, kaynak: ${apiBase})`);
