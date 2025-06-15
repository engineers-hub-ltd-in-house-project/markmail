<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { authStore } from "$lib/stores/authStore";
  import {
    aiApi,
    getContentToneLabel,
    formatReadingTime,
  } from "$lib/services/aiService";
  import type {
    GenerateContentRequest,
    GenerateContentResponse,
    ContentTone,
  } from "$lib/types/ai";

  let isAuthenticated = false;
  let isGenerating = false;
  let error = "";
  let showResults = false;

  // フォームデータ
  let contentType = "email_template";
  let industry = "";
  let targetAudience = "";
  let tone: ContentTone = "professional";
  let language = "ja";
  let existingContent = "";
  let includeVariations = true;
  let includePersonalization = true;

  // 生成結果
  let generatedContent: GenerateContentResponse | null = null;
  let selectedVariation = 0;

  authStore.subscribe((state) => {
    isAuthenticated = state.isAuthenticated;
  });

  onMount(() => {
    if (!isAuthenticated) {
      goto("/auth/login");
    }
  });

  const toneOptions: { value: ContentTone; label: string }[] = [
    { value: "formal", label: "フォーマル" },
    { value: "casual", label: "カジュアル" },
    { value: "professional", label: "プロフェッショナル" },
    { value: "friendly", label: "フレンドリー" },
    { value: "urgent", label: "緊急" },
  ];

  async function generateContent() {
    if (!industry || !targetAudience) {
      error = "業界とターゲット層は必須項目です";
      return;
    }

    isGenerating = true;
    error = "";

    const request: GenerateContentRequest = {
      content_type: contentType,
      context: {
        industry,
        target_audience: targetAudience,
        tone,
        language,
        ...(existingContent && { existing_content: existingContent }),
      },
      options: {
        variations: includeVariations ? 3 : 1,
        include_personalization: includePersonalization,
      },
    };

    try {
      generatedContent = await aiApi.generateContent(request);
      showResults = true;
      selectedVariation = 0;
    } catch (err: any) {
      error = err.message || "コンテンツの生成中にエラーが発生しました";
    } finally {
      isGenerating = false;
    }
  }

  function startOver() {
    showResults = false;
    generatedContent = null;
    selectedVariation = 0;
    existingContent = "";
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text).then(() => {
      // TODO: フィードバック表示
      alert("クリップボードにコピーしました");
    });
  }

  function useInTemplate() {
    if (!generatedContent) return;

    const content =
      generatedContent.variations &&
      generatedContent.variations[selectedVariation]
        ? generatedContent.variations[selectedVariation]
        : generatedContent.content;

    // テンプレート作成画面に遷移（コンテンツを渡す）
    const params = new URLSearchParams({
      content: content,
      variables: generatedContent.suggested_variables.join(","),
    });

    goto(`/templates/new?${params.toString()}`);
  }
</script>

<svelte:head>
  <title>AIコンテンツ生成 | MarkMail</title>
</svelte:head>

