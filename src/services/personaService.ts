import { invokeCommand } from "./ipcClient";
import type {
  Persona,
  PersonaDependencySummary,
} from "../types/persona";

export const personaService = {
  /**
   * Mengambil daftar seluruh persona dengan opsi menyertakan yang diarsipkan.
   */
  async listPersonas(includeArchived = false): Promise<Persona[]> {
    return invokeCommand<Persona[]>("persona_list", { includeArchived });
  },

  /**
   * Mengambil detail satu persona berdasarkan ID.
   */
  async getPersona(personaId: string): Promise<Persona> {
    return invokeCommand<Persona>("persona_get", { personaId });
  },

  /**
   * Membuat persona baru.
   */
  async createPersona(displayName: string, description?: string): Promise<Persona> {
    return invokeCommand<Persona>("persona_create", {
      displayName,
      description: description || null,
    });
  },

  /**
   * Memperbarui nama dan deskripsi persona.
   */
  async updatePersona(
    personaId: string,
    displayName: string,
    description?: string
  ): Promise<Persona> {
    return invokeCommand<Persona>("persona_update", {
      personaId,
      displayName,
      description: description || null,
    });
  },

  /**
   * Menetapkan persona aktif. Tepat satu persona aktif pada satu waktu.
   */
  async setActivePersona(personaId: string): Promise<Persona> {
    return invokeCommand<Persona>("persona_set_active", { personaId });
  },

  /**
   * Mengarsipkan persona.
   */
  async archivePersona(personaId: string): Promise<Persona> {
    return invokeCommand<Persona>("persona_archive", { personaId });
  },

  /**
   * Memulihkan persona dari arsip.
   */
  async restorePersona(personaId: string): Promise<Persona> {
    return invokeCommand<Persona>("persona_restore", { personaId });
  },

  /**
   * Mendapatkan ringkasan ketergantungan data sebelum penghapusan.
   */
  async getDependencySummary(
    personaId: string
  ): Promise<PersonaDependencySummary> {
    return invokeCommand<PersonaDependencySummary>(
      "persona_get_dependency_summary",
      { personaId }
    );
  },

  /**
   * Menghapus persona beserta seluruh datanya secara permanen.
   */
  async deletePersona(personaId: string): Promise<PersonaDependencySummary> {
    return invokeCommand<PersonaDependencySummary>("persona_delete", {
      personaId,
    });
  },
};
