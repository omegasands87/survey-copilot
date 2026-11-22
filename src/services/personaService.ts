import { invokeCommand } from "./ipcClient";
import type { Persona } from "../types/persona";

export const personaService = {
  /**
   * Retrieves all active personas ordered by active status and name.
   */
  async listPersonas(): Promise<Persona[]> {
    return invokeCommand<Persona[]>("persona_list");
  },

  /**
   * Creates a new persona and returns its summary.
   */
  async createPersona(displayName: string): Promise<Persona> {
    return invokeCommand<Persona>("persona_create", { displayName });
  },

  /**
   * Sets the active persona. Exactly one persona is active at a time.
   */
  async setActivePersona(personaId: string): Promise<Persona> {
    return invokeCommand<Persona>("persona_set_active", { personaId });
  },
};
