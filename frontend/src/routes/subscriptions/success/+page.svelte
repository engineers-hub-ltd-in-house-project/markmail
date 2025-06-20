<script>
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";

  let isLoading = true;
  let error = null;

  onMount(() => {
    // セッションIDを取得してサブスクリプションの確認を行う
    const sessionId = $page.url.searchParams.get("session_id");

    if (sessionId) {
      // TODO: バックエンドAPIでセッションの確認を行う
      // Session ID: sessionId
    }

    isLoading = false;
  });

  function goToSubscriptions() {
    goto("/subscriptions");
  }

  function goToDashboard() {
    goto("/");
  }
</script>

<svelte:head>
  <title>お支払いが完了しました | MarkMail</title>
</svelte:head>

<div class="max-w-2xl mx-auto px-4 py-8">
  <div class="bg-white rounded-lg shadow-md p-8 text-center">
    {#if isLoading}
      <div class="flex justify-center items-center py-12">
        <div
          class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"
        ></div>
      </div>
    {:else if error}
      <div class="text-red-600 mb-6">
        <svg
          class="w-16 h-16 mx-auto mb-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
          ></path>
        </svg>
        <h1 class="text-2xl font-bold mb-2">エラーが発生しました</h1>
        <p class="text-gray-600">{error}</p>
      </div>
    {:else}
      <div class="text-green-600 mb-6">
        <svg
          class="w-16 h-16 mx-auto mb-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
          ></path>
        </svg>
        <h1 class="text-3xl font-bold mb-4">お支払いが完了しました！</h1>
      </div>

      <div class="text-gray-600 mb-8">
        <p class="mb-4">
          MarkMailプレミアムプランへのアップグレードありがとうございます。
        </p>
        <p class="mb-4">
          ご登録いただいたメールアドレスに確認メールをお送りしました。
        </p>
        <p>すべてのプレミアム機能が今すぐご利用いただけます。</p>
      </div>

      <div class="bg-gray-50 rounded-lg p-6 mb-8">
        <h2 class="text-lg font-semibold mb-3">
          プレミアムプランで利用可能な機能：
        </h2>
        <ul class="text-left text-sm text-gray-600 space-y-2">
          <li class="flex items-start">
            <svg
              class="w-5 h-5 text-green-500 mr-2 flex-shrink-0"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M5 13l4 4L19 7"
              ></path>
            </svg>
            無制限のメール配信
          </li>
          <li class="flex items-start">
            <svg
              class="w-5 h-5 text-green-500 mr-2 flex-shrink-0"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M5 13l4 4L19 7"
              ></path>
            </svg>
            高度な自動化シーケンス
          </li>
          <li class="flex items-start">
            <svg
              class="w-5 h-5 text-green-500 mr-2 flex-shrink-0"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M5 13l4 4L19 7"
              ></path>
            </svg>
            AI機能（シナリオ生成、コンテンツ生成、件名最適化）
          </li>
          <li class="flex items-start">
            <svg
              class="w-5 h-5 text-green-500 mr-2 flex-shrink-0"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M5 13l4 4L19 7"
              ></path>
            </svg>
            優先サポート
          </li>
        </ul>
      </div>

      <div class="flex flex-col sm:flex-row gap-4 justify-center">
        <button
          on:click={goToDashboard}
          class="px-6 py-3 bg-indigo-600 text-white font-medium rounded-md hover:bg-indigo-700 transition-colors"
        >
          ダッシュボードへ戻る
        </button>
        <button
          on:click={goToSubscriptions}
          class="px-6 py-3 bg-gray-200 text-gray-700 font-medium rounded-md hover:bg-gray-300 transition-colors"
        >
          サブスクリプション管理
        </button>
      </div>
    {/if}
  </div>

  {#if !isLoading && !error}
    <div class="mt-8 text-center text-sm text-gray-500">
      <p>
        ご不明な点がございましたら、
        <a
          href="mailto:support@markmail.com"
          class="text-indigo-600 hover:underline"
        >
          サポートチーム
        </a>
        までお問い合わせください。
      </p>
    </div>
  {/if}
</div>
