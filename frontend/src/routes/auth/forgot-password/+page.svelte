<script lang="ts">
  import { goto } from "$app/navigation";

  let email = "";
  let loading = false;
  let error = "";
  let success = false;

  async function handleSubmit() {
    if (!email) {
      error = "メールアドレスを入力してください";
      return;
    }

    loading = true;
    error = "";

    try {
      const response = await fetch("/api/auth/forgot-password", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ email }),
      });

      if (response.ok) {
        success = true;
      } else {
        const data = await response.json();
        error = data.error || "パスワードリセットのリクエストに失敗しました";
      }
    } catch (err: any) {
      error = "ネットワークエラーが発生しました";
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>パスワードリセット - MarkMail</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center section">
  <div class="container-narrow w-full max-w-md">
    <div class="text-center mb-12 animate-in">
      <h1 class="text-3xl font-light text-black mb-4">
        パスワードをお忘れですか？
      </h1>
      {#if !success}
        <p class="text-gray-600 font-light">
          登録時のメールアドレスを入力してください。パスワードリセット用のリンクをお送りします。
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
          <h2 class="text-xl font-light mb-4">メールを送信しました</h2>
          <p class="text-gray-600 font-light mb-8">
            パスワードリセットの手順をメールでお送りしました。<br />
            メールボックスをご確認ください。
          </p>
          <button
            on:click={() => goto("/auth/login")}
            class="btn-secondary w-full"
          >
            ログインページに戻る
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
          <label for="email" class="label">メールアドレス</label>
          <input
            type="email"
            id="email"
            bind:value={email}
            placeholder="your@email.com"
            class="input-field"
            required
            disabled={loading}
          />
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
            送信中...
          {:else}
            リセットリンクを送信
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
