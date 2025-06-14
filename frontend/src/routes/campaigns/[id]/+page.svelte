<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { campaignService } from "$lib/services/campaignService";
  import type { Campaign } from "$lib/types/campaign";
  import { CampaignStatus } from "$lib/types/campaign";
  import {
    ArrowLeftIcon,
    Calendar,
    Send,
    ClockIcon,
    EditIcon,
    EyeIcon,
    Trash2Icon,
  } from "lucide-svelte";

  // URLパラメータからキャンペーンIDを取得
  const campaignId = $page.params.id;

  // 状態管理
  let isLoading = true;
  let isPreviewLoading = false;
  let isSending = false;
  let isScheduling = false;
  let error: string | null = null;
  let campaign: Campaign | null = null;
  let previewHtml: string | null = null;
  let showPreview = false;
  let showScheduleModal = false;
  let scheduleDate: string = "";
  let sendingProgress = 0;
  let subscriberCount = 0;

  // キャンペーン情報を取得
  async function fetchCampaign() {
    isLoading = true;
    error = null;

    try {
      campaign = await campaignService.getCampaign(campaignId);
      if (campaign.scheduled_at) {
        // ISO形式の日時をローカル日時のinput[type=datetime-local]形式に変換
        scheduleDate = new Date(campaign.scheduled_at)
          .toISOString()
          .slice(0, 16);
      } else {
        // デフォルトで1時間後を設定
        const defaultDate = new Date();
        defaultDate.setHours(defaultDate.getHours() + 1);
        scheduleDate = defaultDate.toISOString().slice(0, 16);
      }
    } catch (err) {
      console.error("キャンペーン取得エラー:", err);
      error = err instanceof Error ? err.message : "不明なエラーが発生しました";
    } finally {
      isLoading = false;
    }
  }

  // キャンペーンプレビューを取得
  async function fetchPreview() {
    if (!campaign) return;

    isPreviewLoading = true;
    error = null;

    try {
      const response = await campaignService.previewCampaign(campaignId);
      previewHtml = response.html;
      showPreview = true;
    } catch (err) {
      console.error("プレビュー取得エラー:", err);
      error =
        err instanceof Error ? err.message : "プレビューの取得に失敗しました";
    } finally {
      isPreviewLoading = false;
    }
  }

  // キャンペーン削除
  async function deleteCampaign() {
    if (!campaign) return;

    if (
      !confirm(
        "このキャンペーンを削除してもよろしいですか？この操作は取り消せません。",
      )
    ) {
      return;
    }

    error = null;

    try {
      await campaignService.deleteCampaign(campaignId);
      goto("/campaigns");
    } catch (err) {
      console.error("キャンペーン削除エラー:", err);
      error =
        err instanceof Error ? err.message : "キャンペーンの削除に失敗しました";
    }
  }

  // キャンペーン送信
  async function sendCampaign() {
    if (!campaign) return;

    // 購読者数を取得
    try {
      const subscribers =
        await campaignService.getCampaignSubscribers(campaignId);
      subscriberCount = subscribers.length;

      if (subscriberCount === 0) {
        error =
          "送信先の購読者が設定されていません。購読者を登録してからお試しください。";
        return;
      }

      if (
        !confirm(
          `このキャンペーンを${subscriberCount}人の購読者に今すぐ送信してもよろしいですか？`,
        )
      ) {
        return;
      }
    } catch (err) {
      console.error("購読者数取得エラー:", err);
      // エンドポイントが存在しない場合は、とりあえず送信を続行（後で修正）
      if (err instanceof Error && err.message.includes("404")) {
        if (!confirm("このキャンペーンを送信してもよろしいですか？")) {
          return;
        }
      } else {
        error = "購読者数の取得に失敗しました";
        return;
      }
    }

    error = null;
    isSending = true;
    sendingProgress = 0;

    try {
      // 送信進捗のシミュレーション（実際にはWebSocketやSSEで実装することを推奨）
      const progressInterval = setInterval(() => {
        if (sendingProgress < 90) {
          sendingProgress += Math.random() * 20;
          if (sendingProgress > 90) sendingProgress = 90;
        }
      }, 500);

      await campaignService.sendCampaign(campaignId);

      clearInterval(progressInterval);
      sendingProgress = 100;

      // 送信成功後、少し待ってからキャンペーン情報を再取得
      setTimeout(() => {
        fetchCampaign();
        isSending = false;
        sendingProgress = 0;
      }, 1500);
    } catch (err) {
      console.error("キャンペーン送信エラー:", err);
      isSending = false;
      sendingProgress = 0;

      // エラーメッセージの詳細化
      if (err instanceof Error) {
        if (err.message.includes("rate limit")) {
          error =
            "送信レート制限に達しました。しばらく待ってから再度お試しください。";
        } else if (err.message.includes("email service")) {
          error =
            "メール送信サービスでエラーが発生しました。管理者に連絡してください。";
        } else {
          error = err.message;
        }
      } else {
        error = "キャンペーンの送信に失敗しました";
      }
    }
  }

  // キャンペーンスケジュール
  async function scheduleCampaign() {
    if (!campaign || !scheduleDate) return;

    error = null;
    isScheduling = true;

    try {
      const scheduledAt = new Date(scheduleDate).toISOString();
      await campaignService.scheduleCampaign(campaignId, {
        scheduled_at: scheduledAt,
      });

      // スケジュール後、モーダルを閉じてキャンペーン情報を再取得
      showScheduleModal = false;
      fetchCampaign();
    } catch (err) {
      console.error("キャンペーンスケジュールエラー:", err);
      error =
        err instanceof Error
          ? err.message
          : "キャンペーンのスケジュールに失敗しました";
    } finally {
      isScheduling = false;
    }
  }

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
    const baseClass = "px-3 py-1 text-sm font-medium rounded-full";

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

  // 初期読み込み
  onMount(() => {
    fetchCampaign();
  });

  // キャンペーンの編集可否
  $: canEdit = campaign?.status === CampaignStatus.DRAFT;

  // キャンペーンの送信可否
  $: canSend =
    campaign?.status === CampaignStatus.DRAFT ||
    campaign?.status === CampaignStatus.SCHEDULED;
