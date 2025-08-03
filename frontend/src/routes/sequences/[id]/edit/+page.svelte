<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { sequenceService } from "$lib/services/sequenceService";
  import { templateApi } from "$lib/services/api";
  import type {
    SequenceWithSteps,
    SequenceStep,
    UpdateSequenceRequest,
    CreateSequenceStepRequest,
    StepType,
  } from "$lib/types/sequence";
  import type { Template } from "$lib/types/template";

  let sequence: SequenceWithSteps | null = null;
  let templates: Template[] = [];
  let loading = true;
  let saving = false;
  let error = "";
  let successMessage = "";

  // フォームフィールド
  let name = "";
  let description = "";
  let status = "";

  // ステップ編集
  let editingStep: SequenceStep | null = null;
  let showAddStep = false;
  let newStepType: StepType = "email";
  let newStepTemplateId = "";
  let newStepDelayMinutes = 60;
  let newStepDelayUnit: "minutes" | "hours" | "days" = "hours";

  $: sequenceId = $page.params.id;
  $: sortedSteps =
    sequence?.steps?.sort((a, b) => a.step_order - b.step_order) || [];

  onMount(async () => {
    await Promise.all([loadSequence(), loadTemplates()]);
  });

  async function loadSequence() {
    try {
      loading = true;
      sequence = await sequenceService.getSequence(sequenceId);
      if (sequence) {
        name = sequence.name;
        description = sequence.description || "";
        status = sequence.status;
      }
    } catch (err) {
      error =
        err instanceof Error
          ? err.message
          : "シーケンスの読み込みに失敗しました";
    } finally {
      loading = false;
    }
  }

  async function loadTemplates() {
    try {
      const result = await templateApi.getTemplates();
      if (result.data) {
        templates = result.data.templates;
      }
    } catch (err) {
      console.error("Failed to load templates:", err);
    }
  }

  async function handleSave() {
    if (!name.trim()) {
      error = "シーケンス名を入力してください";
      return;
    }

    try {
      saving = true;
      error = "";

      const data: UpdateSequenceRequest = {
        name: name.trim(),
        description: description.trim() || undefined,
        status: status as any,
      };

      await sequenceService.updateSequence(sequenceId, data);
      successMessage = "シーケンスを保存しました";
      setTimeout(() => {
        successMessage = "";
      }, 3000);
    } catch (err) {
      error =
        err instanceof Error ? err.message : "シーケンスの保存に失敗しました";
    } finally {
      saving = false;
    }
  }

  async function handleAddStep() {
    try {
      error = "";

      let delayMinutes = newStepDelayMinutes;
      if (newStepDelayUnit === "hours") {
        delayMinutes = newStepDelayMinutes * 60;
      } else if (newStepDelayUnit === "days") {
        delayMinutes = newStepDelayMinutes * 1440;
      }

      // 最大のstep_orderを取得して、それに1を加える
      const currentSteps = sortedSteps || [];
      const maxStepOrder =
        currentSteps.length > 0
          ? Math.max(...currentSteps.map((s) => s.step_order))
          : 0;

      console.log("Current steps:", currentSteps);
      console.log("Max step order:", maxStepOrder);

      const data: CreateSequenceStepRequest = {
        name: newStepType === "email" ? "メール送信" : "待機",
        step_order: maxStepOrder + 1,
        step_type: newStepType,
        delay_value: delayMinutes,
        delay_unit: newStepDelayUnit,
        template_id:
          newStepType === "email" && newStepTemplateId
            ? newStepTemplateId
            : undefined,
      };

      console.log("Creating step with data:", data);

      await sequenceService.createSequenceStep(sequenceId, data);
      await loadSequence();

      // フォームをリセット
      showAddStep = false;
      newStepType = "email";
      newStepTemplateId = "";
      newStepDelayMinutes = 60;
      newStepDelayUnit = "hours";
    } catch (err) {
      error =
        err instanceof Error ? err.message : "ステップの追加に失敗しました";
    }
  }

  async function handleDeleteStep(stepId: number) {
    if (!confirm("このステップを削除してもよろしいですか？")) {
      return;
    }

    try {
      await sequenceService.deleteSequenceStep(sequenceId, stepId);
      await loadSequence();
    } catch (err) {
      error =
        err instanceof Error ? err.message : "ステップの削除に失敗しました";
    }
  }

  function getStepIcon(stepType: string) {
    switch (stepType) {
      case "email":
        return `<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
        </svg>`;
      case "wait":
        return `<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>`;
      default:
        return `<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>`;
    }
  }

  function getTemplateName(templateId: string | undefined): string {
    if (!templateId) return "未設定";
    const template = templates.find((t) => t.id === templateId);
    return template ? template.name : `テンプレート #${templateId}`;
  }
</script>

