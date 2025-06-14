<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { campaignService } from "$lib/services/campaignService";
  import type { CampaignListResponse, Campaign } from "$lib/types/campaign";
  import { CampaignStatus } from "$lib/types/campaign";
  import { PlusIcon, RefreshCwIcon, SearchIcon } from "lucide-svelte";

  // 状態管理
  let isLoading = true;
  let error: string | null = null;
  let campaignList: Campaign[] = [];
  let total = 0;
  let limit = 10;
  let offset = 0;
  let searchTerm = "";
  let currentStatus: CampaignStatus | undefined = undefined;

  // キャンペーン一覧を取得
  async function fetchCampaigns() {
    isLoading = true;
    error = null;

    try {
      const response: CampaignListResponse =
        await campaignService.listCampaigns(limit, offset, currentStatus);
      campaignList = response.campaigns;
      total = response.total;
    } catch (err) {
      console.error("キャンペーン一覧取得エラー:", err);
      error = err instanceof Error ? err.message : "不明なエラーが発生しました";
    } finally {
      isLoading = false;
    }
  }

  // ステータスでフィルタリング
  function filterByStatus(status?: CampaignStatus | string) {
    currentStatus = status;
    offset = 0; // 最初のページに戻る
    fetchCampaigns();
  }

  // キャンペーンを削除
  async function deleteCampaign(id: string) {
    if (!confirm("このキャンペーンを削除してもよろしいですか？")) {
      return;
    }

    try {
      await campaignService.deleteCampaign(id);
      // 成功したら一覧を再読み込み
      fetchCampaigns();
    } catch (err) {
      console.error("キャンペーン削除エラー:", err);
      error = err instanceof Error ? err.message : "不明なエラーが発生しました";
    }
  }

  // 初期読み込み
  onMount(() => {
    fetchCampaigns();
  });

  // 検索フィルター関数
  $: filteredCampaigns = searchTerm
    ? campaignList.filter(
        (campaign) =>
          campaign.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
          campaign.description
            ?.toLowerCase()
            .includes(searchTerm.toLowerCase()) ||
          false,
      )
    : campaignList;

  // キャンペーンステータスのフォーマット
  function formatStatus(status: CampaignStatus): string {
    const statusMap: Record<CampaignStatus, string> = {
      [CampaignStatus.DRAFT]: "下書き",
      [CampaignStatus.SCHEDULED]: "予定済み",
      [CampaignStatus.SENDING]: "送信中",
      [CampaignStatus.SENT]: "送信済み",
      [CampaignStatus.CANCELED]: "キャンセル",
    };
    return statusMap[status] || status;
  }

  // ステータスに応じたバッジスタイルを取得
  function getStatusBadgeClass(status: CampaignStatus): string {
    switch (status) {
      case CampaignStatus.DRAFT:
        return "badge badge-info";
      case CampaignStatus.SCHEDULED:
        return "badge badge-info";
      case CampaignStatus.SENDING:
        return "badge badge-warning";
      case CampaignStatus.SENT:
        return "badge badge-success";
      case CampaignStatus.CANCELED:
        return "badge badge-error";
      default:
        return "badge";
    }
  }

  // 日付フォーマット
  function formatDate(dateString?: string): string {
    if (!dateString) return "未設定";
    const date = new Date(dateString);
    return new Intl.DateTimeFormat("ja-JP", {
      year: "numeric",
      month: "numeric",
      day: "numeric",
      hour: "numeric",
      minute: "numeric",
    }).format(date);
  }
</script>

