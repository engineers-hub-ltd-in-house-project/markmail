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

  function getStatusColor(status: string) {
    switch (status) {
      case "published":
        return "bg-green-100 text-green-800";
      case "draft":
        return "bg-gray-100 text-gray-800";
      case "archived":
        return "bg-red-100 text-red-800";
      default:
        return "bg-gray-100 text-gray-800";
    }
  }

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

<div class="container mx-auto px-4 py-8">
  <div class="mb-8">
    <div class="flex justify-between items-center">
      <div>
        <h1 class="text-3xl font-bold text-gray-800">フォーム</h1>
        <p class="text-gray-600 mt-2">
          購読者を獲得するためのフォームを作成・管理
        </p>
      </div>
      <a
        href="/forms/new"
        class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
      >
        <Plus class="w-5 h-5 mr-2" />
        新規作成
      </a>
    </div>
  </div>

  {#if loading}
    <div class="flex justify-center items-center h-64">
      <div
        class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"
      ></div>
    </div>
  {:else if error}
    <div
      class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg"
    >
      {error}
    </div>
  {:else if forms.length === 0}
    <div class="text-center py-12">
      <div
        class="inline-flex items-center justify-center w-16 h-16 bg-gray-100 rounded-full mb-4"
      >
        <svg
          class="w-8 h-8 text-gray-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
          />
        </svg>
      </div>
      <h3 class="text-lg font-medium text-gray-900 mb-2">
        フォームがありません
      </h3>
      <p class="text-gray-500 mb-6">最初のフォームを作成してみましょう</p>
      <a
        href="/forms/new"
        class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
      >
        <Plus class="w-5 h-5 mr-2" />
        フォームを作成
      </a>
    </div>
  {:else}
    <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
      {#each forms as form}
        <div
          class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden hover:shadow-md transition-shadow"
        >
          <div class="p-6">
            <div class="flex justify-between items-start mb-4">
              <h3 class="text-lg font-semibold text-gray-900 truncate flex-1">
                {form.name}
              </h3>
              <span
                class={`px-2 py-1 text-xs font-medium rounded-full ${getStatusColor(form.status)}`}
              >
                {getStatusText(form.status)}
              </span>
            </div>

            {#if form.description}
              <p class="text-gray-600 text-sm mb-4 line-clamp-2">
                {form.description}
              </p>
            {/if}

            <div class="flex items-center text-sm text-gray-500 mb-4">
              <BarChart class="w-4 h-4 mr-1" />
              <span>{form.submission_count} 件の送信</span>
            </div>

            <div class="flex gap-2">
              <a
                href="/forms/{form.id}/edit"
                class="flex-1 inline-flex items-center justify-center px-3 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors text-sm"
              >
                <Edit3 class="w-4 h-4 mr-1" />
                編集
              </a>

              {#if form.status === "published"}
                <a
                  href="/forms/{form.id}/public"
                  target="_blank"
                  class="inline-flex items-center justify-center px-3 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors text-sm"
                  title="プレビュー"
                >
                  <Eye class="w-4 h-4" />
                </a>

                <button
                  on:click={() => copyEmbedCode(form)}
                  class="inline-flex items-center justify-center px-3 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors text-sm"
                  title="埋め込みコードをコピー"
                >
                  <Copy class="w-4 h-4" />
                </button>
              {/if}

              <button
                on:click={() => deleteForm(form)}
                class="inline-flex items-center justify-center px-3 py-2 bg-red-100 text-red-700 rounded-lg hover:bg-red-200 transition-colors text-sm"
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
