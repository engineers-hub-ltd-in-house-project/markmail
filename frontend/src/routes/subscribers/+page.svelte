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
        return "badge badge-success";
      case SubscriberStatus.UNSUBSCRIBED:
        return "badge";
      case SubscriberStatus.BOUNCED:
        return "badge badge-warning";
      case SubscriberStatus.COMPLAINED:
        return "badge badge-error";
      default:
        return "badge";
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

<div class="section animate-in">
  <div class="container-wide">
    <div class="flex justify-between items-center mb-12">
      <div>
        <h1 class="page-header">購読者管理</h1>
        <p class="page-subtitle">メールリストの管理とセグメント化</p>
      </div>
      <div class="flex space-x-4">
        <button
          class="btn-secondary"
          on:click={() => goto("/subscribers/import")}
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
              d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
            />
          </svg>
          CSVインポート
        </button>
        <button class="btn-primary" on:click={() => goto("/subscribers/new")}>
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
          新規購読者
        </button>
      </div>
    </div>

    <!-- フィルターと検索 -->
    <div class="card mb-8">
      <div class="flex flex-col md:flex-row gap-6">
        <div class="flex-1">
          <label for="search" class="label">検索</label>
          <div class="relative">
            <div
              class="absolute inset-y-0 left-0 flex items-center pl-5 pointer-events-none"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-5 w-5 text-gray-400"
                viewBox="0 0 20 20"
                fill="currentColor"
              >
                <path
                  fill-rule="evenodd"
                  d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z"
                  clip-rule="evenodd"
                />
              </svg>
            </div>
            <input
              id="search"
              type="text"
              bind:value={searchInput}
              placeholder="メールアドレスまたは名前で検索..."
              class="input-field pl-12"
              on:keypress={(e) => e.key === "Enter" && handleSearch()}
            />
          </div>
        </div>

        <div class="md:w-48">
          <label for="tag" class="label">タグ</label>
          <select
            id="tag"
            bind:value={selectedTag}
            class="input-field"
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

        <div class="md:w-48">
          <label for="status" class="label">ステータス</label>
          <select
            id="status"
            bind:value={selectedStatus}
            class="input-field"
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
      <div class="card bg-red-50 border-red-100 mb-6">
        <p class="text-red-600 font-light">{error}</p>
      </div>
    {/if}

    <!-- 購読者リスト -->
    <div class="card overflow-hidden p-0">
      {#if isLoading}
        <div class="p-12 text-center">
          <div
            class="w-12 h-12 border-2 border-gray-900 border-t-transparent rounded-full animate-spin mx-auto"
          ></div>
          <p class="mt-4 text-gray-600 font-light">読み込み中...</p>
        </div>
      {:else if subscribers.length === 0}
        <div class="p-12 text-center">
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
                d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
              />
            </svg>
          </div>
          <h3 class="text-xl font-light text-gray-900 mb-2">
            購読者がありません
          </h3>
          <p class="text-gray-600 font-light mb-8">
            新しい購読者を追加またはCSVファイルをインポートしてください
          </p>
          <div class="flex justify-center space-x-4">
            <button
              class="btn-secondary"
              on:click={() => goto("/subscribers/import")}
            >
              CSVインポート
            </button>
            <button
              class="btn-primary"
              on:click={() => goto("/subscribers/new")}
            >
              新規購読者を追加
            </button>
          </div>
        </div>
      {:else}
        <div class="overflow-x-auto">
          <table class="table-minimal">
            <thead>
              <tr>
                <th
                  class="cursor-pointer hover:text-gray-900"
                  on:click={() => changeSort("email")}
                >
                  メールアドレス
                  {#if sortBy === "email"}
                    <span class="ml-1">{sortOrder === "ASC" ? "↑" : "↓"}</span>
                  {/if}
                </th>
                <th
                  class="cursor-pointer hover:text-gray-900"
                  on:click={() => changeSort("name")}
                >
                  名前
                  {#if sortBy === "name"}
                    <span class="ml-1">{sortOrder === "ASC" ? "↑" : "↓"}</span>
                  {/if}
                </th>
                <th
                  class="cursor-pointer hover:text-gray-900"
                  on:click={() => changeSort("status")}
                >
                  ステータス
                  {#if sortBy === "status"}
                    <span class="ml-1">{sortOrder === "ASC" ? "↑" : "↓"}</span>
                  {/if}
                </th>
                <th
                  class="cursor-pointer hover:text-gray-900"
                  on:click={() => changeSort("subscribed_at")}
                >
                  購読日
                  {#if sortBy === "subscribed_at"}
                    <span class="ml-1">{sortOrder === "ASC" ? "↑" : "↓"}</span>
                  {/if}
                </th>
                <th>タグ</th>
                <th class="text-right">操作</th>
              </tr>
            </thead>
            <tbody>
              {#each subscribers as subscriber}
                <tr>
                  <td>
                    <a
                      href={`/subscribers/${subscriber.id}`}
                      class="text-gray-900 hover:text-black font-light hover:underline"
                    >
                      {subscriber.email}
                    </a>
                  </td>
                  <td class="font-light">{subscriber.name || "-"}</td>
                  <td>
                    <span class={getStatusBadgeClass(subscriber.status)}>
                      {getStatusText(subscriber.status)}
                    </span>
                  </td>
                  <td class="text-sm text-gray-600 font-light">
                    {formatDate(subscriber.subscribed_at)}
                  </td>
                  <td>
                    <div class="flex flex-wrap gap-2">
                      {#each subscriber.tags as tag}
                        <span class="badge badge-info">
                          {tag}
                        </span>
                      {/each}
                    </div>
                  </td>
                  <td class="text-right">
                    <div class="flex items-center justify-end space-x-2">
                      <a
                        href={`/subscribers/${subscriber.id}`}
                        class="icon-button"
                        title="詳細"
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
                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                          />
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                          />
                        </svg>
                      </a>
                      <a
                        href={`/subscribers/${subscriber.id}/edit`}
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
                      </a>
                      <button
                        class="icon-button hover:bg-red-50 hover:text-red-600"
                        on:click={() => deleteSubscriber(subscriber.id)}
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
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>

        <!-- ページネーション -->
        {#if totalPages > 1}
          <div
            class="px-6 py-4 flex items-center justify-between border-t border-gray-100"
          >
            <div class="sm:flex-1 sm:flex sm:items-center sm:justify-between">
              <div>
                <p class="text-sm text-gray-700">
                  全 <span class="font-medium">{total}</span> 件中
                  <span class="font-medium"
                    >{(currentPage - 1) * limit + 1}</span
                  >
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
                            on:click={() =>
                              goToPage(totalPages - 4 + index + 1)}
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
</div>

<style>
  @keyframes fadeInUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .animate-in {
    animation: fadeInUp 0.6s ease-out;
  }
</style>
