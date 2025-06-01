<script>
	import { onMount } from 'svelte';

	let campaigns = [];
	let isLoading = true;

	onMount(async () => {
		// TODO: APIからキャンペーン一覧を取得
		await new Promise(resolve => setTimeout(resolve, 1000));
		campaigns = [
			{
				id: '1',
				name: '新年キャンペーン',
				status: 'active',
				sent_count: 1250,
				open_rate: 24.5,
				created_at: '2024-01-01T00:00:00Z'
			},
			{
				id: '2',
				name: 'ウィークリーニュース',
				status: 'draft',
				sent_count: 0,
				open_rate: 0,
				created_at: '2024-01-15T10:00:00Z'
			}
		];
		isLoading = false;
	});

	function getStatusBadge(status) {
		switch (status) {
			case 'active':
				return 'bg-green-100 text-green-800';
			case 'draft':
				return 'bg-gray-100 text-gray-800';
			case 'completed':
				return 'bg-blue-100 text-blue-800';
			default:
				return 'bg-gray-100 text-gray-800';
		}
	}

	function getStatusText(status) {
		switch (status) {
			case 'active':
				return '配信中';
			case 'draft':
				return '下書き';
			case 'completed':
				return '完了';
			default:
				return '不明';
		}
	}

	function formatDate(dateString) {
		return new Date(dateString).toLocaleDateString('ja-JP');
	}
</script>

<svelte:head>
	<title>キャンペーン - MarkMail</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 py-8">
	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
		<div class="sm:flex sm:items-center">
			<div class="sm:flex-auto">
				<h1 class="text-3xl font-bold text-gray-900">キャンペーン</h1>
				<p class="mt-2 text-sm text-gray-700">
					メールキャンペーンを作成・管理し、配信状況を確認できます。
				</p>
			</div>
			<div class="mt-4 sm:mt-0 sm:ml-16 sm:flex-none">
				<button
					type="button"
					class="inline-flex items-center justify-center rounded-md border border-transparent bg-green-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2 sm:w-auto"
				>
					<svg class="-ml-1 mr-2 h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
					</svg>
					新しいキャンペーン
				</button>
			</div>
		</div>

		{#if isLoading}
			<div class="mt-8 flex justify-center">
				<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-green-600"></div>
			</div>
		{:else}
			<div class="mt-8 grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
				{#each campaigns as campaign}
					<div class="bg-white overflow-hidden shadow rounded-lg">
						<div class="p-5">
							<div class="flex items-center">
								<div class="flex-shrink-0">
									<div class="w-8 h-8 bg-green-500 rounded-md flex items-center justify-center">
										<svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-5 5v-5zM4 19h6v-2H4v2zM16 3H4v14h12V3z"></path>
										</svg>
									</div>
								</div>
								<div class="ml-5 w-0 flex-1">
									<dl>
										<dt class="text-sm font-medium text-gray-500 truncate">
											{campaign.name}
										</dt>
										<dd class="flex items-center text-lg font-medium text-gray-900">
											<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {getStatusBadge(campaign.status)}">
												{getStatusText(campaign.status)}
											</span>
										</dd>
									</dl>
								</div>
							</div>
						</div>
						<div class="bg-gray-50 px-5 py-3">
							<div class="text-sm">
								<div class="grid grid-cols-2 gap-4">
									<div>
										<dt class="font-medium text-gray-500">送信数</dt>
										<dd class="mt-1 text-gray-900">{campaign.sent_count.toLocaleString()}</dd>
									</div>
									<div>
										<dt class="font-medium text-gray-500">開封率</dt>
										<dd class="mt-1 text-gray-900">{campaign.open_rate}%</dd>
									</div>
								</div>
								<div class="mt-3">
									<dt class="font-medium text-gray-500">作成日</dt>
									<dd class="mt-1 text-gray-900">{formatDate(campaign.created_at)}</dd>
								</div>
							</div>
						</div>
						<div class="bg-gray-50 px-5 py-3 border-t border-gray-200">
							<div class="flex space-x-3">
								<button class="text-sm text-blue-600 hover:text-blue-900 font-medium">編集</button>
								<button class="text-sm text-green-600 hover:text-green-900 font-medium">詳細</button>
								<button class="text-sm text-red-600 hover:text-red-900 font-medium">削除</button>
							</div>
						</div>
					</div>
				{/each}
			</div>

			{#if campaigns.length === 0}
				<div class="text-center py-12">
					<svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-5 5v-5zM4 19h6v-2H4v2zM16 3H4v14h12V3z"></path>
					</svg>
					<h3 class="mt-2 text-sm font-medium text-gray-900">キャンペーンがありません</h3>
					<p class="mt-1 text-sm text-gray-500">最初のメールキャンペーンを作成してみましょう。</p>
					<div class="mt-6">
						<button
							type="button"
							class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
						>
							<svg class="-ml-1 mr-2 h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
							</svg>
							新しいキャンペーン
						</button>
					</div>
				</div>
			{/if}
		{/if}
	</div>
</div> 