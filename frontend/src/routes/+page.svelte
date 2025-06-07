<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { authStore } from "$lib/stores/authStore";

  let isAuthenticated = false;

  authStore.subscribe((state) => {
    isAuthenticated = state.isAuthenticated;
  });

  onMount(() => {
    // 認証状態によって適切なページにリダイレクト
    if (isAuthenticated) {
      goto("/templates");
    } else {
      goto("/auth/login");
    }
  });
</script>

<svelte:head>
  <title
    >MarkMail - エンジニア向けマークダウンベースメールマーケティングツール</title
  >
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50">
  <div class="text-center">
    <div class="flex justify-center items-center mb-4">
      <div
        class="w-12 h-12 bg-blue-600 rounded-lg flex items-center justify-center mr-3"
      >
        <span class="text-white font-bold text-xl">M</span>
      </div>
      <h1 class="text-4xl font-bold text-gray-900">MarkMail</h1>
    </div>

    <div
      class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"
    ></div>
    <p class="mt-4 text-gray-600">リダイレクト中...</p>
  </div>
</div>
