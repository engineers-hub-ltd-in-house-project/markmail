<script lang="ts">
  import { goto } from "$app/navigation";
  import DOMPurify from "dompurify";
  import { marked } from "marked";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import type { Template, UpdateTemplateRequest } from "$lib/types/template";
  import { authStore } from "$lib/stores/authStore";
  import { templateApi, markdownApi } from "$lib/services/api";

  const templateId = $page.params.id;

  let name = "";
  let subjectTemplate = "";
  let markdownContent = "";
  let isPublic = false;
  let variablesText = "";

  let previewHtml = "";
  let previewSubject = "";
  let showPreview = false;
  let saving = false;
  let loading = true;
  let error = "";

  // 認証状態を監視
  authStore.subscribe((state) => {
    // 非認証状態のリダイレクト
    if (!state.isAuthenticated && typeof window !== "undefined") {
      goto("/auth/login");
    }
  });

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

      // サーバーサイドでマークダウンをレンダリング
      if (content) {
        const renderResponse = await markdownApi.renderMarkdown(
          content,
          variables,
        );
        if (!renderResponse.error && renderResponse.data) {
          previewHtml = DOMPurify.sanitize(renderResponse.data.html);
        } else {
          // ローカル変換をフォールバックとして使用
          let html = await marked(content);

          // 変数を置換
          Object.entries(variables).forEach(([key, value]) => {
            const placeholder = `{{${key}}}`;
            html = html.replace(new RegExp(placeholder, "g"), value);
          });

          previewHtml = DOMPurify.sanitize(html);
        }
      }

      // 件名プレビュー
      if (subject) {
        let processedSubject = subject;

        // 変数を置換
        Object.entries(variables).forEach(([key, value]) => {
          const placeholder = `{{${key}}}`;
          processedSubject = processedSubject.replace(
            new RegExp(placeholder, "g"),
            value,
          );
        });

        previewSubject = processedSubject;
      }
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

  function variablesToText(variables: Record<string, string> = {}): string {
    return Object.entries(variables)
      .map(([key, value]) => `${key}=${value}`)
      .join("\n");
  }

  async function loadTemplate(id: string) {
    try {
      loading = true;
      error = "";

      const templateResponse = await templateApi.getTemplate(id);

      if (templateResponse.error) {
        throw new Error(templateResponse.error);
      }

      const template = templateResponse.data as Template;

      // フォームにデータを設定
      name = template.name;
      subjectTemplate = template.subject_template;
      markdownContent = template.markdown_content;
      isPublic = template.is_public;
      variablesText = variablesToText(template.variables);
    } catch (err) {
      error = err instanceof Error ? err.message : "エラーが発生しました";
      console.error("Template loading error:", err);
    } finally {
      loading = false;
    }
  }

  async function updateTemplate() {
    if (!name.trim() || !subjectTemplate.trim() || !markdownContent.trim()) {
      error = "テンプレート名、件名、内容は必須です";
      return;
    }

    try {
      saving = true;
      error = "";

      const variables = parseVariables(variablesText);

      const templateData: UpdateTemplateRequest = {
        name: name.trim(),
        subject_template: subjectTemplate.trim(),
        markdown_content: markdownContent.trim(),
        variables,
        is_public: isPublic,
      };

      const updateResponse = await templateApi.updateTemplate(
        templateId,
        templateData,
      );

      if (updateResponse.error) {
        throw new Error(updateResponse.error);
      }

      goto(`/templates/${templateId}`);
    } catch (err) {
      error = err instanceof Error ? err.message : "エラーが発生しました";
      console.error("Template update error:", err);
    } finally {
      saving = false;
    }
  }

  function insertVariable(variable: string) {
    const placeholder = `{{${variable}}}`;
    markdownContent += placeholder;
    subjectTemplate += placeholder;
  }

  onMount(async () => {
    await loadTemplate(templateId);
  });
</script>

<svelte:head>
  <title>テンプレート編集 - MarkMail</title>
</svelte:head>

<div class="max-w-7xl mx-auto px-4 py-8">
  <div class="flex items-center justify-between mb-8">
    <h1 class="text-3xl font-bold text-gray-900">テンプレート編集</h1>
    <div class="flex items-center space-x-4">
      <button
        on:click={() => (showPreview = !showPreview)}
        class="text-gray-600 hover:text-gray-900 px-4 py-2 border border-gray-300 rounded-lg transition-colors"
      >
        {showPreview ? "エディター" : "プレビュー"}
      </button>
      <button
        on:click={() => goto(`/templates/${templateId}`)}
        class="text-gray-600 hover:text-gray-900 px-4 py-2 border border-gray-300 rounded-lg transition-colors"
      >
        キャンセル
      </button>
      <button
        on:click={updateTemplate}
        disabled={saving}
        class="bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white px-4 py-2 rounded-lg font-medium transition-colors"
      >
        {saving ? "保存中..." : "更新"}
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

  {#if loading}
    <div class="text-center py-12">
      <div
        class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"
      ></div>
      <p class="mt-4 text-gray-600">読み込み中...</p>
    </div>
  {:else}
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
          <h3 class="text-lg font-medium text-gray-900 mb-4">
            マークダウン記法
          </h3>
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
  {/if}
</div>
