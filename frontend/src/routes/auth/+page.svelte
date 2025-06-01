<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	onMount(() => {
		// デフォルトでログインページにリダイレクト
		goto('/auth/login');
	});
</script>

<svelte:head>
	<title>認証 - MarkMail</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50">
	<div class="text-center">
		<h1 class="text-2xl font-bold text-gray-900 mb-4">認証ページ</h1>
		<p class="text-gray-600">ログインページにリダイレクトしています...</p>
	</div>
</div> 