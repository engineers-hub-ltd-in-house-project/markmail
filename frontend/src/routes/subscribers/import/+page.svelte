<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { subscriberService } from "$lib/services/subscriberService";
  import type { ImportSubscribersResponse } from "$lib/types/subscriber";

  // 状態変数
  let isLoading = false;
  let isImporting = false;
  let error: string | null = null;
  let success: string | null = null;
  let selectedFile: File | null = null;
  let fileInputError: string | null = null;
  let importResult: ImportSubscribersResponse | null = null;
  let availableTags: string[] = [];
  let tagInput = "";

  // ファイル選択ハンドラー
  function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      selectedFile = input.files[0];
      fileInputError = null;

      // CSVファイルかどうかのチェック
      if (!selectedFile.name.endsWith(".csv")) {
        fileInputError = "CSVファイルを選択してください";
        selectedFile = null;
      } else if (selectedFile.size > 10 * 1024 * 1024) {
        // 10MBの上限
        fileInputError = "ファイルサイズは10MB以下にしてください";
        selectedFile = null;
      }
    } else {
      selectedFile = null;
    }
  }

  // CSVインポート
  async function importSubscribers() {
    if (!selectedFile) {
      fileInputError = "CSVファイルを選択してください";
      return;
    }

    isImporting = true;
    error = null;
    success = null;
    importResult = null;

    try {
      importResult = await subscriberService.importSubscribers({
        file: selectedFile,
        tag: tagInput || undefined,
      });

      success = `${importResult.imported}件の購読者を正常にインポートしました`;
      // ファイル入力をリセット
      selectedFile = null;
      const fileInput = document.getElementById("csv-file") as HTMLInputElement;
      if (fileInput) {
        fileInput.value = "";
      }
    } catch (err) {
      console.error("インポートエラー:", err);
      error = err instanceof Error ? err.message : "不明なエラーが発生しました";
    } finally {
      isImporting = false;
    }
  }

  // タグのリストを取得
  async function loadTags() {
    try {
      availableTags = await subscriberService.getSubscriberTags();
    } catch (err) {
      console.error("タグ取得エラー:", err);
      availableTags = [];
    }
  }

  // 初期化
  onMount(async () => {
    isLoading = true;
    await loadTags();
    isLoading = false;
  });
</script>

<svelte:head>
  <title>購読者CSVインポート | MarkMail</title>
</svelte:head>

