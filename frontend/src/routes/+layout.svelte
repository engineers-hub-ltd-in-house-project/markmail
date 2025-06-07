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
</script>

{#if !isAuthPage}
  <div class="min-h-screen bg-gray-50">
    <nav class="bg-white shadow-sm border-b border-gray-200">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between h-16">
          <div class="flex items-center">
            <a href="/" class="flex items-center space-x-2">
              <div
                class="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center"
              >
                <span class="text-white font-bold text-sm">M</span>
              </div>
              <span class="text-xl font-bold text-gray-900">MarkMail</span>
            </a>
          </div>

          <div class="flex items-center space-x-4">
            {#if isAuthenticated}
              <!-- 認証済みの場合のナビゲーション -->
              <a
                href="/templates"
                class="text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium"
              >
                テンプレート
              </a>
              <a
                href="/campaigns"
                class="text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium"
              >
                キャンペーン
              </a>
              <a
                href="/subscribers"
                class="text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium"
              >
                購読者
              </a>

              <!-- ユーザーメニュー -->
              <div class="relative ml-3">
                <div>
                  <button
                    type="button"
                    class="flex items-center max-w-xs text-sm rounded-full focus:outline-none"
                    id="user-menu"
                    aria-expanded={showDropdown}
                    aria-haspopup="true"
                    on:click={() => (showDropdown = !showDropdown)}
                  >
                    <span class="sr-only">メニューを開く</span>
                    <div
                      class="h-8 w-8 rounded-full bg-gray-200 flex items-center justify-center text-gray-600"
                    >
                      {#if user?.avatar_url}
                        <img
                          src={user.avatar_url}
                          alt={user.name}
                          class="h-8 w-8 rounded-full"
                        />
                      {:else}
                        <span class="text-sm font-medium"
                          >{user?.name?.charAt(0) || "U"}</span
                        >
                      {/if}
                    </div>
                  </button>
                </div>

                {#if showDropdown}
                  <div
                    class="origin-top-right absolute right-0 mt-2 w-48 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 focus:outline-none"
                    role="menu"
                    aria-orientation="vertical"
                    aria-labelledby="user-menu"
                  >
                    <div class="py-1">
                      <span
                        class="block px-4 py-2 text-sm text-gray-700 border-b"
                      >
                        {user?.name || "ユーザー"}
                      </span>
                      <a
                        href="/profile"
                        class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                        role="menuitem"
                      >
                        プロフィール
                      </a>
                      <a
                        href="/settings"
                        class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                        role="menuitem"
                      >
                        設定
                      </a>
                      <button
                        on:click={handleLogout}
                        class="block w-full text-left px-4 py-2 text-sm text-red-600 hover:bg-gray-100"
                        role="menuitem"
                      >
                        ログアウト
                      </button>
                    </div>
                  </div>
                {/if}
              </div>
            {:else}
              <!-- 未認証の場合のナビゲーション -->
              <a
                href="/auth/login"
                class="text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium"
              >
                ログイン
              </a>
              <a
                href="/auth/register"
                class="bg-blue-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-blue-700"
              >
                新規登録
              </a>
            {/if}
          </div>
        </div>
      </div>
    </nav>

    <main>
      <slot />
    </main>
  </div>
{:else}
  <!-- 認証画面ではヘッダーを表示しない -->
  <main>
    <slot />
  </main>
{/if}
