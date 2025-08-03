<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { authStore } from "$lib/stores/authStore";
  import { aiApi, formatOpenRate } from "$lib/services/aiService";
  import type {
    OptimizeSubjectRequest,
    OptimizeSubjectResponse,
    Language,
  } from "$lib/types/ai";

  let isAuthenticated = false;
  let isOptimizing = false;
  let error = "";
  let showResults = false;

  // フォームデータ
  let originalSubject = "";
  let targetAudience = "";
  let campaignGoal = "";
  let variationsCount = 5;
  let language: Language = "ja";

  // 最適化結果
  let optimizationResult: OptimizeSubjectResponse | null = null;

  authStore.subscribe((state) => {
    isAuthenticated = state.isAuthenticated;
  });

  onMount(() => {
    if (!isAuthenticated) {
      goto("/auth/login");
    }
  });

  // サンプル件名
  const subjectExamples = [
    "【期間限定】夏のセール開催中！",
    "あなたのビジネスを次のレベルへ",
    "無料トライアルのご案内",
    "新製品リリースのお知らせ",
    "ウェビナー開催：成功の秘訣を公開",
  ];

  async function optimizeSubject() {
    if (!originalSubject || !targetAudience) {
      error = "件名とターゲット層は必須項目です";
      return;
    }

    isOptimizing = true;
    error = "";

    const request: OptimizeSubjectRequest = {
      original_subject: originalSubject,
      target_audience: targetAudience,
      campaign_goal: campaignGoal || undefined,
      variations_count: variationsCount,
      language,
    };

    try {
      optimizationResult = await aiApi.optimizeSubject(request);
      showResults = true;
    } catch (err: any) {
      error = err.message || "件名の最適化中にエラーが発生しました";
    } finally {
      isOptimizing = false;
    }
  }

  function selectExample(subject: string) {
    originalSubject = subject;
  }

  function startOver() {
    showResults = false;
    optimizationResult = null;
    originalSubject = "";
    targetAudience = "";
    campaignGoal = "";
  }

  function useSubject(subject: string) {
    // テンプレート作成画面に遷移
    const params = new URLSearchParams({
      subject: subject,
    });
    goto(`/templates/new?${params.toString()}`);
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text).then(() => {
      alert("クリップボードにコピーしました");
    });
  }

  // 開封率による色分け
  function getOpenRateColor(rate: number): string {
    if (rate >= 0.3) return "text-green-600";
    if (rate >= 0.2) return "text-blue-600";
    if (rate >= 0.15) return "text-yellow-600";
    return "text-gray-600";
  }

  function getOpenRateBgColor(rate: number): string {
    if (rate >= 0.3) return "bg-green-50";
    if (rate >= 0.2) return "bg-blue-50";
    if (rate >= 0.15) return "bg-yellow-50";
    return "bg-gray-50";
  }
</script>

<svelte:head>
  <title>AI件名最適化 | MarkMail</title>
</svelte:head>