</script>

<div class="max-w-7xl mx-auto px-4 py-8">
  <!-- ヘッダー -->
  <div class="mb-8">
    <a
      href="/campaigns"
      class="text-blue-600 hover:text-blue-900 font-medium flex items-center mb-4"
    >
      <ArrowLeftIcon class="w-5 h-5 mr-2" />
      キャンペーン一覧に戻る
    </a>
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
  {:else if campaign}
    <div
      class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden"
    >
      <!-- キャンペーンヘッダー -->
      <div class="p-6 border-b border-gray-200">
        <div
          class="flex flex-col md:flex-row md:items-center justify-between gap-4"
        >
          <div>
            <div class="flex items-center mb-2">
              <h1 class="text-2xl font-bold text-gray-900">{campaign.name}</h1>
              <span class={`ml-4 ${getStatusBadgeClass(campaign.status)}`}>
                {formatStatus(campaign.status)}
              </span>
            </div>
            {#if campaign.description}
              <p class="text-gray-600">{campaign.description}</p>
            {/if}
          </div>

          <div class="flex space-x-3">
            {#if canEdit}
              <a
                href={`/campaigns/${campaignId}/edit`}
                class="flex items-center px-4 py-2 bg-white border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50 font-medium"
              >
                <EditIcon class="w-4 h-4 mr-2" />
                編集
              </a>
            {/if}

            <button
              class="flex items-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 font-medium disabled:opacity-50 disabled:cursor-not-allowed"
              on:click={fetchPreview}
              disabled={isPreviewLoading}
            >
              <EyeIcon class="w-4 h-4 mr-2" />
              {isPreviewLoading ? "プレビュー取得中..." : "プレビュー"}
            </button>

            {#if canSend}
              {#if campaign.status !== CampaignStatus.SCHEDULED}
                <button
                  class="flex items-center px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 font-medium"
                  on:click={() => (showScheduleModal = true)}
                >
                  <Calendar class="w-4 h-4 mr-2" />
                  スケジュール
                </button>
              {/if}

              <button
                class="flex items-center px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 font-medium disabled:opacity-50 disabled:cursor-not-allowed"
                on:click={sendCampaign}
                disabled={isSending}
              >
                {#if isSending}
                  <div
                    class="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"
                  ></div>
                  送信中...
                {:else}
                  <Send class="w-4 h-4 mr-2" />
                  今すぐ送信
                {/if}
              </button>
            {/if}

            {#if campaign.status === CampaignStatus.DRAFT}
              <button
                class="flex items-center px-4 py-2 bg-white border border-red-300 text-red-600 rounded-md hover:bg-red-50 font-medium"
                on:click={deleteCampaign}
              >
                <Trash2Icon class="w-4 h-4 mr-2" />
                削除
              </button>
            {/if}
          </div>
        </div>
      </div>

      <!-- 送信進捗バー -->
      {#if isSending}
        <div class="px-6 py-4 bg-blue-50 border-b border-blue-200">
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm font-medium text-blue-700">
              キャンペーンを送信中...
            </span>
            <span class="text-sm text-blue-600">
              {Math.round(sendingProgress)}%
            </span>
          </div>
          <div class="w-full bg-blue-200 rounded-full h-2">
            <div
              class="bg-blue-600 h-2 rounded-full transition-all duration-300"
              style="width: {sendingProgress}%"
            ></div>
          </div>
          {#if subscriberCount > 0}
            <p class="text-xs text-blue-600 mt-2">
              {subscriberCount}人の購読者にメールを送信しています
            </p>
          {/if}
        </div>
      {/if}

      <!-- キャンペーン詳細情報 -->
      <div class="p-6">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <h2 class="text-lg font-semibold mb-4 text-gray-900">
              キャンペーン情報
            </h2>
            <dl class="space-y-3">
              <div class="flex">
                <dt class="w-32 text-sm font-medium text-gray-500">
                  ステータス
                </dt>
                <dd class="flex-1 text-sm text-gray-900">
                  <span class={getStatusBadgeClass(campaign.status)}>
                    {formatStatus(campaign.status)}
                  </span>
                </dd>
              </div>

              <div class="flex">
                <dt class="w-32 text-sm font-medium text-gray-500">
                  メール件名
                </dt>
                <dd class="flex-1 text-sm text-gray-900">{campaign.subject}</dd>
              </div>

              <div class="flex">
                <dt class="w-32 text-sm font-medium text-gray-500">
                  送信予定日時
                </dt>
                <dd class="flex-1 text-sm text-gray-900 flex items-center">
                  <ClockIcon class="w-4 h-4 mr-2 text-gray-400" />
                  {formatDate(campaign.scheduled_at)}
                </dd>
              </div>

              <div class="flex">
                <dt class="w-32 text-sm font-medium text-gray-500">送信日時</dt>
                <dd class="flex-1 text-sm text-gray-900">
                  {campaign.sent_at ? formatDate(campaign.sent_at) : "未送信"}
                </dd>
              </div>

              <div class="flex">
                <dt class="w-32 text-sm font-medium text-gray-500">作成日時</dt>
                <dd class="flex-1 text-sm text-gray-900">
                  {formatDate(campaign.created_at)}
                </dd>
              </div>

              <div class="flex">
                <dt class="w-32 text-sm font-medium text-gray-500">更新日時</dt>
                <dd class="flex-1 text-sm text-gray-900">
                  {formatDate(campaign.updated_at)}
                </dd>
              </div>
            </dl>
          </div>

          <div>
            <h2 class="text-lg font-semibold mb-4 text-gray-900">送信統計</h2>
            <div class="bg-gray-50 p-4 rounded-lg">
              <dl class="grid grid-cols-2 gap-4">
                <div>
                  <dt class="text-sm font-medium text-gray-500">送信数</dt>
                  <dd class="mt-1 text-3xl font-semibold text-gray-900">
                    {campaign.stats.sent_count || 0}
                  </dd>
                </div>

                <div>
                  <dt class="text-sm font-medium text-gray-500">開封率</dt>
                  <dd class="mt-1 text-3xl font-semibold text-gray-900">
                    {campaign.stats.sent_count > 0
                      ? `${Math.round(campaign.stats.open_rate * 100)}%`
                      : "0%"}
                  </dd>
                </div>

                <div>
                  <dt class="text-sm font-medium text-gray-500">開封数</dt>
                  <dd class="mt-1 text-xl font-semibold text-gray-900">
                    {campaign.stats.opened_count || 0}
                  </dd>
                </div>

                <div>
                  <dt class="text-sm font-medium text-gray-500">クリック率</dt>
                  <dd class="mt-1 text-xl font-semibold text-gray-900">
                    {campaign.stats.sent_count > 0
                      ? `${Math.round(campaign.stats.click_rate * 100)}%`
                      : "0%"}
                  </dd>
                </div>
              </dl>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- プレビューモーダル -->
    {#if showPreview && previewHtml}
      <div
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      >
        <div
          class="bg-white rounded-lg max-w-4xl max-h-[90vh] w-full overflow-hidden shadow-xl"
        >
          <div
            class="flex items-center justify-between px-6 py-4 border-b border-gray-200"
          >
            <h2 class="text-xl font-semibold text-gray-900">
              キャンペーンプレビュー
            </h2>
            <button
              class="text-gray-400 hover:text-gray-500"
              on:click={() => (showPreview = false)}
            >
              ✕
            </button>
          </div>
          <div class="p-6 overflow-y-auto max-h-[calc(90vh-128px)]">
            <h3 class="text-lg font-medium text-gray-900 mb-2">
              件名: {campaign.subject}
            </h3>
            <div class="prose max-w-none border rounded-md p-4 bg-gray-50">
              {@html previewHtml}
            </div>
          </div>
          <div class="px-6 py-4 border-t border-gray-200 flex justify-end">
            <button
              class="bg-blue-600 hover:bg-blue-700 text-white font-medium px-4 py-2 rounded-lg"
              on:click={() => (showPreview = false)}
            >
              閉じる
            </button>
          </div>
        </div>
      </div>
    {/if}

    <!-- スケジュールモーダル -->
    {#if showScheduleModal}
      <div
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      >
        <div
          class="bg-white rounded-lg max-w-lg w-full overflow-hidden shadow-xl"
        >
          <div
            class="flex items-center justify-between px-6 py-4 border-b border-gray-200"
          >
            <h2 class="text-xl font-semibold text-gray-900">
              送信スケジュール設定
            </h2>
            <button
              class="text-gray-400 hover:text-gray-500"
              on:click={() => (showScheduleModal = false)}
            >
              ✕
            </button>
          </div>
          <div class="p-6">
            <p class="mb-4 text-gray-600">
              キャンペーン「{campaign.name}」の送信日時を設定します。
            </p>

            <div class="mb-4">
              <label
                for="schedule-date"
                class="block text-sm font-medium text-gray-700 mb-1"
              >
                送信日時を設定
              </label>
              <input
                id="schedule-date"
                type="datetime-local"
                bind:value={scheduleDate}
                min={new Date().toISOString().slice(0, 16)}
                class="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              />
            </div>
          </div>
          <div
            class="px-6 py-4 border-t border-gray-200 flex justify-end space-x-3"
          >
            <button
              class="px-4 py-2 border border-gray-300 rounded-md text-gray-700 bg-white hover:bg-gray-50"
              on:click={() => (showScheduleModal = false)}
            >
              キャンセル
            </button>
            <button
              class="bg-blue-600 hover:bg-blue-700 text-white font-medium px-4 py-2 rounded-lg flex items-center disabled:opacity-50 disabled:cursor-not-allowed"
              on:click={scheduleCampaign}
              disabled={isScheduling}
            >
              {#if isScheduling}
                <div
                  class="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"
                ></div>
                設定中...
              {:else}
                <Calendar class="w-4 h-4 mr-2" />
                スケジュール設定
              {/if}
            </button>
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>
