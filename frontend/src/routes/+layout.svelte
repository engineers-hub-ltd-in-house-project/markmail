<script lang="ts">
  import '../app.css';
</script>

<div class="min-h-screen bg-gray-50">
  <nav class="bg-white shadow-sm border-b border-gray-200">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="flex justify-between h-16">
        <div class="flex items-center">
          <a href="/" class="flex items-center space-x-2">
            <div class="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center">
              <span class="text-white font-bold text-sm">M</span>
            </div>
            <span class="text-xl font-bold text-gray-900">MarkMail</span>
          </a>
        </div>
        
        <div class="flex items-center space-x-4">
          <a href="/templates" class="text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium">
            テンプレート
          </a>
          <a href="/campaigns" class="text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium">
            キャンペーン
          </a>
          <a href="/subscribers" class="text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium">
            購読者
          </a>
          <a href="/dashboard" class="bg-blue-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-blue-700">
            ダッシュボード
          </a>
        </div>
      </div>
    </div>
  </nav>

  <main>
    <slot />
  </main>
</div> 