<script lang="ts">
  import { goto } from "$app/navigation";
  import DOMPurify from "dompurify";
  import { marked } from "marked";
  import { onMount } from "svelte";

  type CreateTemplateRequest = {
    name: string;
    subject_template: string;
    markdown_content: string;
    variables?: Record<string, string>;
    is_public?: boolean;
  };

  let name = "";
  let subjectTemplate = "";
  let markdownContent = "";
  let isPublic = false;
  let variablesText = "";

  let previewHtml = "";
  let previewSubject = "";
  let showPreview = false;
  let saving = false;
  let error = "";

  // マークダウンプレビューを更新
  $: updatePreview(markdownContent, subjectTemplate, variablesText);

  async function updatePreview(content: string, subject: string, vars: string) {
    if (!content && !subject) {
      previewHtml = "";
      previewSubject = "";
      return;
    }

    try {
      const variables = parseVariables(vars);

      // マークダウンをHTMLに変換
      let html = await marked(content);
      let processedSubject = subject;

      // 変数を置換
      Object.entries(variables).forEach(([key, value]) => {
        const placeholder = `{{${key}}}`;
        html = html.replace(new RegExp(placeholder, "g"), value);
        processedSubject = processedSubject.replace(
          new RegExp(placeholder, "g"),
          value,
        );
      });

      previewHtml = DOMPurify.sanitize(html);
      previewSubject = processedSubject;
    } catch (err) {
      console.error("Preview error:", err);
      previewHtml = '<p class="text-red-500">プレビューエラー</p>';
    }
  }

  function parseVariables(text: string): Record<string, string> {
    const vars: Record<string, string> = {};
    if (!text.trim()) return vars;

    try {
      const lines = text.split("\n");
      lines.forEach((line) => {
        const trimmed = line.trim();
        if (trimmed && trimmed.includes("=")) {
          const [key, ...valueParts] = trimmed.split("=");
          const value = valueParts.join("=").trim();
          if (key.trim() && value) {
            vars[key.trim()] = value;
          }
        }
      });
    } catch (err) {
      console.error("Variables parsing error:", err);
    }

    return vars;
  }

  async function saveTemplate() {
    if (!name.trim() || !subjectTemplate.trim() || !markdownContent.trim()) {
      error = "テンプレート名、件名、内容は必須です";
      return;
    }

    try {
      saving = true;
      error = "";

      const variables = parseVariables(variablesText);

      const templateData: CreateTemplateRequest = {
        name: name.trim(),
        subject_template: subjectTemplate.trim(),
        markdown_content: markdownContent.trim(),
        variables,
        is_public: isPublic,
      };

      const response = await fetch("/api/templates", {
        method: "POST",
        headers: {
          Authorization: `Bearer ${localStorage.getItem("token")}`,
          "Content-Type": "application/json",
        },
        body: JSON.stringify(templateData),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(
          errorData.message ||
            `テンプレートの作成に失敗しました: ${response.status}`,
        );
      }

      goto("/templates");
    } catch (err) {
      error = err instanceof Error ? err.message : "エラーが発生しました";
      console.error("Template creation error:", err);
    } finally {
      saving = false;
    }
  }

  function insertVariable(variable: string) {
    const placeholder = `{{${variable}}}`;
    markdownContent += placeholder;
    subjectTemplate += placeholder;
  }

  onMount(() => {
    // サンプルデータを設定
    markdownContent =
      "# こんにちは " +
      "{{user_name}}" +
      " さん\n\n" +
      "{{company_name}}" +
      "からのお知らせです。\n\n" +
      "## 重要なお知らせ\n\n" +
      "ご登録いただきありがとうございます。\n\n" +
      "よろしくお願いいたします。";

    variablesText = "user_name=田中太郎\ncompany_name=株式会社MarkMail";
  });
</script>

<svelte:head>
  <title>新しいテンプレート - MarkMail</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-4 py-8">
  <div class="flex items-center justify-between mb-8">
    <h1 class="text-3xl font-bold text-gray-900">新しいテンプレート</h1>
    <div class="flex items-center space-x-4">
      <button
        on:click={() => (showPreview = !showPreview)}
        class="text-gray-600 hover:text-gray-900 px-4 py-2 border border-gray-300 rounded-lg transition-colors"
      >
        {showPreview ? "エディター" : "プレビュー"}
      </button>
      <button
        on:click={() => goto("/templates")}
        class="text-gray-600 hover:text-gray-900 px-4 py-2 border border-gray-300 rounded-lg transition-colors"
      >
        キャンセル
      </button>
      <button
        on:click={saveTemplate}
        disabled={saving}
        class="bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white px-4 py-2 rounded-lg font-medium transition-colors"
      >
        {saving ? "保存中..." : "保存"}
      </button>
    </div>
  </div>

  {#if error}
    <div
      class="mb-6 bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg"
    >
      {error}
    </div>
  {/if}

  <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
    <!-- メインエディター -->
    <div class="lg:col-span-2">
      <div class="bg-white border border-gray-200 rounded-lg p-6">
        <!-- 基本情報 -->
        <div class="space-y-4 mb-6">
          <div>
            <label
              for="name"
              class="block text-sm font-medium text-gray-700 mb-2"
            >
              テンプレート名 *
            </label>
            <input
              id="name"
              type="text"
              bind:value={name}
              placeholder="例: ウェルカムメール"
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <div>
            <label
              for="subject"
              class="block text-sm font-medium text-gray-700 mb-2"
            >
              件名テンプレート *
            </label>
            <input
              id="subject"
              type="text"
              bind:value={subjectTemplate}
              placeholder={"例: " +
                "{{company_name}}" +
                "へようこそ、" +
                "{{user_name}}" +
                "さん！"}
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>

          <div class="flex items-center">
            <input
              id="is_public"
              type="checkbox"
              bind:checked={isPublic}
              class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
            />
            <label for="is_public" class="ml-2 block text-sm text-gray-700">
              公開テンプレートにする
            </label>
          </div>
        </div>

        <!-- エディター/プレビュー -->
        {#if showPreview}
          <div class="space-y-4">
            <h3 class="text-lg font-medium text-gray-900">プレビュー</h3>

            {#if previewSubject}
              <div class="bg-gray-50 p-4 rounded-lg">
                <h4 class="text-sm font-medium text-gray-700 mb-2">件名:</h4>
                <p class="text-lg">{previewSubject}</p>
              </div>
            {/if}

            <div class="bg-gray-50 p-4 rounded-lg">
              <h4 class="text-sm font-medium text-gray-700 mb-2">
                メール内容:
              </h4>
              <div class="prose max-w-none">
                {@html previewHtml}
              </div>
            </div>
          </div>
        {:else}
          <div>
            <label
              for="content"
              class="block text-sm font-medium text-gray-700 mb-2"
            >
              マークダウン内容 *
            </label>
            <textarea
              id="content"
              bind:value={markdownContent}
              placeholder={"# タイトル\n\nこんにちは " +
                "{{user_name}}" +
                " さん\n\n" +
                "{{company_name}}" +
                "からのお知らせです。"}
              rows="20"
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm"
            ></textarea>
          </div>
        {/if}
      </div>
    </div>

    <!-- サイドバー -->
    <div class="space-y-6">
      <!-- 変数設定 -->
      <div class="bg-white border border-gray-200 rounded-lg p-6">
        <h3 class="text-lg font-medium text-gray-900 mb-4">テスト変数</h3>
        <p class="text-sm text-gray-600 mb-4">
          プレビュー用の変数値を設定してください。<br />
          形式: <code>変数名=値</code>
        </p>

        <textarea
          bind:value={variablesText}
          placeholder="user_name=田中太郎&#10;company_name=株式会社例"
          rows="8"
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm"
        ></textarea>
      </div>

      <!-- よく使う変数 -->
      <div class="bg-white border border-gray-200 rounded-lg p-6">
        <h3 class="text-lg font-medium text-gray-900 mb-4">よく使う変数</h3>
        <div class="space-y-2">
          {#each ["user_name", "company_name", "email", "date", "unsubscribe_url"] as variable}
            <button
              on:click={() => insertVariable(variable)}
              class="block w-full text-left px-3 py-2 text-sm bg-gray-50 hover:bg-gray-100 rounded border transition-colors"
            >
              <code>{"{{" + variable + "}}"}</code>
            </button>
          {/each}
        </div>
      </div>

      <!-- マークダウンヘルプ -->
      <div class="bg-white border border-gray-200 rounded-lg p-6">
        <h3 class="text-lg font-medium text-gray-900 mb-4">マークダウン記法</h3>
        <div class="space-y-2 text-sm">
          <div><code># 見出し1</code></div>
          <div><code>## 見出し2</code></div>
          <div><code>**太字**</code></div>
          <div><code>*斜体*</code></div>
          <div><code>[リンク](URL)</code></div>
          <div><code>- リスト項目</code></div>
        </div>
      </div>
    </div>
  </div>
</div>
