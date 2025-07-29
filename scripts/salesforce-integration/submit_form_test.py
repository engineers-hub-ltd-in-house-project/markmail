#!/usr/bin/env python3
import requests
import json
from datetime import datetime

# API設定
API_BASE_URL = "http://localhost:3000/api"
FORM_ID = "92c55a20-85cf-4418-a127-5ccdf39c4c0f"

# テストデータ
test_data = {
    # 基本情報
    "last_name": "田中",
    "first_name": "太郎",
    "email": f"tanaka.taro.{datetime.now().strftime('%Y%m%d%H%M%S')}@example.com",
    "company": "テスト株式会社",
    "state": "東京都",
    
    # プログラミング言語スキル
    "java": "実務経験豊富（メイン言語として使用）",
    "python": "専門性が高い（技術選定・指導可能）",
    "javascript_typescript": "実務経験あり",
    "c_cpp": "基礎知識",
    "csharp": "実務経験あり",
    "php": "未経験",
    "go": "実務経験あり",
    "ruby": "基礎知識",
    "swift": "未経験",
    "kotlin": "基礎知識",
    "other_languages": "Rust, Scala",
    
    # 技術スタック（チェックボックス）
    "react": True,
    "nextjs": True,
    "django": False,
    "ruby_on_rails": False,
    "react_native": True,
    "postgresql": True,
    "sql_server": False,
    "kubernetes": True,
    "azure": False,
    "vue_js": False,
    "svelte": False,
    "flask": True,
    "laravel": False,
    "flutter": False,
    "mongodb": True,
    "redis": True,
    "aws": True,
    "jenkins": False,
    "angular": False,
    "spring": True,
    "express": True,
    "asp_net": False,
    "mysql": True,
    "oracle": False,
    "docker": True,
    "gcp": False,
    "github_actions": True,
    "other_tech": "Terraform, Ansible",
    
    # その他の情報
    "experience": "10年以上のエンジニア経験があります。\nフルスタックエンジニアとして、フロントエンドからバックエンド、インフラまで幅広く対応可能です。\nアーキテクチャ設計やチームリードの経験もあります。",
    "github_url": "https://github.com/tanaka-taro",
    "portfolio_url": "https://tanaka-taro.dev",
    "newsletter_opt_in": True
}

print(f"フォーム送信テスト開始")
print(f"メールアドレス: {test_data['email']}")

# フォーム送信（dataフィールドにラップする）
print("\nフォームを送信中...")
submit_response = requests.post(
    f"{API_BASE_URL}/forms/{FORM_ID}/submit",
    json={"data": test_data}
)

if submit_response.status_code == 201:
    print("✅ フォーム送信成功!")
    submission = submit_response.json()
    print(f"送信ID: {submission['id']}")
    print(f"送信時刻: {submission['submitted_at']}")
    if submission.get('subscriber_id'):
        print(f"購読者ID: {submission['subscriber_id']}")
    
    # サーバーログでSalesforceリード作成を確認
    print("\n⚠️  サーバーログを確認してください:")
    print("- 'Salesforceリードを作成しました' のメッセージ")
    print("- または 'Salesforceリード作成エラー' のエラーメッセージ")
else:
    print(f"❌ フォーム送信エラー: {submit_response.status_code}")
    print(submit_response.text)

print("\n送信データサマリー:")
print(f"- 名前: {test_data['last_name']} {test_data['first_name']}")
print(f"- 会社: {test_data['company']}")
print(f"- 都道府県: {test_data['state']}")
print(f"- 主要スキル: Python（専門性が高い）, Java（実務経験豊富）")
print(f"- 使用技術: React, Next.js, AWS, Docker, Kubernetes など")