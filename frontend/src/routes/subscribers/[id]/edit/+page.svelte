<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { subscriberService } from "$lib/services/subscriberService";
  import type {
    Subscriber,
    UpdateSubscriberRequest,
  } from "$lib/types/subscriber";
  import { SubscriberStatus } from "$lib/types/subscriber";

  // 状態変数
  let subscriber: Subscriber | null = null;
  let isLoading = true;
  let isSaving = false;
  let error: string | null = null;
  let tags: string[] = [];
  let availableTags: string[] = [];
  let newTag = "";
  let customFields: { key: string; value: string }[] = [];

  // 購読者ID
  const subscriberId = $page.params.id;

  // フォームデータ
  let formData: UpdateSubscriberRequest = {
    name: "",
    status: SubscriberStatus.ACTIVE,
    tags: [],
    custom_fields: {},
  };

  // カスタムフィールドを追加
  function addCustomField() {
    customFields = [...customFields, { key: "", value: "" }];
  }

  // カスタムフィールドを削除
  function removeCustomField(index: number) {
    customFields = customFields.filter((_, i) => i !== index);
  }

  // タグを追加
  function addTag() {
    if (newTag && !tags.includes(newTag)) {
      tags = [...tags, newTag];
      newTag = "";
    }
  }

  // タグを削除
  function removeTag(tag: string) {
    tags = tags.filter((t) => t !== tag);
  }

  // フォーム検証
  function validateForm(): boolean {
    // カスタムフィールドのキーが重複していないかチェック
    const keys = customFields
      .filter((field) => field.key.trim() !== "")
      .map((field) => field.key.trim());

    if (new Set(keys).size !== keys.length) {
      error = "カスタムフィールドのキーが重複しています";
      return false;
    }

    return true;
  }

  // 購読者情報を更新
  async function updateSubscriber() {
    if (!validateForm()) {
      return;
    }

    // カスタムフィールドを処理
    const validCustomFields = customFields.filter(
      (field) => field.key.trim() !== "",
    );
    const customFieldsObject: Record<string, any> = {};

    validCustomFields.forEach((field) => {
      customFieldsObject[field.key.trim()] = field.value;
    });

    // 更新データを準備
    const data: UpdateSubscriberRequest = {
      name: formData.name || undefined,
      status: formData.status,
      tags: tags,
      custom_fields: customFieldsObject,
    };

    isSaving = true;
    error = null;

    try {
      await subscriberService.updateSubscriber(subscriberId, data);
      goto(`/subscribers/${subscriberId}`);
    } catch (err) {
      console.error("購読者更新エラー:", err);
      error = err instanceof Error ? err.message : "不明なエラーが発生しました";
    } finally {
      isSaving = false;
    }
  }

  // 購読者詳細を取得
  async function loadSubscriber() {
    isLoading = true;
    error = null;

    try {
      subscriber = await subscriberService.getSubscriber(subscriberId);

      // フォームデータを設定
      formData = {
        name: subscriber.name || "",
        status: subscriber.status,
        tags: subscriber.tags,
        custom_fields: subscriber.custom_fields,
      };

      tags = [...subscriber.tags];

      // カスタムフィールドを設定
      customFields = Object.entries(subscriber.custom_fields).map(
        ([key, value]) => ({
          key,
          value: String(value), // 値を文字列に変換
        }),
      );

      // カスタムフィールドがない場合は空のフィールドを追加
      if (customFields.length === 0) {
        customFields = [{ key: "", value: "" }];
      }
    } catch (err) {
      console.error("購読者詳細取得エラー:", err);
      error =
        err instanceof Error ? err.message : "購読者詳細の取得に失敗しました";
    } finally {
      isLoading = false;
    }
  }

  // タグのリストを取得
  async function loadTags() {
    try {
      availableTags = await subscriberService.getSubscriberTags();
    } catch (err) {
      console.error("タグ取得エラー:", err);
      availableTags = [];
    }
  }

  // 初期化
  onMount(async () => {
    await Promise.all([loadSubscriber(), loadTags()]);
  });
</script>

<svelte:head>
  <title>購読者編集 | MarkMail</title>
</svelte:head>

