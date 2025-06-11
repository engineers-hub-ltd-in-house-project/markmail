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
        return "一時停止";
      case "archived":
        return "アーカイブ済み";
      default:
        return status;
    }
  }
</script>

<div class="container mx-auto px-4 py-8">
  <div class="flex justify-between items-center mb-6">
    <h1 class="text-3xl font-bold">シーケンス</h1>
    <button
      on:click={() => goto("/sequences/new")}
      class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
    >
      新規シーケンス作成
    </button>
  </div>

  {#if error}
    <div
      class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4"
    >
      {error}
    </div>
  {/if}

  <div class="mb-6">
    <input
      type="text"
      bind:value={searchTerm}
      placeholder="シーケンスを検索..."
      class="w-full md:w-1/3 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
    />
  </div>

  {#if loading}
    <div class="flex justify-center items-center h-64">
      <div
        class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"
      ></div>
    </div>
  {:else if filteredSequences.length === 0}
    <div class="text-center py-12">
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
          d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
        />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900">
        シーケンスがありません
      </h3>
      <p class="mt-1 text-sm text-gray-500">
        {searchTerm
          ? "検索条件に一致するシーケンスが見つかりませんでした。"
          : "最初のシーケンスを作成しましょう。"}
      </p>
      {#if !searchTerm}
        <div class="mt-6">
          <button
            on:click={() => goto("/sequences/new")}
            class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700"
          >
            <svg
              class="-ml-1 mr-2 h-5 w-5"
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
      {/if}
    </div>
  {:else}
    <div class="bg-white shadow overflow-hidden sm:rounded-md">
      <ul class="divide-y divide-gray-200">
        {#each filteredSequences as sequence}
          <li>
            <div class="px-4 py-4 sm:px-6 hover:bg-gray-50">
              <div class="flex items-center justify-between">
                <div class="flex items-center">
                  <div class="flex-shrink-0">
                    <svg
                      class="h-10 w-10 text-gray-400"
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
                  <div class="ml-4">
                    <div class="text-sm font-medium text-gray-900">
                      <a
                        href="/sequences/{sequence.id}"
                        class="hover:text-blue-600"
                      >
                        {sequence.name}
                      </a>
                    </div>
                    {#if sequence.description}
                      <div class="text-sm text-gray-500">
                        {sequence.description}
                      </div>
                    {/if}
                    <div class="mt-2 flex items-center text-sm text-gray-500">
                      <span
                        class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {getStatusBadgeClass(
                          sequence.status,
                        )}"
                      >
                        {getStatusText(sequence.status)}
                      </span>
                      <span class="ml-3">
                        トリガー: {sequenceService.formatTriggerType(
                          sequence.trigger_type,
                        )}
                      </span>
                    </div>
                  </div>
                </div>
                <div class="flex items-center space-x-2">
                  {#if sequence.status === "active"}
                    <button
                      on:click={() => handleStatusChange(sequence, "paused")}
                      class="text-yellow-600 hover:text-yellow-900"
                      title="一時停止"
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
                          d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z"
                        />
                      </svg>
                    </button>
                  {:else if sequence.status === "paused"}
                    <button
                      on:click={() => handleStatusChange(sequence, "active")}
                      class="text-green-600 hover:text-green-900"
                      title="再開"
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
                    class="text-indigo-600 hover:text-indigo-900"
                    title="編集"
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
                        d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                      />
                    </svg>
                  </a>
                  <button
                    on:click={() => handleDelete(sequence.id)}
                    class="text-red-600 hover:text-red-900"
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
            </div>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</div>
