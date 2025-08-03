<script>
  import { goto } from "$app/navigation";

  function goToSubscriptions() {
    goto("/subscriptions");
  }

  function goToDashboard() {
    goto("/");
  }

  function retryPayment() {
    // サブスクリプションページに戻って再度支払いを試みる
    goto("/subscriptions");
  }
</script>

<svelte:head>
  <title>お支払いがキャンセルされました | MarkMail</title>
</svelte:head>

<div class="max-w-2xl mx-auto px-4 py-8">
  <div class="bg-white rounded-lg shadow-md p-8 text-center">
    <div class="text-yellow-600 mb-6">
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
          d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
        ></path>
      </svg>
      <h1 class="text-3xl font-bold mb-4">お支払いがキャンセルされました</h1>
    </div>

    <div class="text-gray-600 mb-8">
      <p class="mb-4">お支払い手続きがキャンセルされました。</p>
      <p class="mb-4">プレミアムプランへのアップグレードは完了していません。</p>
      <p>引き続き無料プランの機能をご利用いただけます。</p>
    </div>

    <div class="bg-blue-50 border border-blue-200 rounded-lg p-6 mb-8">
      <h2 class="text-lg font-semibold mb-3 text-blue-800">
        プレミアムプランをご検討中の方へ
      </h2>
      <p class="text-sm text-blue-700 mb-4">
        プレミアムプランでは以下の機能をご利用いただけます：
      </p>
      <ul class="text-left text-sm text-blue-700 space-y-2">
        <li class="flex items-start">
          <svg
            class="w-5 h-5 text-blue-500 mr-2 flex-shrink-0"
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
            class="w-5 h-5 text-blue-500 mr-2 flex-shrink-0"
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
          AI機能でコンテンツ作成を効率化
        </li>
        <li class="flex items-start">
          <svg
            class="w-5 h-5 text-blue-500 mr-2 flex-shrink-0"
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
          高度な自動化機能
        </li>
      </ul>
    </div>

    <div class="flex flex-col sm:flex-row gap-4 justify-center">
      <button
        on:click={retryPayment}
        class="px-6 py-3 bg-indigo-600 text-white font-medium rounded-md hover:bg-indigo-700 transition-colors"
      >
        もう一度試す
      </button>
      <button
        on:click={goToDashboard}
        class="px-6 py-3 bg-gray-200 text-gray-700 font-medium rounded-md hover:bg-gray-300 transition-colors"
      >
        ダッシュボードへ戻る
      </button>
    </div>
  </div>

  <div class="mt-8 text-center text-sm text-gray-500">
    <p>
      お支払いに関してご不明な点がございましたら、
      <a
        href="mailto:support@markmail.com"
        class="text-indigo-600 hover:underline"
      >
        サポートチーム
      </a>
      までお問い合わせください。
    </p>
    <p class="mt-2">
      または、
      <button
        on:click={goToSubscriptions}
        class="text-indigo-600 hover:underline focus:outline-none"
      >
        プランの詳細を確認
      </button>
      してください。
    </p>
  </div>
</div>
