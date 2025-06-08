# ビルドステージ
FROM node:20-alpine AS builder

WORKDIR /app

# 依存関係のキャッシュのために、まずpackage.jsonとpackage-lock.jsonをコピー
COPY package*.json ./

# 依存関係をインストール
RUN npm ci

# ソースコードをコピー
COPY . .

# ビルド時の環境変数
ARG VITE_API_URL
ENV VITE_API_URL=$VITE_API_URL

# アプリケーションをビルド
RUN npm run build || true

# ビルド出力ディレクトリを確認
RUN ls -la /app/.svelte-kit/output/client || true

# 実行ステージ - Nginxを使用
FROM nginx:alpine

# Nginxの設定ファイルをコピー
COPY --from=builder /app/nginx.conf /etc/nginx/nginx.conf

# ビルドしたファイルをコピー（SvelteKitの出力ディレクトリから）
COPY --from=builder /app/.svelte-kit/output/client /usr/share/nginx/html
COPY --from=builder /app/.svelte-kit/output/prerendered/pages /usr/share/nginx/html

# 非rootユーザーで実行するための設定
RUN chown -R nginx:nginx /usr/share/nginx/html && \
    chmod -R 755 /usr/share/nginx/html && \
    chown -R nginx:nginx /var/cache/nginx && \
    chown -R nginx:nginx /var/log/nginx && \
    chown -R nginx:nginx /etc/nginx/conf.d && \
    touch /var/run/nginx.pid && \
    chown -R nginx:nginx /var/run/nginx.pid && \
    chown nginx:nginx /etc/nginx/nginx.conf && \
    rm -f /etc/nginx/conf.d/default.conf

# 非rootユーザーに切り替え
USER nginx

EXPOSE 8080

# ヘルスチェック
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://127.0.0.1:8080/ || exit 1

CMD ["nginx", "-g", "daemon off;"]