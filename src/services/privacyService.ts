import { invokeCommand } from './ipcClient';
import type {
  CleanupTempFilesInput,
  DiagnosticsLogEntry,
  InspectOutboundDataInput,
  OutboundDataDisclosure,
  PrivacySettings,
  RecordLogInput,
  RetentionCleanupResult,
  RetentionStatus,
  SystemDiagnosticsInfo,
  UpdatePrivacySettingsInput,
} from '../types/privacy';

export class PrivacyService {
  /**
   * Mengambil pengaturan privasi dan retensi aplikasi terkini
   */
  static async getSettings(): Promise<PrivacySettings> {
    return invokeCommand<PrivacySettings>('privacy_get_settings');
  }

  /**
   * Memperbarui pengaturan privasi dan retensi aplikasi
   */
  static async updateSettings(input: UpdatePrivacySettingsInput): Promise<PrivacySettings> {
    return invokeCommand<PrivacySettings>('privacy_update_settings', { input });
  }

  /**
   * Menginspeksi transparansi data keluar (what data leaves the device) sebelum dikirim ke AI
   */
  static async inspectOutboundData(input: InspectOutboundDataInput): Promise<OutboundDataDisclosure> {
    return invokeCommand<OutboundDataDisclosure>('privacy_inspect_outbound_data', { input });
  }

  /**
   * Mengambil status penggunaan disk untuk tangkapan layar sementara dan kebijakan retensi aktif
   */
  static async getRetentionStatus(): Promise<RetentionStatus> {
    return invokeCommand<RetentionStatus>('privacy_get_retention_status');
  }

  /**
   * Membersihkan berkas tangkapan layar sementara di disk secara manual
   */
  static async cleanupTempFiles(input: CleanupTempFilesInput = {}): Promise<RetentionCleanupResult> {
    return invokeCommand<RetentionCleanupResult>('privacy_cleanup_temp_files', { input });
  }

  /**
   * Mengambil riwayat log diagnostik yang selalu teredaksi
   */
  static async getDiagnosticLogs(
    limit?: number,
    level?: string,
    category?: string
  ): Promise<DiagnosticsLogEntry[]> {
    return invokeCommand<DiagnosticsLogEntry[]>('privacy_get_diagnostic_logs', {
      limit,
      level,
      category,
    });
  }

  /**
   * Mengosongkan seluruh riwayat log diagnostik lokal
   */
  static async clearDiagnosticLogs(): Promise<number> {
    return invokeCommand<number>('privacy_clear_diagnostic_logs');
  }

  /**
   * Mengambil informasi diagnostik sistem (SQLite, WAL, versi, jumlah entitas)
   */
  static async getSystemDiagnostics(): Promise<SystemDiagnosticsInfo> {
    return invokeCommand<SystemDiagnosticsInfo>('privacy_get_system_diagnostics');
  }

  /**
   * Mencatat entri log diagnostik baru (otomatis disensor di backend)
   */
  static async recordDiagnosticLog(input: RecordLogInput): Promise<DiagnosticsLogEntry> {
    return invokeCommand<DiagnosticsLogEntry>('privacy_record_diagnostic_log', { input });
  }
}
