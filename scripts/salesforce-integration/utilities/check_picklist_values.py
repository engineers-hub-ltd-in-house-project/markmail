#!/usr/bin/env python3
import subprocess
import json

# Leadオブジェクトの情報を取得
print("Lead オブジェクトのメタデータを取得中...")
result = subprocess.run([
    "sf", "sobject", "describe", "-s", "Lead", "-o", "markmail-org", "--json"
], capture_output=True, text=True)

if result.returncode != 0:
    print("エラー:", result.stderr)
    exit(1)

data = json.loads(result.stdout)
fields = data['result']['fields']

# プログラミング言語フィールドの選択リスト値を確認
language_fields = [
    "Java__c", "Python__c", "JavaScript_TypeScript__c", 
    "C_C__c", "C__c", "PHP__c", "Go__c", "Ruby__c", 
    "Swift__c", "Kotlin__c"
]

print("\nプログラミング言語フィールドの選択リスト値:")
print("=" * 80)

for field_name in language_fields:
    for field in fields:
        if field['name'] == field_name:
            print(f"\n{field_name}:")
            if 'picklistValues' in field and field['picklistValues']:
                for value in field['picklistValues']:
                    active = "✓" if value.get('active', False) else "✗"
                    print(f"  {active} {value['value']}")
            else:
                print(f"  タイプ: {field['type']}")
                print("  選択リスト値が定義されていません")
            break