<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { authStore, type User } from "$lib/stores/authStore";

  let name = "";
  let email = "";
  let password = "";
  let confirmPassword = "";
  let isSubmitting = false;
  let error = "";

  // ログイン状態確認
  onMount(() => {
    // 認証状態のチェック
    authStore.subscribe((state) => {
      if (state.isAuthenticated) {
        goto("/templates");
      }
    });
  });

  async function handleRegister() {
    // バリデーション
    if (!name.trim()) {
      error = "名前を入力してください";
      return;
    }

    if (!email.trim() || !email.includes("@")) {
      error = "有効なメールアドレスを入力してください";
      return;
    }

    if (!password || password.length < 8) {
      error = "パスワードは8文字以上で入力してください";
      return;
    }

    if (password !== confirmPassword) {
      error = "パスワードと確認用パスワードが一致しません";
      return;
    }

    try {
      isSubmitting = true;
      error = "";

      const response = await fetch("/api/auth/register", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          name,
          email,
          password,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(
          errorData.message ||
            `ユーザー登録に失敗しました: ${
              response.status === 409
                ? "このメールアドレスは既に使用されています"
                : response.status
            }`,
        );
      }

      const data = await response.json();

      // 認証ストアを更新
      authStore.login(data.token, data.refresh_token, data.user);

      // リダイレクト
      goto("/templates");
    } catch (err) {
      error = err instanceof Error ? err.message : "エラーが発生しました";
      console.error("Registration error:", err);
    } finally {
      isSubmitting = false;
    }
  }
</script>

<svelte:head>
  <title>ユーザー登録 - MarkMail</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-white">
  <div class="w-full max-w-md px-6">
    <!-- Logo and Header -->
    <div class="text-center mb-12 animate-in">
      <h1 class="text-2xl tracking-tight font-light text-black mb-8">
        MARKMAIL
      </h1>
      <h2 class="text-3xl md:text-4xl font-light text-black">Get started</h2>
      <p class="mt-3 text-gray-600 font-light">
        無料でアカウントを作成しましょう
      </p>
    </div>

    {#if error}
      <div
        class="mb-6 p-4 bg-red-50 border border-red-100 rounded-2xl animate-in"
      >
        <p class="text-sm text-red-600 font-light">
          {error}
        </p>
      </div>
    {/if}

    <form
      class="space-y-6 animate-in"
      on:submit|preventDefault={handleRegister}
    >
      <div class="space-y-4">
        <div>
          <label for="name" class="label">お名前</label>
          <input
            id="name"
            name="name"
            type="text"
            required
            bind:value={name}
            class="input-field"
            placeholder="山田 太郎"
          />
        </div>

        <div>
          <label for="email-address" class="label">メールアドレス</label>
          <input
            id="email-address"
            name="email"
            type="email"
            autocomplete="email"
            required
            bind:value={email}
            class="input-field"
            placeholder="your@email.com"
          />
        </div>

        <div>
          <label for="password" class="label">パスワード</label>
          <input
            id="password"
            name="password"
            type="password"
            autocomplete="new-password"
            required
            bind:value={password}
            class="input-field"
            placeholder="8文字以上で入力"
          />
          <p class="mt-1 text-xs text-gray-500 font-light">
            8文字以上の安全なパスワードを設定してください
          </p>
        </div>

        <div>
          <label for="confirm-password" class="label">パスワード（確認）</label>
          <input
            id="confirm-password"
            name="confirm-password"
            type="password"
            autocomplete="new-password"
            required
            bind:value={confirmPassword}
            class="input-field"
            placeholder="パスワードを再入力"
          />
        </div>
      </div>

      <div class="space-y-4">
        <button
          type="submit"
          disabled={isSubmitting}
          class="btn-primary w-full {isSubmitting
            ? 'opacity-50 cursor-not-allowed'
            : ''}"
        >
          {#if isSubmitting}
            <span class="inline-flex items-center">
              <svg
                class="animate-spin -ml-1 mr-3 h-5 w-5 text-white"
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
                ></path>
              </svg>
              登録中...
            </span>
          {:else}
            無料で始める
            <svg
              class="w-5 h-5 ml-2"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13 7l5 5m0 0l-5 5m5-5H6"
              />
            </svg>
          {/if}
        </button>

        <p class="text-xs text-center text-gray-500 font-light">
          登録することで、利用規約とプライバシーポリシーに同意したものとみなされます
        </p>
      </div>

      <div class="pt-6 text-center border-t border-gray-100">
        <p class="text-sm text-gray-600 font-light">
          既にアカウントをお持ちの方は
          <a href="/auth/login" class="text-black hover:underline font-normal">
            ログイン
          </a>
        </p>
      </div>
    </form>

    <!-- Back to Home -->
    <div class="mt-12 text-center">
      <a
        href="/lp"
        class="text-sm text-gray-500 hover:text-gray-700 transition-colors font-light inline-flex items-center"
      >
        <svg
          class="w-4 h-4 mr-2"
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
        ホームに戻る
      </a>
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
