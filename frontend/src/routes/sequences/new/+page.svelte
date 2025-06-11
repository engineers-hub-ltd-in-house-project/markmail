<script lang="ts">
  import { goto } from "$app/navigation";
  import { sequenceService } from "$lib/services/sequenceService";
  import type { CreateSequenceRequest, TriggerType } from "$lib/types/sequence";

  let name = "";
  let description = "";
  let triggerType: TriggerType = "manual";
  let error = "";
  let loading = false;

  const triggerTypes: {
    value: TriggerType;
    label: string;
    description: string;
  }[] = [
    {
      value: "manual",
      label: "手動",
      description: "手動で購読者を登録します",
    },
    {
      value: "subscriber_created",
      label: "購読者登録時",
      description: "新規購読者が登録された時に自動的に開始します",
    },
    {
      value: "form_submission",
      label: "フォーム送信時",
      description: "特定のフォームが送信された時に開始します",
    },
    {
      value: "tag_added",
      label: "タグ追加時",
      description: "購読者に特定のタグが追加された時に開始します",
    },
  ];

  async function handleSubmit() {
    if (!name.trim()) {
      error = "シーケンス名を入力してください";
      return;
    }

    try {
      loading = true;
      error = "";

      const data: CreateSequenceRequest = {
        name: name.trim(),
        description: description.trim() || undefined,
        trigger_type: triggerType,
      };

      const sequence = await sequenceService.createSequence(data);
      goto(`/sequences/${sequence.id}/edit`);
    } catch (err) {
      error =
        err instanceof Error ? err.message : "シーケンスの作成に失敗しました";
    } finally {
      loading = false;
    }
  }
</script>

<div class="container mx-auto px-4 py-8 max-w-2xl">
  <h1 class="text-3xl font-bold mb-6">新規シーケンス作成</h1>

  {#if error}
    <div
      class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4"
    >
      {error}
    </div>
  {/if}

  <form on:submit|preventDefault={handleSubmit} class="space-y-6">
    <div>
      <label for="name" class="block text-sm font-medium text-gray-700 mb-2">
        シーケンス名 <span class="text-red-500">*</span>
      </label>
      <input
        id="name"
        type="text"
        bind:value={name}
        placeholder="例: 新規登録者ウェルカムシリーズ"
        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        disabled={loading}
      />
    </div>

    <div>
      <label
        for="description"
        class="block text-sm font-medium text-gray-700 mb-2"
      >
        説明
      </label>
      <textarea
        id="description"
        bind:value={description}
        rows="3"
        placeholder="このシーケンスの目的や内容を記載してください"
        class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        disabled={loading}
      />
    </div>

    <div>
      <label class="block text-sm font-medium text-gray-700 mb-2">
        トリガータイプ <span class="text-red-500">*</span>
      </label>
      <div class="space-y-3">
        {#each triggerTypes as trigger}
          <label class="flex items-start cursor-pointer">
            <input
              type="radio"
              bind:group={triggerType}
              value={trigger.value}
              class="mt-1 mr-3"
              disabled={loading}
            />
            <div>
              <div class="font-medium">{trigger.label}</div>
              <div class="text-sm text-gray-600">{trigger.description}</div>
            </div>
          </label>
        {/each}
      </div>
    </div>

    <div class="flex justify-end space-x-3 pt-4">
      <button
        type="button"
        on:click={() => goto("/sequences")}
        class="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
        disabled={loading}
      >
        キャンセル
      </button>
      <button
        type="submit"
        class="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed"
        disabled={loading}
      >
        {#if loading}
          <span class="inline-flex items-center">
            <svg
              class="animate-spin -ml-1 mr-3 h-5 w-5 text-white"
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
              ></circle>
              <path
                class="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              ></path>
            </svg>
            作成中...
          </span>
        {:else}
          次へ（ステップ設定）
        {/if}
      </button>
    </div>
  </form>
</div>
