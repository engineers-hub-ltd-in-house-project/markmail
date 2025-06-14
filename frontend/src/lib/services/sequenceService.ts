import { get } from "svelte/store";
import { authStore } from "$lib/stores/authStore";
import type {
  Sequence,
  SequenceStep,
  SequenceWithSteps,
  CreateSequenceRequest,
  UpdateSequenceRequest,
  CreateSequenceStepRequest,
  UpdateSequenceStepRequest,
  SequenceEnrollment,
} from "$lib/types/sequence";

const API_BASE_URL = "/api";

class SequenceService {
  private async request<T>(
    path: string,
    options: RequestInit = {},
  ): Promise<T> {
    const auth = get(authStore);
    if (!auth) {
      throw new Error("Not authenticated");
    }

    const response = await fetch(`${API_BASE_URL}${path}`, {
      ...options,
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${auth.token}`,
        ...options.headers,
      },
    });

    if (response.status === 401) {
      authStore.logout();
      throw new Error("Authentication failed");
    }

    if (!response.ok) {
      const error = await response
        .json()
        .catch(() => ({ message: "Request failed" }));
      throw new Error(error.message || `HTTP ${response.status}`);
    }

    // 204 No Content の場合は空のレスポンスを返す
    if (response.status === 204) {
      return undefined as T;
    }

    return response.json();
  }

  // シーケンスのCRUD操作

  async listSequences(): Promise<Sequence[]> {
    return this.request<Sequence[]>("/sequences");
  }

  async getSequence(id: string): Promise<SequenceWithSteps> {
    return this.request<SequenceWithSteps>(`/sequences/${id}/full`);
  }

  async createSequence(data: CreateSequenceRequest): Promise<Sequence> {
    return this.request<Sequence>("/sequences", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  async updateSequence(
    id: string,
    data: UpdateSequenceRequest,
  ): Promise<Sequence> {
    return this.request<Sequence>(`/sequences/${id}`, {
      method: "PUT",
      body: JSON.stringify(data),
    });
  }

  async deleteSequence(id: string): Promise<void> {
    await this.request<void>(`/sequences/${id}`, {
      method: "DELETE",
    });
  }

  // ステップ管理

  async getSequenceSteps(sequenceId: string): Promise<SequenceStep[]> {
    return this.request<SequenceStep[]>(`/sequences/${sequenceId}/steps`);
  }

  async createSequenceStep(
    sequenceId: string,
    data: CreateSequenceStepRequest,
  ): Promise<SequenceStep> {
    return this.request<SequenceStep>(`/sequences/${sequenceId}/steps`, {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  async updateSequenceStep(
    sequenceId: string,
    stepId: string,
    data: UpdateSequenceStepRequest,
  ): Promise<SequenceStep> {
    return this.request<SequenceStep>(
      `/sequences/${sequenceId}/steps/${stepId}`,
      {
        method: "PUT",
        body: JSON.stringify(data),
      },
    );
  }

  async deleteSequenceStep(sequenceId: string, stepId: string): Promise<void> {
    await this.request<void>(`/sequences/${sequenceId}/steps/${stepId}`, {
      method: "DELETE",
    });
  }

  // ステップの並び替え
  async reorderSteps(
    sequenceId: string,
    stepOrders: { step_id: string; order: number }[],
  ): Promise<void> {
    await this.request<void>(`/sequences/${sequenceId}/steps/reorder`, {
      method: "POST",
      body: JSON.stringify({ steps: stepOrders }),
    });
  }

  // エンロールメント管理

  async getSequenceEnrollments(
    sequenceId: string,
  ): Promise<SequenceEnrollment[]> {
    return this.request<SequenceEnrollment[]>(
      `/sequences/${sequenceId}/enrollments`,
    );
  }

  async enrollSubscriber(
    sequenceId: string,
    subscriberId: string,
  ): Promise<SequenceEnrollment> {
    return this.request<SequenceEnrollment>(`/sequences/${sequenceId}/enroll`, {
      method: "POST",
      body: JSON.stringify({ subscriber_id: subscriberId }),
    });
  }

  async pauseEnrollment(enrollmentId: string): Promise<void> {
    await this.request<void>(`/enrollments/${enrollmentId}/pause`, {
      method: "POST",
    });
  }

  async resumeEnrollment(enrollmentId: string): Promise<void> {
    await this.request<void>(`/enrollments/${enrollmentId}/resume`, {
      method: "POST",
    });
  }

  async cancelEnrollment(enrollmentId: string): Promise<void> {
    await this.request<void>(`/enrollments/${enrollmentId}/cancel`, {
      method: "POST",
    });
  }

  // シーケンスのアクティベーション

  async activateSequence(id: string): Promise<void> {
    await this.request<void>(`/sequences/${id}/activate`, {
      method: "POST",
    });
  }

  async pauseSequence(id: string): Promise<void> {
    await this.request<void>(`/sequences/${id}/pause`, {
      method: "POST",
    });
  }

  async archiveSequence(id: string): Promise<void> {
    await this.request<void>(`/sequences/${id}/archive`, {
      method: "POST",
    });
  }

  // ユーティリティメソッド

  formatTriggerType(trigger: string): string {
    const triggers: Record<string, string> = {
      manual: "手動",
      subscriber_created: "購読者登録時",
      form_submission: "フォーム送信時",
      tag_added: "タグ追加時",
    };
    return triggers[trigger] || trigger;
  }

  formatStepType(stepType: string): string {
    const types: Record<string, string> = {
      email: "メール送信",
      wait: "待機",
      condition: "条件分岐",
      tag: "タグ追加",
    };
    return types[stepType] || stepType;
  }

  formatDelay(value: number, unit: string): string {
    switch (unit) {
      case "minutes":
        return `${value}分`;
      case "hours":
        return `${value}時間`;
      case "days":
        return `${value}日`;
      default:
        return `${value} ${unit}`;
    }
  }
}

export const sequenceService = new SequenceService();
