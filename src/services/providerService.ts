import { invokeCommand } from './ipcClient';
import type {
  ConnectionTestResult,
  CreateProviderConfigInput,
  ModelOption,
  ProviderConfigSummary,
  UpdateProviderConfigInput,
} from '../types/provider';

/**
 * Service klien untuk pengelolaan konfigurasi AI Provider dan Adapter di Survey Copilot.
 *
 * Jaminan Keamanan:
 * - Kunci API asli tidak pernah disentuh oleh service ini.
 * - Konfigurasi hanya mereferensikan kredensial melalui opaque reference `credentialRef`.
 * - Pengujian koneksi dilakukan di backend Rust menggunakan kredensial dari Windows Credential Manager.
 */
export const ProviderService = {
  /**
   * Membuat konfigurasi provider baru.
   */
  async createProviderConfig(input: CreateProviderConfigInput): Promise<ProviderConfigSummary> {
    return invokeCommand<ProviderConfigSummary>('provider_config_create', { input });
  },

  /**
   * Mengambil detail konfigurasi provider berdasarkan ID.
   */
  async getProviderConfig(id: string): Promise<ProviderConfigSummary> {
    return invokeCommand<ProviderConfigSummary>('provider_config_get', { id });
  },

  /**
   * Mengambil seluruh konfigurasi provider yang tersimpan.
   */
  async listProviderConfigs(): Promise<ProviderConfigSummary[]> {
    return invokeCommand<ProviderConfigSummary[]>('provider_config_list');
  },

  /**
   * Memperbarui konfigurasi provider.
   */
  async updateProviderConfig(input: UpdateProviderConfigInput): Promise<ProviderConfigSummary> {
    return invokeCommand<ProviderConfigSummary>('provider_config_update', { input });
  },

  /**
   * Menghapus konfigurasi provider.
   */
  async deleteProviderConfig(id: string): Promise<boolean> {
    return invokeCommand<boolean>('provider_config_delete', { id });
  },

  /**
   * Menguji koneksi provider secara real-time via backend.
   */
  async testConnection(id: string): Promise<ConnectionTestResult> {
    return invokeCommand<ConnectionTestResult>('provider_config_test_connection', { id });
  },

  /**
   * Mengambil model rekomendasi berdasarkan tipe provider.
   */
  async getRecommendedModels(providerType: string): Promise<ModelOption[]> {
    return invokeCommand<ModelOption[]>('provider_get_recommended_models', { providerType });
  },
};