<div class="container mx-auto px-4 py-6">
  <div class="mb-6">
    <a
      href={`/subscribers/${subscriberId}`}
      class="text-indigo-600 hover:text-indigo-800 flex items-center"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="h-5 w-5 mr-1"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z"
          clip-rule="evenodd"
        />
      </svg>
      購読者詳細に戻る
    </a>
  </div>

  {#if error}
    <div class="bg-red-100 border-l-4 border-red-500 text-red-700 p-4 mb-4">
      <p>{error}</p>
    </div>
  {/if}

  {#if isLoading}
    <div class="bg-white shadow rounded-lg p-6">
      <div class="animate-pulse flex space-x-4">
        <div class="flex-1 space-y-4 py-1">
          <div class="h-4 bg-gray-200 rounded w-3/4"></div>
          <div class="space-y-2">
            <div class="h-4 bg-gray-200 rounded"></div>
            <div class="h-4 bg-gray-200 rounded w-5/6"></div>
          </div>
        </div>
      </div>
    </div>
  {:else if subscriber}
    <div class="bg-white shadow rounded-lg overflow-hidden">
      <div class="p-6 border-b border-gray-200">
        <h1 class="text-xl font-semibold text-gray-900">購読者編集</h1>
        <p class="mt-1 text-sm text-gray-500">{subscriber.email}</p>
      </div>

      <div class="p-6">
        <form on:submit|preventDefault={updateSubscriber} class="space-y-6">
          <!-- 基本情報 -->
          <div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
            <div>
              <label
                for="email"
                class="block text-sm font-medium text-gray-700"
              >
                メールアドレス
              </label>
              <input
                id="email"
                type="email"
                value={subscriber.email}
                disabled
                class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 bg-gray-50 text-gray-500 cursor-not-allowed"
              />
              <p class="mt-1 text-xs text-gray-500">
                メールアドレスは変更できません
              </p>
            </div>
            <div>
              <label for="name" class="block text-sm font-medium text-gray-700">
                名前
              </label>
              <input
                id="name"
                type="text"
                bind:value={formData.name}
                class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
              />
            </div>
          </div>

          <!-- ステータス -->
          <div>
            <label
              for="status"
              class="block text-sm font-medium text-gray-700 mb-1"
            >
              ステータス
            </label>
            <select
              id="status"
              bind:value={formData.status}
              class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
            >
              <option value={SubscriberStatus.ACTIVE}>有効</option>
              <option value={SubscriberStatus.UNSUBSCRIBED}>購読解除</option>
              <option value={SubscriberStatus.BOUNCED}>バウンス</option>
              <option value={SubscriberStatus.COMPLAINED}>スパム報告</option>
            </select>
          </div>

          <!-- タグ -->
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">
              タグ
            </label>
            <div class="flex flex-wrap gap-2 mb-2">
              {#each tags as tag}
                <span
                  class="bg-indigo-100 text-indigo-800 text-sm px-2 py-1 rounded-full flex items-center"
                >
                  {tag}
                  <button
                    type="button"
                    class="ml-1 text-indigo-600 hover:text-indigo-800"
                    on:click={() => removeTag(tag)}
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      class="h-4 w-4"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M6 18L18 6M6 6l12 12"
                      />
                    </svg>
                  </button>
                </span>
              {/each}
            </div>

            <div class="flex space-x-2">
              <div class="flex-1">
                <input
                  type="text"
                  list="available-tags"
                  bind:value={newTag}
                  placeholder="タグを入力..."
                  class="block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                />
                <datalist id="available-tags">
                  {#each availableTags as tag}
                    <option value={tag} />
                  {/each}
                </datalist>
              </div>
              <button
                type="button"
                class="bg-indigo-600 text-white px-3 py-2 rounded hover:bg-indigo-700"
                on:click={addTag}
              >
                追加
              </button>
            </div>
          </div>

          <!-- カスタムフィールド -->
          <div>
            <div class="flex justify-between mb-1">
              <label class="block text-sm font-medium text-gray-700">
                カスタムフィールド
              </label>
              <button
                type="button"
                class="text-indigo-600 hover:text-indigo-800 text-sm"
                on:click={addCustomField}
              >
                + フィールド追加
              </button>
            </div>

            {#each customFields as field, index}
              <div class="flex space-x-2 mb-2">
                <input
                  type="text"
                  placeholder="キー"
                  bind:value={field.key}
                  class="block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                />
                <input
                  type="text"
                  placeholder="値"
                  bind:value={field.value}
                  class="block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
                />
                <button
                  type="button"
                  class="text-red-600 hover:text-red-800"
                  on:click={() => removeCustomField(index)}
                  disabled={customFields.length === 1}
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-5 w-5"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z"
                      clip-rule="evenodd"
                    />
                  </svg>
                </button>
              </div>
            {/each}
          </div>

          <!-- 送信ボタン -->
          <div class="flex justify-end">
            <button
              type="button"
              class="bg-white py-2 px-4 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 hover:bg-gray-50 mr-3"
              on:click={() => goto(`/subscribers/${subscriberId}`)}
            >
              キャンセル
            </button>
            <button
              type="submit"
              class="bg-indigo-600 py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
              disabled={isSaving}
            >
              {#if isSaving}
                <span class="flex items-center">
                  <svg
                    class="animate-spin -ml-1 mr-2 h-4 w-4 text-white"
                    xmlns="http://www.w3.org/2000/svg"
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
                    />
                  </svg>
                  保存中...
                </span>
              {:else}
                保存する
              {/if}
            </button>
          </div>
        </form>
      </div>
    </div>
  {:else if !isLoading && !error}
    <div class="bg-white shadow rounded-lg p-6">
      <p class="text-gray-500">購読者が見つかりませんでした。</p>
    </div>
  {/if}
</div>
