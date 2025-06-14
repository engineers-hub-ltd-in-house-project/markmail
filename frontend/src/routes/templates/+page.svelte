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

<div class="section animate-in">
  <div class="container-wide">
    <div class="flex justify-between items-center mb-12">
      <div>
        <h1 class="page-header">メールテンプレート</h1>
        <p class="page-subtitle">Markdownでメールテンプレートを作成・管理</p>
      </div>
      <button on:click={() => goto("/templates/new")} class="btn-primary">
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
            d="M12 6v6m0 0v6m0-6h6m-6 0H6"
          />
        </svg>
        新しいテンプレート
      </button>
    </div>

    {#if loading}
      <div class="text-center py-24">
        <div class="inline-block">
          <div
            class="w-12 h-12 border-2 border-gray-900 border-t-transparent rounded-full animate-spin"
          ></div>
        </div>
        <p class="mt-4 text-gray-600 font-light">読み込み中...</p>
      </div>
    {:else if error}
      <div class="card bg-red-50 border-red-100 text-center py-8">
        <svg
          class="mx-auto h-12 w-12 text-red-400 mb-4"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
        <p class="text-red-600 font-light">{error}</p>
        <button on:click={loadTemplates} class="mt-4 btn-secondary btn-sm">
          再試行
        </button>
      </div>
    {:else if templates.length === 0}
      <div class="text-center py-24">
        <div class="max-w-sm mx-auto">
          <div
            class="w-20 h-20 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-6"
          >
            <svg
              class="w-10 h-10 text-gray-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="1.5"
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
            </svg>
          </div>
          <h3 class="text-xl font-light text-gray-900 mb-2">
            テンプレートがありません
          </h3>
          <p class="text-gray-600 font-light mb-8">
            最初のメールテンプレートを作成しましょう
          </p>
          <button on:click={() => goto("/templates/new")} class="btn-primary">
            テンプレートを作成
          </button>
        </div>
      </div>
    {:else}
      <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {#each templates as template (template.id)}
          <div class="card group">
            <div class="flex items-start justify-between mb-4">
              <h3 class="text-xl font-light text-gray-900 truncate pr-2">
                {template.name}
              </h3>
              {#if template.is_public}
                <span class="badge badge-success"> 公開 </span>
              {/if}
            </div>

            <p class="text-gray-600 font-light line-clamp-2 mb-6">
              {template.subject_template}
            </p>

            <div class="text-xs text-gray-500 font-light mb-6">
              <p>作成: {formatDate(template.created_at)}</p>
              <p>更新: {formatDate(template.updated_at)}</p>
            </div>

            <div
              class="flex items-center justify-between pt-6 border-t border-gray-100"
            >
              <button
                on:click={() => goto(`/templates/${template.id}`)}
                class="text-gray-700 hover:text-black transition-colors font-light text-sm inline-flex items-center"
              >
                <svg
                  class="w-4 h-4 mr-1"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                  />
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                  />
                </svg>
                表示
              </button>
              <div class="flex items-center space-x-3">
                <button
                  on:click={() => goto(`/templates/${template.id}/edit`)}
                  class="icon-button"
                  title="編集"
                >
                  <svg
                    class="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                    />
                  </svg>
                </button>
                <button
                  on:click={() => deleteTemplate(template.id)}
                  class="icon-button hover:bg-red-50 hover:text-red-600"
                  title="削除"
                >
                  <svg
                    class="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                    />
                  </svg>
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
