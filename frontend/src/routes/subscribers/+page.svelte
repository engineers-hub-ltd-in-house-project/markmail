<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { subscriberService } from "$lib/services/subscriberService";
  import type {
    SubscriberListResponse,
    Subscriber,
  } from "$lib/types/subscriber";
  import { SubscriberStatus } from "$lib/types/subscriber";

  // 状態変数
  let subscribers: Subscriber[] = [];
  let total = 0;
  let isLoading = true;
  let error: string | null = null;
  let searchInput = "";
  let selectedTag = "";
  let selectedStatus = "";
  let tags: string[] = [];

  // ページング
  let limit = 10;
  let currentPage = 1;
  let totalPages = 0;

  // 並べ替え
  let sortBy = "created_at";
  let sortOrder: "ASC" | "DESC" = "DESC";

  // フィルタリングされた購読者リスト
  $: {
    totalPages = Math.ceil(total / limit);
  }

  // ステータスに応じたバッジの色を返す
  function getStatusBadgeClass(status: SubscriberStatus): string {
    switch (status) {
      case SubscriberStatus.ACTIVE:
        return "bg-green-100 text-green-800";
      case SubscriberStatus.UNSUBSCRIBED:
        return "bg-gray-100 text-gray-800";
      case SubscriberStatus.BOUNCED:
        return "bg-yellow-100 text-yellow-800";
      case SubscriberStatus.COMPLAINED:
        return "bg-red-100 text-red-800";
      default:
        return "bg-gray-100 text-gray-800";
    }
  }

  // ステータスの日本語表示
  function getStatusText(status: SubscriberStatus): string {
    switch (status) {
      case SubscriberStatus.ACTIVE:
        return "有効";
      case SubscriberStatus.UNSUBSCRIBED:
        return "購読解除";
      case SubscriberStatus.BOUNCED:
        return "バウンス";
      case SubscriberStatus.COMPLAINED:
        return "スパム報告";
      default:
        return status;
    }
  }

  // 日付フォーマット
  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString("ja-JP", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  // 購読者一覧を取得
  async function loadSubscribers() {
    isLoading = true;
    error = null;

    try {
      // オフセットを計算
      const offset = (currentPage - 1) * limit;

      const response: SubscriberListResponse =
        await subscriberService.listSubscribers(
          limit,
          offset,
          searchInput,
          selectedTag,
          selectedStatus,
          sortBy,
          sortOrder,
        );

      subscribers = response.subscribers;
      total = response.total;
    } catch (err) {
      console.error("購読者リスト取得エラー:", err);
      error =
        err instanceof Error ? err.message : "購読者リストの取得に失敗しました";
      subscribers = [];
      total = 0;
    } finally {
      isLoading = false;
    }
  }

  // 購読者タグを取得
  async function loadTags() {
    try {
      tags = await subscriberService.getSubscriberTags();
    } catch (err) {
      console.error("タグ取得エラー:", err);
      tags = [];
    }
  }

  // 購読者を削除
  async function deleteSubscriber(id: string) {
    if (!confirm("この購読者を削除してもよろしいですか？")) {
      return;
    }

    try {
      await subscriberService.deleteSubscriber(id);
      // 再読み込み
      loadSubscribers();
    } catch (err) {
      console.error("購読者削除エラー:", err);
      error = err instanceof Error ? err.message : "購読者の削除に失敗しました";
    }
  }

  // 検索ハンドラー
  function handleSearch() {
    currentPage = 1;
    loadSubscribers();
  }

  // ソート変更
  function changeSort(column: string) {
    if (sortBy === column) {
      // 同じカラムの場合は昇順・降順を切り替え
      sortOrder = sortOrder === "ASC" ? "DESC" : "ASC";
    } else {
      sortBy = column;
      sortOrder = "ASC";
    }
    loadSubscribers();
  }

  // ページ遷移
  function goToPage(page: number) {
    if (page < 1 || page > totalPages) return;
    currentPage = page;
    loadSubscribers();
  }

  // 初期化
  onMount(async () => {
    await Promise.all([loadSubscribers(), loadTags()]);
  });
</script>

<svelte:head>
  <title>購読者管理 | MarkMail</title>
</svelte:head>

<div class="container mx-auto px-4 py-6">
  <div class="flex justify-between items-center mb-6">
    <h1 class="text-2xl font-bold">購読者管理</h1>
    <div class="space-x-2">
      <button
        class="bg-indigo-600 text-white px-4 py-2 rounded hover:bg-indigo-700"
        on:click={() => goto("/subscribers/new")}
      >
        新規購読者
      </button>
      <button
        class="bg-indigo-600 text-white px-4 py-2 rounded hover:bg-indigo-700"
        on:click={() => goto("/subscribers/import")}
      >
        CSVインポート
      </button>
    </div>
  </div>

  <!-- フィルターと検索 -->
  <div class="bg-white shadow rounded-lg p-4 mb-6">
    <div class="flex flex-col md:flex-row gap-4">
      <div class="flex-1">
        <label for="search" class="block text-sm font-medium text-gray-700 mb-1"
          >検索</label
        >
        <div class="relative">
          <input
            id="search"
            type="text"
            bind:value={searchInput}
            placeholder="メールアドレスまたは名前で検索..."
            class="w-full p-2 border border-gray-300 rounded-md"
            on:keypress={(e) => e.key === "Enter" && handleSearch()}
          />
          <button
            class="absolute right-2 top-2 text-gray-400 hover:text-indigo-700"
            on:click={handleSearch}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="h-5 w-5"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fill-rule="evenodd"
                d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
                clip-rule="evenodd"
              />
            </svg>
          </button>
        </div>
      </div>

      <div>
        <label for="tag" class="block text-sm font-medium text-gray-700 mb-1"
          >タグ</label
        >
        <select
          id="tag"
          bind:value={selectedTag}
          class="w-full p-2 border border-gray-300 rounded-md"
          on:change={() => {
            currentPage = 1;
            loadSubscribers();
          }}
        >
          <option value="">すべてのタグ</option>
          {#each tags as tag}
            <option value={tag}>{tag}</option>
          {/each}
        </select>
      </div>

      <div>
        <label for="status" class="block text-sm font-medium text-gray-700 mb-1"
          >ステータス</label
        >
        <select
          id="status"
          bind:value={selectedStatus}
          class="w-full p-2 border border-gray-300 rounded-md"
          on:change={() => {
            currentPage = 1;
            loadSubscribers();
          }}
        >
          <option value="">すべてのステータス</option>
          <option value={SubscriberStatus.ACTIVE}>有効</option>
          <option value={SubscriberStatus.UNSUBSCRIBED}>購読解除</option>
          <option value={SubscriberStatus.BOUNCED}>バウンス</option>
          <option value={SubscriberStatus.COMPLAINED}>スパム報告</option>
        </select>
      </div>
    </div>
  </div>

  <!-- エラーメッセージ -->
  {#if error}
    <div
      class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4"
    >
      <p>{error}</p>
    </div>
  {/if}

  <!-- 購読者リスト -->
  <div class="bg-white shadow rounded-lg overflow-hidden">
    {#if isLoading}
      <div class="p-6 text-center">
        <div
          class="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600 mx-auto"
        ></div>
        <p class="mt-2 text-gray-600">読み込み中...</p>
      </div>
    {:else if subscribers.length === 0}
      <div class="p-6 text-center">
        <p class="text-gray-600">購読者がありません</p>
      </div>
    {:else}
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gray-50">
            <tr>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer"
                on:click={() => changeSort("email")}
              >
                メールアドレス
                {#if sortBy === "email"}
                  <span class="ml-1">{sortOrder === "ASC" ? "↑" : "↓"}</span>
                {/if}
              </th>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer"
                on:click={() => changeSort("name")}
              >
                名前
                {#if sortBy === "name"}
                  <span class="ml-1">{sortOrder === "ASC" ? "↑" : "↓"}</span>
                {/if}
              </th>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer"
                on:click={() => changeSort("status")}
              >
                ステータス
                {#if sortBy === "status"}
                  <span class="ml-1">{sortOrder === "ASC" ? "↑" : "↓"}</span>
                {/if}
              </th>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer"
                on:click={() => changeSort("subscribed_at")}
              >
                購読日
                {#if sortBy === "subscribed_at"}
                  <span class="ml-1">{sortOrder === "ASC" ? "↑" : "↓"}</span>
                {/if}
              </th>
              <th
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                タグ
              </th>
              <th
                class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                操作
              </th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-200">
            {#each subscribers as subscriber}
              <tr class="hover:bg-gray-50">
                <td class="px-6 py-4 whitespace-nowrap">
                  <a
                    href={`/subscribers/${subscriber.id}`}
                    class="text-indigo-600 hover:text-indigo-900 font-medium"
                  >
                    {subscriber.email}
                  </a>
                </td>
                <td class="px-6 py-4 whitespace-nowrap"
                  >{subscriber.name || "-"}</td
                >
                <td class="px-6 py-4 whitespace-nowrap">
                  <span
                    class={`px-2 py-1 text-xs rounded-full ${getStatusBadgeClass(subscriber.status)}`}
                  >
                    {getStatusText(subscriber.status)}
                  </span>
                </td>
                <td class="px-6 py-4 whitespace-nowrap"
                  >{formatDate(subscriber.subscribed_at)}</td
                >
                <td class="px-6 py-4">
                  <div class="flex flex-wrap gap-1">
                    {#each subscriber.tags as tag}
                      <span
                        class="bg-indigo-100 text-indigo-800 text-xs px-2 py-1 rounded-full"
                      >
                        {tag}
                      </span>
                    {/each}
                  </div>
                </td>
                <td
                  class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium"
                >
                  <a
                    href={`/subscribers/${subscriber.id}/edit`}
                    class="text-indigo-600 hover:text-indigo-900 mr-4"
                  >
                    編集
                  </a>
                  <button
                    class="text-red-600 hover:text-red-900"
                    on:click={() => deleteSubscriber(subscriber.id)}
                  >
                    削除
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <!-- ページネーション -->
      {#if totalPages > 1}
        <div
          class="bg-white px-4 py-3 flex items-center justify-between border-t border-gray-200 sm:px-6"
        >
          <div class="sm:flex-1 sm:flex sm:items-center sm:justify-between">
            <div>
              <p class="text-sm text-gray-700">
                全 <span class="font-medium">{total}</span> 件中
                <span class="font-medium">{(currentPage - 1) * limit + 1}</span>
                -
                <span class="font-medium"
                  >{Math.min(currentPage * limit, total)}</span
                > 件を表示
              </p>
            </div>
            <div class="mt-3 sm:mt-0">
              <nav
                class="relative z-0 inline-flex rounded-md shadow-sm -space-x-px"
                aria-label="Pagination"
              >
                <button
                  class="relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                  disabled={currentPage === 1}
                  on:click={() => goToPage(currentPage - 1)}
                >
                  <span class="sr-only">前へ</span>
                  <svg
                    class="h-5 w-5"
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z"
                      clip-rule="evenodd"
                    />
                  </svg>
                </button>

                {#each Array(Math.min(5, totalPages)) as _, index}
                  {#if totalPages <= 5}
                    <button
                      class={`relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium ${
                        currentPage === index + 1
                          ? "text-indigo-600 bg-indigo-50"
                          : "text-gray-700 hover:bg-gray-50"
                      }`}
                      on:click={() => goToPage(index + 1)}
                    >
                      {index + 1}
                    </button>
                  {:else}
                    <!-- 5ページ以上ある場合の表示 -->
                    {#if currentPage <= 3}
                      <!-- 前半表示 -->
                      {#if index + 1 <= 5}
                        <button
                          class={`relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium ${
                            currentPage === index + 1
                              ? "text-indigo-600 bg-indigo-50"
                              : "text-gray-700 hover:bg-gray-50"
                          }`}
                          on:click={() => goToPage(index + 1)}
                        >
                          {index + 1}
                        </button>
                      {/if}
                    {:else if currentPage >= totalPages - 2}
                      <!-- 後半表示 -->
                      {#if index + 1 > 0}
                        <button
                          class={`relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium ${
                            currentPage === totalPages - 4 + index + 1
                              ? "text-indigo-600 bg-indigo-50"
                              : "text-gray-700 hover:bg-gray-50"
                          }`}
                          on:click={() => goToPage(totalPages - 4 + index + 1)}
                        >
                          {totalPages - 4 + index + 1}
                        </button>
                      {/if}
                    {:else}
                      <!-- 中間表示 -->
                      {#if index === 0}
                        <button
                          class="relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium text-gray-700 hover:bg-gray-50"
                          on:click={() => goToPage(1)}
                        >
                          1
                        </button>
                      {:else if index === 1}
                        {#if currentPage > 3}
                          <span
                            class="relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium text-gray-700"
                          >
                            ...
                          </span>
                        {/if}
                      {:else if index === 2}
                        <button
                          class="relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium text-indigo-600 bg-indigo-50"
                          on:click={() => goToPage(currentPage)}
                        >
                          {currentPage}
                        </button>
                      {:else if index === 3}
                        {#if currentPage < totalPages - 2}
                          <span
                            class="relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium text-gray-700"
                          >
                            ...
                          </span>
                        {/if}
                      {:else if index === 4}
                        <button
                          class="relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium text-gray-700 hover:bg-gray-50"
                          on:click={() => goToPage(totalPages)}
                        >
                          {totalPages}
                        </button>
                      {/if}
                    {/if}
                  {/if}
                {/each}

                <button
                  class="relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                  disabled={currentPage === totalPages}
                  on:click={() => goToPage(currentPage + 1)}
                >
                  <span class="sr-only">次へ</span>
                  <svg
                    class="h-5 w-5"
                    xmlns="http://www.w3.org/2000/svg"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
                      clip-rule="evenodd"
                    />
                  </svg>
                </button>
              </nav>
            </div>
          </div>
        </div>
      {/if}
    {/if}
  </div>
</div>
