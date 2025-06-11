<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { sequenceService } from "$lib/services/sequenceService";
  import type {
    SequenceWithSteps,
    SequenceStepWithTemplate,
  } from "$lib/types/sequence";

  let sequence: SequenceWithSteps | null = null;
  let loading = true;
  let error = "";

  $: sequenceId = $page.params.id;

  onMount(async () => {
    await loadSequence();
  });

  async function loadSequence() {
    try {
      loading = true;
      sequence = await sequenceService.getSequence(sequenceId);
    } catch (err) {
      error =
        err instanceof Error
          ? err.message
          : "シーケンスの読み込みに失敗しました";
    } finally {
      loading = false;
    }
  }

  function getStepIcon(stepType: string) {
    switch (stepType) {
      case "email":
        return `<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
        </svg>`;
      case "wait":
        return `<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>`;
      case "condition":
        return `<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
        </svg>`;
      default:
        return `<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>`;
    }
  }

  function getStatusBadgeClass(status: string) {
    switch (status) {
      case "draft":
        return "bg-blue-100 text-blue-800";
      case "active":
        return "bg-green-100 text-green-800";
      case "paused":
        return "bg-yellow-100 text-yellow-800";
      case "archived":
        return "bg-gray-100 text-gray-800";
      default:
        return "bg-gray-100 text-gray-800";
    }
  }

  function getStatusText(status: string) {
    switch (status) {
      case "draft":
        return "下書き";
      case "active":
        return "実行中";
      case "paused":
        return "一憂停止";
      case "archived":
        return "アーカイブ済み";
      default:
        return status;
    }
  }
</script>

<div class="container mx-auto px-4 py-8">
  {#if loading}
    <div class="flex justify-center items-center h-64">
      <div
        class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"
      ></div>
    </div>
  {:else if error}
    <div
      class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4"
    >
      {error}
    </div>
  {:else if sequence}
    <div class="mb-6">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-3xl font-bold">{sequence.name}</h1>
          {#if sequence.description}
            <p class="text-gray-600 mt-2">{sequence.description}</p>
          {/if}
          <div class="mt-4 flex items-center space-x-4">
            <span
              class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium {getStatusBadgeClass(
                sequence.status,
              )}"
            >
              {getStatusText(sequence.status)}
            </span>
            <span class="text-sm text-gray-500">
              トリガー: {sequenceService.formatTriggerType(
                sequence.trigger_type,
              )}
            </span>
          </div>
        </div>
        <div class="flex space-x-2">
          <button
            on:click={() => goto(`/sequences/${sequence.id}/edit`)}
            class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
          >
            編集
          </button>
          <button
            on:click={() => goto("/sequences")}
            class="bg-gray-300 hover:bg-gray-400 text-gray-800 font-bold py-2 px-4 rounded"
          >
            一覧に戻る
          </button>
        </div>
      </div>
    </div>

    <div class="bg-white shadow rounded-lg p-6">
      <h2 class="text-xl font-semibold mb-4">シーケンスステップ</h2>

      {#if sequence.steps && sequence.steps.length > 0}
        <div class="space-y-4">
          <!-- トリガー -->
          <div class="flex items-start">
            <div class="flex-shrink-0">
              <div
                class="flex items-center justify-center h-10 w-10 rounded-full bg-blue-100 text-blue-600"
              >
                <svg
                  class="h-6 w-6"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6"
                  />
                </svg>
              </div>
            </div>
            <div class="ml-4 flex-1">
              <div class="text-sm font-medium text-gray-900">トリガー</div>
              <div class="text-sm text-gray-500">
                {sequenceService.formatTriggerType(sequence.trigger_type)}
              </div>
            </div>
          </div>

          <!-- 接続線 -->
          <div class="ml-5 h-8 border-l-2 border-gray-300"></div>

          <!-- ステップ -->
          {#each sequence.steps.sort((a, b) => a.step_order - b.step_order) as step, index}
            <div class="flex items-start">
              <div class="flex-shrink-0">
                <div
                  class="flex items-center justify-center h-10 w-10 rounded-full {step.step_type ===
                  'email'
                    ? 'bg-green-100 text-green-600'
                    : step.step_type === 'wait'
                      ? 'bg-yellow-100 text-yellow-600'
                      : 'bg-purple-100 text-purple-600'}"
                >
                  {@html getStepIcon(step.step_type)}
                </div>
              </div>
              <div class="ml-4 flex-1">
                <div class="bg-gray-50 rounded-lg p-4">
                  <div class="flex items-center justify-between mb-2">
                    <h3 class="text-sm font-medium text-gray-900">
                      ステップ {step.step_order}: {sequenceService.formatStepType(
                        step.step_type,
                      )}
                    </h3>
                  </div>

                  {#if step.step_type === "email" && step.template_id}
                    <div class="text-sm text-gray-600">
                      <p>テンプレート: #{step.template_id}</p>
                    </div>
                  {:else if step.step_type === "wait"}
                    <div class="text-sm text-gray-600">
                      <p>
                        待機時間: {sequenceService.formatDelay(
                          step.delay_value,
                          step.delay_unit,
                        )}
                      </p>
                    </div>
                  {:else if step.step_type === "condition" && step.conditions}
                    <div class="text-sm text-gray-600">
                      <p>条件: {step.conditions.length}個の条件</p>
                    </div>
                  {/if}
                </div>
              </div>
            </div>

            {#if index < sequence.steps.length - 1}
              <!-- 接続線 -->
              <div class="ml-5 h-8 border-l-2 border-gray-300"></div>
            {/if}
          {/each}

          <!-- 終了 -->
          <div class="ml-5 h-8 border-l-2 border-gray-300"></div>
          <div class="flex items-start">
            <div class="flex-shrink-0">
              <div
                class="flex items-center justify-center h-10 w-10 rounded-full bg-gray-100 text-gray-600"
              >
                <svg
                  class="h-6 w-6"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
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
            <div class="ml-4 flex-1">
              <div class="text-sm font-medium text-gray-900">完了</div>
            </div>
          </div>
        </div>
      {:else}
        <div class="text-center py-8">
          <svg
            class="mx-auto h-12 w-12 text-gray-400"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 6v6m0 0v6m0-6h6m-6 0H6"
            />
          </svg>
          <p class="mt-2 text-sm text-gray-500">
            ステップがまだ設定されていません
          </p>
          <button
            on:click={() => goto(`/sequences/${sequence.id}/edit`)}
            class="mt-4 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700"
          >
            ステップを追加
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>
