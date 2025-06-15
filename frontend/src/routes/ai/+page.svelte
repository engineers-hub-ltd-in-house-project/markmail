<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { authStore } from "$lib/stores/authStore";

  let isAuthenticated = false;

  authStore.subscribe((state) => {
    isAuthenticated = state.isAuthenticated;
  });

  onMount(() => {
    if (!isAuthenticated) {
      goto("/auth/login");
    }
  });
</script>

<svelte:head>
  <title>AI機能 | MarkMail</title>
</svelte:head>

{#if isAuthenticated}
  <div class="container mx-auto px-4 py-8">
    <!-- ヘッダー -->
    <div class="mb-8">
      <h1 class="text-3xl font-bold text-gray-900">AI機能</h1>
      <p class="mt-2 text-gray-600">
        AIを活用してマーケティングキャンペーンを効率的に作成しましょう
      </p>
    </div>

    <!-- AI機能カード -->
    <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
      <!-- シナリオ生成 -->
      <a
        href="/ai/scenarios/new"
        class="group relative rounded-lg border bg-white p-6 shadow-sm transition-all hover:shadow-md"
      >
        <div class="mb-4">
          <div
            class="inline-flex h-12 w-12 items-center justify-center rounded-lg bg-blue-100 text-blue-600"
          >
            <svg
              class="h-6 w-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </div>
        </div>
        <h3 class="mb-2 text-lg font-semibold text-gray-900">
          マーケティングシナリオ生成
        </h3>
        <p class="text-sm text-gray-600">
          業界、ターゲット、ゴールを指定するだけで、完全なマーケティングファネルを自動生成します
        </p>
        <div
          class="mt-4 flex items-center text-sm font-medium text-blue-600 group-hover:text-blue-700"
        >
          シナリオを生成
          <svg
            class="ml-1 h-4 w-4"
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
        </div>
      </a>

      <!-- コンテンツ生成 -->
      <a
        href="/ai/content/new"
        class="group relative rounded-lg border bg-white p-6 shadow-sm transition-all hover:shadow-md"
      >
        <div class="mb-4">
          <div
            class="inline-flex h-12 w-12 items-center justify-center rounded-lg bg-purple-100 text-purple-600"
          >
            <svg
              class="h-6 w-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
              />
            </svg>
          </div>
        </div>
        <h3 class="mb-2 text-lg font-semibold text-gray-900">コンテンツ生成</h3>
        <p class="text-sm text-gray-600">
          メールコンテンツを自動生成し、ターゲットに合わせてトーンやスタイルをカスタマイズします
        </p>
        <div
          class="mt-4 flex items-center text-sm font-medium text-purple-600 group-hover:text-purple-700"
        >
          コンテンツを生成
          <svg
            class="ml-1 h-4 w-4"
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
        </div>
      </a>

      <!-- 件名最適化 -->
      <a
        href="/ai/subject-optimizer"
        class="group relative rounded-lg border bg-white p-6 shadow-sm transition-all hover:shadow-md"
      >
        <div class="mb-4">
          <div
            class="inline-flex h-12 w-12 items-center justify-center rounded-lg bg-green-100 text-green-600"
          >
            <svg
              class="h-6 w-6"
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
          </div>
        </div>
        <h3 class="mb-2 text-lg font-semibold text-gray-900">件名最適化</h3>
        <p class="text-sm text-gray-600">
          既存の件名を分析し、開封率を向上させる最適化されたバリエーションを提案します
        </p>
        <div
          class="mt-4 flex items-center text-sm font-medium text-green-600 group-hover:text-green-700"
        >
          件名を最適化
          <svg
            class="ml-1 h-4 w-4"
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
        </div>
      </a>
    </div>

    <!-- 統計情報 -->
    <div class="mt-12 rounded-lg bg-gray-50 p-6">
      <h2 class="mb-4 text-lg font-semibold text-gray-900">AI機能の効果</h2>
      <div class="grid gap-4 sm:grid-cols-3">
        <div>
          <p class="text-2xl font-bold text-blue-600">80%</p>
          <p class="text-sm text-gray-600">キャンペーン作成時間の削減</p>
        </div>
        <div>
          <p class="text-2xl font-bold text-purple-600">45%</p>
          <p class="text-sm text-gray-600">平均開封率の向上</p>
        </div>
        <div>
          <p class="text-2xl font-bold text-green-600">3x</p>
          <p class="text-sm text-gray-600">コンテンツ生産性の向上</p>
        </div>
      </div>
    </div>
  </div>
{/if}