{#if isAuthenticated}
  <div class="container mx-auto px-4 py-8">
    <!-- ヘッダー -->
    <div class="mb-8">
      <div class="flex items-center gap-2 text-sm text-gray-600 mb-2">
        <a href="/ai" class="hover:text-gray-900">AI機能</a>
        <span>/</span>
        <span>件名最適化</span>
      </div>
      <h1 class="text-3xl font-bold text-gray-900">AI件名最適化</h1>
      <p class="mt-2 text-gray-600">
        メールの件名を分析し、開封率を向上させる最適化案を提案します
      </p>
    </div>

    {#if !showResults}
      <!-- 入力フォーム -->
      <div class="max-w-3xl">
        <form on:submit|preventDefault={optimizeSubject} class="space-y-6">
          <!-- 元の件名 -->
          <div>
            <label
              for="originalSubject"
              class="block text-sm font-medium text-gray-700 mb-2"
            >
              最適化したい件名 <span class="text-red-500">*</span>
            </label>
            <input
              type="text"
              id="originalSubject"
              bind:value={originalSubject}
              placeholder="例: 新商品のご案内"
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              required
            />
            <div class="mt-2">
              <p class="text-xs text-gray-500 mb-2">サンプル:</p>
              <div class="flex flex-wrap gap-2">
                {#each subjectExamples as example}
                  <button
                    type="button"
                    on:click={() => selectExample(example)}
                    class="text-xs px-3 py-1 rounded-full border hover:bg-gray-50
                      {originalSubject === example
                      ? 'bg-blue-50 border-blue-300 text-blue-700'
                      : 'border-gray-300 text-gray-600'}"
                  >
                    {example}
                  </button>
                {/each}
              </div>
            </div>
          </div>

          <!-- ターゲット層 -->
          <div>
            <label
              for="targetAudience"
              class="block text-sm font-medium text-gray-700 mb-2"
            >
              ターゲット層 <span class="text-red-500">*</span>
            </label>
            <input
              type="text"
              id="targetAudience"
              bind:value={targetAudience}
              placeholder="例: 30-40代のビジネスパーソン"
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              required
            />
          </div>

          <!-- キャンペーンゴール -->
          <div>
            <label
              for="campaignGoal"
              class="block text-sm font-medium text-gray-700 mb-2"
            >
              キャンペーンの目的（オプション）
            </label>
            <input
              type="text"
              id="campaignGoal"
              bind:value={campaignGoal}
              placeholder="例: セールの告知、ウェビナー参加促進"
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            />
          </div>

          <!-- バリエーション数 -->
          <div>
            <label
              for="variationsCount"
              class="block text-sm font-medium text-gray-700 mb-2"
            >
              生成するバリエーション数
            </label>
            <select
              id="variationsCount"
              bind:value={variationsCount}
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            >
              <option value={3}>3つ</option>
              <option value={5}>5つ</option>
              <option value={7}>7つ</option>
              <option value={10}>10つ</option>
            </select>
          </div>

          <!-- 出力言語 -->
          <div>
            <label
              for="language"
              class="block text-sm font-medium text-gray-700 mb-2"
            >
              出力言語
            </label>
            <select
              id="language"
              bind:value={language}
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            >
              <option value="ja">日本語</option>
              <option value="en">英語</option>
            </select>
            <p class="mt-1 text-sm text-gray-500">
              最適化された件名の言語を選択してください
            </p>
          </div>

          <!-- エラーメッセージ -->
          {#if error}
            <div class="rounded-md bg-red-50 p-4">
              <p class="text-sm text-red-800">{error}</p>
            </div>
          {/if}

          <!-- ボタン -->
          <div class="flex gap-4">
            <button
              type="submit"
              disabled={isOptimizing}
              class="btn-primary flex items-center gap-2"
            >
              {#if isOptimizing}
                <svg
                  class="animate-spin h-4 w-4"
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
                  />
                  <path
                    class="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
                  />
                </svg>
                最適化中...
              {:else}
                <svg
                  class="h-4 w-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M13 10V3L4 14h7v7l9-11h-7z"
                  />
                </svg>
                件名を最適化
              {/if}
            </button>
            <a href="/ai" class="btn-secondary"> キャンセル </a>
          </div>
        </form>
      </div>
    {:else if optimizationResult}
      <!-- 最適化結果 -->
      <div class="max-w-4xl">
        <div class="mb-6 rounded-lg bg-green-50 p-4">
          <p class="text-sm font-medium text-green-800">
            件名の最適化が完了しました！
          </p>
        </div>

        <!-- 元の件名 -->
        <div class="mb-6 rounded-lg border bg-white p-6">
          <h3 class="text-sm font-medium text-gray-500 mb-2">元の件名</h3>
          <p class="text-lg text-gray-900">{originalSubject}</p>
          <div class="mt-2 text-sm text-gray-600">
            ターゲット層: {targetAudience}
            {#if campaignGoal}
              <span class="ml-3">目的: {campaignGoal}</span>
            {/if}
          </div>
        </div>

        <!-- 最適化されたバリエーション -->
        <div class="mb-6">
          <h3 class="text-lg font-semibold text-gray-900 mb-4">
            最適化されたバリエーション
          </h3>
          <div class="space-y-3">
            {#each optimizationResult.optimized_subjects as variation, index}
              <div
                class="rounded-lg border bg-white p-4 hover:shadow-sm transition-shadow
                {index === optimizationResult.best_pick
                  ? 'border-blue-500'
                  : ''}"
              >
                <div class="flex items-start justify-between">
                  <div class="flex-1">
                    {#if index === optimizationResult.best_pick}
                      <span
                        class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800 mb-2"
                      >
                        おすすめ
                      </span>
                    {/if}
                    <h4 class="font-medium text-gray-900 mb-2">
                      {variation.subject}
                    </h4>
                    <p class="text-sm text-gray-600 mb-2">
                      {variation.reasoning}
                    </p>
                    <div class="flex items-center gap-4">
                      <span
                        class="text-sm {getOpenRateColor(
                          variation.predicted_open_rate,
                        )}"
                      >
                        予測開封率:
                        <span class="font-semibold text-lg">
                          {formatOpenRate(variation.predicted_open_rate)}
                        </span>
                      </span>
                      <div
                        class="flex-1 h-2 bg-gray-200 rounded-full overflow-hidden"
                      >
                        <div
                          class="h-full {getOpenRateBgColor(
                            variation.predicted_open_rate,
                          )} 
                            {getOpenRateColor(
                            variation.predicted_open_rate,
                          ).replace('text-', 'bg-')}"
                          style="width: {variation.predicted_open_rate * 100}%"
                        />
                      </div>
                    </div>
                  </div>
                  <div class="ml-4 flex gap-2">
                    <button
                      on:click={() => copyToClipboard(variation.subject)}
                      class="text-gray-500 hover:text-gray-700"
                      title="コピー"
                    >
                      <svg
                        class="h-5 w-5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2"
                          d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                        />
                      </svg>
                    </button>
                    <button
                      on:click={() => useSubject(variation.subject)}
                      class="text-blue-600 hover:text-blue-700"
                      title="使用する"
                    >
                      <svg
                        class="h-5 w-5"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2"
                          d="M9 5l7 7-7 7"
                        />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>

        <!-- 開封率の説明 -->
        <div class="mb-6 rounded-lg bg-gray-50 p-6">
          <h3 class="text-sm font-semibold text-gray-900 mb-3">開封率の目安</h3>
          <div class="grid grid-cols-4 gap-4 text-sm">
            <div class="flex items-center gap-2">
              <div class="w-4 h-4 rounded bg-green-500"></div>
              <span>30%以上: 優秀</span>
            </div>
            <div class="flex items-center gap-2">
              <div class="w-4 h-4 rounded bg-blue-500"></div>
              <span>20-30%: 良好</span>
            </div>
            <div class="flex items-center gap-2">
              <div class="w-4 h-4 rounded bg-yellow-500"></div>
              <span>15-20%: 平均的</span>
            </div>
            <div class="flex items-center gap-2">
              <div class="w-4 h-4 rounded bg-gray-400"></div>
              <span>15%未満: 改善余地あり</span>
            </div>
          </div>
        </div>

        <!-- アクションボタン -->
        <div class="flex gap-4">
          <button
            on:click={() =>
              useSubject(
                optimizationResult?.optimized_subjects[
                  optimizationResult.best_pick
                ].subject || "",
              )}
            class="btn-primary"
          >
            おすすめの件名を使用
          </button>
          <button on:click={startOver} class="btn-secondary">
            別の件名を最適化
          </button>
        </div>
      </div>
    {/if}
  </div>
{/if}
