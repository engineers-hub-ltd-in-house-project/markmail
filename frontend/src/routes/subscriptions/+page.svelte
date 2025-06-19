<script lang="ts">
  import { onMount } from "svelte";
  import { subscriptionService } from "$lib/services/subscriptionService";
  import type {
    SubscriptionPlan,
    SubscriptionDetailsResponse,
    UsageMetric,
  } from "$lib/types/subscription";

  let loading = true;
  let error = "";
  let plans: SubscriptionPlan[] = [];
  let currentSubscription: SubscriptionDetailsResponse | null = null;
  let upgrading = false;

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    try {
      loading = true;
      error = "";

      // プラン一覧と現在のサブスクリプション情報を取得
      const [plansResponse, subscriptionResponse] = await Promise.all([
        subscriptionService.getPlans(),
        subscriptionService.getCurrentSubscription(),
      ]);

      plans = plansResponse.plans;
      currentSubscription = subscriptionResponse;
    } catch (err: any) {
      error = err.message || "データの読み込みに失敗しました";
    } finally {
      loading = false;
    }
  }

  async function handleUpgrade(planId: string) {
    if (
      !confirm(
        "プランをアップグレードしますか？\nStripeの決済画面に移動します。",
      )
    ) {
      return;
    }

    try {
      upgrading = true;
      error = "";
      // Stripe Checkoutへリダイレクト
      await subscriptionService.createCheckoutSession(
        planId,
        `${window.location.origin}/subscriptions/success`,
        `${window.location.origin}/subscriptions`,
      );
    } catch (err: any) {
      error = err.message || "アップグレードに失敗しました";
      upgrading = false;
    }
  }

  function formatPrice(price: number): string {
    return new Intl.NumberFormat("ja-JP", {
      style: "currency",
      currency: "JPY",
    }).format(price);
  }

  function getUsagePercentageClass(metric: UsageMetric): string {
    if (metric.percentage >= 90) return "text-red-600";
    if (metric.percentage >= 70) return "text-yellow-600";
    return "text-green-600";
  }

  function formatLimit(limit: number): string {
    if (limit < 0) return "無制限";
    return limit.toLocaleString();
  }
</script>

