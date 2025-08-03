<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { authStore } from "$lib/stores/authStore";
  import { aiApi } from "$lib/services/aiService";
  import { templateApi } from "$lib/services/api";
  import { formService } from "$lib/services/formService";
  import { sequenceService } from "$lib/services/sequenceService";
  import type {
    GenerateScenarioRequest,
    GenerateScenarioResponse,
    Language,
  } from "$lib/types/ai";
  import type { CreateTemplateRequest } from "$lib/types/template";
  import type { CreateFormRequest } from "$lib/types/form";
  import type {
    CreateSequenceRequest,
    CreateSequenceStepRequest,
    TriggerType,
    StepType,
  } from "$lib/types/sequence";

  let isAuthenticated = false;
  let isGenerating = false;
  let error = "";
  let showResults = false;
  let isImplementing = false;
  let implementationProgress = {
    templates: 0,
    forms: 0,
    sequence: false,
    completed: false,
  };

  // フォームデータ
  let formData: GenerateScenarioRequest = {
    industry: "",
    target_audience: "",
    goal: "",
    additional_context: "",
    language: "ja" as Language,
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
      language: "ja" as Language,
    };
  }

  async function implementScenario() {
    if (!generatedScenario) return;

    isImplementing = true;
    error = "";
    implementationProgress = {
      templates: 0,
      forms: 0,
      sequence: false,
      completed: false,
    };

    try {
      // 1. テンプレートを作成
      const createdTemplates: string[] = [];
      for (let i = 0; i < generatedScenario.templates.length; i++) {
        const template = generatedScenario.templates[i];
        const createRequest: CreateTemplateRequest = {
          name: template.name,
          subject_template: template.subject,
          markdown_content: template.content,
        };

        const response = await templateApi.createTemplate(createRequest);
        if (response.error) {
          throw new Error(`テンプレート作成エラー: ${response.error}`);
        }
        if (response.data) {
          createdTemplates.push(response.data.id);
          implementationProgress.templates = i + 1;
        }
      }

      // 2. フォームを作成
      const createdForms: string[] = [];
      for (let i = 0; i < generatedScenario.forms.length; i++) {
        const generatedForm = generatedScenario.forms[i];
        // スラッグを生成（重複を避けるためタイムスタンプとインデックスを含める）
        const slug =
          generatedForm.name
            .toLowerCase()
            .replace(/\s+/g, "-")
            .replace(/[^a-z0-9-]/g, "") +
          "-" +
          Date.now() +
          "-" +
          i;

        const createRequest: CreateFormRequest = {
          name: generatedForm.name,
          description: generatedForm.description,
          slug: slug,
          markdown_content: `# ${generatedForm.name}\n\n${generatedForm.description || ""}`,
          form_fields: generatedForm.fields.map((field, index) => ({
            field_type: field.field_type as
              | "text"
              | "email"
              | "textarea"
              | "select"
              | "radio"
              | "checkbox",
            name: field.name,
            label: field.label,
            required: field.required,
            display_order: index + 1,
            options: field.options,
          })),
          settings: {
            submit_button_text: "送信",
            success_message: "ご登録ありがとうございました。",
            require_confirmation: true,
          },
        };

        try {
          const createdForm = await formService.create(createRequest);
          createdForms.push(createdForm.id);
          implementationProgress.forms = i + 1;
        } catch (formErr: any) {
          throw new Error(`フォーム作成エラー: ${formErr.message}`);
        }
      }

      // 3. シーケンスを作成
      const triggerConfig: Record<string, any> = {};
      if (
        generatedScenario.sequence.trigger_type === "form_submission" &&
        createdForms.length > 0
      ) {
        triggerConfig.form_id = createdForms[0];
      }

      const createSequenceRequest: CreateSequenceRequest = {
        name: generatedScenario.sequence.name,
        description: generatedScenario.sequence.description,
        trigger_type: generatedScenario.sequence.trigger_type as TriggerType,
        trigger_config: triggerConfig,
      };

      const sequence = await sequenceService.createSequence(
        createSequenceRequest,
      );
      const sequenceId = sequence.id;

      // 4. シーケンスステップを作成
      for (let i = 0; i < generatedScenario.sequence.steps.length; i++) {
        const step = generatedScenario.sequence.steps[i];
        const stepRequest: CreateSequenceStepRequest = {
          name: step.name,
          step_type: step.step_type as StepType,
          step_order: i + 1,
          delay_value: step.delay_value,
          delay_unit: step.delay_unit,
          template_id:
            step.template_index !== undefined
              ? createdTemplates[step.template_index]
              : undefined,
          conditions: step.conditions,
        };

        await sequenceService.createSequenceStep(sequenceId, stepRequest);
      }

      implementationProgress.sequence = true;
      implementationProgress.completed = true;

      // 成功メッセージを表示して、シーケンス管理画面に遷移
      setTimeout(() => {
        goto("/sequences");
      }, 2000);
    } catch (err: any) {
      error = err.message || "シナリオの実装中にエラーが発生しました";
    } finally {
      if (!implementationProgress.completed) {
        isImplementing = false;
      }
    }
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
              bind:value={formData.language}
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            >
              <option value="ja">日本語</option>
              <option value="en">英語</option>
            </select>
            <p class="mt-1 text-sm text-gray-500">
              生成されるシナリオの言語を選択してください
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

        <!-- 実装進捗 -->
        {#if isImplementing}
          <div class="mb-6 rounded-lg border bg-blue-50 p-6">
            <h3 class="text-lg font-semibold text-blue-900 mb-4">実装中...</h3>
            <div class="space-y-3">
              <div class="flex items-center gap-3">
                <div
                  class={`w-5 h-5 rounded-full ${implementationProgress.templates > 0 ? "bg-green-500" : "bg-gray-300"}`}
                >
                  {#if implementationProgress.templates > 0}
                    <svg
                      class="w-5 h-5 text-white"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  {/if}
                </div>
                <span class="text-sm">
                  テンプレート作成 ({implementationProgress.templates}/{generatedScenario
                    .templates.length})
                </span>
              </div>
              <div class="flex items-center gap-3">
                <div
                  class={`w-5 h-5 rounded-full ${implementationProgress.forms > 0 ? "bg-green-500" : "bg-gray-300"}`}
                >
                  {#if implementationProgress.forms === generatedScenario.forms.length}
                    <svg
                      class="w-5 h-5 text-white"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  {/if}
                </div>
                <span class="text-sm">
                  フォーム作成 ({implementationProgress.forms}/{generatedScenario
                    .forms.length})
                </span>
              </div>
              <div class="flex items-center gap-3">
                <div
                  class={`w-5 h-5 rounded-full ${implementationProgress.sequence ? "bg-green-500" : "bg-gray-300"}`}
                >
                  {#if implementationProgress.sequence}
                    <svg
                      class="w-5 h-5 text-white"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path
                        fill-rule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  {/if}
                </div>
                <span class="text-sm"> シーケンス・ステップ作成 </span>
              </div>
            </div>
            {#if implementationProgress.completed}
              <div class="mt-4 rounded bg-green-100 p-3">
                <p class="text-sm text-green-800">
                  実装が完了しました！シーケンス管理画面に移動します...
                </p>
              </div>
            {/if}
          </div>
        {/if}

        <!-- アクションボタン -->
        <div class="flex gap-4">
          <button
            on:click={implementScenario}
            disabled={isImplementing}
            class="btn-primary flex items-center gap-2"
          >
            {#if isImplementing}
              <svg class="animate-spin h-4 w-4" fill="none" viewBox="0 0 24 24">
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
              実装中...
            {:else}
              このシナリオを実装
            {/if}
          </button>
          <button
            on:click={startOver}
            class="btn-secondary"
            disabled={isImplementing}
          >
            新しいシナリオを生成
          </button>
        </div>
      </div>
    {/if}
  </div>
{/if}
