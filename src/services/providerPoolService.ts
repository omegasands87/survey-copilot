import { invokeCommand } from './ipcClient';
import type {
  PoolExecutionResult,
  PoolSettings,
  ProviderHealthInfo,
  UpdatePoolSettingsInput,
} from '../types/providerPool';

export class ProviderPoolService {
  /**
   * Mengambil pengaturan global provider pool dari backend
   */
  static async getSettings(): Promise<PoolSettings> {
    return invokeCommand<PoolSettings>('provider_pool_get_settings');
  }

  /**
   * Memperbarui pengaturan global provider pool
   */
  static async updateSettings(input: UpdatePoolSettingsInput): Promise<PoolSettings> {
    return invokeCommand<PoolSettings>('provider_pool_update_settings', { input });
  }

  /**
   * Mengambil status kesehatan dan konkurensi aktif seluruh provider
   */
  static async listHealth(): Promise<ProviderHealthInfo[]> {
    return invokeCommand<ProviderHealthInfo[]>('provider_pool_list_health');
  }

  /**
   * Menjalankan permintaan AI terstruktur melalui provider pool
   */
  static async executeRequest<T = unknown>(request: {
    requestId: string;
    operation: string;
    modelId: string;
    systemInstruction?: string | null;
    userPayload: string;
    responseSchema?: unknown;
    temperature?: number | null;
    maxTokens?: number | null;
    timeoutMs?: number | null;
  }): Promise<PoolExecutionResult<T>> {
    return invokeCommand<PoolExecutionResult<T>>('provider_pool_execute_request', { request });
  }
}