<div class="container mx-auto px-4 py-8">
  <h1 class="text-3xl font-bold mb-8">サブスクリプション管理</h1>

  {#if loading}
    <div class="text-center">読み込み中...</div>
  {:else if error}
    <div
      class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4"
    >
      {error}
    </div>
  {:else}
    <!-- 現在のプランと使用状況 -->
    {#if currentSubscription}
      <div class="bg-white rounded-lg shadow-md p-6 mb-8">
        <h2 class="text-2xl font-semibold mb-4">現在のプラン</h2>

        <div class="mb-6">
          <div class="flex items-center justify-between mb-2">
            <h3 class="text-xl font-medium">
              {currentSubscription.plan.display_name}
            </h3>
            <span class="text-2xl font-bold text-blue-600">
              {formatPrice(currentSubscription.plan.price)}/月
            </span>
          </div>
          {#if currentSubscription.plan.description}
            <p class="text-gray-600">{currentSubscription.plan.description}</p>
          {/if}
        </div>

        <!-- 使用状況 -->
        <div class="border-t pt-4">
          <h4 class="text-lg font-medium mb-4">使用状況</h4>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="space-y-3">
              <div>
                <div class="flex justify-between mb-1">
                  <span>コンタクト数</span>
                  <span
                    class={getUsagePercentageClass(
                      currentSubscription.usage.contacts,
                    )}
                  >
                    {currentSubscription.usage.contacts.current} / {formatLimit(
                      currentSubscription.usage.contacts.limit,
                    )}
                  </span>
                </div>
                <div class="w-full bg-gray-200 rounded-full h-2">
                  <div
                    class="bg-blue-600 h-2 rounded-full"
                    style="width: {Math.min(
                      currentSubscription.usage.contacts.percentage,
                      100,
                    )}%"
                  ></div>
                </div>
              </div>

              <div>
                <div class="flex justify-between mb-1">
                  <span>月間メール送信数</span>
                  <span
                    class={getUsagePercentageClass(
                      currentSubscription.usage.emails_sent,
                    )}
                  >
                    {currentSubscription.usage.emails_sent.current} / {formatLimit(
                      currentSubscription.usage.emails_sent.limit,
                    )}
                  </span>
                </div>
                <div class="w-full bg-gray-200 rounded-full h-2">
                  <div
                    class="bg-blue-600 h-2 rounded-full"
                    style="width: {Math.min(
                      currentSubscription.usage.emails_sent.percentage,
                      100,
                    )}%"
                  ></div>
                </div>
              </div>

              <div>
                <div class="flex justify-between mb-1">
                  <span>キャンペーン数</span>
                  <span
                    class={getUsagePercentageClass(
                      currentSubscription.usage.campaigns,
                    )}
                  >
                    {currentSubscription.usage.campaigns.current} / {formatLimit(
                      currentSubscription.usage.campaigns.limit,
                    )}
                  </span>
                </div>
                <div class="w-full bg-gray-200 rounded-full h-2">
                  <div
                    class="bg-blue-600 h-2 rounded-full"
                    style="width: {Math.min(
                      currentSubscription.usage.campaigns.percentage,
                      100,
                    )}%"
                  ></div>
                </div>
              </div>
            </div>

            <div class="space-y-3">
              <div>
                <div class="flex justify-between mb-1">
                  <span>テンプレート数</span>
                  <span
                    class={getUsagePercentageClass(
                      currentSubscription.usage.templates,
                    )}
                  >
                    {currentSubscription.usage.templates.current} / {formatLimit(
                      currentSubscription.usage.templates.limit,
                    )}
                  </span>
                </div>
                <div class="w-full bg-gray-200 rounded-full h-2">
                  <div
                    class="bg-blue-600 h-2 rounded-full"
                    style="width: {Math.min(
                      currentSubscription.usage.templates.percentage,
                      100,
                    )}%"
                  ></div>
                </div>
              </div>

              <div>
                <div class="flex justify-between mb-1">
                  <span>シーケンス数</span>
                  <span
                    class={getUsagePercentageClass(
                      currentSubscription.usage.sequences,
                    )}
                  >
                    {currentSubscription.usage.sequences.current} / {formatLimit(
                      currentSubscription.usage.sequences.limit,
                    )}
                  </span>
                </div>
                <div class="w-full bg-gray-200 rounded-full h-2">
                  <div
                    class="bg-blue-600 h-2 rounded-full"
                    style="width: {Math.min(
                      currentSubscription.usage.sequences.percentage,
                      100,
                    )}%"
                  ></div>
                </div>
              </div>

              <div>
                <div class="flex justify-between mb-1">
                  <span>フォーム数</span>
                  <span
                    class={getUsagePercentageClass(
                      currentSubscription.usage.forms,
                    )}
                  >
                    {currentSubscription.usage.forms.current} / {formatLimit(
                      currentSubscription.usage.forms.limit,
                    )}
                  </span>
                </div>
                <div class="w-full bg-gray-200 rounded-full h-2">
                  <div
                    class="bg-blue-600 h-2 rounded-full"
                    style="width: {Math.min(
                      currentSubscription.usage.forms.percentage,
                      100,
                    )}%"
                  ></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    {/if}

    <!-- プラン一覧 -->
    <h2 class="text-2xl font-semibold mb-4">利用可能なプラン</h2>
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
      {#each plans as plan}
        <div
          class="bg-white rounded-lg shadow-md p-6 {currentSubscription?.plan
            .id === plan.id
            ? 'border-2 border-blue-500'
            : ''}"
        >
          <div class="mb-4">
            <h3 class="text-xl font-semibold">{plan.display_name}</h3>
            <p class="text-gray-600">{plan.description || ""}</p>
          </div>

          <div class="mb-6">
            <span class="text-3xl font-bold">{formatPrice(plan.price)}</span>
            <span class="text-gray-600">/月</span>
          </div>

          <div class="space-y-2 mb-6 text-sm">
            <div class="flex items-center">
              <svg
                class="w-4 h-4 mr-2 text-green-500"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fill-rule="evenodd"
                  d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                  clip-rule="evenodd"
                ></path>
              </svg>
              コンタクト数: {formatLimit(plan.contact_limit)}
            </div>
            <div class="flex items-center">
              <svg
                class="w-4 h-4 mr-2 text-green-500"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fill-rule="evenodd"
                  d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                  clip-rule="evenodd"
                ></path>
              </svg>
              月間メール送信数: {formatLimit(plan.monthly_email_limit)}
            </div>
            <div class="flex items-center">
              <svg
                class="w-4 h-4 mr-2 text-green-500"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fill-rule="evenodd"
                  d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                  clip-rule="evenodd"
                ></path>
              </svg>
              キャンペーン数: {formatLimit(plan.campaign_limit)}
            </div>

            {#if plan.ai_features}
              <div class="flex items-center">
                <svg
                  class="w-4 h-4 mr-2 text-green-500"
                  fill="currentColor"
                  viewBox="0 0 20 20"
                >
                  <path
                    fill-rule="evenodd"
                    d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                    clip-rule="evenodd"
                  ></path>
                </svg>
                AI機能
              </div>
            {/if}

            {#if plan.advanced_analytics}
              <div class="flex items-center">
                <svg
                  class="w-4 h-4 mr-2 text-green-500"
                  fill="currentColor"
                  viewBox="0 0 20 20"
                >
                  <path
                    fill-rule="evenodd"
                    d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                    clip-rule="evenodd"
                  ></path>
                </svg>
                高度な分析
              </div>
            {/if}

            {#if plan.api_access}
              <div class="flex items-center">
                <svg
                  class="w-4 h-4 mr-2 text-green-500"
                  fill="currentColor"
                  viewBox="0 0 20 20"
                >
                  <path
                    fill-rule="evenodd"
                    d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                    clip-rule="evenodd"
                  ></path>
                </svg>
                APIアクセス
              </div>
            {/if}
          </div>

          {#if currentSubscription}
            {#if currentSubscription.plan.id === plan.id}
              <button
                class="w-full bg-gray-300 text-gray-700 px-4 py-2 rounded cursor-not-allowed"
                disabled
              >
                現在のプラン
              </button>
            {:else if currentSubscription.plan.price < plan.price}
              <button
                class="w-full bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 transition-colors"
                on:click={() => handleUpgrade(plan.id)}
                disabled={upgrading}
              >
                {upgrading ? "処理中..." : "アップグレード"}
              </button>
            {:else}
              <button
                class="w-full bg-gray-200 text-gray-600 px-4 py-2 rounded cursor-not-allowed"
                disabled
              >
                ダウングレード不可
              </button>
            {/if}
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>
