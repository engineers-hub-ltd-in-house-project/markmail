#!/usr/bin/env python3
import requests
import json
import uuid
import sys

# MarkMail API設定（開発環境）
API_BASE_URL = "https://dev.markmail.engineers-hub.ltd/api"

# ログイン情報（コマンドライン引数から取得）
if len(sys.argv) != 3:
    print("使用方法: python create_markmail_form_dev.py <email> <password>")
    sys.exit(1)

EMAIL = sys.argv[1]
PASSWORD = sys.argv[2]

# ログイン
print("MarkMail開発環境にログイン中...")
login_response = requests.post(
    f"{API_BASE_URL}/auth/login",
    json={"email": EMAIL, "password": PASSWORD}
)

if login_response.status_code != 200:
    print(f"ログインエラー: {login_response.status_code}")
    print(login_response.text)
    exit(1)

auth_data = login_response.json()
print(f"レスポンス: {auth_data}")
token = auth_data.get("access_token") or auth_data.get("token")
if not token:
    print("トークンが見つかりません")
    print(f"レスポンスデータ: {auth_data}")
    exit(1)
    
headers = {"Authorization": f"Bearer {token}"}

user_info = auth_data.get("user", {})
print(f"ログイン成功: ユーザーID {user_info.get('id', 'unknown')}")