<div class="container mx-auto px-4 py-8">
  {#if loading}
    <div class="flex justify-center items-center h-64">
      <div
        class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"
      ></div>
    </div>
  {:else if sequence}
    <div class="mb-6 flex justify-between items-center">
      <h1 class="text-3xl font-bold">シーケンスを編集</h1>
      <button
        on:click={() => goto(`/sequences/${sequenceId}`)}
        class="text-gray-600 hover:text-gray-800"
      >
        詳細表示に戻る
      </button>
    </div>

    {#if error}
      <div
        class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4"
      >
        {error}
      </div>
    {/if}

    {#if successMessage}
      <div
        class="bg-green-100 border border-green-400 text-green-700 px-4 py-3 rounded mb-4"
      >
        {successMessage}
      </div>
    {/if}

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
      <!-- 基本情報 -->
      <div class="bg-white shadow rounded-lg p-6">
        <h2 class="text-xl font-semibold mb-4">基本情報</h2>

        <form on:submit|preventDefault={handleSave} class="space-y-4">
          <div>
            <label
              for="name"
              class="block text-sm font-medium text-gray-700 mb-1"
            >
              シーケンス名
            </label>
            <input
              id="name"
              type="text"
              bind:value={name}
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              disabled={saving}
            />
          </div>

          <div>
            <label
              for="description"
              class="block text-sm font-medium text-gray-700 mb-1"
            >
              説明
            </label>
            <textarea
              id="description"
              bind:value={description}
              rows="3"
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              disabled={saving}
            />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">
              トリガータイプ
            </label>
            <p class="text-gray-600">
              {sequenceService.formatTriggerType(sequence.trigger_type)}
            </p>
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">
              ステータス
            </label>
            <select
              bind:value={status}
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              disabled={saving}
            >
              <option value="draft">下書き</option>
              <option value="active">実行中</option>
              <option value="paused">一時停止</option>
              <option value="archived">アーカイブ済み</option>
            </select>
          </div>

          <div class="pt-4">
            <button
              type="submit"
              class="w-full px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 disabled:opacity-50"
              disabled={saving}
            >
              {saving ? "保存中..." : "保存"}
            </button>
          </div>
        </form>
      </div>

      <!-- ステップビルダー -->
      <div class="bg-white shadow rounded-lg p-6">
        <div class="flex justify-between items-center mb-4">
          <h2 class="text-xl font-semibold">ステップ</h2>
          {#if !showAddStep}
            <button
              on:click={() => (showAddStep = true)}
              class="text-blue-600 hover:text-blue-800 text-sm font-medium"
            >
              + ステップを追加
            </button>
          {/if}
        </div>

        <!-- ステップリスト -->
        <div class="space-y-3">
          {#each sortedSteps as step, index}
            <div class="border border-gray-200 rounded-lg p-4">
              <div class="flex items-start justify-between">
                <div class="flex items-start space-x-3">
                  <div class="flex-shrink-0 mt-1">
                    <div
                      class="w-8 h-8 rounded-full bg-gray-100 flex items-center justify-center text-sm font-medium"
                    >
                      {step.step_order}
                    </div>
                  </div>
                  <div>
                    <div class="flex items-center space-x-2">
                      <span class="inline-flex">
                        {@html getStepIcon(step.step_type)}
                      </span>
                      <span class="font-medium">
                        {sequenceService.formatStepType(step.step_type)}
                      </span>
                    </div>
                    {#if step.step_type === "email"}
                      <p class="text-sm text-gray-600 mt-1">
                        テンプレート: {getTemplateName(step.template_id)}
                      </p>
                    {/if}
                    {#if step.delay_value > 0}
                      <p class="text-sm text-gray-600 mt-1">
                        待機時間: {sequenceService.formatDelay(
                          step.delay_value,
                          step.delay_unit,
                        )}
                      </p>
                    {/if}
                  </div>
                </div>
                <button
                  on:click={() => handleDeleteStep(step.id)}
                  class="text-red-600 hover:text-red-800"
                  title="削除"
                >
                  <svg
                    class="h-5 w-5"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
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
            </div>
          {/each}

          {#if showAddStep}
            <div class="border-2 border-dashed border-gray-300 rounded-lg p-4">
              <h3 class="font-medium mb-3">新しいステップを追加</h3>

              <div class="space-y-3">
                <div>
                  <label class="block text-sm font-medium text-gray-700 mb-1">
                    ステップタイプ
                  </label>
                  <select
                    bind:value={newStepType}
                    class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  >
                    <option value="email">メール送信</option>
                    <option value="wait">待機</option>
                  </select>
                </div>

                {#if newStepType === "email"}
                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">
                      メールテンプレート
                    </label>
                    <select
                      bind:value={newStepTemplateId}
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    >
                      <option value="">選択してください</option>
                      {#each templates as template}
                        <option value={template.id}>{template.name}</option>
                      {/each}
                    </select>
                  </div>
                {/if}

                <div>
                  <label class="block text-sm font-medium text-gray-700 mb-1">
                    待機時間
                  </label>
                  <div class="flex space-x-2">
                    <input
                      type="number"
                      bind:value={newStepDelayMinutes}
                      min="0"
                      class="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    <select
                      bind:value={newStepDelayUnit}
                      class="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    >
                      <option value="minutes">分</option>
                      <option value="hours">時間</option>
                      <option value="days">日</option>
                    </select>
                  </div>
                </div>

                <div class="flex space-x-2 pt-2">
                  <button
                    on:click={handleAddStep}
                    class="flex-1 px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600"
                    disabled={newStepType === "email" && !newStepTemplateId}
                  >
                    追加
                  </button>
                  <button
                    on:click={() => (showAddStep = false)}
                    class="flex-1 px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-50"
                  >
                    キャンセル
                  </button>
                </div>
              </div>
            </div>
          {/if}

          {#if sortedSteps.length === 0 && !showAddStep}
            <div class="text-center py-8 text-gray-500">
              <p>ステップがまだ設定されていません</p>
              <button
                on:click={() => (showAddStep = true)}
                class="mt-2 text-blue-600 hover:text-blue-800"
              >
                最初のステップを追加
              </button>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>
