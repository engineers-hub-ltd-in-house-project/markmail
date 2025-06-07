<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { campaignService } from "$lib/services/campaignService";
  import type { Template } from "$lib/types/template";
  import type { CreateCampaignRequest } from "$lib/types/campaign";
  import { ArrowLeftIcon, SendIcon } from "lucide-svelte";

  // テンプレート取得用
  import { templateApi } from "$lib/services/api";

  // 状態管理
  let isLoading = false;
  let isSaving = false;
  let error: string | null = null;
  let templates: Template[] = [];
  let loadingTemplates = true;
  let templateError: string | null = null;

  // フォームデータ
  let campaignData: CreateCampaignRequest = {
    name: "",
    description: "",
    template_id: "",
    subject: "",
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
    if (!campaignData.name.trim()) {
      validationErrors.name = "キャンペーン名は必須です";
      isValid = false;
    } else {
      validationErrors.name = "";
    }

    // テンプレートIDのバリデーション
    if (!campaignData.template_id) {
      validationErrors.template_id = "テンプレートの選択は必須です";
      isValid = false;
    } else {
      validationErrors.template_id = "";
    }

    // 件名のバリデーション
    if (!campaignData.subject.trim()) {
      validationErrors.subject = "メール件名は必須です";
      isValid = false;
    } else {
      validationErrors.subject = "";
    }

    return isValid;
  }

  // テンプレート一覧を取得
  async function fetchTemplates() {
    loadingTemplates = true;
    templateError = null;

    try {
      const response = await templateApi.getTemplates();

      if (response.error) {
        throw new Error(response.error);
      }

      templates = response.data?.templates || [];
    } catch (err) {
      console.error("テンプレート取得エラー:", err);
      templateError =
        err instanceof Error ? err.message : "不明なエラーが発生しました";
      templates = [];
    } finally {
      loadingTemplates = false;
    }
  }

  // テンプレート選択時の処理
  function onTemplateChange() {
    if (campaignData.template_id) {
      selectedTemplate =
        templates.find((t) => t.id === campaignData.template_id) || null;

      // テンプレートの件名を自動で設定
      if (selectedTemplate && !campaignData.subject) {
        campaignData.subject = selectedTemplate.subject_template;
      }
    } else {
      selectedTemplate = null;
    }
  }

  // キャンペーン作成
  async function createCampaign() {
    if (!validateForm()) {
      return;
    }

    isSaving = true;
    error = null;

    try {
      const campaign = await campaignService.createCampaign(campaignData);

      // 作成後にキャンペーン詳細ページに遷移
      goto(`/campaigns/${campaign.id}`);
    } catch (err) {
      console.error("キャンペーン作成エラー:", err);
      error = err instanceof Error ? err.message : "不明なエラーが発生しました";
      isSaving = false;
    }
  }

  // 初期読み込み
  onMount(() => {
    fetchTemplates();
  });
</script>

<div class="max-w-3xl mx-auto px-4 py-8">
  <!-- ヘッダー -->
  <div class="mb-8">
    <a
      href="/campaigns"
      class="text-blue-600 hover:text-blue-900 font-medium flex items-center mb-4"
    >
      <ArrowLeftIcon class="w-5 h-5 mr-2" />
      キャンペーン一覧に戻る
    </a>

    <h1 class="text-3xl font-bold text-gray-900">新規キャンペーン作成</h1>
  </div>

  <!-- エラーメッセージ -->
  {#if error}
    <div
      class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4"
    >
      <p>{error}</p>
    </div>
  {/if}

  <form
    on:submit|preventDefault={createCampaign}
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
        bind:value={campaignData.name}
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
        bind:value={campaignData.description}
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
      {:else if templateError}
        <div class="mt-1 text-sm text-red-600">{templateError}</div>
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
          bind:value={campaignData.template_id}
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
        <p class="mt-1 text-sm text-red-600">{validationErrors.template_id}</p>
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
        bind:value={campaignData.subject}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
        placeholder="例: 【重要】新製品のお知らせ"
      />
      <p class="mt-1 text-xs text-gray-500">テンプレート変数を使用できます</p>
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

    <!-- 保存ボタン -->
    <div class="flex justify-end pt-4">
      <a
        href="/campaigns"
        class="mr-4 px-4 py-2 border border-gray-300 rounded-md text-gray-700 bg-white hover:bg-gray-50"
      >
        キャンセル
      </a>
      <button
        type="submit"
        class="bg-blue-600 hover:bg-blue-700 text-white font-medium px-4 py-2 rounded-lg flex items-center disabled:opacity-50 disabled:cursor-not-allowed"
        disabled={isSaving || loadingTemplates}
      >
        <SendIcon class="w-4 h-4 mr-2" />
        {isSaving ? "作成中..." : "キャンペーンを作成"}
      </button>
    </div>
  </form>
</div>
