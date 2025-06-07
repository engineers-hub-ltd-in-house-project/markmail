<script lang="ts">
  import { goto } from "$app/navigation";
  import DOMPurify from "dompurify";
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import type { Template, TemplatePreviewResponse } from "$lib/types/template";
  import { authStore } from "$lib/stores/authStore";
  import { templateApi } from "$lib/services/api";

  const templateId = $page.params.id;

  let template: Template | null = null;
  let loading = true;
  let error = "";
  let previewHtml = "";
  let previewSubject = "";
  let showHtml = true;

  // 認証状態を監視
  authStore.subscribe((state) => {
    // 非認証状態のリダイレクト
    if (!state.isAuthenticated && typeof window !== "undefined") {
      goto("/auth/login");
    }
  });

  onMount(async () => {
    await loadTemplate(templateId);
  });

  async function loadTemplate(id: string) {
    try {
      loading = true;
      error = "";

      // テンプレート情報取得
      const templateResponse = await templateApi.getTemplate(id);

      if (templateResponse.error) {
        throw new Error(templateResponse.error);
      }

      template = templateResponse.data as Template;

      // プレビュー取得
      await generatePreview(template);
    } catch (err) {
      error = err instanceof Error ? err.message : "エラーが発生しました";
      console.error("Template loading error:", err);
    } finally {
      loading = false;
    }
  }

  async function generatePreview(template: Template) {
    try {
      const previewResponse = await templateApi.previewTemplate(template.id, {
        variables: template.variables || {},
      });

      if (previewResponse.error) {
        throw new Error(previewResponse.error);
      }

      const previewData = previewResponse.data as TemplatePreviewResponse;
      previewHtml = DOMPurify.sanitize(previewData.html);
      previewSubject = previewData.subject;
    } catch (err) {
      console.error("Preview error:", err);
    }
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString("ja-JP", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }
</script>

<svelte:head>
  <title
    >{template
      ? `${template.name} - MarkMail`
      : "テンプレート詳細 - MarkMail"}</title
  >
</svelte:head>

<div class="max-w-7xl mx-auto px-4 py-8">
  <div class="flex items-center justify-between mb-8">
    <button
      on:click={() => goto("/templates")}
      class="text-blue-600 hover:text-blue-900 font-medium flex items-center"
    >
      <svg
        class="w-5 h-5 mr-2"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M10 19l-7-7m0 0l7-7m-7 7h18"
        />
      </svg>
      テンプレート一覧に戻る
    </button>
  </div>

  {#if loading}
    <div class="text-center py-12">
      <div
        class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"
      ></div>
      <p class="mt-4 text-gray-600">読み込み中...</p>
    </div>
  {:else if error}
    <div
      class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg"
    >
      <p>{error}</p>
      <button
        on:click={() => loadTemplate(templateId)}
        class="mt-2 text-sm bg-red-100 hover:bg-red-200 px-3 py-1 rounded transition-colors"
      >
        再試行
      </button>
    </div>
  {:else if template}
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
      <!-- メイン表示エリア -->
      <div class="lg:col-span-2">
        <div class="bg-white border border-gray-200 rounded-lg p-6 mb-6">
          <div class="flex justify-between items-center mb-6">
            <h1 class="text-2xl font-bold text-gray-900">{template.name}</h1>
            <div class="flex items-center space-x-2">
              {#if template.is_public}
                <span
                  class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800"
                >
                  公開
                </span>
              {/if}
              <button
                on:click={() => goto(`/templates/${template.id}/edit`)}
                class="bg-blue-600 hover:bg-blue-700 text-white px-3 py-1 rounded text-sm transition-colors"
              >
                編集
              </button>
            </div>
          </div>

          <!-- 件名 -->
          <div class="mb-6">
            <h3 class="text-sm font-medium text-gray-700 mb-2">
              件名テンプレート:
            </h3>
            <div class="bg-gray-50 p-3 rounded-lg text-gray-800">
              {template.subject_template}
            </div>
          </div>

          <!-- 表示切り替えボタン -->
          <div class="flex mb-4">
            <button
              on:click={() => (showHtml = true)}
              class={`px-4 py-2 text-sm font-medium rounded-l-lg ${
                showHtml
                  ? "bg-blue-600 text-white"
                  : "bg-gray-100 text-gray-700 hover:bg-gray-200"
              }`}
            >
              プレビュー
            </button>
            <button
              on:click={() => (showHtml = false)}
              class={`px-4 py-2 text-sm font-medium rounded-r-lg ${
                !showHtml
                  ? "bg-blue-600 text-white"
                  : "bg-gray-100 text-gray-700 hover:bg-gray-200"
              }`}
            >
              マークダウン
            </button>
          </div>

          <!-- コンテンツ表示 -->
          {#if showHtml}
            <div class="prose max-w-none border border-gray-200 rounded-lg p-4">
              {@html previewHtml}
            </div>
          {:else}
            <div
              class="font-mono text-sm bg-gray-50 p-4 rounded-lg whitespace-pre-wrap overflow-auto border border-gray-200"
            >
              {template.markdown_content}
            </div>
          {/if}
        </div>

        <div class="flex items-center justify-between">
          <div class="text-sm text-gray-500">
            <p>作成: {formatDate(template.created_at)}</p>
            <p>更新: {formatDate(template.updated_at)}</p>
          </div>

          <div class="flex space-x-2">
            <button
              on:click={() => goto(`/templates/${template.id}/edit`)}
              class="text-blue-600 hover:bg-blue-50 border border-blue-600 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            >
              このテンプレートを編集
            </button>
            <button
              on:click={() => goto("/campaigns/new?template=" + template.id)}
              class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors"
            >
              キャンペーンを作成
            </button>
          </div>
        </div>
      </div>

      <!-- サイドバー -->
      <div class="space-y-6">
        <!-- 変数リスト -->
        <div class="bg-white border border-gray-200 rounded-lg p-6">
          <h3 class="text-lg font-medium text-gray-900 mb-4">
            テンプレート変数
          </h3>

          {#if Object.keys(template.variables || {}).length === 0}
            <p class="text-sm text-gray-500">変数は設定されていません</p>
          {:else}
            <div class="space-y-2">
              {#each Object.entries(template.variables || {}) as [key, value]}
                <div class="text-sm">
                  <span class="font-mono text-blue-700">{`{{${key}}}`}</span>
                  <span class="text-gray-500 ml-2">= {value}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- プレビュー件名 -->
        <div class="bg-white border border-gray-200 rounded-lg p-6">
          <h3 class="text-lg font-medium text-gray-900 mb-4">プレビュー件名</h3>
          <div class="text-sm text-gray-900 p-3 bg-gray-50 rounded-lg">
            {previewSubject || "プレビューできません"}
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
  {:else}
    <div class="text-center py-12">
      <p class="text-lg text-gray-700">テンプレートが見つかりませんでした</p>
    </div>
  {/if}
</div>
