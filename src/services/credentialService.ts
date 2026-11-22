import { invokeCommand } from './ipcClient';
import type {
  CredentialSummary,
  DeleteCredentialResult,
  SaveCredentialInput,
  TestCredentialResult,
  UpdateCredentialInput,
} from '../types/credential';

/**
 * Service klien untuk pengelolaan aman kredensial API provider di Windows Credential Manager.
 *
 * JAMINAN KEAMANAN:
 * - Kunci API asli hanya dikirimkan ke backend melalui perintah `credential_save` / `credential_update`.
 * - Backend tidak pernah mengembalikan secret ke frontend.
 * - Frontend tidak pernah menyimpan secret di localStorage, sessionStorage, atau state jangka panjang.
 */
export const CredentialService = {
  /**
   * Menyimpan kredensial baru ke Windows Credential Manager.
   */
  async saveCredential(input: SaveCredentialInput): Promise<CredentialSummary> {
    return invokeCommand<CredentialSummary>('credential_save', { input });
  },

  /**
   * Mengambil daftar seluruh kredensial yang tersimpan beserta statusnya.
   * Ringkasan TIDAK MEMUAT kunci API rahasia.
   */
  async listCredentials(): Promise<CredentialSummary[]> {
    return invokeCommand<CredentialSummary[]>('credential_list');
  },

  /**
   * Mengambil status satu kredensial berdasarkan ID.
   */
  async getCredentialStatus(id: string): Promise<CredentialSummary> {
    return invokeCommand<CredentialSummary>('credential_get_status', { id });
  },

  /**
   * Memperbarui label atau secret kredensial.
   */
  async updateCredential(input: UpdateCredentialInput): Promise<CredentialSummary> {
    return invokeCommand<CredentialSummary>('credential_update', { input });
  },

  /**
   * Menghapus kredensial dari Windows Credential Manager dan SQLite.
   * Jika kredensial masih digunakan oleh provider configuration, atur `force: true`
   * setelah konfirmasi pengguna.
   */
  async deleteCredential(id: string, force = false): Promise<DeleteCredentialResult> {
    return invokeCommand<DeleteCredentialResult>('credential_delete', { id, force });
  },

  /**
   * Menguji keberadaan dan akses kredensial di Windows Credential Manager.
   */
  async testCredential(id: string): Promise<TestCredentialResult> {
    return invokeCommand<TestCredentialResult>('credential_test', { id });
  },
};
