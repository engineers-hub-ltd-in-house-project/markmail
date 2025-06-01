#!/bin/bash

echo "🚀 MarkMail lefthook セットアップ"
echo "================================"

# lefthook のインストール
echo "📦 lefthook をインストール中..."

if command -v brew &> /dev/null; then
    echo "🍺 Homebrew でインストール中..."
    brew install lefthook
elif command -v go &> /dev/null; then
    echo "🐹 Go でインストール中..."
    go install github.com/evilmartians/lefthook@latest
elif command -v npm &> /dev/null; then
    echo "📦 npm でインストール中..."
    npm install -g @arkweid/lefthook
else
    echo "❌ lefthook のインストールに失敗しました"
    echo ""
    echo "以下のいずれかの方法でインストールしてください："
    echo "• Homebrew: brew install lefthook"
    echo "• Go: go install github.com/evilmartians/lefthook@latest"
    echo "• npm: npm install -g @arkweid/lefthook"
    echo "• 手動: https://github.com/evilmartians/lefthook/releases"
    exit 1
fi

# lefthook の初期化
echo "🪝 lefthook を初期化中..."
lefthook install

# フロントエンドの依存関係をインストール
echo "📦 フロントエンドの依存関係をインストール中..."
cd frontend
npm install
cd ..

# Rust フォーマッターの確認
echo "🦀 Rust フォーマッターを確認中..."
cd backend
if ! rustup component list --installed | grep -q rustfmt; then
    echo "📦 rustfmt をインストール中..."
    rustup component add rustfmt
fi

if ! rustup component list --installed | grep -q clippy; then
    echo "📦 clippy をインストール中..."
    rustup component add clippy
fi
cd ..

echo ""
echo "🎉 lefthook のセットアップが完了しました！"
echo ""
echo "📋 これで以下のタイミングで自動整形が実行されます："
echo "  • git commit 時 → Rust と フロントエンドコードを自動整形"
echo "  • git push 時 → テストを自動実行"
echo ""
echo "📋 手動実行コマンド："
echo "  • 全体フォーマット: npm run format"
echo "  • フロントエンドのみ: npm run format:frontend"
echo "  • バックエンドのみ: npm run format:backend"
echo ""
echo "🚀 これで何も意識せずに git commit するだけで自動整形されます！" 