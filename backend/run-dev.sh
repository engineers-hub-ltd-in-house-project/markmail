#!/bin/bash

# 開発環境用の環境変数を設定
export DATABASE_URL="postgres://markmail:markmail_password@localhost:5432/markmail_dev"
export JWT_SECRET="test-secret-key-for-development"
export MAIL_PROVIDER="mailhog"
export SMTP_FROM="noreply@markmail.dev"
export SMTP_HOST="localhost"
export SMTP_PORT="1025"
export RUST_LOG="markmail_backend=debug,tower_http=debug"

# 開発モードで実行
cargo run