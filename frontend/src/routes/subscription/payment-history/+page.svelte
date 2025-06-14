<script lang="ts">
  import { onMount } from "svelte";
  import { subscriptionService } from "$lib/services/subscriptionService";
  import type { PaymentHistory } from "$lib/types/subscription";

  let loading = true;
  let error = "";
  let payments: PaymentHistory[] = [];
  let limit = 50;
  let offset = 0;
  let hasMore = false;

  onMount(async () => {
    await loadPaymentHistory();
  });

  async function loadPaymentHistory() {
    try {
      loading = true;
      error = "";

      const response = await subscriptionService.getPaymentHistory(
        limit,
        offset,
      );
      payments = response.payments;
      hasMore = response.payments.length === limit;
    } catch (err: any) {
      error = err.message || "支払い履歴の取得に失敗しました";
    } finally {
      loading = false;
    }
  }

  async function loadMore() {
    offset += limit;
    await loadPaymentHistory();
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString("ja-JP", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function formatPrice(amount: number): string {
    return new Intl.NumberFormat("ja-JP", {
      style: "currency",
      currency: "JPY",
    }).format(amount);
  }

  function getStatusBadgeClass(status: string): string {
    switch (status) {
      case "succeeded":
        return "bg-green-100 text-green-800";
      case "failed":
        return "bg-red-100 text-red-800";
      case "pending":
        return "bg-yellow-100 text-yellow-800";
      case "refunded":
        return "bg-gray-100 text-gray-800";
      default:
        return "bg-gray-100 text-gray-800";
    }
  }

  function getStatusLabel(status: string): string {
    switch (status) {
      case "succeeded":
        return "成功";
      case "failed":
        return "失敗";
      case "pending":
        return "処理中";
      case "refunded":
        return "返金済み";
      default:
        return status;
    }
  }
</script>

<div class="container mx-auto px-4 py-8">
  <div class="flex items-center justify-between mb-8">
    <h1 class="text-3xl font-bold">支払い履歴</h1>
    <a href="/subscription" class="text-blue-600 hover:text-blue-800">
      ← サブスクリプション管理に戻る
    </a>
  </div>

  {#if loading && payments.length === 0}
    <div class="text-center">読み込み中...</div>
  {:else if error}
    <div
      class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4"
    >
      {error}
    </div>
  {:else if payments.length === 0}
    <div class="text-center text-gray-500">支払い履歴がありません</div>
  {:else}
    <div class="bg-white rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              日時
            </th>
            <th
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              説明
            </th>
            <th
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              金額
            </th>
            <th
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              ステータス
            </th>
            <th
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              決済ID
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          {#each payments as payment}
            <tr>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {formatDate(payment.paid_at || payment.created_at)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {payment.description || "サブスクリプション料金"}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {formatPrice(payment.amount)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <span
                  class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {getStatusBadgeClass(
                    payment.status,
                  )}"
                >
                  {getStatusLabel(payment.status)}
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {#if payment.stripe_payment_intent_id}
                  <code class="text-xs">{payment.stripe_payment_intent_id}</code
                  >
                {:else}
                  -
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    {#if hasMore}
      <div class="mt-4 text-center">
        <button
          class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 transition-colors"
          on:click={loadMore}
          disabled={loading}
        >
          {loading ? "読み込み中..." : "さらに表示"}
        </button>
      </div>
    {/if}
  {/if}
</div>
