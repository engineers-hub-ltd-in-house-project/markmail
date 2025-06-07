<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { authStore, type User } from "$lib/stores/authStore";
  import { get } from "svelte/store";

  let email = "";
  let password = "";
  let rememberMe = false;
  let isSubmitting = false;
  let error = "";

  // ログイン状態確認
  onMount(() => {
    // 現在の認証状態を即座にチェック
    const currentState = get(authStore);
    if (currentState.isAuthenticated) {
      goto("/templates");
      return;
    }

    // 認証状態の変更を監視
    const unsubscribe = authStore.subscribe((state) => {
      if (state.isAuthenticated) {
        goto("/templates");
      }
    });

    return unsubscribe;
  });

  async function handleLogin() {
    if (!email.trim() || !password) {
      error = "メールアドレスとパスワードを入力してください";
      return;
    }

    try {
      isSubmitting = true;
      error = "";

      const response = await fetch("/api/auth/login", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ email, password }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(
          errorData.message ||
            `ログインに失敗しました: ${response.status === 401 ? "認証情報が正しくありません" : response.status}`,
        );
      }

      const data = await response.json();

      // 認証ストアを更新
      authStore.login(data.token, data.refresh_token, data.user);

      // リダイレクト
      goto("/templates");
    } catch (err) {
      error = err instanceof Error ? err.message : "エラーが発生しました";
      console.error("Login error:", err);
    } finally {
      isSubmitting = false;
    }
  }
</script>

<svelte:head>
  <title>ログイン - MarkMail</title>
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
        アカウントにログイン
      </h2>
    </div>

    {#if error}
      <div class="rounded-md bg-red-50 p-4 border border-red-200">
        <div class="text-sm text-red-700">
          {error}
        </div>
      </div>
    {/if}

    <form class="mt-8 space-y-6" on:submit|preventDefault={handleLogin}>
      <input type="hidden" name="remember" value="true" />
      <div class="rounded-md shadow-sm -space-y-px">
        <div>
          <label for="email-address" class="sr-only">メールアドレス</label>
          <input
            id="email-address"
            name="email"
            type="email"
            autocomplete="email"
            bind:value={email}
            class="appearance-none rounded-t-md relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10"
            placeholder="メールアドレス"
          />
        </div>
        <div>
          <label for="password" class="sr-only">パスワード</label>
          <input
            id="password"
            name="password"
            type="password"
            autocomplete="current-password"
            bind:value={password}
            class="appearance-none rounded-b-md relative block w-full px-3 py-2 border border-gray-300 placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-blue-500 focus:border-blue-500 focus:z-10"
            placeholder="パスワード"
          />
        </div>
      </div>

      <div class="flex items-center justify-between">
        <div class="flex items-center">
          <input
            id="remember-me"
            name="remember-me"
            type="checkbox"
            bind:checked={rememberMe}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
          />
          <label for="remember-me" class="ml-2 block text-sm text-gray-900">
            ログイン状態を保存
          </label>
        </div>

        <div class="text-sm">
          <a
            href="/auth/forgot-password"
            class="text-blue-600 hover:text-blue-500"
          >
            パスワードを忘れた場合
          </a>
        </div>
      </div>

      <div>
        <button
          type="submit"
          disabled={isSubmitting}
          class="group relative w-full flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:bg-blue-400"
        >
          {isSubmitting ? "ログイン中..." : "ログイン"}
        </button>
      </div>

      <div class="text-center">
        <p class="text-sm text-gray-600">
          アカウントをお持ちでない場合は
          <a
            href="/auth/register"
            class="font-medium text-blue-600 hover:text-blue-500"
          >
            ユーザー登録
          </a>
          してください
        </p>
      </div>
    </form>
  </div>
</div>
