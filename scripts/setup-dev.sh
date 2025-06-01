#!/bin/bash

echo "🚀 MarkMail 開発環境セットアップを開始します..."

# 環境変数ファイルの作成
if [ ! -f .env ]; then
    echo "📝 環境変数ファイルを作成中..."
    cp env.example .env
    echo "✅ .env ファイルが作成されました。必要に応じて編集してください。"
else
    echo "ℹ️  .env ファイルは既に存在します。"
fi

# Docker Composeで開発環境を起動
echo "🐳 Docker Compose で開発環境を起動中..."
docker-compose up -d postgres redis mailhog

# PostgreSQLの起動を待機
echo "⏳ PostgreSQLの起動を待機中..."
sleep 10

# Rustの依存関係をチェック
echo "🦀 Rust環境をチェック中..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Rustがインストールされていません。https://rustup.rs/ からインストールしてください。"
    exit 1
fi

# SQLx CLIのインストール
echo "🔧 SQLx CLIをインストール中..."
cargo install sqlx-cli --no-default-features --features postgres

# Node.jsの依存関係をチェック
echo "📦 Node.js環境をチェック中..."
if ! command -v npm &> /dev/null; then
    echo "❌ Node.jsがインストールされていません。https://nodejs.org/ からインストールしてください。"
    exit 1
fi

# フロントエンドの依存関係をインストール
echo "📦 フロントエンドの依存関係をインストール中..."
cd frontend
npm install
cd ..

echo ""
echo "🎉 セットアップが完了しました！"
echo ""
echo "📋 次のステップ:"
echo "1. バックエンドを起動: cd backend && cargo run"
echo "2. フロントエンドを起動: cd frontend && npm run dev"
echo ""
echo "🌐 アクセス先:"
echo "- フロントエンド: http://localhost:5173"
echo "- バックエンドAPI: http://localhost:3000"
echo "- MailHog: http://localhost:8025"
echo ""
echo "🚀 Happy coding with MarkMail!" 