import { invokeCommand } from "./ipcClient";
import type {
  AddFactSourceInput,
  ConflictWarning,
  CreateFactInput,
  FactSource,
  ProfileFact,
  ProfileFactDetail,
  Sensitivity,
  UpdateFactInput,
  VerificationStatus,
} from "../types/fact";

export const factService = {
  /**
   * Mengambil daftar fakta yang terikat secara ketat ke personaId tertentu.
   */
  async listFacts(personaId: string, includeArchived = false): Promise<ProfileFact[]> {
    return invokeCommand<ProfileFact[]>("fact_list", { personaId, includeArchived });
  },

  /**
   * Mengambil detail lengkap fakta beserta seluruh riwayat dan sumber bukti.
   */
  async getFactDetail(personaId: string, factId: string): Promise<ProfileFactDetail> {
    return invokeCommand<ProfileFactDetail>("fact_get_detail", { personaId, factId });
  },

  /**
   * Membuat fakta baru beserta sumber bukti secara atomik.
   */
  async createFact(input: CreateFactInput): Promise<ProfileFact> {
    return invokeCommand<ProfileFact>("fact_create", { input });
  },

  /**
   * Memperbarui nilai atau metadata fakta dan menyimpan riwayat nilai lama.
   */
  async updateFact(input: UpdateFactInput): Promise<ProfileFactDetail> {
    return invokeCommand<ProfileFactDetail>("fact_update", { input });
  },

  /**
   * Memperbarui status verifikasi fakta (confirmed, unverified, disputed, outdated).
   */
  async setFactStatus(
    personaId: string,
    factId: string,
    status: VerificationStatus
  ): Promise<ProfileFactDetail> {
    return invokeCommand<ProfileFactDetail>("fact_set_status", { personaId, factId, status });
  },

  /**
   * Memperbarui klasifikasi sensitivitas fakta (ordinary, sensitive, highly_sensitive).
   */
  async setFactSensitivity(
    personaId: string,
    factId: string,
    sensitivity: Sensitivity
  ): Promise<void> {
    return invokeCommand<void>("fact_set_sensitivity", { personaId, factId, sensitivity });
  },

  /**
   * Menambahkan sumber bukti baru ke fakta yang ada.
   */
  async addFactSource(input: AddFactSourceInput): Promise<FactSource> {
    return invokeCommand<FactSource>("fact_add_source", { input });
  },

  /**
   * Mengarsipkan fakta profil.
   */
  async archiveFact(personaId: string, factId: string): Promise<void> {
    return invokeCommand<void>("fact_archive", { personaId, factId });
  },

  /**
   * Memulihkan fakta dari arsip.
   */
  async restoreFact(personaId: string, factId: string): Promise<void> {
    return invokeCommand<void>("fact_restore", { personaId, factId });
  },

  /**
   * Menghapus fakta profil secara permanen.
   */
  async deleteFact(personaId: string, factId: string): Promise<void> {
    return invokeCommand<void>("fact_delete", { personaId, factId });
  },

  /**
   * Mendeteksi konflik antar-fakta pada profil persona aktif.
   */
  async detectConflicts(personaId: string): Promise<ConflictWarning[]> {
    return invokeCommand<ConflictWarning[]>("fact_detect_conflicts", { personaId });
  },

  /**
   * Mengambil daftar fakta yang layak digunakan sebagai bukti untuk rekomendasi AI.
   */
  async getEligibleFactsForAi(
    personaId: string,
    allowSensitive = false
  ): Promise<ProfileFact[]> {
    return invokeCommand<ProfileFact[]>("fact_get_eligible_for_ai", {
      personaId,
      allowSensitive,
    });
  },
};
