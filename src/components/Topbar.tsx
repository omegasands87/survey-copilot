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
    <header className="topbar" role="banner">
      <div>
        <span className="eyebrow">SURVEY COPILOT</span>
        <h1>{title}</h1>
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
        <label htmlFor="topbar-persona-select" className="sr-only">
          Pilih profil responden aktif:
        </label>
        <select
          id="topbar-persona-select"
          className="persona-select"
          aria-label="Pilih profil responden aktif"
          value={activePersona?.id ?? ""}
          disabled={personas.length === 0}
          onChange={(event) => onSelectPersona(event.target.value)}
        >
          {personas.length === 0 && <option value="">(Belum ada profil aktif)</option>}
          {personas.map((persona) => (
            <option key={persona.id} value={persona.id}>
              👤 {persona.displayName}
            </option>
          ))}
        </select>
      </div>
    </header>
  );
}
