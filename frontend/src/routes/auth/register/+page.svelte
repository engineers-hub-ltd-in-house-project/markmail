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

<div
  class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8"
>
  <div class="max-w-md w-full space-y-8">
    <div>
      <h1 class="text-center text-3xl font-extrabold text-gray-900">
        MarkMail
      </h1>
      <h2 class="mt-6 text-center text-2xl font-bold text-gray-900">
        新規アカウント登録
      </h2>
    </div>

    {#if error}
      <div class="rounded-md bg-red-50 p-4 border border-red-200">
        <div class="text-sm text-red-700">
          {error}
        </div>
      </div>
    {/if}

    <form class="mt-8 space-y-6" on:submit|preventDefault={handleRegister}>
      <div class="rounded-md shadow-sm space-y-3">
        <div>
          <label for="name" class="sr-only">名前</label>
          <input
            id="name"
            name="name"
            type="text"
            required
            bind:value={name}
            class="appearance-none rounded-md relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10"
            placeholder="名前"
          />
        </div>
        <div>
          <label for="email-address" class="sr-only">メールアドレス</label>
          <input
            id="email-address"
            name="email"
            type="email"
            autocomplete="email"
            required
            bind:value={email}
            class="appearance-none rounded-md relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10"
            placeholder="メールアドレス"
          />
        </div>
        <div>
          <label for="password" class="sr-only">パスワード</label>
          <input
            id="password"
            name="password"
            type="password"
            autocomplete="new-password"
            required
            bind:value={password}
            class="appearance-none rounded-md relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10"
            placeholder="パスワード (8文字以上)"
          />
        </div>
        <div>
          <label for="confirm-password" class="sr-only"
            >パスワード（確認用）</label
          >
          <input
            id="confirm-password"
            name="confirm-password"
            type="password"
            autocomplete="new-password"
            required
            bind:value={confirmPassword}
            class="appearance-none rounded-md relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10"
            placeholder="パスワード（確認用）"
          />
        </div>
      </div>

      <div>
        <button
          type="submit"
          disabled={isSubmitting}
          class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:bg-blue-400"
        >
          {isSubmitting ? "登録中..." : "アカウント登録"}
        </button>
      </div>

      <div class="text-center">
        <p class="text-sm text-gray-600">
          既にアカウントをお持ちの場合は
          <a
            href="/auth/login"
            class="font-medium text-blue-600 hover:text-blue-500"
          >
            ログイン
          </a>
          してください
        </p>
      </div>
    </form>
  </div>
</div>