<div class="container mx-auto px-4 py-6">
  <div class="mb-6">
    <a
      href="/subscribers"
      class="text-indigo-600 hover:text-indigo-800 flex items-center"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="h-5 w-5 mr-1"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z"
          clip-rule="evenodd"
        />
      </svg>
      購読者一覧に戻る
    </a>
  </div>

  <div class="bg-white shadow rounded-lg overflow-hidden">
    <div class="p-6 border-b border-gray-200">
      <h1 class="text-xl font-semibold text-gray-900">購読者CSVインポート</h1>
    </div>

    {#if error}
      <div class="bg-red-100 border-l-4 border-red-500 text-red-700 p-4 m-4">
        <p>{error}</p>
      </div>
    {/if}

    {#if success}
      <div
        class="bg-green-100 border-l-4 border-green-500 text-green-700 p-4 m-4"
      >
        <p>{success}</p>
      </div>
    {/if}

    {#if importResult && importResult.failed > 0}
      <div
        class="bg-yellow-100 border-l-4 border-yellow-500 text-yellow-700 p-4 m-4"
      >
        <p>{importResult.failed}件のインポートに失敗しました</p>
        {#if importResult.errors && importResult.errors.length > 0}
          <ul class="list-disc pl-5 mt-2">
            {#each importResult.errors as error}
              <li>{error}</li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}

    <div class="p-6 space-y-6">
      <!-- CSVフォーマット説明 -->
      <div class="bg-gray-50 p-4 rounded border border-gray-200">
        <h3 class="text-md font-medium text-gray-700 mb-2">CSVファイル形式</h3>
        <p class="text-sm text-gray-600 mb-2">
          1行目はヘッダー行として扱われます。以下の列が必要です：
        </p>
        <div class="overflow-x-auto">
          <table class="min-w-full divide-y divide-gray-200">
            <thead class="bg-gray-100">
              <tr>
                <th
                  class="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  列名
                </th>
                <th
                  class="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  必須
                </th>
                <th
                  class="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  説明
                </th>
              </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200">
              <tr>
                <td
                  class="px-4 py-2 whitespace-nowrap text-sm font-medium text-gray-900"
                  >email</td
                >
                <td class="px-4 py-2 whitespace-nowrap text-sm text-gray-700"
                  >必須</td
                >
                <td class="px-4 py-2 text-sm text-gray-700"
                  >購読者のメールアドレス</td
                >
              </tr>
              <tr>
                <td
                  class="px-4 py-2 whitespace-nowrap text-sm font-medium text-gray-900"
                  >name</td
                >
                <td class="px-4 py-2 whitespace-nowrap text-sm text-gray-700"
                  >任意</td
                >
                <td class="px-4 py-2 text-sm text-gray-700">購読者の名前</td>
              </tr>
              <tr>
                <td
                  class="px-4 py-2 whitespace-nowrap text-sm font-medium text-gray-900"
                  >tags</td
                >
                <td class="px-4 py-2 whitespace-nowrap text-sm text-gray-700"
                  >任意</td
                >
                <td class="px-4 py-2 text-sm text-gray-700">
                  カンマ区切りのタグリスト（例: "tag1,tag2,tag3"）
                </td>
              </tr>
              <tr>
                <td
                  class="px-4 py-2 whitespace-nowrap text-sm font-medium text-gray-900"
                >
                  custom_*
                </td>
                <td class="px-4 py-2 whitespace-nowrap text-sm text-gray-700"
                  >任意</td
                >
                <td class="px-4 py-2 text-sm text-gray-700">
                  カスタムフィールド（例:
                  "custom_company"はcompanyという名前のカスタムフィールドになります）
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="mt-4">
          <h4 class="text-sm font-medium text-gray-700 mb-1">例：</h4>
          <pre
            class="bg-gray-100 p-2 text-xs rounded overflow-x-auto">email,name,tags,custom_company,custom_role
user1@example.com,田中太郎,vip，技術者,株式会社サンプル,エンジニア
user2@example.com,山田花子,新規,株式会社テスト,マネージャー</pre>
        </div>
      </div>

      <form on:submit|preventDefault={importSubscribers} class="space-y-6">
        <!-- ファイル選択 -->
        <div>
          <label
            for="csv-file"
            class="block text-sm font-medium text-gray-700 mb-1"
          >
            CSVファイル<span class="text-red-600">*</span>
          </label>
          <input
            id="csv-file"
            type="file"
            accept=".csv"
            on:change={handleFileSelect}
            class="block w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 file:text-sm file:font-semibold file:bg-indigo-50 file:text-indigo-700 hover:file:bg-indigo-100"
          />
          {#if fileInputError}
            <p class="mt-1 text-sm text-red-600">{fileInputError}</p>
          {/if}
          {#if selectedFile}
            <p class="mt-1 text-sm text-gray-500">
              選択したファイル: {selectedFile.name} ({Math.round(
                selectedFile.size / 1024,
              )} KB)
            </p>
          {/if}
        </div>

        <!-- 共通タグ -->
        <div>
          <label for="tag" class="block text-sm font-medium text-gray-700 mb-1">
            共通タグ（すべての購読者に追加）
          </label>
          <input
            id="tag"
            type="text"
            list="available-tags"
            bind:value={tagInput}
            placeholder="すべての購読者に追加するタグを入力..."
            class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
          />
          <datalist id="available-tags">
            {#each availableTags as tag}
              <option value={tag} />
            {/each}
          </datalist>
        </div>

        <!-- 送信ボタン -->
        <div class="flex justify-end">
          <button
            type="button"
            class="bg-white py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 hover:bg-gray-50 mr-3"
            on:click={() => goto("/subscribers")}
          >
            キャンセル
          </button>
          <button
            type="submit"
            class="bg-indigo-600 py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            disabled={isImporting || !selectedFile}
          >
            {#if isImporting}
              <span class="flex items-center">
                <svg
                  class="animate-spin -ml-1 mr-2 h-4 w-4 text-white"
                  xmlns="http://www.w3.org/2000/svg"
                  fill="none"
                  viewBox="0 0 24 24"
                >
                  <circle
                    class="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    stroke-width="4"
                  ></circle>
                  <path
                    class="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  />
                </svg>
                インポート中...
              </span>
            {:else}
              インポート実行
            {/if}
          </button>
        </div>
      </form>
    </div>
  </div>
</div>