{#if isAuthenticated}
  <div class="container mx-auto px-4 py-8">
    <!-- ヘッダー -->
    <div class="mb-8">
      <div class="flex items-center gap-2 text-sm text-gray-600 mb-2">
        <a href="/ai" class="hover:text-gray-900">AI機能</a>
        <span>/</span>
        <span>コンテンツ生成</span>
      </div>
      <h1 class="text-3xl font-bold text-gray-900">AIコンテンツ生成</h1>
      <p class="mt-2 text-gray-600">
        ターゲットに最適化されたメールコンテンツを自動生成します
      </p>
    </div>

    {#if !showResults}
      <!-- 入力フォーム -->
      <div class="max-w-3xl">
        <form on:submit|preventDefault={generateContent} class="space-y-6">
          <!-- コンテンツタイプ -->
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">
              コンテンツタイプ
            </label>
            <select
              bind:value={contentType}
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            >
              <option value="email_template">メールテンプレート</option>
              <option value="subject">件名のみ</option>
            </select>
          </div>

          <!-- 業界 -->
          <div>
            <label
              for="industry"
              class="block text-sm font-medium text-gray-700 mb-2"
            >
              業界 <span class="text-red-500">*</span>
            </label>
            <input
              type="text"
              id="industry"
              bind:value={industry}
              placeholder="例: Eコマース、SaaS、ヘルスケア"
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              required
            />
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
              placeholder="例: 20-30代の働く女性、中小企業の経営者"
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              required
            />
          </div>

          <!-- トーン -->
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-2">
              トーン
            </label>
            <div class="grid grid-cols-3 gap-3">
              {#each toneOptions as option}
                <label
                  class="relative flex cursor-pointer rounded-lg border p-3 hover:bg-gray-50"
                >
                  <input
                    type="radio"
                    bind:group={tone}
                    value={option.value}
                    class="sr-only"
                  />
                  <div class="flex items-center">
                    <div class="text-sm">
                      <p
                        class="font-medium {tone === option.value
                          ? 'text-blue-600'
                          : 'text-gray-900'}"
                      >
                        {option.label}
                      </p>
                    </div>
                  </div>
                  {#if tone === option.value}
                    <div
                      class="absolute -inset-px rounded-lg border-2 border-blue-500"
                    />
                  {/if}
                </label>
              {/each}
            </div>
          </div>

          <!-- 既存コンテンツ -->
          <div>
            <label
              for="existingContent"
              class="block text-sm font-medium text-gray-700 mb-2"
            >
              参考にする既存コンテンツ（オプション）
            </label>
            <textarea
              id="existingContent"
              bind:value={existingContent}
              rows="4"
              placeholder="既存のメールやコンテンツを貼り付けると、そのスタイルを参考に生成します"
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            />
          </div>

          <!-- オプション -->
          <div class="space-y-3">
            <label class="flex items-center">
              <input
                type="checkbox"
                bind:checked={includeVariations}
                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <span class="ml-2 text-sm text-gray-700">
                複数のバリエーションを生成する
              </span>
            </label>
            <label class="flex items-center">
              <input
                type="checkbox"
                bind:checked={includePersonalization}
                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <span class="ml-2 text-sm text-gray-700">
                パーソナライゼーション変数を含める
              </span>
            </label>
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
              disabled={isGenerating}
              class="btn-primary flex items-center gap-2"
            >
              {#if isGenerating}
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
                生成中...
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
                    d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
                  />
                </svg>
                コンテンツを生成
              {/if}
            </button>
            <a href="/ai" class="btn-secondary"> キャンセル </a>
          </div>
        </form>
      </div>
    {:else if generatedContent}
      <!-- 生成結果 -->
      <div class="max-w-4xl">
        <div class="mb-6 rounded-lg bg-green-50 p-4">
          <p class="text-sm font-medium text-green-800">
            コンテンツが正常に生成されました！
          </p>
        </div>

        <!-- メタデータ -->
        <div class="mb-6 grid grid-cols-4 gap-4">
          <div class="rounded-lg bg-gray-50 p-4">
            <p class="text-sm text-gray-500">読了時間</p>
            <p class="text-lg font-semibold text-gray-900">
              {formatReadingTime(
                generatedContent.metadata.estimated_reading_time,
              )}
            </p>
          </div>
          <div class="rounded-lg bg-gray-50 p-4">
            <p class="text-sm text-gray-500">文字数</p>
            <p class="text-lg font-semibold text-gray-900">
              {generatedContent.metadata.word_count}
            </p>
          </div>
          <div class="rounded-lg bg-gray-50 p-4">
            <p class="text-sm text-gray-500">パーソナライゼーション</p>
            <p class="text-lg font-semibold text-gray-900">
              {Math.round(
                generatedContent.metadata.personalization_score * 100,
              )}%
            </p>
          </div>
          <div class="rounded-lg bg-gray-50 p-4">
            <p class="text-sm text-gray-500">明瞭性</p>
            <p class="text-lg font-semibold text-gray-900">
              {Math.round(generatedContent.metadata.clarity_score * 100)}%
            </p>
          </div>
        </div>

        <!-- バリエーション選択 -->
        {#if generatedContent.variations && generatedContent.variations.length > 1}
          <div class="mb-6">
            <label class="block text-sm font-medium text-gray-700 mb-2">
              バリエーションを選択
            </label>
            <div class="flex gap-2">
              {#each generatedContent.variations as _, index}
                <button
                  on:click={() => (selectedVariation = index)}
                  class="px-4 py-2 rounded-md text-sm font-medium transition-colors
                    {selectedVariation === index
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-100 text-gray-700 hover:bg-gray-200'}"
                >
                  バリエーション {index + 1}
                </button>
              {/each}
            </div>
          </div>
        {/if}

        <!-- コンテンツ表示 -->
        <div class="mb-6 rounded-lg border bg-white p-6">
          <div class="mb-4 flex items-center justify-between">
            <h3 class="text-lg font-semibold text-gray-900">
              生成されたコンテンツ
            </h3>
            <button
              on:click={() =>
                copyToClipboard(
                  generatedContent?.variations &&
                    generatedContent.variations[selectedVariation]
                    ? generatedContent.variations[selectedVariation]
                    : generatedContent?.content || "",
                )}
              class="text-sm text-blue-600 hover:text-blue-700 flex items-center gap-1"
            >
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
                  d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                />
              </svg>
              コピー
            </button>
          </div>
          <div class="prose max-w-none">
            <div class="whitespace-pre-wrap text-gray-800">
              {generatedContent.variations &&
              generatedContent.variations[selectedVariation]
                ? generatedContent.variations[selectedVariation]
                : generatedContent.content}
            </div>
          </div>
        </div>

        <!-- 使用変数 -->
        {#if generatedContent.suggested_variables.length > 0}
          <div class="mb-6 rounded-lg border bg-white p-6">
            <h3 class="text-lg font-semibold text-gray-900 mb-3">推奨変数</h3>
            <p class="text-sm text-gray-600 mb-3">
              これらの変数を使用してコンテンツをパーソナライズできます
            </p>
            <div class="flex flex-wrap gap-2">
              {#each generatedContent.suggested_variables as variable}
                <code class="px-3 py-1 bg-gray-100 rounded text-sm">
                  {`{{${variable}}}`}
                </code>
              {/each}
            </div>
          </div>
        {/if}

        <!-- アクションボタン -->
        <div class="flex gap-4">
          <button on:click={useInTemplate} class="btn-primary">
            テンプレートとして使用
          </button>
          <button on:click={startOver} class="btn-secondary">
            新しいコンテンツを生成
          </button>
        </div>
      </div>
    {/if}
  </div>
{/if}
