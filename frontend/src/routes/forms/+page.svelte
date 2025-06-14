<script lang="ts">
  import { onMount } from "svelte";
  import { Plus, Edit3, Trash2, Eye, Copy, BarChart } from "lucide-svelte";
  import { formService } from "$lib/services/formService";
  import type { Form } from "$lib/types/form";

  let forms: Form[] = [];
  let loading = true;
  let error: string | null = null;

  onMount(async () => {
    await loadForms();
  });

  async function loadForms() {
    try {
      loading = true;
      const result = await formService.getAll();
      console.log("Loaded forms:", result);
      forms = result || [];
    } catch (err) {
      error = "フォームの読み込みに失敗しました";
      console.error("Error loading forms:", err);
    } finally {
      loading = false;
    }
  }

  async function deleteForm(form: Form) {
    if (!confirm(`フォーム「${form.name}」を削除してもよろしいですか？`)) {
      return;
    }

    try {
      await formService.delete(form.id);
      await loadForms();
    } catch (err) {
      alert("フォームの削除に失敗しました");
      console.error(err);
    }
  }

  function copyEmbedCode(form: Form) {
    const embedCode = `<iframe src="${window.location.origin}/forms/${form.id}/public" width="100%" height="600" frameborder="0"></iframe>`;
    navigator.clipboard.writeText(embedCode);
    alert("埋め込みコードをコピーしました");
  }

  // getStatusColor関数は削除（badgeクラスを直接使用するため）

  function getStatusText(status: string) {
    switch (status) {
      case "published":
        return "公開中";
      case "draft":
        return "下書き";
      case "archived":
        return "アーカイブ";
      default:
        return status;
    }
  }
</script>

<div class="section animate-in">
  <div class="container-wide">
    <div class="flex justify-between items-center mb-12">
      <div>
        <h1 class="page-header">フォーム</h1>
        <p class="page-subtitle">購読者を獲得するためのフォームを作成・管理</p>
      </div>
      <a href="/forms/new" class="btn-primary">
        <Plus class="w-5 h-5 mr-2" />
        新規作成
      </a>
    </div>

    {#if loading}
      <div class="flex justify-center items-center h-64">
        <div
          class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary"
        ></div>
      </div>
    {:else if error}
      <div class="card">
        <div class="p-6">
          <div class="text-center">
            <div
              class="inline-flex items-center justify-center w-16 h-16 bg-red-100 rounded-full mb-4"
            >
              <svg
                class="w-8 h-8 text-red-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </div>
            <h3 class="text-lg font-semibold text-gray-900 mb-2">
              エラーが発生しました
            </h3>
            <p class="text-gray-600 font-light">{error}</p>
          </div>
        </div>
      </div>
    {:else if forms.length === 0}
      <div class="card">
        <div class="p-12 text-center">
          <div
            class="inline-flex items-center justify-center w-20 h-20 bg-gray-100 rounded-full mb-6"
          >
            <svg
              class="w-10 h-10 text-gray-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="1.5"
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
            </svg>
          </div>
          <h3 class="text-xl font-semibold text-gray-900 mb-3">
            フォームがありません
          </h3>
          <p class="text-gray-600 font-light mb-8">
            最初のフォームを作成してみましょう
          </p>
          <a href="/forms/new" class="btn-primary">
            <Plus class="w-5 h-5 mr-2" />
            フォームを作成
          </a>
        </div>
      </div>
    {:else}
      <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
        {#each forms as form (form.id)}
          <div
            class="card hover:shadow-lg transition-all duration-300 animate-in"
            style="--animation-delay: {forms.indexOf(form) * 50}ms"
          >
            <div class="p-6">
              <div class="flex justify-between items-start mb-4">
                <h3 class="text-lg font-semibold text-gray-900 truncate flex-1">
                  {form.name}
                </h3>
                <span
                  class={form.status === "published"
                    ? "badge-success"
                    : form.status === "draft"
                      ? "badge-secondary"
                      : "badge-danger"}
                >
                  {getStatusText(form.status)}
                </span>
              </div>

              {#if form.description}
                <p class="text-gray-600 text-sm mb-4 line-clamp-2 font-light">
                  {form.description}
                </p>
              {/if}

              <div
                class="flex items-center text-sm text-gray-500 mb-6 font-light"
              >
                <BarChart class="w-4 h-4 mr-2" />
                <span>{form.submission_count} 件の送信</span>
              </div>

              <div class="flex gap-2">
                <a
                  href="/forms/{form.id}/edit"
                  class="btn-secondary flex-1 text-sm"
                >
                  <Edit3 class="w-4 h-4 mr-1" />
                  編集
                </a>

                {#if form.status === "published"}
                  <a
                    href="/forms/{form.id}/public"
                    target="_blank"
                    class="icon-button"
                    title="プレビュー"
                  >
                    <Eye class="w-4 h-4" />
                  </a>

                  <button
                    on:click={() => copyEmbedCode(form)}
                    class="icon-button"
                    title="埋め込みコードをコピー"
                  >
                    <Copy class="w-4 h-4" />
                  </button>
                {/if}

                <button
                  on:click={() => deleteForm(form)}
                  class="icon-button icon-button-danger"
                  title="削除"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
