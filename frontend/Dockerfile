# ── Builder ──────────────────────────────────────────
FROM node:20-bookworm-slim AS builder
WORKDIR /app
COPY package.json ./
RUN npm install --no-audit --no-fund
COPY . ./
ARG NEXT_PUBLIC_API_URL=http://127.0.0.1:18081
ENV NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL
# Rewrites build anında donar; compose ağındaki API adresi burada sabitlenir.
ARG API_PROXY_TARGET=http://api:8080
ENV API_PROXY_TARGET=$API_PROXY_TARGET
RUN npm run build

# ── Runtime (standalone) ─────────────────────────────
FROM node:20-bookworm-slim
RUN useradd -m -u 10002 webuser
WORKDIR /app
ENV NODE_ENV=production PORT=3000 HOSTNAME=0.0.0.0
COPY --from=builder /app/.next/standalone ./
COPY --from=builder /app/.next/static ./.next/static
COPY --from=builder /app/public ./public
USER webuser
EXPOSE 3000
HEALTHCHECK --interval=15s --timeout=5s --retries=5 \
  CMD node -e "fetch('http://127.0.0.1:3000/').then(r=>{if(!r.ok)process.exit(1)}).catch(()=>process.exit(1))"
ENTRYPOINT ["node", "server.js"]
