<script lang="ts">
  import { goto } from "$app/navigation";
  import { Save, X, Plus, Trash2, GripVertical } from "lucide-svelte";
  import { formService } from "$lib/services/formService";
  import type { CreateFormRequest, FormField } from "$lib/types/form";

  let form: CreateFormRequest = {
    name: "",
    description: "",
    markdown_content:
      "# フォームタイトル\n\nフォームの説明文をここに記入します。",
    form_fields: [
      {
        field_type: "email",
        name: "email",
        label: "メールアドレス",
        placeholder: "example@email.com",
        required: true,
        validation_rules: {},
        options: [],
        display_order: 0,
      },
    ],
    settings: {
      submit_button_text: "送信",
      success_message: "フォームを送信しました。ありがとうございます。",
      require_confirmation: true,
    },
  };

  let saving = false;
  let error: string | null = null;

  const fieldTypes = [
    { value: "text", label: "テキスト" },
    { value: "email", label: "メールアドレス" },
    { value: "textarea", label: "テキストエリア" },
    { value: "select", label: "セレクトボックス" },
    { value: "radio", label: "ラジオボタン" },
    { value: "checkbox", label: "チェックボックス" },
  ];

  function addField() {
    const newField: FormField = {
      field_type: "text",
      name: `field_${Date.now()}`,
      label: "新しいフィールド",
      placeholder: "",
      required: false,
      validation_rules: {},
      options: [],
      display_order: form.form_fields!.length,
    };
    form.form_fields = [...(form.form_fields || []), newField];
  }

  function removeField(index: number) {
    form.form_fields = form.form_fields!.filter((_, i) => i !== index);
    // 表示順を更新
    form.form_fields.forEach((field, i) => {
      field.display_order = i;
    });
  }

  function moveField(index: number, direction: "up" | "down") {
    const fields = [...form.form_fields!];
    const newIndex = direction === "up" ? index - 1 : index + 1;

    if (newIndex < 0 || newIndex >= fields.length) return;

    [fields[index], fields[newIndex]] = [fields[newIndex], fields[index]];

    // 表示順を更新
    fields.forEach((field, i) => {
      field.display_order = i;
    });

    form.form_fields = fields;
  }

  function addOption(fieldIndex: number) {
    const field = form.form_fields![fieldIndex];
    if (!field.options) field.options = [];
    field.options.push({
      value: `option_${Date.now()}`,
      label: "新しい選択肢",
    });
    form.form_fields = [...form.form_fields!];
  }

  function removeOption(fieldIndex: number, optionIndex: number) {
    const field = form.form_fields![fieldIndex];
    field.options = field.options!.filter((_, i) => i !== optionIndex);
    form.form_fields = [...form.form_fields!];
  }

  async function save() {
    if (!form.name) {
      error = "フォーム名を入力してください";
      return;
    }

    try {
      saving = true;
      error = null;

      // nameをスラッグ形式に変換
      if (!form.slug) {
        form.slug = form.name
          .toLowerCase()
          .replace(/\s+/g, "-")
          .replace(/[^a-z0-9-]/g, "");
      }

      const created = await formService.create(form);
      goto(`/forms/${created.id}/edit`);
    } catch (err: any) {
      error = err.response?.data?.error || "フォームの作成に失敗しました";
      console.error(err);
    } finally {
      saving = false;
    }
  }
</script>

