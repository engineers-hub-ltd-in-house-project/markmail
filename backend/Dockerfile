# ビルドステージ
FROM rust:1.75-slim AS builder

# 必要なシステムパッケージをインストール
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# 依存関係のキャッシュのために、まずCargo.tomlとCargo.lockをコピー
COPY Cargo.toml Cargo.lock ./

# ダミーのmain.rsを作成して依存関係をビルド
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# ソースコードをコピーして実際のアプリケーションをビルド
COPY src ./src
RUN touch src/main.rs && \
    cargo build --release

# 実行ステージ
FROM debian:bookworm-slim

# 必要なランタイムライブラリをインストール
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 非rootユーザーを作成
RUN useradd -m -u 1001 appuser

WORKDIR /app

# ビルドしたバイナリをコピー
COPY --from=builder /usr/src/app/target/release/markmail-backend /app/markmail-backend

# 実行権限を付与
RUN chmod +x /app/markmail-backend

# 非rootユーザーに切り替え
USER appuser

# 環境変数の設定
ENV RUST_LOG=info
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080

EXPOSE 8080

# ヘルスチェック
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["/app/markmail-backend"]