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
import type { Persona, PersonaDependencySummary } from "./types/persona";
import type {
  ConflictWarning,
  CreateFactInput,
  FactSource,
  ProfileFact,
  ProfileFactDetail,
  Sensitivity,
  UpdateFactInput,
  VerificationStatus,
} from "./types/fact";
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

  const [personas, setPersonas] = useState<Persona[]>([]);
  const [profileError, setProfileError] = useState("");
  const [profileLoading, setProfileLoading] = useState(false);

  const [facts, setFacts] = useState<ProfileFact[]>([]);
  const [conflicts, setConflicts] = useState<ConflictWarning[]>([]);
  const [factError, setFactError] = useState("");
  const [factLoading, setFactLoading] = useState(false);

  async function refreshPersonas() {
    try {
      const result = await personaService.listPersonas(true);
      setPersonas(result);
      setProfileError("");
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
      const [factList, conflictList] = await Promise.all([
        factService.listFacts(personaId),
        factService.detectConflicts(personaId),
      ]);
      setFacts(factList);
      setConflicts(conflictList);
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
      setConflicts([]);
    }
  }, [activePersona?.id]);

  async function handleCreatePersona(displayName: string, description?: string) {
    setProfileLoading(true);
    setProfileError("");
    try {
      await personaService.createPersona(displayName, description);
      await refreshPersonas();
    } catch (error) {
      setProfileError(AppError.from(error).message);
    } finally {
      setProfileLoading(false);
    }
  }

  async function handleUpdatePersona(
    personaId: string,
    displayName: string,
    description?: string
  ) {
    setProfileLoading(true);
    setProfileError("");
    try {
      await personaService.updatePersona(personaId, displayName, description);
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

  async function handleArchivePersona(personaId: string) {
    setProfileError("");
    try {
      await personaService.archivePersona(personaId);
      await refreshPersonas();
    } catch (error) {
      setProfileError(AppError.from(error).message);
    }
  }

  async function handleRestorePersona(personaId: string) {
    setProfileError("");
    try {
      await personaService.restorePersona(personaId);
      await refreshPersonas();
    } catch (error) {
      setProfileError(AppError.from(error).message);
    }
  }

  async function handleGetDependencySummary(
    personaId: string
  ): Promise<PersonaDependencySummary> {
    return personaService.getDependencySummary(personaId);
  }

  async function handleDeletePersona(personaId: string) {
    setProfileError("");
    try {
      await personaService.deletePersona(personaId);
      await refreshPersonas();
    } catch (error) {
      setProfileError(AppError.from(error).message);
    }
  }

  async function handleCreateFact(input: CreateFactInput) {
    if (!activePersona) return;
    setFactLoading(true);
    setFactError("");
    try {
      await factService.createFact(input);
      await refreshFacts(activePersona.id);
    } catch (error) {
      setFactError(AppError.from(error).message);
    } finally {
      setFactLoading(false);
    }
  }

  async function handleUpdateFact(input: UpdateFactInput) {
    if (!activePersona) return;
    setFactLoading(true);
    setFactError("");
    try {
      await factService.updateFact(input);
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

  async function handleUpdateFactSensitivity(
    factId: string,
    sensitivity: Sensitivity
  ) {
    if (!activePersona) return;
    try {
      await factService.setFactSensitivity(activePersona.id, factId, sensitivity);
      await refreshFacts(activePersona.id);
    } catch (error) {
      setFactError(AppError.from(error).message);
    }
  }

  async function handleGetFactDetail(factId: string): Promise<ProfileFactDetail> {
    if (!activePersona) {
      throw new Error("Tidak ada persona yang aktif");
    }
    return factService.getFactDetail(activePersona.id, factId);
  }

  async function handleAddFactSource(input: {
    factId: string;
    sourceType: string;
    title: string;
    description: string;
    referenceUri?: string;
    observedAt?: string;
    notes?: string;
  }): Promise<FactSource> {
    if (!activePersona) {
      throw new Error("Tidak ada persona yang aktif");
    }
    const result = await factService.addFactSource({
      personaId: activePersona.id,
      factId: input.factId,
      sourceType: input.sourceType,
      title: input.title,
      description: input.description,
      referenceUri: input.referenceUri,
      observedAt: input.observedAt,
      notes: input.notes,
    });
    await refreshFacts(activePersona.id);
    return result;
  }

  async function handleDeleteFact(factId: string) {
    if (!activePersona || !window.confirm("Hapus fakta profil ini secara permanen?")) return;
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
          personas={personas.filter((p) => p.status === "active")}
          activePersona={activePersona}
          onSelectPersona={(id) => void handleActivatePersona(id)}
        />

        {page === "home" && (
          <HomeView activePersona={activePersona} onNavigate={setPage} />
        )}

        {page === "copilot" && (
          <CopilotView
            activePersona={activePersona}
          />
        )}

        {page === "profiles" && (
          <ProfilesView
            personas={personas}
            activePersona={activePersona}
            facts={facts}
            conflicts={conflicts}
            profileError={profileError}
            profileLoading={profileLoading}
            factError={factError}
            factLoading={factLoading}
            onCreatePersona={handleCreatePersona}
            onUpdatePersona={handleUpdatePersona}
            onActivatePersona={handleActivatePersona}
            onArchivePersona={handleArchivePersona}
            onRestorePersona={handleRestorePersona}
            onGetDependencySummary={handleGetDependencySummary}
            onDeletePersona={handleDeletePersona}
            onCreateFact={handleCreateFact}
            onUpdateFact={handleUpdateFact}
            onUpdateFactStatus={handleUpdateFactStatus}
            onUpdateFactSensitivity={handleUpdateFactSensitivity}
            onGetFactDetail={handleGetFactDetail}
            onAddFactSource={handleAddFactSource}
            onDeleteFact={handleDeleteFact}
          />
        )}

        {page === "history" && (
          <HistoryView
            activePersona={activePersona}
            onOpenSessionInCopilot={() => {
              setPage("copilot");
            }}
          />
        )}

        {page === "settings" && (
          <SettingsView
            personas={personas}
            activePersona={activePersona}
            onRefreshPersonas={refreshPersonas}
            onDeletePersona={handleDeletePersona}
          />
        )}
      </main>
    </div>
  );
}
