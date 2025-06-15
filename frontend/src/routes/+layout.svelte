<script lang="ts">
  import "../app.css";
  import { authStore, type User } from "$lib/stores/authStore";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { page } from "$app/stores";

  // ログアウト処理
  function handleLogout() {
    authStore.logout();
    goto("/auth/login");
  }

  // ドロップダウンメニュー制御
  let showDropdown = false;

  // ページ遷移時にドロップダウンメニューを閉じる
  $: {
    $page;
    showDropdown = false;
  }

  // サインイン状態
  let isAuthenticated: boolean;
  let user: User | null;

  // ストアから認証状態を監視
  authStore.subscribe((state) => {
    isAuthenticated = state.isAuthenticated;
    user = state.user;
  });

  // 特定のルートでは非表示にする
  $: isAuthPage = $page.url.pathname.startsWith("/auth");
  $: isLpPage = $page.url.pathname === "/lp";
</script>

{#if !isAuthPage && !isLpPage}
  <div class="min-h-screen bg-white">
    <nav
      class="fixed w-full px-6 lg:px-12 py-6 z-50 bg-white/90 backdrop-blur-md border-b border-gray-100"
    >
      <div class="max-w-7xl mx-auto flex justify-between items-center">
        <div class="flex items-center">
          <a href="/" class="text-2xl tracking-tight font-light text-black"
            >MARKMAIL</a
          >
        </div>

        <div class="flex items-center space-x-8">
          {#if isAuthenticated}
            <!-- 認証済みの場合のナビゲーション -->
            <a href="/templates" class="nav-link text-sm"> テンプレート </a>
            <a href="/campaigns" class="nav-link text-sm"> キャンペーン </a>
            <a href="/subscribers" class="nav-link text-sm"> 購読者 </a>
            <a href="/forms" class="nav-link text-sm"> フォーム </a>
            <a href="/sequences" class="nav-link text-sm"> シーケンス </a>
            <a href="/ai" class="nav-link text-sm"> AI機能 </a>

            <!-- ユーザーメニュー -->
            <div class="relative ml-4">
              <div>
                <button
                  type="button"
                  class="flex items-center p-2 rounded-full hover:bg-gray-100 transition-colors"
                  id="user-menu"
                  aria-expanded={showDropdown}
                  aria-haspopup="true"
                  on:click={() => (showDropdown = !showDropdown)}
                >
                  <span class="sr-only">メニューを開く</span>
                  <div
                    class="h-10 w-10 rounded-full bg-gray-900 flex items-center justify-center text-white"
                  >
                    {#if user?.avatar_url}
                      <img
                        src={user.avatar_url}
                        alt={user.name}
                        class="h-10 w-10 rounded-full"
                      />
                    {:else}
                      <span class="text-sm font-light"
                        >{user?.name?.charAt(0) || "U"}</span
                      >
                    {/if}
                  </div>
                </button>
              </div>

              {#if showDropdown}
                <div
                  class="origin-top-right absolute right-0 mt-2 w-56 rounded-2xl shadow-xl bg-white border border-gray-100 overflow-hidden"
                  role="menu"
                  aria-orientation="vertical"
                  aria-labelledby="user-menu"
                >
                  <div class="py-2">
                    <div class="px-6 py-3 border-b border-gray-100">
                      <p class="text-sm font-light text-gray-900">
                        {user?.name || "ユーザー"}
                      </p>
                      <p class="text-xs font-light text-gray-500 mt-1">
                        {user?.email || ""}
                      </p>
                    </div>
                    <a href="/profile" class="dropdown-item" role="menuitem">
                      プロフィール
                    </a>
                    <a href="/settings" class="dropdown-item" role="menuitem">
                      設定
                    </a>
                    <a
                      href="/subscription"
                      class="dropdown-item"
                      role="menuitem"
                    >
                      サブスクリプション
                    </a>
                    <a
                      href="/subscription/payment-history"
                      class="dropdown-item"
                      role="menuitem"
                    >
                      支払い履歴
                    </a>
                    <div class="border-t border-gray-100 mt-2 pt-2">
                      <button
                        on:click={handleLogout}
                        class="dropdown-item w-full text-left text-red-600 hover:bg-red-50"
                        role="menuitem"
                      >
                        ログアウト
                      </button>
                    </div>
                  </div>
                </div>
              {/if}
            </div>
          {:else}
            <!-- 未認証の場合のナビゲーション -->
            <a
              href="/auth/login"
              class="text-sm text-gray-800 hover:text-black transition-colors font-light"
            >
              ログイン
            </a>
            <a href="/auth/register" class="btn-primary btn-sm">
              無料で始める
            </a>
          {/if}
        </div>
      </div>
    </nav>

    <main class="pt-20">
      <slot />
    </main>
  </div>
{:else}
  <!-- 認証画面やLPページではヘッダーを表示しない -->
  <main>
    <slot />
  </main>
{/if}
