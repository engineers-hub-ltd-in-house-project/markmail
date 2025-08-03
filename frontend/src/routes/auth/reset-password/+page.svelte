<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount } from "svelte";

  let newPassword = "";
  let confirmPassword = "";
  let loading = false;
  let error = "";
  let success = false;
  let token = "";
  let showPassword = false;

  onMount(() => {
    token = $page.url.searchParams.get("token") || "";
    if (!token) {
      error = "無効なリセットリンクです";
    }
  });

  async function handleSubmit() {
    error = "";

    if (!newPassword || !confirmPassword) {
      error = "すべてのフィールドを入力してください";
      return;
    }

    if (newPassword.length < 8) {
      error = "パスワードは8文字以上である必要があります";
      return;
    }

    if (newPassword !== confirmPassword) {
      error = "パスワードが一致しません";
      return;
    }

    loading = true;

    try {
      const response = await fetch("/api/auth/reset-password", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          token,
          new_password: newPassword,
        }),
      });

      if (response.ok) {
        success = true;
      } else {
        const data = await response.json();
        error = data.error || "パスワードのリセットに失敗しました";
      }
    } catch (err: any) {
      error = "ネットワークエラーが発生しました";
    } finally {
      loading = false;
    }
  }

  function togglePasswordVisibility() {
    showPassword = !showPassword;
  }
</script>

<svelte:head>
  <title>新しいパスワードの設定 - MarkMail</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center section">
  <div class="container-narrow w-full max-w-md">
    <div class="text-center mb-12 animate-in">
      <h1 class="text-3xl font-light text-black mb-4">
        新しいパスワードの設定
      </h1>
      {#if !success && !error}
        <p class="text-gray-600 font-light">
          新しいパスワードを入力してください
        </p>
      {/if}
    </div>

    {#if success}
      <div class="card animate-in">
        <div class="text-center">
          <div
            class="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-6"
          >
            <svg
              class="w-8 h-8 text-green-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M5 13l4 4L19 7"
              />
            </svg>
          </div>
          <h2 class="text-xl font-light mb-4">
            パスワードがリセットされました
          </h2>
          <p class="text-gray-600 font-light mb-8">
            新しいパスワードでログインできます
          </p>
          <button
            on:click={() => goto("/auth/login")}
            class="btn-primary w-full"
          >
            ログインページへ
          </button>
        </div>
      </div>
    {:else if error && !token}
      <div class="card animate-in">
        <div class="text-center">
          <div
            class="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mx-auto mb-6"
          >
            <svg
              class="w-8 h-8 text-red-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </div>
          <h2 class="text-xl font-light mb-4">無効なリンク</h2>
          <p class="text-gray-600 font-light mb-8">
            リセットリンクが無効または期限切れです
          </p>
          <button
            on:click={() => goto("/auth/forgot-password")}
            class="btn-secondary w-full"
          >
            パスワードリセットをやり直す
          </button>
        </div>
      </div>
    {:else}
      <form on:submit|preventDefault={handleSubmit} class="card animate-in">
        {#if error}
          <div class="mb-6 p-4 bg-red-50 border border-red-200 rounded-xl">
            <p class="text-sm text-red-600">{error}</p>
          </div>
        {/if}

        <div class="mb-6">
          <label for="new-password" class="label">新しいパスワード</label>
          <div class="relative">
            {#if showPassword}
              <input
                type="text"
                id="new-password"
                bind:value={newPassword}
                placeholder="••••••••"
                class="input-field pr-12"
                required
                disabled={loading}
                minlength="8"
              />
            {:else}
              <input
                type="password"
                id="new-password"
                bind:value={newPassword}
                placeholder="••••••••"
                class="input-field pr-12"
                required
                disabled={loading}
                minlength="8"
              />
            {/if}
            <button
              type="button"
              on:click={togglePasswordVisibility}
              class="absolute right-3 top-1/2 -translate-y-1/2 p-2 text-gray-500 hover:text-gray-700"
            >
              {#if showPassword}
                <svg
                  class="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"
                  />
                </svg>
              {:else}
                <svg
                  class="w-5 h-5"
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
              {/if}
            </button>
          </div>
          <p class="text-xs text-gray-500 mt-2 font-light">
            8文字以上で入力してください
          </p>
        </div>

        <div class="mb-8">
          <label for="confirm-password" class="label">パスワード（確認）</label>
          {#if showPassword}
            <input
              type="text"
              id="confirm-password"
              bind:value={confirmPassword}
              placeholder="••••••••"
              class="input-field"
              required
              disabled={loading}
            />
          {:else}
            <input
              type="password"
              id="confirm-password"
              bind:value={confirmPassword}
              placeholder="••••••••"
              class="input-field"
              required
              disabled={loading}
            />
          {/if}
        </div>

        <button
          type="submit"
          disabled={loading}
          class="btn-primary w-full mb-6"
        >
          {#if loading}
            <svg
              class="animate-spin h-5 w-5 mr-3"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
            >
              <circle
                cx="12"
                cy="12"
                r="10"
                stroke-opacity="0.25"
                stroke-width="4"
              />
              <path
                d="M4 12a8 8 0 018-8v8z"
                stroke-opacity="0.75"
                stroke-width="4"
              />
            </svg>
            パスワードをリセット中...
          {:else}
            パスワードをリセット
          {/if}
        </button>

        <div class="text-center">
          <a
            href="/auth/login"
            class="text-sm text-gray-600 hover:text-black font-light"
          >
            ログインページに戻る
          </a>
        </div>
      </form>
    {/if}
  </div>
</div>

<style>
  .animate-in {
    animation: fadeInUp 0.6s ease-out;
  }

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
</style>
