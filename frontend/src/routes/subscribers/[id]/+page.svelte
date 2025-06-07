<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { subscriberService } from "$lib/services/subscriberService";
  import type { Subscriber } from "$lib/types/subscriber";
  import { SubscriberStatus } from "$lib/types/subscriber";

  // 状態変数
  let subscriber: Subscriber | null = null;
  let isLoading = true;
  let error: string | null = null;

  // 購読者ID
  const subscriberId = $page.params.id;

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
  function formatDate(dateString?: string): string {
    if (!dateString) return "-";
    return new Date(dateString).toLocaleDateString("ja-JP", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  // 購読者詳細を取得
  async function loadSubscriber() {
    isLoading = true;
    error = null;

    try {
      subscriber = await subscriberService.getSubscriber(subscriberId);
    } catch (err) {
      console.error("購読者詳細取得エラー:", err);
      error =
        err instanceof Error ? err.message : "購読者詳細の取得に失敗しました";
    } finally {
      isLoading = false;
    }
  }

  // 購読者を削除
  async function deleteSubscriber() {
    if (!confirm("この購読者を削除してもよろしいですか？")) {
      return;
    }

    try {
      await subscriberService.deleteSubscriber(subscriberId);
      goto("/subscribers");
    } catch (err) {
      console.error("購読者削除エラー:", err);
      error = err instanceof Error ? err.message : "購読者の削除に失敗しました";
    }
  }

  // 初期化
  onMount(() => {
    loadSubscriber();
  });
</script>

<svelte:head>
  <title
    >{subscriber ? `${subscriber.email} | 購読者詳細` : "購読者詳細"} | MarkMail</title
  >
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

  {#if error}
    <div class="bg-red-100 border-l-4 border-red-500 text-red-700 p-4 mb-4">
      <p>{error}</p>
    </div>
  {/if}

  {#if isLoading}
    <div class="bg-white shadow rounded-lg p-6">
      <div class="animate-pulse flex space-x-4">
        <div class="flex-1 space-y-4 py-1">
          <div class="h-4 bg-gray-200 rounded w-3/4"></div>
          <div class="space-y-2">
            <div class="h-4 bg-gray-200 rounded"></div>
            <div class="h-4 bg-gray-200 rounded w-5/6"></div>
          </div>
        </div>
      </div>
    </div>
  {:else if subscriber}
    <div class="bg-white shadow rounded-lg overflow-hidden">
      <!-- ヘッダー -->
      <div
        class="p-6 sm:flex sm:items-center sm:justify-between border-b border-gray-200"
      >
        <div>
          <h1 class="text-xl font-semibold text-gray-900">購読者詳細</h1>
          <p class="mt-1 text-sm text-gray-500">ID: {subscriber.id}</p>
        </div>
        <div class="mt-4 sm:mt-0 flex space-x-3">
          <a
            href={`/subscribers/${subscriber.id}/edit`}
            class="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
          >
            <svg
              class="-ml-1 mr-2 h-5 w-5 text-gray-500"
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z"
              />
            </svg>
            編集
          </a>
          <button
            class="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-red-600 hover:bg-red-700"
            on:click={deleteSubscriber}
          >
            <svg
              class="-ml-1 mr-2 h-5 w-5"
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fill-rule="evenodd"
                d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z"
                clip-rule="evenodd"
              />
            </svg>
            削除
          </button>
        </div>
      </div>

      <!-- 基本情報 -->
      <div class="px-6 py-5 border-b border-gray-200">
        <h2 class="text-lg font-medium text-gray-900 mb-4">基本情報</h2>
        <dl class="grid grid-cols-1 gap-x-4 gap-y-6 sm:grid-cols-2">
          <div>
            <dt class="text-sm font-medium text-gray-500">メールアドレス</dt>
            <dd class="mt-1 text-sm text-gray-900">{subscriber.email}</dd>
          </div>
          <div>
            <dt class="text-sm font-medium text-gray-500">名前</dt>
            <dd class="mt-1 text-sm text-gray-900">{subscriber.name || "-"}</dd>
          </div>
          <div>
            <dt class="text-sm font-medium text-gray-500">ステータス</dt>
            <dd class="mt-1 text-sm text-gray-900">
              <span
                class={`px-2 py-1 text-xs rounded-full ${getStatusBadgeClass(subscriber.status)}`}
              >
                {getStatusText(subscriber.status)}
              </span>
            </dd>
          </div>
          <div>
            <dt class="text-sm font-medium text-gray-500">登録日時</dt>
            <dd class="mt-1 text-sm text-gray-900">
              {formatDate(subscriber.subscribed_at)}
            </dd>
          </div>
          {#if subscriber.unsubscribed_at}
            <div>
              <dt class="text-sm font-medium text-gray-500">購読解除日時</dt>
              <dd class="mt-1 text-sm text-gray-900">
                {formatDate(subscriber.unsubscribed_at)}
              </dd>
            </div>
          {/if}
          <div>
            <dt class="text-sm font-medium text-gray-500">作成日時</dt>
            <dd class="mt-1 text-sm text-gray-900">
              {formatDate(subscriber.created_at)}
            </dd>
          </div>
          <div>
            <dt class="text-sm font-medium text-gray-500">更新日時</dt>
            <dd class="mt-1 text-sm text-gray-900">
              {formatDate(subscriber.updated_at)}
            </dd>
          </div>
        </dl>
      </div>

      <!-- タグ -->
      <div class="px-6 py-5 border-b border-gray-200">
        <h2 class="text-lg font-medium text-gray-900 mb-4">タグ</h2>
        {#if subscriber.tags.length === 0}
          <p class="text-sm text-gray-500">タグはありません</p>
        {:else}
          <div class="flex flex-wrap gap-2">
            {#each subscriber.tags as tag}
              <span
                class="bg-indigo-100 text-indigo-800 text-sm px-2 py-1 rounded-full"
              >
                {tag}
              </span>
            {/each}
          </div>
        {/if}
      </div>

      <!-- カスタムフィールド -->
      <div class="px-6 py-5">
        <h2 class="text-lg font-medium text-gray-900 mb-4">
          カスタムフィールド
        </h2>
        {#if Object.keys(subscriber.custom_fields).length === 0}
          <p class="text-sm text-gray-500">カスタムフィールドはありません</p>
        {:else}
          <dl class="grid grid-cols-1 gap-x-4 gap-y-6 sm:grid-cols-2">
            {#each Object.entries(subscriber.custom_fields) as [key, value]}
              <div>
                <dt class="text-sm font-medium text-gray-500">{key}</dt>
                <dd class="mt-1 text-sm text-gray-900">{value}</dd>
              </div>
            {/each}
          </dl>
        {/if}
      </div>
    </div>
  {:else if !isLoading && !error}
    <div class="bg-white shadow rounded-lg p-6">
      <p class="text-gray-500">購読者が見つかりませんでした。</p>
    </div>
  {/if}
</div>
