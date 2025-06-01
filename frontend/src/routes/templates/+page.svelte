<script>
	import { onMount } from 'svelte';

	let templates = [];
	let isLoading = true;

	onMount(async () => {
		// TODO: APIからテンプレート一覧を取得
		await new Promise(resolve => setTimeout(resolve, 1000)); // 模擬的な遅延
		templates = [
			{
				id: '1',
				name: 'ニュースレター',
				description: '週次ニュースレターのテンプレート',
				created_at: '2024-01-15T10:00:00Z',
				updated_at: '2024-01-20T15:30:00Z'
			},
			{
				id: '2',
				name: 'プロモーション',
				description: 'セール・キャンペーン用のテンプレート',
				created_at: '2024-01-10T09:00:00Z',
				updated_at: '2024-01-18T11:45:00Z'
			}
		];
		isLoading = false;
	});

	function formatDate(dateString) {
		return new Date(dateString).toLocaleDateString('ja-JP');
	}
</script>

<svelte:head>
	<title>テンプレート - MarkMail</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 py-8">
	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
		<div class="sm:flex sm:items-center">
			<div class="sm:flex-auto">
				<h1 class="text-3xl font-bold text-gray-900">テンプレート</h1>
				<p class="mt-2 text-sm text-gray-700">
					メールテンプレートを作成・管理できます。マークダウンで美しいメールを作成しましょう。
				</p>
			</div>
			<div class="mt-4 sm:mt-0 sm:ml-16 sm:flex-none">
				<button
					type="button"
					class="inline-flex items-center justify-center rounded-md border border-transparent bg-blue-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 sm:w-auto"
				>
					<svg class="-ml-1 mr-2 h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
					</svg>
					新しいテンプレート
				</button>
			</div>
		</div>

		{#if isLoading}
			<div class="mt-8 flex justify-center">
				<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
			</div>
		{:else}
			<div class="mt-8 flex flex-col">
				<div class="-my-2 -mx-4 overflow-x-auto sm:-mx-6 lg:-mx-8">
					<div class="inline-block min-w-full py-2 align-middle md:px-6 lg:px-8">
						<div class="overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg">
							<table class="min-w-full divide-y divide-gray-300">
								<thead class="bg-gray-50">
									<tr>
										<th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
											テンプレート名
										</th>
										<th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
											説明
										</th>
										<th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
											作成日
										</th>
										<th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
											更新日
										</th>
										<th scope="col" class="relative px-6 py-3">
											<span class="sr-only">アクション</span>
										</th>
									</tr>
								</thead>
								<tbody class="bg-white divide-y divide-gray-200">
									{#each templates as template}
										<tr class="hover:bg-gray-50">
											<td class="px-6 py-4 whitespace-nowrap">
												<div class="flex items-center">
													<div class="flex-shrink-0 h-10 w-10">
														<div class="h-10 w-10 rounded-lg bg-blue-100 flex items-center justify-center">
															<svg class="h-6 w-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
																<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
															</svg>
														</div>
													</div>
													<div class="ml-4">
														<div class="text-sm font-medium text-gray-900">{template.name}</div>
													</div>
												</div>
											</td>
											<td class="px-6 py-4 whitespace-nowrap">
												<div class="text-sm text-gray-900">{template.description}</div>
											</td>
											<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
												{formatDate(template.created_at)}
											</td>
											<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
												{formatDate(template.updated_at)}
											</td>
											<td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
												<div class="flex space-x-2">
													<button class="text-blue-600 hover:text-blue-900">編集</button>
													<button class="text-green-600 hover:text-green-900">プレビュー</button>
													<button class="text-red-600 hover:text-red-900">削除</button>
												</div>
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					</div>
				</div>
			</div>

			{#if templates.length === 0}
				<div class="text-center py-12">
					<svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
					</svg>
					<h3 class="mt-2 text-sm font-medium text-gray-900">テンプレートがありません</h3>
					<p class="mt-1 text-sm text-gray-500">最初のメールテンプレートを作成してみましょう。</p>
					<div class="mt-6">
						<button
							type="button"
							class="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
						>
							<svg class="-ml-1 mr-2 h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"></path>
							</svg>
							新しいテンプレート
						</button>
					</div>
				</div>
			{/if}
		{/if}
	</div>
</div> 