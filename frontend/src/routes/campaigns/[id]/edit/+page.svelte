<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { campaignService } from "$lib/services/campaignService";
  import type { Campaign, UpdateCampaignRequest } from "$lib/types/campaign";
  import { CampaignStatus } from "$lib/types/campaign";

  import type { Template } from "$lib/types/template";
  import { templateApi } from "$lib/services/api";
  import { ArrowLeftIcon, SaveIcon, EyeIcon } from "lucide-svelte";

  // ステータスのフォーマット関数
  function formatStatus(status: string): string {
    const statusMap: Record<string, string> = {
      draft: "下書き",
      scheduled: "予定済み",
      sending: "送信中",
      sent: "送信済み",
      canceled: "キャンセル",
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

  // URLパラメータからキャンペーンIDを取得
  const campaignId = $page.params.id;

  // 状態管理
  let isLoading = true;
  let isSaving = false;
  let isPreviewLoading = false;
  let error: string | null = null;
  let campaign: Campaign | null = null;
  let templates: Template[] = [];
  let loadingTemplates = true;
  let previewHtml: string | null = null;
  let showPreview = false;

  // フォームデータ
  let formData: UpdateCampaignRequest = {
    name: "",
    description: "",
    subject: "",
    template_id: "",
  };

  // バリデーション状態
  let validationErrors = {
    name: "",
    template_id: "",
    subject: "",
  };

  // 選択されたテンプレートを追跡
  let selectedTemplate: Template | null = null;

  // フォームバリデーション
  function validateForm(): boolean {
    let isValid = true;

    // 名前のバリデーション
    if (!formData.name) {
      validationErrors.name = "キャンペーン名は必須です";
      isValid = false;
    } else {
      validationErrors.name = "";
    }

    // テンプレートIDのバリデーション
    if (!formData.template_id) {
      validationErrors.template_id = "テンプレートの選択は必須です";
      isValid = false;
    } else {
      validationErrors.template_id = "";
    }

    // 件名のバリデーション
    if (!formData.subject) {
      validationErrors.subject = "メール件名は必須です";
      isValid = false;
    } else {
      validationErrors.subject = "";
    }

    return isValid;
  }

  // キャンペーン情報とテンプレート一覧を取得
  async function fetchData() {
    isLoading = true;
    error = null;

    try {
      // 並列で両方のデータを取得
      const [campaignData, templatesResponse] = await Promise.all([
        campaignService.getCampaign(campaignId),
        templateApi.getTemplates(),
      ]);

      if (templatesResponse.error) {
        throw new Error(templatesResponse.error);
      }

      const templatesData = templatesResponse.data;

      campaign = campaignData;
      templates = templatesData.templates;

      // フォームにデータを設定
      formData = {
        name: campaign.name,
        description: campaign.description,
        subject: campaign.subject,
        template_id: campaign.template_id,
      };

      // 選択中のテンプレート設定
      selectedTemplate =
        templates.find((t) => t.id === campaign.template_id) || null;
    } catch (err) {
      console.error("データ取得エラー:", err);
      error = err instanceof Error ? err.message : "不明なエラーが発生しました";
    } finally {
      isLoading = false;
      loadingTemplates = false;
    }
  }

  // テンプレート選択時の処理
  function onTemplateChange() {
    if (formData.template_id) {
      selectedTemplate =
        templates.find((t) => t.id === formData.template_id) || null;
    } else {
      selectedTemplate = null;
    }
  }

  // キャンペーン更新
  async function updateCampaign() {
    if (!validateForm()) {
      return;
    }

    isSaving = true;
    error = null;

    try {
      await campaignService.updateCampaign(campaignId, formData);

      // 更新成功後、キャンペーン詳細ページに遷移
      goto(`/campaigns/${campaignId}`);
    } catch (err) {
      console.error("キャンペーン更新エラー:", err);
      error = err instanceof Error ? err.message : "不明なエラーが発生しました";
      isSaving = false;
    }
  }

  // キャンペーンプレビューを取得
  async function fetchPreview() {
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

  // 初期読み込み
  onMount(() => {
    fetchData();
  });

  // 編集可能かどうかの判定
  $: canEdit = campaign && campaign.status === CampaignStatus.DRAFT;
</script>

<div class="max-w-3xl mx-auto px-4 py-8">
  <!-- ヘッダー -->
  <div class="mb-8">
    <a
      href={`/campaigns/${campaignId}`}
      class="text-blue-600 hover:text-blue-900 font-medium flex items-center mb-4"
    >
      <ArrowLeftIcon class="w-5 h-5 mr-2" />
      キャンペーン詳細に戻る
    </a>

    <h1 class="text-3xl font-bold text-gray-900">キャンペーン編集</h1>
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
    <!-- 編集不可の場合 -->
  {:else if campaign && !canEdit}
    <div
      class="bg-yellow-100 border border-yellow-400 text-yellow-800 px-4 py-3 rounded mb-6"
    >
      <p>
        このキャンペーンは「{formatStatus(
          campaign.status,
        )}」状態のため編集できません。
      </p>
    </div>
    <div class="flex justify-center">
      <a
        href={`/campaigns/${campaignId}`}
        class="bg-blue-600 hover:bg-blue-700 text-white font-medium px-4 py-2 rounded-lg"
      >
        キャンペーン詳細に戻る
      </a>
    </div>
    <!-- 編集フォーム -->
  {:else if campaign}
    <form
      on:submit|preventDefault={updateCampaign}
      class="space-y-6 bg-white p-6 rounded-lg shadow-sm border border-gray-200"
    >
      <!-- キャンペーン名 -->
      <div>
        <label
          for="campaign-name"
          class="block text-sm font-medium text-gray-700"
        >
          キャンペーン名 <span class="text-red-600">*</span>
        </label>
        <input
          id="campaign-name"
          type="text"
          bind:value={formData.name}
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
          placeholder="例: 新製品リリース通知"
        />
        {#if validationErrors.name}
          <p class="mt-1 text-sm text-red-600">{validationErrors.name}</p>
        {/if}
      </div>

      <!-- 説明 -->
      <div>
        <label
          for="campaign-description"
          class="block text-sm font-medium text-gray-700"
        >
          説明
        </label>
        <textarea
          id="campaign-description"
          bind:value={formData.description}
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
          rows="3"
          placeholder="キャンペーンの目的や内容の説明"
        ></textarea>
      </div>

      <!-- テンプレート選択 -->
      <div>
        <label
          for="template-select"
          class="block text-sm font-medium text-gray-700"
        >
          テンプレート <span class="text-red-600">*</span>
        </label>
        {#if loadingTemplates}
          <div class="mt-1 text-sm text-gray-500">
            テンプレートを読み込み中...
          </div>
        {:else if templates.length === 0}
          <div class="mt-1 text-sm text-gray-500">
            使用可能なテンプレートがありません。
            <a href="/templates/new" class="text-blue-600 hover:underline">
              新しいテンプレートを作成してください。
            </a>
          </div>
        {:else}
          <select
            id="template-select"
            bind:value={formData.template_id}
            on:change={onTemplateChange}
            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
          >
            <option value="">テンプレートを選択してください</option>
            {#each templates as template}
              <option value={template.id}>{template.name}</option>
            {/each}
          </select>
        {/if}
        {#if validationErrors.template_id}
          <p class="mt-1 text-sm text-red-600">
            {validationErrors.template_id}
          </p>
        {/if}
      </div>

      <!-- 件名 -->
      <div>
        <label
          for="campaign-subject"
          class="block text-sm font-medium text-gray-700"
        >
          メール件名 <span class="text-red-600">*</span>
        </label>
        <input
          id="campaign-subject"
          type="text"
          bind:value={formData.subject}
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
          placeholder="例: 【重要】新製品のお知らせ"
        />
        <p class="mt-1 text-xs text-gray-500">
          テンプレート変数（例: {{ name }}）を使用できます
        </p>
        {#if validationErrors.subject}
          <p class="mt-1 text-sm text-red-600">{validationErrors.subject}</p>
        {/if}
      </div>

      <!-- テンプレートプレビュー -->
      {#if selectedTemplate}
        <div>
          <h3 class="text-sm font-medium text-gray-700 mb-2">
            テンプレートプレビュー
          </h3>
          <div class="bg-gray-50 border border-gray-200 rounded-md p-4">
            <p class="text-sm font-medium mb-2">
              テンプレート: {selectedTemplate.name}
            </p>
            <div class="text-sm text-gray-600 border-t border-gray-200 pt-2">
              <p class="mb-1">
                <strong>件名テンプレート:</strong>
                {selectedTemplate.subject_template}
              </p>
              <p>
                <strong>変数:</strong>
                {Object.keys(selectedTemplate.variables).length > 0
                  ? Object.keys(selectedTemplate.variables).join(", ")
                  : "なし"}
              </p>
            </div>
          </div>
        </div>
      {/if}

      <!-- キャンペーン情報 -->
      <div class="pt-4 border-t border-gray-200">
        <h3 class="text-sm font-medium text-gray-700 mb-2">キャンペーン情報</h3>
        <div class="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span class="text-gray-500">ステータス:</span>
            <span class={getStatusBadgeClass(campaign.status)}>
              {formatStatus(campaign.status)}
            </span>
          </div>
          <div>
            <span class="text-gray-500">作成日時:</span>
            <span class="text-gray-900">{formatDate(campaign.created_at)}</span>
          </div>
          {#if campaign.scheduled_at}
            <div>
              <span class="text-gray-500">予定日時:</span>
              <span class="text-gray-900"
                >{formatDate(campaign.scheduled_at)}</span
              >
            </div>
          {/if}
          {#if campaign.sent_at}
            <div>
              <span class="text-gray-500">送信日時:</span>
              <span class="text-gray-900">{formatDate(campaign.sent_at)}</span>
            </div>
          {/if}
        </div>
      </div>

      <!-- ボタン群 -->
      <div class="flex justify-between pt-4">
        <div class="flex space-x-3">
          <button
            type="button"
            class="px-4 py-2 border border-gray-300 rounded-md text-gray-700 bg-white hover:bg-gray-50 flex items-center"
            on:click={fetchPreview}
            disabled={isPreviewLoading}
          >
            <EyeIcon class="w-4 h-4 mr-2" />
            {isPreviewLoading ? "プレビュー取得中..." : "プレビュー確認"}
          </button>
        </div>

        <div class="flex space-x-3">
          <a
            href={`/campaigns/${campaignId}`}
            class="px-4 py-2 border border-gray-300 rounded-md text-gray-700 bg-white hover:bg-gray-50"
          >
            キャンセル
          </a>
          <button
            type="submit"
            class="bg-blue-600 hover:bg-blue-700 text-white font-medium px-4 py-2 rounded-lg flex items-center disabled:opacity-50 disabled:cursor-not-allowed"
            disabled={isSaving}
          >
            <SaveIcon class="w-4 h-4 mr-2" />
            {isSaving ? "保存中..." : "保存"}
          </button>
        </div>
      </div>
    </form>
  {/if}

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
            件名: {formData.subject || campaign?.subject}
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
</div>
