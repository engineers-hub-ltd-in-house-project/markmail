<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { formService } from "$lib/services/formService";
  import type { Form } from "$lib/types/form";

  let form: Form | null = null;
  let loading = true;
  let error = "";
  let submitting = false;
  let submitted = false;
  let formData: Record<string, any> = {};

  onMount(async () => {
    await loadForm();
  });

  async function loadForm() {
    try {
      loading = true;
      error = "";
      const id = $page.params.id;
      form = await formService.getPublicForm(id);

      console.log("Loaded form:", form); // デバッグ用

      // Initialize form data with default values
      if (form?.form_fields && Array.isArray(form.form_fields)) {
        form.form_fields.forEach((field: any) => {
          if (field.default_value) {
            formData[field.name] = field.default_value;
          } else if (field.field_type === "checkbox") {
            formData[field.name] = false;
          } else {
            formData[field.name] = "";
          }
        });
      } else {
        console.warn("Form fields is not an array:", form?.form_fields);
        // form_fieldsがない場合は空配列を設定
        if (form) {
          form.form_fields = [];
        }
      }
    } catch (err) {
      console.error("Failed to load form:", err);
      error = "フォームの読み込みに失敗しました";
    } finally {
      loading = false;
    }
  }

  async function handleSubmit() {
    if (!form) return;

    // Validate required fields
    const missingFields = form.form_fields
      .filter((field: any) => field.required && !formData[field.name])
      .map((field: any) => field.label || field.name);

    if (missingFields.length > 0) {
      error = `必須項目を入力してください: ${missingFields.join(", ")}`;
      return;
    }

    try {
      submitting = true;
      error = "";
      await formService.submitForm(form.id, formData);
      submitted = true;

      // Clear form data
      formData = {};
      if (form.form_fields) {
        form.form_fields.forEach((field: any) => {
          if (field.default_value) {
            formData[field.name] = field.default_value;
          } else if (field.field_type === "checkbox") {
            formData[field.name] = false;
          } else {
            formData[field.name] = "";
          }
        });
      }
    } catch (err) {
      console.error("Failed to submit form:", err);
      error = "フォームの送信に失敗しました";
    } finally {
      submitting = false;
    }
  }

  function renderMarkdown(text: string): string {
    // Basic markdown rendering - in production, use a proper markdown library
    return text
      .replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>")
      .replace(/\*(.*?)\*/g, "<em>$1</em>")
      .replace(/\n/g, "<br>");
  }
</script>