<div class="section animate-in">
  <div class="container-wide">
    <div class="flex justify-between items-center mb-12">
      <div>
        <h1 class="page-header">キャンペーン管理</h1>
        <p class="page-subtitle">メールキャンペーンの作成・配信・分析</p>
      </div>
      <a href="/campaigns/new" class="btn-primary">
        <PlusIcon class="w-5 h-5 mr-2" />
        新規キャンペーン作成
      </a>
    </div>

    <!-- フィルター・検索 -->
    <div class="card mb-8">
      <div
        class="flex flex-col md:flex-row md:items-center justify-between gap-6"
      >
        <!-- ステータスフィルター -->
        <div class="flex flex-wrap gap-3">
          <button
            class={!currentStatus
              ? "btn-primary btn-sm"
              : "btn-secondary btn-sm"}
            on:click={() => filterByStatus(undefined)}
          >
            全て
          </button>
          {#each Object.values(CampaignStatus) as status}
            <button
              class={currentStatus === status
                ? "btn-primary btn-sm"
                : "btn-secondary btn-sm"}
              on:click={() => filterByStatus(status)}
            >
              {formatStatus(status)}
            </button>
          {/each}
        </div>

        <!-- 検索ボックス -->
        <div class="relative md:w-80">
          <div
            class="absolute inset-y-0 left-0 flex items-center pl-5 pointer-events-none"
          >
            <SearchIcon class="w-5 h-5 text-gray-400" />
          </div>
          <input
            type="text"
            class="input-field pl-12"
            placeholder="キャンペーン名、説明で検索..."
            bind:value={searchTerm}
          />
        </div>
      </div>
    </div>

    <!-- エラーメッセージ -->
    {#if error}
      <div class="card bg-red-50 border-red-100 mb-6">
        <p class="text-red-600 font-light">{error}</p>
      </div>
    {/if}

    <!-- 読み込み中表示 -->
    {#if isLoading}
      <div class="text-center py-24">
        <div class="inline-block">
          <div
            class="w-12 h-12 border-2 border-gray-900 border-t-transparent rounded-full animate-spin"
          ></div>
        </div>
        <p class="mt-4 text-gray-600 font-light">読み込み中...</p>
      </div>
    {:else if filteredCampaigns.length === 0}
      <div class="text-center py-24">
        <div class="max-w-sm mx-auto">
          <div
            class="w-20 h-20 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-6"
          >
            <svg
              class="w-10 h-10 text-gray-400"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="1.5"
                d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
              />
            </svg>
          </div>
          {#if searchTerm}
            <h3 class="text-xl font-light text-gray-900 mb-2">
              検索結果がありません
            </h3>
            <p class="text-gray-600 font-light">
              別のキーワードで検索してください
            </p>
          {:else if currentStatus}
            <h3 class="text-xl font-light text-gray-900 mb-2">
              「{formatStatus(currentStatus)}」のキャンペーンはありません
            </h3>
            <p class="text-gray-600 font-light">
              他のステータスで絞り込んでみてください
            </p>
          {:else}
            <h3 class="text-xl font-light text-gray-900 mb-2">
              キャンペーンがありません
            </h3>
            <p class="text-gray-600 font-light mb-8">
              最初のキャンペーンを作成しましょう
            </p>
            <a href="/campaigns/new" class="btn-primary">
              最初のキャンペーンを作成
            </a>
          {/if}
        </div>
      </div>
    {:else}
      <!-- キャンペーン一覧 -->
      <div class="card overflow-hidden p-0">
        <div class="overflow-x-auto">
          <table class="table-minimal">
            <thead>
              <tr>
                <th>キャンペーン名</th>
                <th>ステータス</th>
                <th>予定日時</th>
                <th>配信数</th>
                <th>開封率</th>
                <th>作成日</th>
                <th>アクション</th>
              </tr>
            </thead>
            <tbody>
              {#each filteredCampaigns as campaign}
                <tr>
                  <td>
                    <div class="font-light text-gray-900">
                      {campaign.name}
                    </div>
                    {#if campaign.description}
                      <div
                        class="text-sm text-gray-500 truncate max-w-xs font-light"
                      >
                        {campaign.description}
                      </div>
                    {/if}
                  </td>
                  <td>
                    <span class={getStatusBadgeClass(campaign.status)}>
                      {formatStatus(campaign.status)}
                    </span>
                  </td>
                  <td class="text-sm text-gray-600 font-light">
                    {formatDate(campaign.scheduled_at)}
                  </td>
                  <td class="text-sm text-gray-600 font-light">
                    {campaign.stats.recipient_count}
                  </td>
                  <td class="text-sm text-gray-600 font-light">
                    {campaign.stats.sent_count > 0
                      ? `${Math.round(campaign.stats.open_rate * 100)}%`
                      : "0%"}
                  </td>
                  <td class="text-sm text-gray-600 font-light">
                    {formatDate(campaign.created_at)}
                  </td>
                  <td>
                    <div class="flex items-center space-x-2">
                      <a
                        href={`/campaigns/${campaign.id}`}
                        class="icon-button"
                        title="詳細"
                      >
                        <svg
                          class="w-4 h-4"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24"
                        >
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                          />
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                          />
                        </svg>
                      </a>
                      <a
                        href={`/campaigns/${campaign.id}/edit`}
                        class="icon-button"
                        title="編集"
                      >
                        <svg
                          class="w-4 h-4"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24"
                        >
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                          />
                        </svg>
                      </a>
                      {#if campaign.status === CampaignStatus.DRAFT}
                        <button
                          class="icon-button hover:bg-red-50 hover:text-red-600"
                          on:click={() => deleteCampaign(campaign.id)}
                          title="削除"
                        >
                          <svg
                            class="w-4 h-4"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                          >
                            <path
                              stroke-linecap="round"
                              stroke-linejoin="round"
                              stroke-width="2"
                              d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                            />
                          </svg>
                        </button>
                      {/if}
                    </div>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>

      <!-- ページネーション -->
      {#if total > limit}
        <div class="mt-6 flex items-center justify-between">
          <div class="text-sm text-gray-600 font-light">
            全 <span class="font-normal">{total}</span> 件中
            <span class="font-normal">{offset + 1}</span> から
            <span class="font-normal">{Math.min(offset + limit, total)}</span> 件を表示
          </div>
          <div class="flex space-x-3">
            <button
              class="btn-secondary btn-sm"
              disabled={offset === 0}
              on:click={() => {
                offset = Math.max(0, offset - limit);
                fetchCampaigns();
              }}
            >
              <svg
                class="w-4 h-4 mr-1"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M15 19l-7-7 7-7"
                />
              </svg>
              前へ
            </button>
            <button
              class="btn-secondary btn-sm"
              disabled={offset + limit >= total}
              on:click={() => {
                offset = offset + limit;
                fetchCampaigns();
              }}
            >
              次へ
              <svg
                class="w-4 h-4 ml-1"
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
      {/if}
    {/if}

    <!-- 更新ボタン -->
    <div class="flex justify-center mt-12">
      <button
        class="btn-secondary"
        on:click={fetchCampaigns}
        disabled={isLoading}
      >
        <RefreshCwIcon class="w-5 h-5 mr-2 {isLoading ? 'animate-spin' : ''}" />
        {isLoading ? "更新中..." : "更新"}
      </button>
    </div>
  </div>
</div>

<style>
  @keyframes fadeInUp {
    from {
      opacity: 0;
      transform: translateY(20px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .animate-in {
    animation: fadeInUp 0.6s ease-out;
  }
</style>
