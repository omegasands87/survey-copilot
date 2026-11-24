export type PersonaStatus = "active" | "archived" | "disabled";

export interface Persona {
  id: string;
  displayName: string;
  description?: string | null;
  isActive: boolean;
  status: PersonaStatus;
  createdAt?: string;
  updatedAt?: string;
  archivedAt?: string | null;
  factsCount?: number;
  sessionsCount?: number;
}

export interface CreatePersonaInput {
  displayName: string;
  description?: string;
}

export interface UpdatePersonaInput {
  displayName: string;
  description?: string;
}

export interface PersonaDependencySummary {
  personaId: string;
  displayName: string;
  factsCount: number;
  sourcesCount: number;
  sessionsCount: number;
  questionsCount: number;
  recommendationsCount: number;
  answersCount: number;
  fitResultsCount: number;
  attemptsCount: number;
  totalAssociatedRecords: number;
}

export interface ProfileImportResult {
  persona: Persona;
  importedFacts: number;
  importedSources: number;
}