<svelte:head>
  <title>{form?.name || "Loading..."}</title>
  {#if form?.description}
    <meta name="description" content={form.description} />
  {/if}
</svelte:head>

<div class="min-h-screen bg-gray-50 py-8">
  <div class="max-w-2xl mx-auto px-4">
    {#if loading}
      <div class="bg-white rounded-lg shadow p-8">
        <div class="flex justify-center">
          <div
            class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"
          ></div>
        </div>
      </div>
    {:else if error && !form}
      <div class="bg-white rounded-lg shadow p-8">
        <div class="text-center">
          <p class="text-red-600">{error}</p>
        </div>
      </div>
    {:else if form}
      <div class="bg-white rounded-lg shadow">
        {#if form.settings?.header_content}
          <div class="p-8 border-b">
            <div class="prose max-w-none">
              {@html renderMarkdown(form.settings.header_content)}
            </div>
          </div>
        {/if}

        <div class="p-8">
          <h1 class="text-2xl font-bold mb-4">{form.name}</h1>

          {#if form.description}
            <p class="text-gray-600 mb-6">{form.description}</p>
          {/if}

          {#if submitted}
            <div
              class="bg-green-50 border border-green-200 text-green-800 px-4 py-3 rounded mb-6"
            >
              {form.settings?.success_message ||
                "フォームの送信が完了しました。ありがとうございます。"}
            </div>
          {/if}

          {#if error}
            <div
              class="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded mb-6"
            >
              {error}
            </div>
          {/if}

          <form on:submit|preventDefault={handleSubmit} class="space-y-6">
            {#if form.form_fields && Array.isArray(form.form_fields)}
              {#each form.form_fields as field}
                <div>
                  <label class="block text-sm font-medium text-gray-700 mb-1">
                    {field.label || field.name}
                    {#if field.required}
                      <span class="text-red-500">*</span>
                    {/if}
                  </label>

                  {#if field.description}
                    <p class="text-sm text-gray-500 mb-2">
                      {field.description}
                    </p>
                  {/if}

                  {#if field.field_type === "text"}
                    <input
                      type="text"
                      bind:value={formData[field.name]}
                      placeholder={field.placeholder || ""}
                      required={field.required}
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  {:else if field.field_type === "email"}
                    <input
                      type="email"
                      bind:value={formData[field.name]}
                      placeholder={field.placeholder || ""}
                      required={field.required}
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  {:else if field.field_type === "url"}
                    <input
                      type="url"
                      bind:value={formData[field.name]}
                      placeholder={field.placeholder || ""}
                      required={field.required}
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  {:else if field.field_type === "tel"}
                    <input
                      type="tel"
                      bind:value={formData[field.name]}
                      placeholder={field.placeholder || ""}
                      required={field.required}
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  {:else if field.field_type === "number"}
                    <input
                      type="number"
                      bind:value={formData[field.name]}
                      placeholder={field.placeholder || ""}
                      required={field.required}
                      min={field.validation?.min}
                      max={field.validation?.max}
                      step={field.validation?.step}
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  {:else if field.field_type === "textarea"}
                    <textarea
                      bind:value={formData[field.name]}
                      placeholder={field.placeholder || ""}
                      required={field.required}
                      rows={field.validation?.rows || 4}
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    ></textarea>
                  {:else if field.field_type === "select"}
                    <select
                      bind:value={formData[field.name]}
                      required={field.required}
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    >
                      <option value="">選択してください</option>
                      {#if field.options}
                        {#each field.options as option}
                          <option value={option.value}>{option.label}</option>
                        {/each}
                      {/if}
                    </select>
                  {:else if field.field_type === "radio"}
                    <div class="space-y-2">
                      {#if field.options}
                        {#each field.options as option}
                          <label class="flex items-center">
                            <input
                              type="radio"
                              bind:group={formData[field.name]}
                              value={option.value}
                              required={field.required}
                              class="mr-2"
                            />
                            <span>{option.label}</span>
                          </label>
                        {/each}
                      {/if}
                    </div>
                  {:else if field.field_type === "checkbox"}
                    <label class="flex items-center">
                      <input
                        type="checkbox"
                        bind:checked={formData[field.name]}
                        required={field.required}
                        class="mr-2"
                      />
                      <span>{field.placeholder || "チェックする"}</span>
                    </label>
                  {:else if field.field_type === "date"}
                    <input
                      type="date"
                      bind:value={formData[field.name]}
                      required={field.required}
                      min={field.validation?.min}
                      max={field.validation?.max}
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  {:else if field.field_type === "time"}
                    <input
                      type="time"
                      bind:value={formData[field.name]}
                      required={field.required}
                      min={field.validation?.min}
                      max={field.validation?.max}
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  {:else if field.field_type === "file"}
                    <input
                      type="file"
                      on:change={(e) => {
                        const files = e.currentTarget.files;
                        if (files && files.length > 0) {
                          formData[field.name] = files[0];
                        }
                      }}
                      required={field.required}
                      accept={field.validation?.accept}
                      class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  {/if}
                </div>
              {/each}
            {/if}

            <div class="pt-4">
              <button
                type="submit"
                disabled={submitting}
                class="w-full bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
              >
                {#if submitting}
                  送信中...
                {:else}
                  {form.settings?.submit_button_text || "送信"}
                {/if}
              </button>
            </div>
          </form>
        </div>

        {#if form.settings?.footer_content}
          <div class="p-8 border-t">
            <div class="prose max-w-none">
              {@html renderMarkdown(form.settings.footer_content)}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
