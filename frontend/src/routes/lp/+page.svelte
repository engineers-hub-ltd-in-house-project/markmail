<script lang="ts">
  import { goto } from "$app/navigation";
</script>

<svelte:head>
  <title
    >MarkMail - エンジニア向けマークダウンベースメールマーケティングツール</title
  >
  <meta
    name="description"
    content="MarkMailは、エンジニアのためのシンプルで強力なメールマーケティングツールです。Markdownでメールを作成し、効率的にキャンペーンを管理できます。"
  />
</svelte:head>

<div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
  <nav class="container mx-auto px-6 py-8">
    <div class="flex justify-between items-center">
      <div class="flex items-center">
        <div
          class="w-10 h-10 bg-blue-600 rounded-lg flex items-center justify-center mr-3"
        >
          <span class="text-white font-bold text-xl">M</span>
        </div>
        <h1 class="text-2xl font-bold text-gray-900">MarkMail</h1>
      </div>
      <div class="space-x-4">
        <button
          on:click={() => goto("/auth/login")}
          class="px-4 py-2 text-gray-700 hover:text-gray-900 transition-colors"
        >
          ログイン
        </button>
        <button
          on:click={() => goto("/auth/register")}
          class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          無料で始める
        </button>
      </div>
    </div>
  </nav>

  <main class="container mx-auto px-6 py-16">
    <section class="text-center mb-20">
      <h2 class="text-5xl font-bold text-gray-900 mb-6">
        エンジニアのための<br />
        シンプルなメールマーケティング
      </h2>
      <p class="text-xl text-gray-600 mb-8 max-w-2xl mx-auto">
        Markdownでメールを作成し、効率的にキャンペーンを管理。
        技術者にとって使いやすいインターフェースで、メールマーケティングを始めましょう。
      </p>
      <button
        on:click={() => goto("/auth/register")}
        class="px-8 py-4 bg-blue-600 text-white text-lg font-semibold rounded-lg hover:bg-blue-700 transition-colors shadow-lg"
      >
        無料アカウントを作成
      </button>
    </section>

    <section class="grid md:grid-cols-3 gap-8 mb-20">
      <div class="bg-white p-8 rounded-lg shadow-md">
        <div
          class="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center mb-4"
        >
          <svg
            class="w-6 h-6 text-blue-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
            />
          </svg>
        </div>
        <h3 class="text-xl font-semibold mb-3">Markdownエディタ</h3>
        <p class="text-gray-600">
          慣れ親しんだMarkdown記法でメールを作成。リアルタイムプレビューで確認しながら編集できます。
        </p>
      </div>

      <div class="bg-white p-8 rounded-lg shadow-md">
        <div
          class="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center mb-4"
        >
          <svg
            class="w-6 h-6 text-green-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
            />
          </svg>
        </div>
        <h3 class="text-xl font-semibold mb-3">キャンペーン管理</h3>
        <p class="text-gray-600">
          メールキャンペーンの作成、送信、追跡をシンプルなインターフェースで管理できます。
        </p>
      </div>

      <div class="bg-white p-8 rounded-lg shadow-md">
        <div
          class="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center mb-4"
        >
          <svg
            class="w-6 h-6 text-purple-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
            />
          </svg>
        </div>
        <h3 class="text-xl font-semibold mb-3">購読者管理</h3>
        <p class="text-gray-600">
          購読者リストの管理、セグメント化、インポート/エクスポート機能を提供します。
        </p>
      </div>
    </section>

    <section class="bg-white rounded-lg shadow-lg p-12 text-center">
      <h2 class="text-3xl font-bold mb-4">今すぐ始めましょう</h2>
      <p class="text-lg text-gray-600 mb-8">
        無料アカウントを作成して、MarkMailの全機能をお試しください。
      </p>
      <div class="space-x-4">
        <button
          on:click={() => goto("/auth/register")}
          class="px-8 py-3 bg-blue-600 text-white font-semibold rounded-lg hover:bg-blue-700 transition-colors"
        >
          アカウント作成
        </button>
        <button
          on:click={() => goto("/auth/login")}
          class="px-8 py-3 border border-gray-300 text-gray-700 font-semibold rounded-lg hover:bg-gray-50 transition-colors"
        >
          ログイン
        </button>
      </div>
    </section>
  </main>

  <footer class="container mx-auto px-6 py-8 mt-20 border-t border-gray-200">
    <div class="text-center text-gray-600">
      <p>&copy; 2024 MarkMail. All rights reserved.</p>
    </div>
  </footer>
</div>
