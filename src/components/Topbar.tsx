import type { Persona } from "../types/persona";

interface TopbarProps {
  title: string;
  personas: Persona[];
  activePersona?: Persona;
  onSelectPersona: (personaId: string) => void;
}

export function Topbar({
  title,
  personas,
  activePersona,
  onSelectPersona,
}: TopbarProps) {
  return (
    <header className="topbar">
      <div>
        <span className="eyebrow">SURVEY COPILOT</span>
        <h1>{title}</h1>
      </div>
      <select
        className="persona-select"
        aria-label="Profil responden aktif"
        value={activePersona?.id ?? ""}
        disabled={personas.length === 0}
        onChange={(event) => onSelectPersona(event.target.value)}
      >
        {personas.length === 0 && <option value="">Buat profil dahulu</option>}
        {personas.map((persona) => (
          <option key={persona.id} value={persona.id}>
            {persona.displayName}
          </option>
        ))}
      </select>
    </header>
  );
}
