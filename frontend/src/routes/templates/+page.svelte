<script lang="ts">
  import { goto } from "$app/navigation";
  import type { Template } from "$lib/types/template";
  import { onMount } from "svelte";
  import { authStore } from "$lib/stores/authStore";
  import { templateApi } from "$lib/services/api";

  let templates: Template[] = [];
  let loading = true;
  let error = "";
  let token: string | null = null;

  // 認証状態を監視
  authStore.subscribe((state) => {
    token = state.token;

    // 非認証状態のリダイレクト
    if (!state.isAuthenticated && typeof window !== "undefined") {
      goto("/auth/login");
    }
  });

  onMount(async () => {
    await loadTemplates();
  });

  async function loadTemplates() {
    try {
      loading = true;
      error = "";

      if (!token) {
        throw new Error("認証されていません。ログインしてください。");
      }

      const response = await templateApi.getTemplates(50, 0);

      if (response.error) {
        throw new Error(response.error);
      }

      templates = response.data?.templates || [];
    } catch (err) {
      error = err instanceof Error ? err.message : "エラーが発生しました";
      console.error("Templates loading error:", err);
    } finally {
      loading = false;
    }
  }

  async function deleteTemplate(id: string) {
    if (!confirm("このテンプレートを削除しますか？")) {
      return;
    }

    try {
      const response = await templateApi.deleteTemplate(id);

      if (response.error) {
        throw new Error(response.error);
      }

      // 成功メッセージを表示
      alert("テンプレートを削除しました");

      // リストを再読み込み
      await loadTemplates();
    } catch (err) {
      alert(err instanceof Error ? err.message : "エラーが発生しました");
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
  <title>テンプレート一覧 - MarkMail</title>
</svelte:head>

<div class="max-w-6xl mx-auto px-4 py-8">
  <div class="flex justify-between items-center mb-8">
    <h1 class="text-3xl font-bold text-gray-900">メールテンプレート</h1>
    <button
      on:click={() => goto("/templates/new")}
      class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg font-medium transition-colors"
    >
      新しいテンプレート
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
        on:click={loadTemplates}
        class="mt-2 text-sm bg-red-100 hover:bg-red-200 px-3 py-1 rounded transition-colors"
      >
        再試行
      </button>
    </div>
  {:else if templates.length === 0}
    <div class="text-center py-12">
      <div class="max-w-sm mx-auto">
        <svg
          class="mx-auto h-12 w-12 text-gray-400"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width={2}
            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
          />
        </svg>
        <h3 class="mt-2 text-sm font-medium text-gray-900">
          テンプレートがありません
        </h3>
        <p class="mt-1 text-sm text-gray-500">
          最初のメールテンプレートを作成しましょう。
        </p>
        <div class="mt-6">
          <button
            on:click={() => goto("/templates/new")}
            class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg font-medium transition-colors"
          >
            テンプレートを作成
          </button>
        </div>
      </div>
    </div>
  {:else}
    <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
      {#each templates as template}
        <div
          class="bg-white border border-gray-200 rounded-lg shadow-sm hover:shadow-md transition-shadow"
        >
          <div class="p-6">
            <div class="flex items-start justify-between">
              <h3 class="text-lg font-medium text-gray-900 truncate pr-2">
                {template.name}
              </h3>
              <div class="flex items-center space-x-1">
                {#if template.is_public}
                  <span
                    class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800"
                  >
                    公開
                  </span>
                {/if}
              </div>
            </div>

            <p class="mt-2 text-sm text-gray-600 line-clamp-2">
              {template.subject_template}
            </p>

            <div class="mt-4 text-xs text-gray-500">
              <p>作成: {formatDate(template.created_at)}</p>
              <p>更新: {formatDate(template.updated_at)}</p>
            </div>

            <div class="mt-6 flex items-center space-x-3">
              <button
                on:click={() => goto(`/templates/${template.id}`)}
                class="text-blue-600 hover:text-blue-900 text-sm font-medium"
              >
                表示
              </button>
              <button
                on:click={() => goto(`/templates/${template.id}/edit`)}
                class="text-gray-600 hover:text-gray-900 text-sm font-medium"
              >
                編集
              </button>
              <button
                on:click={() => deleteTemplate(template.id)}
                class="text-red-600 hover:text-red-900 text-sm font-medium"
              >
                削除
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