<div class="container mx-auto px-4 py-8 max-w-4xl">
  <div class="mb-8">
    <div class="flex justify-between items-center">
      <h1 class="text-3xl font-bold text-gray-800">新規フォーム作成</h1>
      <a
        href="/forms"
        class="inline-flex items-center px-4 py-2 text-gray-600 hover:text-gray-800 transition-colors"
      >
        <X class="w-5 h-5 mr-2" />
        キャンセル
      </a>
    </div>
  </div>

  {#if error}
    <div
      class="mb-6 bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg"
    >
      {error}
    </div>
  {/if}

  <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
    <div class="mb-6">
      <label for="name" class="block text-sm font-medium text-gray-700 mb-2">
        フォーム名 <span class="text-red-500">*</span>
      </label>
      <input
        type="text"
        id="name"
        bind:value={form.name}
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="お問い合わせフォーム"
      />
    </div>

    <div class="mb-6">
      <label
        for="description"
        class="block text-sm font-medium text-gray-700 mb-2"
      >
        説明（任意）
      </label>
      <textarea
        id="description"
        bind:value={form.description}
        rows="2"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        placeholder="このフォームの用途を説明してください"
      />
    </div>

    <div class="mb-6">
      <label
        for="markdown_content"
        class="block text-sm font-medium text-gray-700 mb-2"
      >
        フォーム説明文（Markdown）
      </label>
      <textarea
        id="markdown_content"
        bind:value={form.markdown_content}
        rows="6"
        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-sm"
      />
    </div>

    <div class="mb-6">
      <div class="flex justify-between items-center mb-4">
        <h3 class="text-lg font-medium text-gray-900">フィールド設定</h3>
        <button
          type="button"
          on:click={addField}
          class="inline-flex items-center px-3 py-1 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700 transition-colors"
        >
          <Plus class="w-4 h-4 mr-1" />
          フィールドを追加
        </button>
      </div>

      <div class="space-y-4">
        {#each form.form_fields || [] as field, index}
          <div class="bg-gray-50 p-4 rounded-lg border border-gray-200">
            <div class="flex items-start gap-4">
              <button
                type="button"
                class="mt-2 text-gray-400 hover:text-gray-600 cursor-move"
                title="ドラッグして並び替え"
              >
                <GripVertical class="w-5 h-5" />
              </button>

              <div class="flex-1 space-y-3">
                <div class="grid grid-cols-2 gap-3">
                  <div>
                    <label class="block text-xs font-medium text-gray-700 mb-1">
                      フィールドタイプ
                    </label>
                    <select
                      bind:value={field.field_type}
                      class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                    >
                      {#each fieldTypes as type}
                        <option value={type.value}>{type.label}</option>
                      {/each}
                    </select>
                  </div>

                  <div>
                    <label class="block text-xs font-medium text-gray-700 mb-1">
                      フィールド名（英数字）
                    </label>
                    <input
                      type="text"
                      bind:value={field.name}
                      class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                      placeholder="field_name"
                    />
                  </div>
                </div>

                <div>
                  <label class="block text-xs font-medium text-gray-700 mb-1">
                    ラベル
                  </label>
                  <input
                    type="text"
                    bind:value={field.label}
                    class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                    placeholder="表示ラベル"
                  />
                </div>

                {#if field.field_type !== "select" && field.field_type !== "radio" && field.field_type !== "checkbox"}
                  <div>
                    <label class="block text-xs font-medium text-gray-700 mb-1">
                      プレースホルダー
                    </label>
                    <input
                      type="text"
                      bind:value={field.placeholder}
                      class="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                      placeholder="入力例"
                    />
                  </div>
                {/if}

                {#if field.field_type === "select" || field.field_type === "radio" || field.field_type === "checkbox"}
                  <div>
                    <div class="flex justify-between items-center mb-2">
                      <label class="block text-xs font-medium text-gray-700">
                        選択肢
                      </label>
                      <button
                        type="button"
                        on:click={() => addOption(index)}
                        class="text-xs text-blue-600 hover:text-blue-700"
                      >
                        + 選択肢を追加
                      </button>
                    </div>
                    {#each field.options || [] as option, optIndex}
                      <div class="flex gap-2 mb-2">
                        <input
                          type="text"
                          bind:value={option.label}
                          class="flex-1 px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                          placeholder="選択肢"
                        />
                        <button
                          type="button"
                          on:click={() => removeOption(index, optIndex)}
                          class="text-red-600 hover:text-red-700"
                        >
                          <Trash2 class="w-4 h-4" />
                        </button>
                      </div>
                    {/each}
                  </div>
                {/if}

                <div class="flex items-center">
                  <input
                    type="checkbox"
                    id="required_{index}"
                    bind:checked={field.required}
                    class="mr-2"
                  />
                  <label for="required_{index}" class="text-sm text-gray-700">
                    必須項目
                  </label>
                </div>
              </div>

              <div class="flex flex-col gap-1">
                <button
                  type="button"
                  on:click={() => moveField(index, "up")}
                  disabled={index === 0}
                  class="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
                  title="上に移動"
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
                      d="M5 15l7-7 7 7"
                    />
                  </svg>
                </button>
                <button
                  type="button"
                  on:click={() => moveField(index, "down")}
                  disabled={index === (form.form_fields?.length || 0) - 1}
                  class="p-1 text-gray-400 hover:text-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
                  title="下に移動"
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
                      d="M19 9l-7 7-7-7"
                    />
                  </svg>
                </button>
                <button
                  type="button"
                  on:click={() => removeField(index)}
                  class="p-1 text-red-600 hover:text-red-700"
                  title="削除"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <div class="mb-6">
      <h3 class="text-lg font-medium text-gray-900 mb-4">フォーム設定</h3>

      <div class="space-y-4">
        <div>
          <label
            for="submit_button_text"
            class="block text-sm font-medium text-gray-700 mb-2"
          >
            送信ボタンのテキスト
          </label>
          <input
            type="text"
            id="submit_button_text"
            bind:value={form.settings.submit_button_text}
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        <div>
          <label
            for="success_message"
            class="block text-sm font-medium text-gray-700 mb-2"
          >
            送信完了メッセージ
          </label>
          <textarea
            id="success_message"
            bind:value={form.settings.success_message}
            rows="2"
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        <div class="flex items-center">
          <input
            type="checkbox"
            id="require_confirmation"
            bind:checked={form.settings.require_confirmation}
            class="mr-2"
          />
          <label for="require_confirmation" class="text-sm text-gray-700">
            メールアドレス確認を必須にする
          </label>
        </div>
      </div>
    </div>

    <div class="flex justify-end gap-3">
      <a
        href="/forms"
        class="px-4 py-2 text-gray-600 hover:text-gray-800 transition-colors"
      >
        キャンセル
      </a>
      <button
        on:click={save}
        disabled={saving}
        class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {#if saving}
          <div
            class="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"
          ></div>
          保存中...
        {:else}
          <Save class="w-5 h-5 mr-2" />
          保存
        {/if}
      </button>
    </div>
  </div>
</div>
