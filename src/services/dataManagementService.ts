import { invokeCommand } from "./ipcClient";
import type {
  AllRecordCounts,
  ExportSummary,
  RetentionPolicyInfo,
  WipeReport,
} from "../types/dataManagement";

export const dataManagementService = {
  /**
   * Mengambil ringkasan jumlah data dan indikator privasi sebelum ekspor.
   */
  async getExportSummary(personaId?: string): Promise<ExportSummary> {
    return invokeCommand<ExportSummary>("export_get_summary", {
      personaId: personaId || null,
    });
  },

  /**
   * Menghasilkan teks dokumen JSON ekspor lengkap.
   */
  async generateExportJson(personaId?: string): Promise<string> {
    return invokeCommand<string>("export_generate_json", {
      personaId: personaId || null,
    });
  },

  /**
   * Menghitung seluruh record yang ada di dalam database sebelum konfirmasi penghapusan menyeluruh.
   */
  async getWipeSummary(): Promise<AllRecordCounts> {
    return invokeCommand<AllRecordCounts>("data_get_wipe_summary");
  },

  /**
   * Menghapus seluruh data aplikasi secara permanen dengan teks konfirmasi 'HAPUS SEMUA DATA'.
   */
  async wipeAllData(confirmationText: string): Promise<WipeReport> {
    return invokeCommand<WipeReport>("data_wipe_all", {
      confirmationText,
    });
  },

  /**
   * Mengambil informasi kebijakan retensi dan siklus privasi data.
   */
  async getRetentionInfo(): Promise<RetentionPolicyInfo> {
    return invokeCommand<RetentionPolicyInfo>("data_get_retention_info");
  },

  /**
   * Memicu pengunduhan berkas JSON secara aman di browser/webview.
   */
  triggerFileDownload(jsonContent: string, filename: string): void {
    const blob = new Blob([jsonContent], { type: "application/json;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  },
};
