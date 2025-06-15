<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { authStore } from "$lib/stores/authStore";
  import { aiApi } from "$lib/services/aiService";
  import type {
    GenerateScenarioRequest,
    GenerateScenarioResponse,
  } from "$lib/types/ai";

  let isAuthenticated = false;
  let isGenerating = false;
  let error = "";
  let showResults = false;

  // フォームデータ
  let formData: GenerateScenarioRequest = {
    industry: "",
    target_audience: "",
    goal: "",
    additional_context: "",
  };

  // 生成結果
  let generatedScenario: GenerateScenarioResponse | null = null;

  authStore.subscribe((state) => {
    isAuthenticated = state.isAuthenticated;
  });

  onMount(() => {
    if (!isAuthenticated) {
      goto("/auth/login");
    }
  });

  // よく使われる業界の例
  const industryExamples = [
    "Eコマース",
    "SaaS",
    "ヘルスケア",
    "教育",
    "不動産",
    "フィットネス",
    "旅行・観光",
    "フード・レストラン",
  ];

  // ゴールの例
  const goalExamples = [
    "新規顧客獲得",
    "既存顧客のリテンション向上",
    "商品の販売促進",
    "ブランド認知度向上",
    "イベント参加者の増加",
    "アップセル・クロスセル",
  ];

  async function generateScenario() {
    if (!formData.industry || !formData.target_audience || !formData.goal) {
      error = "業界、ターゲット層、ゴールは必須項目です";
      return;
    }

    isGenerating = true;
    error = "";

    try {
      generatedScenario = await aiApi.generateScenario(formData);
      showResults = true;
    } catch (err: any) {
      error = err.message || "シナリオの生成中にエラーが発生しました";
    } finally {
      isGenerating = false;
    }
  }

  function selectIndustry(industry: string) {
    formData.industry = industry;
  }

  function selectGoal(goal: string) {
    formData.goal = goal;
  }

  function startOver() {
    showResults = false;
    generatedScenario = null;
    formData = {
      industry: "",
      target_audience: "",
      goal: "",
      additional_context: "",
    };
  }

  async function implementScenario() {
    // TODO: 実装機能を追加
    alert(
      "この機能は開発中です。生成されたシナリオを基に手動で作成してください。",
    );
  }
</script>

<svelte:head>
  <title>AIシナリオ生成 | MarkMail</title>
</svelte:head>

