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
RUN npm run build

# 実行ステージ - Nginxを使用
FROM nginx:alpine

# Nginxの設定ファイルをコピー
COPY --from=builder /app/nginx.conf /etc/nginx/conf.d/default.conf

# ビルドしたファイルをコピー
COPY --from=builder /app/build /usr/share/nginx/html

# 非rootユーザーで実行するための設定
RUN chown -R nginx:nginx /usr/share/nginx/html && \
    chmod -R 755 /usr/share/nginx/html && \
    chown -R nginx:nginx /var/cache/nginx && \
    chown -R nginx:nginx /var/log/nginx && \
    chown -R nginx:nginx /etc/nginx/conf.d && \
    touch /var/run/nginx.pid && \
    chown -R nginx:nginx /var/run/nginx.pid

# 非rootユーザーに切り替え
USER nginx

EXPOSE 80

# ヘルスチェック
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:80/ || exit 1

CMD ["nginx", "-g", "daemon off;"]