# フォームデータの準備
form_data = {
    "name": "エンジニアスキルシート",
    "description": "Engineers Hub Co.,Ltd のエンジニアスキル登録フォームです。",
    "markdown_content": "# エンジニアスキルシート\n\nEngineers Hub Co.,Ltd のエンジニアスキル登録フォームです。",
    "status": "published",
    "form_fields": [
        # 基本情報
        {
            "id": str(uuid.uuid4()),
            "name": "last_name",
            "label": "姓",
            "field_type": "text",
            "required": True,
            "display_order": 1
        },
        {
            "id": str(uuid.uuid4()),
            "name": "first_name",
            "label": "名",
            "field_type": "text",
            "required": True,
            "display_order": 2
        },
        {
            "id": str(uuid.uuid4()),
            "name": "email",
            "label": "メールアドレス",
            "field_type": "email",
            "required": True,
            "display_order": 3
        },
        {
            "id": str(uuid.uuid4()),
            "name": "company",
            "label": "会社名",
            "field_type": "text",
            "required": False,
            "display_order": 4
        },
        {
            "id": str(uuid.uuid4()),
            "name": "state",
            "label": "都道府県",
            "field_type": "text",
            "required": False,
            "display_order": 5
        },
        # プログラミング言語スキル
        {
            "id": str(uuid.uuid4()),
            "name": "java",
            "label": "Java",
            "field_type": "select",
            "required": False,
            "display_order": 6,
            "options": [
                {"value": "未経験", "label": "未経験"},
                {"value": "基礎知識", "label": "基礎知識"},
                {"value": "実務経験あり", "label": "実務経験あり"},
                {"value": "実務経験豊富", "label": "実務経験豊富（メイン言語として使用）"},
                {"value": "専門性が高い", "label": "専門性が高い（技術選定・指導可能）"}
            ]
        },
        {
            "id": str(uuid.uuid4()),
            "name": "python",
            "label": "Python",
            "field_type": "select",
            "required": False,
            "display_order": 7,
            "options": [
                {"value": "未経験", "label": "未経験"},
                {"value": "基礎知識", "label": "基礎知識"},
                {"value": "実務経験あり", "label": "実務経験あり"},
                {"value": "実務経験豊富", "label": "実務経験豊富（メイン言語として使用）"},
                {"value": "専門性が高い", "label": "専門性が高い（技術選定・指導可能）"}
            ]
        },
        {
            "id": str(uuid.uuid4()),
            "name": "javascript_typescript",
            "label": "JavaScript/TypeScript",
            "field_type": "select",
            "required": False,
            "display_order": 8,
            "options": [
                {"value": "未経験", "label": "未経験"},
                {"value": "基礎知識", "label": "基礎知識"},
                {"value": "実務経験あり", "label": "実務経験あり"},
                {"value": "実務経験豊富", "label": "実務経験豊富（メイン言語として使用）"},
                {"value": "専門性が高い", "label": "専門性が高い（技術選定・指導可能）"}
            ]
        },
        {
            "id": str(uuid.uuid4()),
            "name": "c_cpp",
            "label": "C/C++",
            "field_type": "select",
            "required": False,
            "display_order": 9,
            "options": [
                {"value": "未経験", "label": "未経験"},
                {"value": "基礎知識", "label": "基礎知識"},
                {"value": "実務経験あり", "label": "実務経験あり"},
                {"value": "実務経験豊富", "label": "実務経験豊富（メイン言語として使用）"},
                {"value": "専門性が高い", "label": "専門性が高い（技術選定・指導可能）"}
            ]
        },
        {
            "id": str(uuid.uuid4()),
            "name": "csharp",
            "label": "C#",
            "field_type": "select",
            "required": False,
            "display_order": 10,
            "options": [
                {"value": "未経験", "label": "未経験"},
                {"value": "基礎知識", "label": "基礎知識"},
                {"value": "実務経験あり", "label": "実務経験あり"},
                {"value": "実務経験豊富", "label": "実務経験豊富（メイン言語として使用）"},
                {"value": "専門性が高い", "label": "専門性が高い（技術選定・指導可能）"}
            ]
        },
        {
            "id": str(uuid.uuid4()),
            "name": "php",
            "label": "PHP",
            "field_type": "select",
            "required": False,
            "display_order": 11,
            "options": [
                {"value": "未経験", "label": "未経験"},
                {"value": "基礎知識", "label": "基礎知識"},
                {"value": "実務経験あり", "label": "実務経験あり"},
                {"value": "実務経験豊富", "label": "実務経験豊富（メイン言語として使用）"},
                {"value": "専門性が高い", "label": "専門性が高い（技術選定・指導可能）"}
            ]
        },
        {
            "id": str(uuid.uuid4()),
            "name": "go",
            "label": "Go",
            "field_type": "select",
            "required": False,
            "display_order": 12,
            "options": [
                {"value": "未経験", "label": "未経験"},
                {"value": "基礎知識", "label": "基礎知識"},
                {"value": "実務経験あり", "label": "実務経験あり"},
                {"value": "実務経験豊富", "label": "実務経験豊富（メイン言語として使用）"},
                {"value": "専門性が高い", "label": "専門性が高い（技術選定・指導可能）"}
            ]
        },
        {
            "id": str(uuid.uuid4()),
            "name": "ruby",
            "label": "Ruby",
            "field_type": "select",
            "required": False,
            "display_order": 13,
            "options": [
                {"value": "未経験", "label": "未経験"},
                {"value": "基礎知識", "label": "基礎知識"},
                {"value": "実務経験あり", "label": "実務経験あり"},
                {"value": "実務経験豊富", "label": "実務経験豊富（メイン言語として使用）"},
                {"value": "専門性が高い", "label": "専門性が高い（技術選定・指導可能）"}
            ]
        },
        {
            "id": str(uuid.uuid4()),
            "name": "swift",
            "label": "Swift",
            "field_type": "select",
            "required": False,
            "display_order": 14,
            "options": [
                {"value": "未経験", "label": "未経験"},
                {"value": "基礎知識", "label": "基礎知識"},
                {"value": "実務経験あり", "label": "実務経験あり"},
                {"value": "実務経験豊富", "label": "実務経験豊富（メイン言語として使用）"},
                {"value": "専門性が高い", "label": "専門性が高い（技術選定・指導可能）"}
            ]
        },
        {
            "id": str(uuid.uuid4()),
            "name": "kotlin",
            "label": "Kotlin",
            "field_type": "select",
            "required": False,
            "display_order": 15,
            "options": [
                {"value": "未経験", "label": "未経験"},
                {"value": "基礎知識", "label": "基礎知識"},
                {"value": "実務経験あり", "label": "実務経験あり"},
                {"value": "実務経験豊富", "label": "実務経験豊富（メイン言語として使用）"},
                {"value": "専門性が高い", "label": "専門性が高い（技術選定・指導可能）"}
            ]
        },
        {
            "id": str(uuid.uuid4()),
            "name": "other_languages",
            "label": "その他のプログラミング言語（カンマ区切りで記入）",
            "field_type": "text",
            "required": False,
            "display_order": 16
        },
        # フレームワーク・技術スタック
        {
            "id": str(uuid.uuid4()),
            "name": "react",
            "label": "React",
            "field_type": "checkbox",
            "required": False,
            "display_order": 17
        },
        {
            "id": str(uuid.uuid4()),
            "name": "nextjs",
            "label": "Next.js",
            "field_type": "checkbox",
            "required": False,
            "display_order": 18
        },
        {
            "id": str(uuid.uuid4()),
            "name": "django",
            "label": "Django",
            "field_type": "checkbox",
            "required": False,
            "display_order": 19
        },
        {
            "id": str(uuid.uuid4()),
            "name": "ruby_on_rails",
            "label": "Ruby on Rails",
            "field_type": "checkbox",
            "required": False,
            "display_order": 20
        },
        {
            "id": str(uuid.uuid4()),
            "name": "react_native",
            "label": "React Native",
            "field_type": "checkbox",
            "required": False,
            "display_order": 21
        },
        {
            "id": str(uuid.uuid4()),
            "name": "postgresql",
            "label": "PostgreSQL",
            "field_type": "checkbox",
            "required": False,
            "display_order": 22
        },
        {
            "id": str(uuid.uuid4()),
            "name": "sql_server",
            "label": "SQL Server",
            "field_type": "checkbox",
            "required": False,
            "display_order": 23
        },
        {
            "id": str(uuid.uuid4()),
            "name": "kubernetes",
            "label": "Kubernetes",
            "field_type": "checkbox",
            "required": False,
            "display_order": 24
        },
        {
            "id": str(uuid.uuid4()),
            "name": "azure",
            "label": "Azure",
            "field_type": "checkbox",
            "required": False,
            "display_order": 25
        },
        {
            "id": str(uuid.uuid4()),
            "name": "vue_js",
            "label": "Vue.js",
            "field_type": "checkbox",
            "required": False,
            "display_order": 26
        },
        {
            "id": str(uuid.uuid4()),
            "name": "svelte",
            "label": "Svelte",
            "field_type": "checkbox",
            "required": False,
            "display_order": 27
        },
        {
            "id": str(uuid.uuid4()),
            "name": "flask",
            "label": "Flask",
            "field_type": "checkbox",
            "required": False,
            "display_order": 28
        },
        {
            "id": str(uuid.uuid4()),
            "name": "laravel",
            "label": "Laravel",
            "field_type": "checkbox",
            "required": False,
            "display_order": 29
        },
        {
            "id": str(uuid.uuid4()),
            "name": "flutter",
            "label": "Flutter",
            "field_type": "checkbox",
            "required": False,
            "display_order": 30
        },
        {
            "id": str(uuid.uuid4()),
            "name": "mongodb",
            "label": "MongoDB",
            "field_type": "checkbox",
            "required": False,
            "display_order": 31
        },
        {
            "id": str(uuid.uuid4()),
            "name": "redis",
            "label": "Redis",
            "field_type": "checkbox",
            "required": False,
            "display_order": 32
        },
        {
            "id": str(uuid.uuid4()),
            "name": "aws",
            "label": "AWS",
            "field_type": "checkbox",
            "required": False,
            "display_order": 33
        },
        {
            "id": str(uuid.uuid4()),
            "name": "jenkins",
            "label": "Jenkins",
            "field_type": "checkbox",
            "required": False,
            "display_order": 34
        },
        {
            "id": str(uuid.uuid4()),
            "name": "angular",
            "label": "Angular",
            "field_type": "checkbox",
            "required": False,
            "display_order": 35
        },
        {
            "id": str(uuid.uuid4()),
            "name": "spring",
            "label": "Spring",
            "field_type": "checkbox",
            "required": False,
            "display_order": 36
        },
        {
            "id": str(uuid.uuid4()),
            "name": "express",
            "label": "Express",
            "field_type": "checkbox",
            "required": False,
            "display_order": 37
        },
        {
            "id": str(uuid.uuid4()),
            "name": "asp_net",
            "label": "ASP.NET",
            "field_type": "checkbox",
            "required": False,
            "display_order": 38
        },
        {
            "id": str(uuid.uuid4()),
            "name": "mysql",
            "label": "MySQL",
            "field_type": "checkbox",
            "required": False,
            "display_order": 39
        },
        {
            "id": str(uuid.uuid4()),
            "name": "oracle",
            "label": "Oracle",
            "field_type": "checkbox",
            "required": False,
            "display_order": 40
        },
        {
            "id": str(uuid.uuid4()),
            "name": "docker",
            "label": "Docker",
            "field_type": "checkbox",
            "required": False,
            "display_order": 41
        },
        {
            "id": str(uuid.uuid4()),
            "name": "gcp",
            "label": "GCP",
            "field_type": "checkbox",
            "required": False,
            "display_order": 42
        },
        {
            "id": str(uuid.uuid4()),
            "name": "github_actions",
            "label": "GitHub Actions",
            "field_type": "checkbox",
            "required": False,
            "display_order": 43
        },
        {
            "id": str(uuid.uuid4()),
            "name": "other_tech",
            "label": "その他のフレームワーク・技術（カンマ区切りで記入）",
            "field_type": "text",
            "required": False,
            "display_order": 44
        },
        # その他の情報
        {
            "id": str(uuid.uuid4()),
            "name": "experience",
            "label": "業務経験・アピールポイントなど",
            "field_type": "textarea",
            "required": False,
            "display_order": 45,
            "validation": {
                "rows": 5
            }
        },
        {
            "id": str(uuid.uuid4()),
            "name": "github_url",
            "label": "GitHub URL（お持ちの場合）",
            "field_type": "url",
            "required": False,
            "display_order": 46,
            "placeholder": "https://github.com/username"
        },
        {
            "id": str(uuid.uuid4()),
            "name": "portfolio_url",
            "label": "ポートフォリオURL（お持ちの場合）",
            "field_type": "url",
            "required": False,
            "display_order": 47,
            "placeholder": "https://example.com"
        },
        {
            "id": str(uuid.uuid4()),
            "name": "newsletter_opt_in",
            "label": "Engineers Hub Co.,Ltdからのお知らせを受け取る",
            "field_type": "checkbox",
            "required": False,
            "display_order": 48,
            "default_value": True
        }
    ],
    "settings": {
        "submit_button_text": "送信する",
        "success_message": "エンジニアスキルシートの送信が完了しました。ありがとうございます。",
        "header_content": "# エンジニアスキルシート\n\nEngineers Hub Co.,Ltd のエンジニアスキル登録フォームです。\n以下の項目にご記入ください。"
    }
}

# フォームを作成
print("\nフォームを作成中...")
create_response = requests.post(
    f"{API_BASE_URL}/forms",
    headers=headers,
    json=form_data
)

if create_response.status_code != 201:
    print(f"フォーム作成エラー: {create_response.status_code}")
    print(create_response.text)
    exit(1)

created_form = create_response.json()
form_id = created_form["id"]

print(f"\nフォーム作成成功!")
print(f"フォームID: {form_id}")
print(f"フォーム名: {created_form['name']}")
print(f"ステータス: {created_form['status']}")
print(f"\nフォームURL:")
print(f"https://dev.markmail.engineers-hub.ltd/forms/{form_id}/public")
print(f"\n埋め込みコード:")
print(f'<iframe src="https://dev.markmail.engineers-hub.ltd/forms/{form_id}/public" width="100%" height="1200" frameborder="0"></iframe>')