{#if isAuthenticated}
  <div class="container mx-auto px-4 py-8">
    <!-- ヘッダー -->
    <div class="mb-8">
      <div class="flex items-center gap-2 text-sm text-gray-600 mb-2">
        <a href="/ai" class="hover:text-gray-900">AI機能</a>
        <span>/</span>
        <span>シナリオ生成</span>
      </div>
      <h1 class="text-3xl font-bold text-gray-900">
        AIマーケティングシナリオ生成
      </h1>
      <p class="mt-2 text-gray-600">
        業界とゴールを指定して、完全なマーケティングファネルを自動生成します
      </p>
    </div>

    {#if !showResults}
      <!-- 入力フォーム -->
      <div class="max-w-3xl">
        <form on:submit|preventDefault={generateScenario} class="space-y-6">
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
              bind:value={formData.industry}
              placeholder="例: Eコマース、SaaS、ヘルスケア"
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              required
            />
            <div class="mt-2 flex flex-wrap gap-2">
              {#each industryExamples as example}
                <button
                  type="button"
                  on:click={() => selectIndustry(example)}
                  class="text-xs px-3 py-1 rounded-full border hover:bg-gray-50
                    {formData.industry === example
                    ? 'bg-blue-50 border-blue-300 text-blue-700'
                    : 'border-gray-300 text-gray-600'}"
                >
                  {example}
                </button>
              {/each}
            </div>
          </div>

          <!-- ターゲット層 -->
          <div>
            <label
              for="target_audience"
              class="block text-sm font-medium text-gray-700 mb-2"
            >
              ターゲット層 <span class="text-red-500">*</span>
            </label>
            <input
              type="text"
              id="target_audience"
              bind:value={formData.target_audience}
              placeholder="例: 20-30代の働く女性、中小企業の経営者"
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              required
            />
          </div>

          <!-- ゴール -->
          <div>
            <label
              for="goal"
              class="block text-sm font-medium text-gray-700 mb-2"
            >
              マーケティングゴール <span class="text-red-500">*</span>
            </label>
            <input
              type="text"
              id="goal"
              bind:value={formData.goal}
              placeholder="例: 新規顧客獲得、リテンション向上"
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              required
            />
            <div class="mt-2 flex flex-wrap gap-2">
              {#each goalExamples as example}
                <button
                  type="button"
                  on:click={() => selectGoal(example)}
                  class="text-xs px-3 py-1 rounded-full border hover:bg-gray-50
                    {formData.goal === example
                    ? 'bg-blue-50 border-blue-300 text-blue-700'
                    : 'border-gray-300 text-gray-600'}"
                >
                  {example}
                </button>
              {/each}
            </div>
          </div>

          <!-- 追加コンテキスト -->
          <div>
            <label
              for="additional_context"
              class="block text-sm font-medium text-gray-700 mb-2"
            >
              追加情報（オプション）
            </label>
            <textarea
              id="additional_context"
              bind:value={formData.additional_context}
              rows="3"
              placeholder="特別な要件や制約があれば記入してください"
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            />
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
                    d="M13 10V3L4 14h7v7l9-11h-7z"
                  />
                </svg>
                シナリオを生成
              {/if}
            </button>
            <a href="/ai" class="btn-secondary"> キャンセル </a>
          </div>
        </form>
      </div>
    {:else if generatedScenario}
      <!-- 生成結果 -->
      <div class="max-w-5xl">
        <div class="mb-6 rounded-lg bg-green-50 p-4">
          <p class="text-sm font-medium text-green-800">
            シナリオが正常に生成されました！
          </p>
        </div>

        <!-- シナリオ概要 -->
        <div class="mb-8 rounded-lg border bg-white p-6">
          <h2 class="text-xl font-semibold text-gray-900 mb-2">
            {generatedScenario.scenario_name}
          </h2>
          <p class="text-gray-600">{generatedScenario.description}</p>

          <div class="mt-4 grid grid-cols-3 gap-4 text-sm">
            <div>
              <span class="text-gray-500">業界:</span>
              <span class="ml-2 font-medium">{formData.industry}</span>
            </div>
            <div>
              <span class="text-gray-500">ターゲット:</span>
              <span class="ml-2 font-medium">{formData.target_audience}</span>
            </div>
            <div>
              <span class="text-gray-500">ゴール:</span>
              <span class="ml-2 font-medium">{formData.goal}</span>
            </div>
          </div>
        </div>

        <!-- シーケンス -->
        <div class="mb-8">
          <h3 class="text-lg font-semibold text-gray-900 mb-4">
            メールシーケンス
          </h3>
          <div class="rounded-lg border bg-white p-6">
            <h4 class="font-medium text-gray-900">
              {generatedScenario.sequence.name}
            </h4>
            <p class="text-sm text-gray-600 mb-4">
              {generatedScenario.sequence.description}
            </p>

            <div class="space-y-3">
              {#each generatedScenario.sequence.steps as step, index}
                <div class="flex items-start gap-3">
                  <div
                    class="flex-shrink-0 w-8 h-8 rounded-full bg-blue-100 text-blue-600 flex items-center justify-center text-sm font-medium"
                  >
                    {index + 1}
                  </div>
                  <div class="flex-1">
                    <p class="font-medium text-gray-900">{step.name}</p>
                    <div class="text-sm text-gray-600">
                      {#if step.step_type === "email"}
                        メール送信
                        {#if step.template_index !== undefined}
                          （テンプレート #{step.template_index + 1}）
                        {/if}
                      {:else if step.step_type === "wait"}
                        {step.delay_value}{step.delay_unit === "days"
                          ? "日"
                          : step.delay_unit === "hours"
                            ? "時間"
                            : "分"}待機
                      {:else}
                        {step.step_type}
                      {/if}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        </div>

        <!-- フォーム -->
        {#if generatedScenario.forms.length > 0}
          <div class="mb-8">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">
              リードキャプチャフォーム
            </h3>
            <div class="grid gap-4 md:grid-cols-2">
              {#each generatedScenario.forms as form}
                <div class="rounded-lg border bg-white p-6">
                  <h4 class="font-medium text-gray-900">{form.name}</h4>
                  <p class="text-sm text-gray-600 mb-3">{form.description}</p>
                  <div class="space-y-2">
                    {#each form.fields as field}
                      <div class="text-sm">
                        <span class="font-medium">{field.label}</span>
                        <span class="text-gray-500 ml-2"
                          >({field.field_type})</span
                        >
                        {#if field.required}
                          <span class="text-red-500 ml-1">*</span>
                        {/if}
                      </div>
                    {/each}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- テンプレート -->
        {#if generatedScenario.templates.length > 0}
          <div class="mb-8">
            <h3 class="text-lg font-semibold text-gray-900 mb-4">
              メールテンプレート
            </h3>
            <div class="space-y-4">
              {#each generatedScenario.templates as template, index}
                <div class="rounded-lg border bg-white p-6">
                  <div class="mb-3">
                    <span class="text-xs font-medium text-gray-500"
                      >テンプレート #{index + 1}</span
                    >
                    <h4 class="font-medium text-gray-900">{template.name}</h4>
                  </div>
                  <div class="mb-3">
                    <p class="text-sm text-gray-500">件名:</p>
                    <p class="font-medium">{template.subject}</p>
                  </div>
                  <div class="mb-3">
                    <p class="text-sm text-gray-500">本文:</p>
                    <div
                      class="mt-1 p-3 bg-gray-50 rounded text-sm whitespace-pre-wrap"
                    >
                      {template.content}
                    </div>
                  </div>
                  {#if template.variables.length > 0}
                    <div>
                      <p class="text-sm text-gray-500">使用変数:</p>
                      <div class="mt-1 flex flex-wrap gap-2">
                        {#each template.variables as variable}
                          <code class="text-xs px-2 py-1 bg-gray-100 rounded">
                            {`{{${variable}}}`}
                          </code>
                        {/each}
                      </div>
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- アクションボタン -->
        <div class="flex gap-4">
          <button on:click={implementScenario} class="btn-primary">
            このシナリオを実装
          </button>
          <button on:click={startOver} class="btn-secondary">
            新しいシナリオを生成
          </button>
        </div>
      </div>
    {/if}
  </div>
{/if}
