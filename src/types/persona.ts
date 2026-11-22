export interface Persona {
  id: string;
  displayName: string;
  isActive: boolean;
  status: string;
}

export interface CreatePersonaInput {
  displayName: string;
}
