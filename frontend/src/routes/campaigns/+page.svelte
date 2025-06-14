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
    const baseClass = "px-2 py-1 text-xs font-medium rounded-full";

    switch (status) {
      case CampaignStatus.DRAFT:
        return `${baseClass} bg-gray-200 text-gray-800`;
      case CampaignStatus.SCHEDULED:
        return `${baseClass} bg-blue-100 text-blue-800`;
      case CampaignStatus.SENDING:
        return `${baseClass} bg-yellow-100 text-yellow-800`;
      case CampaignStatus.SENT:
        return `${baseClass} bg-green-100 text-green-800`;
      case CampaignStatus.CANCELED:
        return `${baseClass} bg-red-100 text-red-800`;
      default:
        return baseClass;
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

<div class="max-w-7xl mx-auto px-4 py-8">
  <div class="flex items-center justify-between mb-6">
    <h1 class="text-3xl font-bold text-gray-900">キャンペーン管理</h1>
    <a
      href="/campaigns/new"
      class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg font-medium flex items-center"
    >
      <PlusIcon class="w-4 h-4 mr-2" />
      新規キャンペーン作成
    </a>
  </div>

  <!-- フィルター・検索 -->
  <div class="bg-white p-4 rounded-lg shadow-sm border border-gray-200 mb-6">
    <div
      class="flex flex-col md:flex-row md:items-center justify-between gap-4"
    >
      <!-- ステータスフィルター -->
      <div class="flex flex-wrap gap-2">
        <button
          class={`px-3 py-1.5 text-sm font-medium rounded-md ${!currentStatus ? "bg-blue-600 text-white" : "bg-gray-100 text-gray-700 hover:bg-gray-200"}`}
          on:click={() => filterByStatus(undefined)}
        >
          全て
        </button>
        {#each Object.values(CampaignStatus) as status}
          <button
            class={`px-3 py-1.5 text-sm font-medium rounded-md ${currentStatus === status ? "bg-blue-600 text-white" : "bg-gray-100 text-gray-700 hover:bg-gray-200"}`}
            on:click={() => filterByStatus(status)}
          >
            {formatStatus(status)}
          </button>
        {/each}
      </div>

      <!-- 検索ボックス -->
      <div class="relative">
        <div
          class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none"
        >
          <SearchIcon class="w-4 h-4 text-gray-500" />
        </div>
        <input
          type="text"
          class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full pl-10 p-2.5"
          placeholder="キャンペーン名、説明で検索..."
          bind:value={searchTerm}
        />
      </div>
    </div>
  </div>

  <!-- エラーメッセージ -->
  {#if error}
    <div
      class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4"
    >
      <p>{error}</p>
    </div>
  {/if}

  <!-- 読み込み中表示 -->
  {#if isLoading}
    <div class="text-center py-12">
      <div
        class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"
      ></div>
      <p class="mt-4 text-gray-600">読み込み中...</p>
    </div>
  {:else if filteredCampaigns.length === 0}
    <div
      class="bg-white p-8 rounded-lg shadow-sm border border-gray-200 text-center"
    >
      {#if searchTerm}
        <p class="text-lg text-gray-600">
          検索条件に一致するキャンペーンはありません。
        </p>
      {:else if currentStatus}
        <p class="text-lg text-gray-600">
          「{formatStatus(currentStatus)}」のキャンペーンはありません。
        </p>
      {:else}
        <p class="text-lg text-gray-600">
          キャンペーンはまだ作成されていません。
        </p>
        <a
          href="/campaigns/new"
          class="inline-block mt-4 bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg font-medium"
        >
          最初のキャンペーンを作成
        </a>
      {/if}
    </div>
  {:else}
    <!-- キャンペーン一覧 -->
    <div
      class="bg-white overflow-hidden shadow-sm rounded-lg border border-gray-200"
    >
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gray-50">
            <tr>
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >キャンペーン名</th
              >
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >ステータス</th
              >
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >予定日時</th
              >
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >配信数</th
              >
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >開封率</th
              >
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >作成日</th
              >
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >アクション</th
              >
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-200">
            {#each filteredCampaigns as campaign}
              <tr class="hover:bg-gray-50">
                <td class="px-6 py-4 whitespace-nowrap">
                  <div class="text-sm font-medium text-gray-900">
                    {campaign.name}
                  </div>
                  {#if campaign.description}
                    <div class="text-sm text-gray-500 truncate max-w-xs">
                      {campaign.description}
                    </div>
                  {/if}
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <span class={getStatusBadgeClass(campaign.status)}>
                    {formatStatus(campaign.status)}
                  </span>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {formatDate(campaign.scheduled_at)}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {campaign.stats.recipient_count}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {campaign.stats.sent_count > 0
                    ? `${Math.round(campaign.stats.open_rate * 100)}%`
                    : "0%"}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {formatDate(campaign.created_at)}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
                  <div class="flex space-x-2">
                    <a
                      href={`/campaigns/${campaign.id}`}
                      class="text-blue-600 hover:text-blue-900"
                    >
                      詳細
                    </a>
                    <a
                      href={`/campaigns/${campaign.id}/edit`}
                      class="text-indigo-600 hover:text-indigo-900"
                    >
                      編集
                    </a>
                    {#if campaign.status === CampaignStatus.DRAFT}
                      <button
                        class="text-red-600 hover:text-red-900"
                        on:click={() => deleteCampaign(campaign.id)}
                      >
                        削除
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
      <div class="mt-4 flex items-center justify-between">
        <div class="text-sm text-gray-700">
          全 <span class="font-medium">{total}</span> 件中
          <span class="font-medium">{offset + 1}</span> から
          <span class="font-medium">{Math.min(offset + limit, total)}</span> 件を表示
        </div>
        <div class="flex space-x-2">
          <button
            class="px-3 py-1.5 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
            disabled={offset === 0}
            on:click={() => {
              offset = Math.max(0, offset - limit);
              fetchCampaigns();
            }}
          >
            前へ
          </button>
          <button
            class="px-3 py-1.5 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
            disabled={offset + limit >= total}
            on:click={() => {
              offset = offset + limit;
              fetchCampaigns();
            }}
          >
            次へ
          </button>
        </div>
      </div>
    {/if}
  {/if}

  <!-- 更新ボタン -->
  <div class="flex justify-center mt-8">
    <button
      class="flex items-center px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
      on:click={fetchCampaigns}
      disabled={isLoading}
    >
      <RefreshCwIcon class="w-4 h-4 mr-2" />
      {isLoading ? "更新中..." : "更新"}
    </button>
  </div>
</div>
