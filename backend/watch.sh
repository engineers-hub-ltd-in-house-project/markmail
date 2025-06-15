#!/bin/bash
# cargo-watch を使った開発サーバー起動スクリプト

echo "🚀 Starting development server with auto-reload..."
echo "📁 Watching: src/ and .env files"
echo "🔄 Auto-restart on file changes"
echo ""

# .envファイルから環境変数を読み込む
if [ -f ../.env ]; then
    set -a  # 自動的にエクスポート
    source ../.env
    set +a
fi

cargo watch -c -w src -w ../.env -x run