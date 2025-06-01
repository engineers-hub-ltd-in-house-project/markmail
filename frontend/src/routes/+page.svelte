<script lang="ts">
  import { onMount } from 'svelte';
  
  let mounted = false;
  
  onMount(() => {
    mounted = true;
  });
</script>

<svelte:head>
  <title>MarkMail - エンジニア向けマークダウンメールマーケティング</title>
  <meta name="description" content="マークダウンでメールテンプレートを作成し、効率的なメールマーケティングを実現" />
</svelte:head>

<div class="bg-gradient-to-br from-blue-50 to-indigo-100 min-h-screen">
  <!-- ヒーローセクション -->
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-20 pb-16">
    <div class="text-center">
      <h1 class="text-4xl md:text-6xl font-bold text-gray-900 mb-6">
        <span class="text-blue-600">マークダウン</span>で<br />
        メールマーケティングを<br />
        <span class="text-indigo-600">革新</span>する
      </h1>
      
      <p class="text-xl text-gray-600 mb-8 max-w-3xl mx-auto">
        エンジニア向けに設計されたMarkMailで、マークダウンを使って美しいメールテンプレートを作成し、
        効率的なメールマーケティングを実現しましょう。
      </p>
      
      <div class="flex flex-col sm:flex-row gap-4 justify-center">
        <a href="/auth/register" 
           class="bg-blue-600 text-white px-8 py-3 rounded-lg text-lg font-medium hover:bg-blue-700 transition-colors">
          無料で始める
        </a>
        <a href="/templates" 
           class="bg-white text-blue-600 px-8 py-3 rounded-lg text-lg font-medium border-2 border-blue-600 hover:bg-blue-50 transition-colors">
          テンプレートを見る
        </a>
      </div>
    </div>
  </div>

  <!-- 機能紹介セクション -->
  <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
    <div class="text-center mb-16">
      <h2 class="text-3xl font-bold text-gray-900 mb-4">
        エンジニアのためのメールマーケティング
      </h2>
      <p class="text-lg text-gray-600">
        コードを書くように、メールテンプレートを作成
      </p>
    </div>

    <div class="grid md:grid-cols-3 gap-8">
      <!-- マークダウンエディター -->
      <div class="bg-white rounded-xl p-6 shadow-lg">
        <div class="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center mb-4">
          <svg class="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
          </svg>
        </div>
        <h3 class="text-xl font-semibold text-gray-900 mb-2">マークダウンエディター</h3>
        <p class="text-gray-600">
          慣れ親しんだマークダウン記法でメールテンプレートを作成。シンタックスハイライトとリアルタイムプレビュー付き。
        </p>
      </div>

      <!-- テンプレート管理 -->
      <div class="bg-white rounded-xl p-6 shadow-lg">
        <div class="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center mb-4">
          <svg class="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
          </svg>
        </div>
        <h3 class="text-xl font-semibold text-gray-900 mb-2">テンプレート管理</h3>
        <p class="text-gray-600">
          再利用可能なテンプレートライブラリ。変数機能でパーソナライズされたメールを効率的に作成。
        </p>
      </div>

      <!-- GitHub連携 -->
      <div class="bg-white rounded-xl p-6 shadow-lg">
        <div class="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center mb-4">
          <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"></path>
          </svg>
        </div>
        <h3 class="text-xl font-semibold text-gray-900 mb-2">GitHub連携</h3>
        <p class="text-gray-600">
          GitHubリポジトリからマークダウンファイルを直接インポート。バージョン管理と連携したワークフロー。
        </p>
      </div>
    </div>
  </div>

  <!-- CTA セクション -->
  <div class="bg-blue-600 text-white py-16">
    <div class="max-w-4xl mx-auto text-center px-4 sm:px-6 lg:px-8">
      <h2 class="text-3xl font-bold mb-4">
        今すぐMarkMailを始めよう
      </h2>
      <p class="text-xl mb-8 text-blue-100">
        無料プランで全機能をお試しいただけます
      </p>
      <a href="/auth/register" 
         class="bg-white text-blue-600 px-8 py-3 rounded-lg text-lg font-medium hover:bg-gray-100 transition-colors">
        無料アカウント作成
      </a>
    </div>
  </div>
</div> 