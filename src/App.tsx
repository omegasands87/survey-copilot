import { useEffect, useState } from "react";
import { Sidebar, type Page } from "./components/Sidebar";
import { Topbar } from "./components/Topbar";
import { HomeView } from "./components/views/HomeView";
import { CopilotView } from "./components/views/CopilotView";
import { ProfilesView } from "./components/views/ProfilesView";
import { HistoryView } from "./components/views/HistoryView";
import { SettingsView } from "./components/views/SettingsView";
import { personaService } from "./services/personaService";
import { factService } from "./services/factService";
import type { Persona } from "./types/persona";
import type { ProfileFact, VerificationStatus, Sensitivity } from "./types/fact";
import { AppError } from "./types/error";

const pageTitles: Record<Page, string> = {
  home: "Beranda",
  profiles: "Profil",
  copilot: "Copilot",
  history: "Riwayat",
  settings: "Pengaturan",
};

export default function App() {
  const [page, setPage] = useState<Page>("home");
  const [question, setQuestion] = useState("");

  const [personas, setPersonas] = useState<Persona[]>([]);
  const [profileError, setProfileError] = useState("");
  const [profileLoading, setProfileLoading] = useState(false);

  const [facts, setFacts] = useState<ProfileFact[]>([]);
  const [factError, setFactError] = useState("");
  const [factLoading, setFactLoading] = useState(false);

  async function refreshPersonas() {
    try {
      const result = await personaService.listPersonas();
      setPersonas(result);
    } catch (error) {
      setProfileError(AppError.from(error).message);
    }
  }

  useEffect(() => {
    void refreshPersonas();
  }, []);

  const activePersona = personas.find((persona) => persona.isActive);

  async function refreshFacts(personaId: string) {
    try {
      const result = await factService.listFacts(personaId);
      setFacts(result);
      setFactError("");
    } catch (error) {
      setFactError(AppError.from(error).message);
    }
  }

  useEffect(() => {
    if (activePersona) {
      void refreshFacts(activePersona.id);
    } else {
      setFacts([]);
    }
  }, [activePersona?.id]);

  async function handleCreatePersona(displayName: string) {
    setProfileLoading(true);
    setProfileError("");
    try {
      await personaService.createPersona(displayName);
      await refreshPersonas();
    } catch (error) {
      setProfileError(AppError.from(error).message);
    } finally {
      setProfileLoading(false);
    }
  }

  async function handleActivatePersona(personaId: string) {
    setProfileError("");
    try {
      await personaService.setActivePersona(personaId);
      await refreshPersonas();
    } catch (error) {
      setProfileError(AppError.from(error).message);
    }
  }

  async function handleCreateFact(factData: {
    category: string;
    attribute: string;
    value: string;
    sourceDescription: string;
    verificationStatus: VerificationStatus;
    sensitivity: Sensitivity;
  }) {
    if (!activePersona) return;
    setFactLoading(true);
    setFactError("");
    try {
      await factService.createFact({
        personaId: activePersona.id,
        category: factData.category,
        attribute: factData.attribute,
        value: factData.value,
        sourceDescription: factData.sourceDescription,
        verificationStatus: factData.verificationStatus,
        sensitivity: factData.sensitivity,
      });
      await refreshFacts(activePersona.id);
    } catch (error) {
      setFactError(AppError.from(error).message);
    } finally {
      setFactLoading(false);
    }
  }

  async function handleUpdateFactStatus(
    factId: string,
    status: VerificationStatus
  ) {
    if (!activePersona) return;
    try {
      await factService.setFactStatus(activePersona.id, factId, status);
      await refreshFacts(activePersona.id);
    } catch (error) {
      setFactError(AppError.from(error).message);
    }
  }

  async function handleDeleteFact(factId: string) {
    if (!activePersona || !window.confirm("Hapus informasi profil ini?")) return;
    try {
      await factService.deleteFact(activePersona.id, factId);
      await refreshFacts(activePersona.id);
    } catch (error) {
      setFactError(AppError.from(error).message);
    }
  }

  return (
    <div className="app-shell">
      <Sidebar activePage={page} onSelectPage={setPage} />

      <main className="main-area">
        <Topbar
          title={pageTitles[page]}
          personas={personas}
          activePersona={activePersona}
          onSelectPersona={(id) => void handleActivatePersona(id)}
        />

        {page === "home" && (
          <HomeView activePersona={activePersona} onNavigate={setPage} />
        )}

        {page === "copilot" && (
          <CopilotView
            question={question}
            activePersona={activePersona}
            onChangeQuestion={setQuestion}
          />
        )}

        {page === "profiles" && (
          <ProfilesView
            personas={personas}
            activePersona={activePersona}
            facts={facts}
            profileError={profileError}
            profileLoading={profileLoading}
            factError={factError}
            factLoading={factLoading}
            onCreatePersona={handleCreatePersona}
            onActivatePersona={handleActivatePersona}
            onCreateFact={handleCreateFact}
            onUpdateFactStatus={handleUpdateFactStatus}
            onDeleteFact={handleDeleteFact}
          />
        )}

        {page === "history" && <HistoryView />}

        {page === "settings" && <SettingsView />}
      </main>
    </div>
  );
}
