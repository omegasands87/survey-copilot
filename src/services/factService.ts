import { invokeCommand } from "./ipcClient";
import type { CreateFactInput, ProfileFact, VerificationStatus } from "../types/fact";

export const factService = {
  /**
   * Lists facts strictly scoped to the specified personaId.
   */
  async listFacts(personaId: string): Promise<ProfileFact[]> {
    return invokeCommand<ProfileFact[]>("fact_list", { personaId });
  },

  /**
   * Creates a new fact and source atomically for the specified persona.
   */
  async createFact(input: CreateFactInput): Promise<ProfileFact> {
    return invokeCommand<ProfileFact>("fact_create", {
      personaId: input.personaId,
      category: input.category,
      attribute: input.attribute,
      value: input.value,
      sourceDescription: input.sourceDescription,
      verificationStatus: input.verificationStatus,
      sensitivity: input.sensitivity,
    });
  },

  /**
   * Updates verification status for a persona's fact.
   */
  async setFactStatus(
    personaId: string,
    factId: string,
    status: VerificationStatus
  ): Promise<void> {
    return invokeCommand<void>("fact_set_status", { personaId, factId, status });
  },

  /**
   * Deletes a persona's fact if not referenced by survey history.
   */
  async deleteFact(personaId: string, factId: string): Promise<void> {
    return invokeCommand<void>("fact_delete", { personaId, factId });
  },
};
