<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  let scrollY = 0;
  let innerHeight = 0;
  let heroRef: HTMLElement;
  let featuresRef: HTMLElement;

  // Intersection Observer for scroll animations
  onMount(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add("animate-in");
          }
        });
      },
      { threshold: 0.1 },
    );

    // Observe all sections
    document.querySelectorAll(".observe-section").forEach((el) => {
      observer.observe(el);
    });

    return () => observer.disconnect();
  });
</script>

<svelte:window bind:scrollY bind:innerHeight />

<svelte:head>
  <title>MarkMail - AI時代のマーケティングオートメーション</title>
  <meta
    name="description"
    content="MarkMailは、AIと相性抜群のマーケティングオートメーションツール。Markdown形式でAIが理解しやすいメールを作成し、次世代のマーケティングを実現します。"
  />
</svelte:head>

<div class="min-h-screen bg-white">
  <!-- Navigation -->
  <nav
    class="fixed w-full px-6 lg:px-12 py-6 z-50 bg-white/90 backdrop-blur-md border-b border-gray-100 transition-all duration-300"
    style="background-color: {scrollY > 50
      ? 'rgba(255, 255, 255, 0.95)'
      : 'transparent'}; 
              border-bottom-color: {scrollY > 50 ? '#f3f4f6' : 'transparent'};"
  >
    <div class="max-w-7xl mx-auto flex justify-between items-center">
      <div class="flex items-center">
        <h1 class="text-2xl tracking-tight font-light text-black">MARKMAIL</h1>
      </div>
      <div class="flex items-center space-x-8">
        <button
          on:click={() => goto("/auth/login")}
          class="text-sm text-gray-800 hover:text-black transition-colors font-light"
        >
          ログイン
        </button>
        <button
          on:click={() => goto("/auth/register")}
          class="px-6 py-3 bg-black text-white text-sm hover:bg-gray-900 transition-all duration-200 font-light rounded-full"
        >
          無料で始める
        </button>
      </div>
    </div>
  </nav>

  <main class="w-full">
    <!-- Hero Section -->
    <section
      bind:this={heroRef}
      class="px-6 lg:px-12 pt-32 pb-20 md:pt-48 md:pb-32 max-w-7xl mx-auto observe-section opacity-0 translate-y-10 transition-all duration-1000"
    >
      <div class="max-w-4xl">
        <h2
          class="text-5xl md:text-7xl font-light leading-tight text-black mb-8 tracking-tight"
        >
          AIと相性抜群の<br />
          <span
            class="text-gradient bg-gradient-to-r from-gray-900 via-gray-700 to-gray-900 bg-clip-text text-transparent"
            >メールマーケティング</span
          >
        </h2>
        <p
          class="text-xl md:text-2xl text-gray-600 mb-6 font-light leading-relaxed max-w-3xl"
        >
          AIが理解しやすいMarkdown形式でメールを作成。
          従来のツールとは異なる、次世代のマーケティングオートメーション。
        </p>
        <p class="text-lg text-gray-500 mb-12 max-w-3xl font-light">
          フォームビルダー、メールシーケンス、CRM連携など、
          マーケティングオートメーションに必要な全ての機能を提供。
        </p>
        <div class="flex flex-col sm:flex-row gap-4">
          <button
            on:click={() => goto("/auth/register")}
            class="group px-8 py-4 bg-black text-white hover:bg-gray-900 transition-all duration-300 font-light text-lg rounded-full flex items-center justify-center"
          >
            無料アカウントを作成
            <svg
              class="w-5 h-5 ml-2 group-hover:translate-x-1 transition-transform"
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
          </button>
          <button
            on:click={() => goto("/auth/login")}
            class="px-8 py-4 border border-gray-300 text-black hover:border-black transition-all duration-300 font-light text-lg rounded-full"
          >
            デモを見る
          </button>
        </div>
      </div>

      <!-- Scroll indicator -->
      <div
        class="absolute bottom-8 left-1/2 transform -translate-x-1/2 animate-bounce"
      >
        <svg
          class="w-6 h-6 text-gray-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M19 14l-7 7m0 0l-7-7m7 7V3"
          />
        </svg>
      </div>
    </section>

    <!-- Features Section -->
    <section
      bind:this={featuresRef}
      class="px-6 lg:px-12 py-20 bg-gradient-to-b from-gray-50 to-white"
    >
      <div class="max-w-7xl mx-auto">
        <div
          class="text-center mb-16 observe-section opacity-0 translate-y-10 transition-all duration-1000"
        >
          <h2 class="text-4xl md:text-5xl font-light text-black mb-4">
            シンプルで強力な機能
          </h2>
          <p class="text-xl text-gray-600 font-light">
            必要な機能だけを、最高の形で提供します
          </p>
        </div>

        <div class="grid md:grid-cols-3 gap-8 mb-12">
          <!-- Feature 1 -->
          <div
            class="observe-section opacity-0 translate-y-10 transition-all duration-1000 delay-100"
          >
            <div
              class="bg-white p-10 rounded-2xl hover:shadow-xl transition-all duration-300 border border-gray-100 group"
            >
              <div
                class="mb-6 w-12 h-12 bg-black rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform"
              >
                <svg
                  class="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="1.5"
                    d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                  />
                </svg>
              </div>
              <h3 class="text-2xl font-light mb-4 text-black">
                Markdownエディタ
              </h3>
              <p class="text-gray-600 font-light leading-relaxed">
                慣れ親しんだMarkdown記法でメールを作成。リアルタイムプレビューで確認しながら編集できます。
              </p>
            </div>
          </div>

          <!-- Feature 2 -->
          <div
            class="observe-section opacity-0 translate-y-10 transition-all duration-1000 delay-200"
          >
            <div
              class="bg-white p-10 rounded-2xl hover:shadow-xl transition-all duration-300 border border-gray-100 group"
            >
              <div
                class="mb-6 w-12 h-12 bg-black rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform"
              >
                <svg
                  class="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="1.5"
                    d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
                  />
                </svg>
              </div>
              <h3 class="text-2xl font-light mb-4 text-black">
                キャンペーン管理
              </h3>
              <p class="text-gray-600 font-light leading-relaxed">
                メールキャンペーンの作成、送信、追跡をシンプルなインターフェースで管理できます。
              </p>
            </div>
          </div>

          <!-- Feature 3 -->
          <div
            class="observe-section opacity-0 translate-y-10 transition-all duration-1000 delay-300"
          >
            <div
              class="bg-white p-10 rounded-2xl hover:shadow-xl transition-all duration-300 border border-gray-100 group"
            >
              <div
                class="mb-6 w-12 h-12 bg-black rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform"
              >
                <svg
                  class="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="1.5"
                    d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
                  />
                </svg>
              </div>
              <h3 class="text-2xl font-light mb-4 text-black">購読者管理</h3>
              <p class="text-gray-600 font-light leading-relaxed">
                購読者リストの管理、セグメント化、インポート/エクスポート機能を提供します。
              </p>
            </div>
          </div>

          <!-- Feature 4 -->
          <div
            class="observe-section opacity-0 translate-y-10 transition-all duration-1000 delay-400"
          >
            <div
              class="bg-white p-10 rounded-2xl hover:shadow-xl transition-all duration-300 border border-gray-100 group"
            >
              <div
                class="mb-6 w-12 h-12 bg-black rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform"
              >
                <svg
                  class="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="1.5"
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                  />
                </svg>
              </div>
              <h3 class="text-2xl font-light mb-4 text-black">
                フォームビルダー
              </h3>
              <p class="text-gray-600 font-light leading-relaxed">
                マークダウンベースのフォーム作成機能。埋め込み可能なフォームで購読者を獲得できます。
              </p>
            </div>
          </div>

          <!-- Feature 5 -->
          <div
            class="observe-section opacity-0 translate-y-10 transition-all duration-1000 delay-500"
          >
            <div
              class="bg-white p-10 rounded-2xl hover:shadow-xl transition-all duration-300 border border-gray-100 group"
            >
              <div
                class="mb-6 w-12 h-12 bg-black rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform"
              >
                <svg
                  class="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="1.5"
                    d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
              </div>
              <h3 class="text-2xl font-light mb-4 text-black">
                メールシーケンス
              </h3>
              <p class="text-gray-600 font-light leading-relaxed">
                自動配信シーケンスを設計。ウェルカムメールや育成メールを自動化できます。
              </p>
            </div>
          </div>

          <!-- Feature 6 -->
          <div
            class="observe-section opacity-0 translate-y-10 transition-all duration-1000 delay-600"
          >
            <div
              class="bg-white p-10 rounded-2xl hover:shadow-xl transition-all duration-300 border border-gray-100 group"
            >
              <div
                class="mb-6 w-12 h-12 bg-black rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform"
              >
                <svg
                  class="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="1.5"
                    d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
                  />
                </svg>
              </div>
              <h3 class="text-2xl font-light mb-4 text-black">
                外部システム連携
              </h3>
              <p class="text-gray-600 font-light leading-relaxed">
                Salesforce、HubSpot、Zapierなど主要なツールとシームレスに連携。
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Roadmap Section -->
    <section class="px-6 lg:px-12 py-20 bg-white">
      <div class="max-w-7xl mx-auto">
        <div
          class="text-center mb-16 observe-section opacity-0 translate-y-10 transition-all duration-1000"
        >
          <h2 class="text-4xl md:text-5xl font-light text-black mb-4">
            今後のアップデート
          </h2>
          <p class="text-xl text-gray-600 font-light">
            継続的に進化し続けるプラットフォーム
          </p>
        </div>

        <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
          <div
            class="observe-section opacity-0 translate-y-10 transition-all duration-1000 delay-100"
          >
            <div
              class="bg-gray-50 p-8 rounded-2xl hover:shadow-xl transition-all duration-300 border border-gray-100 group"
            >
              <div
                class="mb-6 w-12 h-12 bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform"
              >
                <svg
                  class="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
                  />
                </svg>
              </div>
              <h4 class="text-xl font-light mb-2 text-gray-900">
                ダブルオプトイン
              </h4>
              <p class="text-gray-600 font-light text-sm leading-relaxed">
                確実な同意取得とスパム防止機能
              </p>
            </div>
          </div>

          <div
            class="observe-section opacity-0 translate-y-10 transition-all duration-1000 delay-200"
          >
            <div
              class="bg-gray-50 p-8 rounded-2xl hover:shadow-xl transition-all duration-300 border border-gray-100 group"
            >
              <div
                class="mb-6 w-12 h-12 bg-gradient-to-br from-green-500 to-green-600 rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform"
              >
                <svg
                  class="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                  />
                </svg>
              </div>
              <h4 class="text-xl font-light mb-2 text-gray-900">
                配信設計ツール
              </h4>
              <p class="text-gray-600 font-light text-sm leading-relaxed">
                ビジュアルフローエディタで複雑な配信を設計
              </p>
            </div>
          </div>

          <div
            class="observe-section opacity-0 translate-y-10 transition-all duration-1000 delay-300"
          >
            <div
              class="bg-gray-50 p-8 rounded-2xl hover:shadow-xl transition-all duration-300 border border-gray-100 group"
            >
              <div
                class="mb-6 w-12 h-12 bg-gradient-to-br from-purple-500 to-purple-600 rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform"
              >
                <svg
                  class="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01"
                  />
                </svg>
              </div>
              <h4 class="text-xl font-light mb-2 text-gray-900">A/Bテスト</h4>
              <p class="text-gray-600 font-light text-sm leading-relaxed">
                件名、コンテンツ、送信時刻の最適化
              </p>
            </div>
          </div>

          <div
            class="observe-section opacity-0 translate-y-10 transition-all duration-1000 delay-400"
          >
            <div
              class="bg-gray-50 p-8 rounded-2xl hover:shadow-xl transition-all duration-300 border border-gray-100 group"
            >
              <div
                class="mb-6 w-12 h-12 bg-gradient-to-br from-orange-500 to-orange-600 rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform"
              >
                <svg
                  class="w-6 h-6 text-white"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
                  />
                </svg>
              </div>
              <h4 class="text-xl font-light mb-2 text-gray-900">AI機能</h4>
              <p class="text-gray-600 font-light text-sm leading-relaxed">
                コンテンツ生成と最適化提案
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Benefits Section -->
    <section class="px-6 lg:px-12 py-20 bg-gray-50">
      <div class="max-w-7xl mx-auto">
        <div class="grid md:grid-cols-2 gap-12">
          <div
            class="observe-section opacity-0 translate-x-10 transition-all duration-1000"
          >
            <div
              class="bg-white p-10 rounded-2xl shadow-sm border border-gray-100"
            >
              <div class="text-3xl mb-6">🔐</div>
              <h3 class="text-2xl font-light mb-4 text-gray-900">
                エンタープライズレベルのセキュリティ
              </h3>
              <ul class="space-y-4 text-gray-700 font-light">
                <li class="flex items-start">
                  <svg
                    class="w-5 h-5 text-green-500 mr-3 mt-0.5 flex-shrink-0"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                      clip-rule="evenodd"
                    />
                  </svg>
                  OWASP Top 10準拠のセキュリティ対策
                </li>
                <li class="flex items-start">
                  <svg
                    class="w-5 h-5 text-green-500 mr-3 mt-0.5 flex-shrink-0"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                      clip-rule="evenodd"
                    />
                  </svg>
                  SQLインジェクション・XSS・CSRF対策
                </li>
                <li class="flex items-start">
                  <svg
                    class="w-5 h-5 text-green-500 mr-3 mt-0.5 flex-shrink-0"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                      clip-rule="evenodd"
                    />
                  </svg>
                  エンドツーエンド暗号化
                </li>
              </ul>
            </div>
          </div>

          <div
            class="observe-section opacity-0 -translate-x-10 transition-all duration-1000"
          >
            <div
              class="bg-white p-10 rounded-2xl shadow-sm border border-gray-100"
            >
              <div class="text-3xl mb-6">⚡</div>
              <h3 class="text-2xl font-light mb-4 text-gray-900">
                高速パフォーマンス
              </h3>
              <ul class="space-y-4 text-gray-700 font-light">
                <li class="flex items-start">
                  <svg
                    class="w-5 h-5 text-green-500 mr-3 mt-0.5 flex-shrink-0"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                      clip-rule="evenodd"
                    />
                  </svg>
                  Redisキャッシュによる高速処理
                </li>
                <li class="flex items-start">
                  <svg
                    class="w-5 h-5 text-green-500 mr-3 mt-0.5 flex-shrink-0"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                      clip-rule="evenodd"
                    />
                  </svg>
                  非同期処理による効率的な配信
                </li>
                <li class="flex items-start">
                  <svg
                    class="w-5 h-5 text-green-500 mr-3 mt-0.5 flex-shrink-0"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                  >
                    <path
                      fill-rule="evenodd"
                      d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                      clip-rule="evenodd"
                    />
                  </svg>
                  スケーラブルなアーキテクチャ
                </li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- CTA Section -->
    <section class="px-6 lg:px-12 py-32 bg-black text-white">
      <div
        class="max-w-4xl mx-auto text-center observe-section opacity-0 translate-y-10 transition-all duration-1000"
      >
        <h2 class="text-4xl md:text-5xl font-light mb-6">今すぐ始めましょう</h2>
        <p class="text-xl mb-12 text-gray-300 font-light">
          無料アカウントを作成して、MarkMailの全機能をお試しください。
        </p>
        <div class="flex flex-col sm:flex-row gap-4 justify-center">
          <button
            on:click={() => goto("/auth/register")}
            class="group px-8 py-4 bg-white text-black hover:bg-gray-100 transition-all duration-300 font-light text-lg rounded-full flex items-center justify-center"
          >
            アカウント作成
            <svg
              class="w-5 h-5 ml-2 group-hover:translate-x-1 transition-transform"
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
          </button>
          <button
            on:click={() => goto("/auth/login")}
            class="px-8 py-4 border border-white text-white hover:bg-white hover:text-black transition-all duration-300 font-light text-lg rounded-full"
          >
            ログイン
          </button>
        </div>
      </div>
    </section>
  </main>

  <!-- Footer -->
  <footer class="px-6 lg:px-12 py-12 bg-white border-t border-gray-100">
    <div class="max-w-7xl mx-auto text-center">
      <p class="text-gray-500 font-light">
        &copy; 2024 MarkMail. All rights reserved.
      </p>
    </div>
  </footer>
</div>

<style>
  /* CSS animations */
  :global(.observe-section) {
    transition-property: opacity, transform;
    transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
  }

  :global(.animate-in) {
    opacity: 1 !important;
    transform: translateY(0) translateX(0) !important;
  }

  .text-gradient {
    background-size: 200% 200%;
    animation: gradient 3s ease infinite;
  }

  @keyframes gradient {
    0% {
      background-position: 0% 50%;
    }
    50% {
      background-position: 100% 50%;
    }
    100% {
      background-position: 0% 50%;
    }
  }
</style>
