<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { sequenceService } from "$lib/services/sequenceService";
  import type { Sequence } from "$lib/types/sequence";

  let sequences: Sequence[] = [];
  let loading = true;
  let error = "";
  let searchTerm = "";

  $: filteredSequences = sequences.filter(
    (sequence) =>
      sequence.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      sequence.description?.toLowerCase().includes(searchTerm.toLowerCase()),
  );

  onMount(async () => {
    await loadSequences();
  });

  async function loadSequences() {
    try {
      loading = true;
      sequences = await sequenceService.listSequences();
    } catch (err) {
      error =
        err instanceof Error
          ? err.message
          : "シーケンスの読み込みに失敗しました";
    } finally {
      loading = false;
    }
  }

  async function handleStatusChange(
    sequence: Sequence,
    newStatus: "active" | "paused",
  ) {
    try {
      if (newStatus === "active") {
        await sequenceService.activateSequence(sequence.id);
      } else {
        await sequenceService.pauseSequence(sequence.id);
      }
      await loadSequences();
    } catch (err) {
      error =
        err instanceof Error ? err.message : "ステータスの更新に失敗しました";
    }
  }

  async function handleDelete(id: number) {
    if (!confirm("このシーケンスを削除してもよろしいですか？")) {
      return;
    }

    try {
      await sequenceService.deleteSequence(id);
      await loadSequences();
    } catch (err) {
      error =
        err instanceof Error ? err.message : "シーケンスの削除に失敗しました";
    }
  }

  function getStatusBadgeClass(status: string) {
    switch (status) {
      case "draft":
        return "badge-primary";
      case "active":
        return "badge-success";
      case "paused":
        return "badge-warning";
      case "archived":
        return "badge-neutral";
      default:
        return "badge-neutral";
    }
  }

  function getStatusText(status: string) {
    switch (status) {
      case "draft":
        return "下書き";
      case "active":
        return "実行中";
      case "paused":
        return "一時停止";
      case "archived":
        return "アーカイブ済み";
      default:
        return status;
    }
  }
</script>

<div class="section animate-in">
  <div class="container-wide">
    <div class="flex justify-between items-center mb-12">
      <div>
        <h1 class="page-header">シーケンス</h1>
        <p class="page-subtitle">自動化されたメールシーケンスを管理</p>
      </div>
      <button on:click={() => goto("/sequences/new")} class="btn-primary">
        <svg
          class="w-5 h-5 mr-2"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 4v16m8-8H4"
          />
        </svg>
        新規シーケンス作成
      </button>
    </div>

    {#if error}
      <div class="alert alert-error mb-6">
        <svg
          class="w-5 h-5 mr-2"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
        {error}
      </div>
    {/if}

    <div class="mb-8">
      <div class="relative max-w-md">
        <svg
          class="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
          />
        </svg>
        <input
          type="text"
          bind:value={searchTerm}
          placeholder="シーケンスを検索..."
          class="form-input pl-10"
        />
      </div>
    </div>

    {#if loading}
      <div class="flex justify-center items-center h-64">
        <div
          class="animate-spin rounded-full h-12 w-12 border-4 border-gray-900 border-t-transparent"
        ></div>
      </div>
    {:else if filteredSequences.length === 0}
      <div class="card">
        <div class="text-center py-16">
          <div
            class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-gray-100 mb-4"
          >
            <svg
              class="w-8 h-8 text-gray-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
              />
            </svg>
          </div>
          <h3 class="text-lg font-medium text-gray-900 mb-2">
            シーケンスがありません
          </h3>
          <p class="text-gray-500 font-light mb-6">
            {searchTerm
              ? "検索条件に一致するシーケンスが見つかりませんでした。"
              : "最初のシーケンスを作成して、メールマーケティングを自動化しましょう。"}
          </p>
          {#if !searchTerm}
            <button on:click={() => goto("/sequences/new")} class="btn-primary">
              <svg
                class="w-5 h-5 mr-2"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 4v16m8-8H4"
                />
              </svg>
              新規シーケンス作成
            </button>
          {/if}
        </div>
      </div>
    {:else}
      <div class="grid gap-4">
        {#each filteredSequences as sequence, i}
          <div
            class="card hover:shadow-lg transition-shadow duration-200 animate-fade-in"
            style="animation-delay: {i * 50}ms"
          >
            <div class="flex items-start justify-between">
              <div class="flex items-start">
                <div class="flex-shrink-0">
                  <div
                    class="w-12 h-12 rounded-full bg-gray-100 flex items-center justify-center"
                  >
                    <svg
                      class="w-6 h-6 text-gray-600"
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
                <div class="ml-4">
                  <h3 class="text-lg font-medium text-gray-900">
                    <a
                      href="/sequences/{sequence.id}"
                      class="hover:text-gray-700 transition-colors"
                    >
                      {sequence.name}
                    </a>
                  </h3>
                  {#if sequence.description}
                    <p class="text-gray-600 font-light mt-1">
                      {sequence.description}
                    </p>
                  {/if}
                  <div class="mt-3 flex items-center gap-4 text-sm">
                    <span class={getStatusBadgeClass(sequence.status)}>
                      {getStatusText(sequence.status)}
                    </span>
                    <span class="text-gray-500 font-light">
                      トリガー: {sequenceService.formatTriggerType(
                        sequence.trigger_type,
                      )}
                    </span>
                  </div>
                </div>
              </div>
              <div class="flex items-center gap-2 ml-4">
                {#if sequence.status === "active"}
                  <button
                    on:click={() => handleStatusChange(sequence, "paused")}
                    class="icon-button"
                    title="一時停止"
                  >
                    <svg
                      class="w-5 h-5"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z"
                      />
                    </svg>
                  </button>
                {:else if sequence.status === "paused"}
                  <button
                    on:click={() => handleStatusChange(sequence, "active")}
                    class="icon-button"
                    title="再開"
                  >
                    <svg
                      class="w-5 h-5"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
                      />
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                      />
                    </svg>
                  </button>
                {/if}
                <a
                  href="/sequences/{sequence.id}/edit"
                  class="icon-button"
                  title="編集"
                >
                  <svg
                    class="w-5 h-5"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                    />
                  </svg>
                </a>
                <button
                  on:click={() => handleDelete(sequence.id)}
                  class="icon-button hover:text-red-600"
                  title="削除"
                >
                  <svg
                    class="w-5 h-5"
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
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .animate-fade-in {
    animation: fade-in 0.3s ease-out forwards;
    opacity: 0;
  }
</style>
