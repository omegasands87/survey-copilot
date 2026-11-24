import { useMemo, useState } from "react";
import { Sidebar, type Page } from "../components/Sidebar";
import { Topbar } from "../components/Topbar";
import { HomeView } from "../components/views/HomeView";
import { ProfilesView } from "../components/views/ProfilesView";
import type { Persona, PersonaDependencySummary } from "../types/persona";
import type {
  ConflictWarning,
  CreateFactInput,
  FactSource,
  ProfileFact,
  ProfileFactDetail,
  Sensitivity,
  UpdateFactInput,
  VerificationStatus,
} from "../types/fact";
import "./preview.css";

const DEMO_TIME = "2026-11-24T09:00:00.000Z";

const initialPersonas: Persona[] = [
  {
    id: "demo-persona-01",
    displayName: "Profil Demo 01",
    description: "Contoh profil responden untuk evaluasi tampilan.",
    isActive: true,
    status: "active",
    createdAt: DEMO_TIME,
    updatedAt: DEMO_TIME,
    factsCount: 5,
    sessionsCount: 3,
  },
  {
    id: "demo-persona-02",
    displayName: "Profil Demo 02",
    description: "Profil demo kedua untuk menguji pergantian persona.",
    isActive: false,
    status: "active",
    createdAt: DEMO_TIME,
    updatedAt: DEMO_TIME,
    factsCount: 4,
    sessionsCount: 2,
  },
];

const initialFacts: ProfileFact[] = [
  {
    id: "demo-fact-01",
    personaId: "demo-persona-01",
    category: "Demografi",
    factKey: "age",
    attribute: "Usia",
    dataType: "number",
    value: 32,
    normalizedValue: "32",
    unit: "tahun",
    verificationStatus: "confirmed",
    sensitivity: "ordinary",
    sourceDescription: "Contoh input pengguna",
    validFrom: "2026-01-01",
    validUntil: null,
    isValid: true,
    hasConflict: false,
    sourcesCount: 1,
    createdAt: DEMO_TIME,
    updatedAt: DEMO_TIME,
  },
  {
    id: "demo-fact-02",
    personaId: "demo-persona-01",
    category: "Pekerjaan",
    factKey: "employment_status",
    attribute: "Status pekerjaan",
    dataType: "string",
    value: "Karyawan swasta",
    normalizedValue: "Karyawan swasta",
    unit: null,
    verificationStatus: "confirmed",
    sensitivity: "ordinary",
    sourceDescription: "Contoh input pengguna",
    validFrom: "2026-01-01",
    validUntil: null,
    isValid: true,
    hasConflict: false,
    sourcesCount: 1,
    createdAt: DEMO_TIME,
    updatedAt: DEMO_TIME,
  },
  {
    id: "demo-fact-03",
    personaId: "demo-persona-01",
    category: "Teknologi",
    factKey: "primary_device",
    attribute: "Perangkat utama",
    dataType: "string",
    value: "Android",
    normalizedValue: "Android",
    unit: null,
    verificationStatus: "unverified",
    sensitivity: "ordinary",
    sourceDescription: "Contoh input pengguna",
    validFrom: null,
    validUntil: null,
    isValid: true,
    hasConflict: false,
    sourcesCount: 0,
    createdAt: DEMO_TIME,
    updatedAt: DEMO_TIME,
  },
  {
    id: "demo-fact-04",
    personaId: "demo-persona-01",
    category: "Rumah tangga",
    factKey: "household_size",
    attribute: "Jumlah anggota rumah tangga",
    dataType: "number",
    value: 4,
    normalizedValue: "4",
    unit: "orang",
    verificationStatus: "confirmed",
    sensitivity: "ordinary",
    sourceDescription: "Contoh input pengguna",
    validFrom: null,
    validUntil: null,
    isValid: true,
    hasConflict: false,
    sourcesCount: 1,
    createdAt: DEMO_TIME,
    updatedAt: DEMO_TIME,
  },
  {
    id: "demo-fact-05",
    personaId: "demo-persona-01",
    category: "Preferensi",
    factKey: "shopping_channel",
    attribute: "Kanal belanja pilihan",
    dataType: "string",
    value: "Marketplace",
    normalizedValue: "Marketplace",
    unit: null,
    verificationStatus: "unverified",
    sensitivity: "ordinary",
    sourceDescription: "Contoh input pengguna",
    validFrom: null,
    validUntil: null,
    isValid: true,
    hasConflict: false,
    sourcesCount: 0,
    createdAt: DEMO_TIME,
    updatedAt: DEMO_TIME,
  },
];

const pageTitles: Record<Page, string> = {
  home: "Beranda",
  profiles: "Profil",
  copilot: "Copilot",
  history: "Riwayat",
  settings: "Pengaturan",
};

const demoSessions = [
  { id: "S-2401", name: "Studi pengalaman belanja online", platform: "Panel Demo", date: "24 Nov 2026", status: "Selesai", questions: 12 },
  { id: "S-2318", name: "Penggunaan perangkat digital", platform: "Panel Demo", date: "22 Nov 2026", status: "Berjalan", questions: 8 },
  { id: "S-2290", name: "Preferensi layanan keuangan", platform: "Panel Demo", date: "18 Nov 2026", status: "Disaring", questions: 5 },
];

function PreviewCopilotView() {
  const [question, setQuestion] = useState("Dalam 30 hari terakhir, seberapa sering Anda berbelanja melalui marketplace?");
  const [showAnswer, setShowAnswer] = useState(false);
  const [captureNotice, setCaptureNotice] = useState(false);

  return (
    <section className="content preview-page" aria-label="Preview Copilot">
      <div className="section-heading">
        <div>
          <span className="eyebrow">QUESTION WORKSPACE</span>
          <h2>Copilot Survei</h2>
          <p className="preview-subtitle">Contoh tampilan pembacaan pertanyaan, pemeriksaan bukti, dan rekomendasi jawaban.</p>
        </div>
        <span className="preview-static-pill">SIMULASI UI</span>
      </div>

      <div className="preview-toolbar">
        <div>
          <span className="preview-muted-label">SESI AKTIF</span>
          <strong>Studi pengalaman belanja online</strong>
        </div>
        <span className="preview-session-tag">Profil: Profil Demo 01</span>
      </div>

      <div className="preview-copilot-grid">
        <div className="preview-panel preview-question-panel">
          <div className="preview-panel-heading">
            <div>
              <span className="preview-muted-label">01 / PERTANYAAN</span>
              <h3>Masukkan pertanyaan survei</h3>
            </div>
            <span className="preview-step-number">01</span>
          </div>

          <div className="preview-action-row">
            <button type="button" className="secondary-button" onClick={() => setCaptureNotice(true)}>
              ▣ Simulasikan screenshot
            </button>
            <button type="button" className="secondary-button" onClick={() => setQuestion("")}>
              Tempel / ketik teks
            </button>
          </div>
          {captureNotice && (
            <div className="preview-inline-notice" role="status">
              Mode preview: capture desktop tidak tersedia. Ini hanya contoh state setelah screenshot.
              <button type="button" onClick={() => setCaptureNotice(false)} aria-label="Tutup notifikasi">×</button>
            </div>
          )}
          <label className="preview-field-label" htmlFor="preview-question-text">Teks pertanyaan</label>
          <textarea
            id="preview-question-text"
            className="preview-question-input"
            value={question}
            onChange={(event) => setQuestion(event.target.value)}
            rows={5}
          />
          <div className="preview-field-footer">
            <span>{question.length} karakter</span>
            <span className="preview-neutral-chip">Demo text • tidak disimpan</span>
          </div>

          <div className="preview-divider" />
          <div className="preview-panel-heading compact">
            <div>
              <span className="preview-muted-label">02 / STRUKTUR</span>
              <h3>Opsi jawaban</h3>
            </div>
          </div>
          <div className="preview-option-list">
            {["Tidak pernah", "1–2 kali", "3–5 kali", "Lebih dari 5 kali"].map((option, index) => (
              <label className="preview-option" key={option}>
                <input type="radio" name="preview-option" defaultChecked={index === 1} />
                <span>{option}</span>
              </label>
            ))}
          </div>
          <div className="preview-action-row preview-action-row-end">
            <button type="button" className="secondary-button" onClick={() => { setQuestion("Dalam 30 hari terakhir, seberapa sering Anda berbelanja melalui marketplace?"); setShowAnswer(false); }}>
              Reset contoh
            </button>
            <button type="button" className="primary-button" disabled={!question.trim()} onClick={() => setShowAnswer(true)}>
              Tampilkan rekomendasi demo <span aria-hidden="true">→</span>
            </button>
          </div>
        </div>

        <aside className="preview-side-stack">
          <div className="preview-panel">
            <div className="preview-panel-heading compact">
              <div>
                <span className="preview-muted-label">KONTEKS PROFIL</span>
                <h3>Profil Demo 01</h3>
              </div>
              <span className="preview-avatar">AP</span>
            </div>
            <div className="preview-mini-facts">
              <div><span>Usia</span><strong>32 tahun</strong></div>
              <div><span>Status pekerjaan</span><strong>Karyawan swasta</strong></div>
              <div><span>Perangkat utama</span><strong>Android</strong></div>
            </div>
            <p className="preview-footnote">Semua data pada layar ini adalah data sintetis untuk evaluasi tampilan.</p>
          </div>

          <div className="preview-panel preview-evidence-panel">
            <span className="preview-muted-label">PEMERIKSAAN BUKTI</span>
            <h3>Status kecocokan profil</h3>
            <div className="preview-evidence-status"><span className="preview-status-dot" /> Ada konteks yang relevan</div>
            <p>Contoh panel ini menunjukkan bagaimana kecocokan, fakta pendukung, dan kebutuhan klarifikasi dapat ditampilkan.</p>
            <div className="preview-evidence-line"><span>Fakta terkait</span><strong>2</strong></div>
            <div className="preview-evidence-line"><span>Perlu konfirmasi</span><strong>1</strong></div>
          </div>
        </aside>
      </div>

      {showAnswer && (
        <div className="preview-panel preview-answer-panel" role="status">
          <div className="preview-panel-heading compact">
            <div>
              <span className="preview-muted-label">CONTOH SINTETIS</span>
              <h3>Rekomendasi jawaban demo</h3>
            </div>
            <span className="preview-neutral-chip">Bukan jawaban AI aktual</span>
          </div>
          <p>Contoh: “1–2 kali”, berdasarkan fakta demo yang terlihat pada panel konteks. Nilai ini bukan fakta responden dan tidak disimpan.</p>
        </div>
      )}
    </section>
  );
}

function PreviewHistoryView() {
  const [filter, setFilter] = useState("Semua");
  const visibleSessions = demoSessions.filter((session) => filter === "Semua" || session.status === filter);
  return (
    <section className="content preview-page" aria-label="Preview Riwayat">
      <div className="section-heading">
        <div>
          <span className="eyebrow">SURVEY MEMORY</span>
          <h2>Riwayat Survei</h2>
          <p className="preview-subtitle">Contoh daftar sesi dan hasil survei untuk menilai hierarchy informasi.</p>
        </div>
        <span className="preview-static-pill">DATA DEMO</span>
      </div>
      <div className="preview-stat-grid">
        <article className="preview-stat-card"><span>Total sesi</span><strong>24</strong><small>Contoh data</small></article>
        <article className="preview-stat-card"><span>Pertanyaan tersimpan</span><strong>186</strong><small>Contoh data</small></article>
        <article className="preview-stat-card"><span>Jawaban dikonfirmasi</span><strong>142</strong><small>Contoh data</small></article>
      </div>
      <div className="preview-panel">
        <div className="preview-list-toolbar">
          <div><h3>Aktivitas terbaru</h3><p>Daftar sesi per persona demo.</p></div>
          <div className="preview-filter-row">
            {["Semua", "Selesai", "Berjalan", "Disaring"].map((item) => (
              <button key={item} type="button" className={filter === item ? "filter-tab active" : "filter-tab"} onClick={() => setFilter(item)}>{item}</button>
            ))}
          </div>
        </div>
        <div className="preview-session-list">
          {visibleSessions.map((session) => (
            <article className="preview-session-row" key={session.id}>
              <div className="preview-session-icon">▤</div>
              <div className="preview-session-main">
                <strong>{session.name}</strong>
                <span>{session.platform} · {session.date} · {session.questions} pertanyaan</span>
              </div>
              <span className={`preview-session-status ${session.status === "Selesai" ? "is-done" : session.status === "Berjalan" ? "is-active" : "is-muted"}`}>{session.status}</span>
              <button type="button" className="text-button" aria-label={`Buka preview ${session.name}`}>Detail →</button>
            </article>
          ))}
          {visibleSessions.length === 0 && <p className="preview-empty-state">Tidak ada sesi untuk filter ini.</p>}
        </div>
      </div>
    </section>
  );
}

function PreviewSettingsView() {
  const [activeTab, setActiveTab] = useState("Umum");
  const [privacyEnabled, setPrivacyEnabled] = useState(true);
  const tabs = ["Umum", "Provider", "Privasi", "Data"];
  return (
    <section className="content preview-page" aria-label="Preview Pengaturan">
      <div className="section-heading">
        <div>
          <span className="eyebrow">PREFERENCES</span>
          <h2>Pengaturan</h2>
          <p className="preview-subtitle">Pratinjau struktur panel dan pengelompokan kontrol aplikasi.</p>
        </div>
        <span className="preview-static-pill">MOCK CONTROLS</span>
      </div>
      <div className="preview-settings-tabs" role="tablist" aria-label="Kategori pengaturan demo">
        {tabs.map((tab) => <button key={tab} type="button" role="tab" aria-selected={activeTab === tab} className={activeTab === tab ? "active" : ""} onClick={() => setActiveTab(tab)}>{tab}</button>)}
      </div>
      {activeTab === "Umum" && (
        <div className="preview-settings-grid">
          <div className="preview-panel">
            <span className="preview-muted-label">PREFERENSI TAMPILAN</span>
            <h3>Pengalaman aplikasi</h3>
            <div className="preview-setting-row"><div><strong>Notifikasi di aplikasi</strong><span>Informasi status dan hasil tindakan.</span></div><span className="preview-toggle is-on" aria-label="Contoh toggle aktif" /></div>
            <div className="preview-setting-row"><div><strong>Konfirmasi sebelum menghapus</strong><span>Menampilkan konfirmasi sebelum aksi destruktif.</span></div><span className="preview-toggle is-on" aria-label="Contoh toggle aktif" /></div>
          </div>
          <div className="preview-panel">
            <span className="preview-muted-label">KONEKSI LOKAL</span>
            <h3>Status layanan</h3>
            <div className="preview-provider-row"><span className="preview-status-dot" /><div><strong>Database lokal</strong><span>Mock state • tidak terhubung</span></div><span className="preview-neutral-chip">Demo</span></div>
            <div className="preview-provider-row"><span className="preview-status-dot is-off" /><div><strong>Provider AI</strong><span>Tidak ada request dari preview</span></div><span className="preview-neutral-chip">Off</span></div>
          </div>
        </div>
      )}
      {activeTab === "Provider" && (
        <div className="preview-panel">
          <span className="preview-muted-label">PROVIDER CONFIGURATION</span>
          <h3>Provider AI (contoh tampilan)</h3>
          <div className="preview-provider-row"><span className="preview-provider-logo">G</span><div><strong>Gemini</strong><span>Konfigurasi demonstrasi · tidak ada API key</span></div><span className="preview-neutral-chip">Mock</span></div>
          <p className="preview-footnote">Pengaturan provider yang sebenarnya hanya tersedia dalam aplikasi desktop. Preview tidak menghubungi provider.</p>
        </div>
      )}
      {activeTab === "Privasi" && (
        <div className="preview-panel">
          <span className="preview-muted-label">PRIVACY & CONSENT</span>
          <h3>Kontrol privasi</h3>
          <div className="preview-setting-row"><div><strong>Contoh preferensi privasi</strong><span>Interaksi ini hanya mengubah state sementara.</span></div><button type="button" className={privacyEnabled ? "preview-toggle is-on" : "preview-toggle"} aria-label="Toggle contoh privasi" aria-pressed={privacyEnabled} onClick={() => setPrivacyEnabled((value) => !value)} /></div>
          <p className="preview-footnote">Tidak ada setting yang disimpan atau diterapkan ke aplikasi.</p>
        </div>
      )}
      {activeTab === "Data" && (
        <div className="preview-panel">
          <span className="preview-muted-label">LOCAL DATA</span>
          <h3>Ekspor dan penghapusan data</h3>
          <p>Panel ini hanya menunjukkan penempatan informasi retensi, ekspor, dan data berbahaya.</p>
          <div className="preview-action-row">
            <button type="button" className="secondary-button" disabled>Pratinjau ekspor (nonaktif)</button>
            <button type="button" className="danger-button" disabled>Hapus data demo (nonaktif)</button>
          </div>
        </div>
      )}
      <div className="preview-disclaimer">Semua kontrol di halaman ini bersifat demonstrasi dan tidak mengubah aplikasi desktop.</div>
    </section>
  );
}

function makeDependencySummary(persona: Persona, facts: ProfileFact[]): PersonaDependencySummary {
  const count = facts.filter((fact) => fact.personaId === persona.id).length;
  return {
    personaId: persona.id,
    displayName: persona.displayName,
    factsCount: count,
    sourcesCount: count,
    sessionsCount: persona.sessionsCount ?? 0,
    questionsCount: 0,
    recommendationsCount: 0,
    answersCount: 0,
    fitResultsCount: 0,
    attemptsCount: 0,
    totalAssociatedRecords: count + (persona.sessionsCount ?? 0),
  };
}

/**
 * Isolated browser preview. This module imports no service or Tauri IPC client.
 * All interactions are in-memory React state and disappear on refresh.
 */
export function PreviewApp() {
  const [page, setPage] = useState<Page>("home");
  const [personas, setPersonas] = useState<Persona[]>(initialPersonas);
  const [activePersonaId, setActivePersonaId] = useState("demo-persona-01");
  const [facts, setFacts] = useState<ProfileFact[]>(initialFacts);
  const activePersona = personas.find((persona) => persona.id === activePersonaId && persona.status === "active");
  const activeFacts = useMemo(() => facts.filter((fact) => fact.personaId === activePersonaId), [facts, activePersonaId]);

  async function createPersona(displayName: string, description?: string) {
    const id = `demo-persona-${Date.now()}`;
    setPersonas((current) => [...current, {
      id, displayName, description: description ?? "Profil demo yang dibuat di browser.",
      isActive: false, status: "active", createdAt: DEMO_TIME, updatedAt: DEMO_TIME, factsCount: 0, sessionsCount: 0,
    }]);
  }

  async function updatePersona(personaId: string, displayName: string, description?: string) {
    setPersonas((current) => current.map((persona) => persona.id === personaId
      ? { ...persona, displayName, description, updatedAt: DEMO_TIME }
      : persona));
  }

  async function activatePersona(personaId: string) {
    setActivePersonaId(personaId);
    setPersonas((current) => current.map((persona) => ({ ...persona, isActive: persona.id === personaId })));
  }

  async function archivePersona(personaId: string) {
    setPersonas((current) => current.map((persona) => persona.id === personaId
      ? { ...persona, status: "archived", isActive: false }
      : persona));
    if (activePersonaId === personaId) {
      const next = personas.find((persona) => persona.id !== personaId && persona.status === "active");
      setActivePersonaId(next?.id ?? "");
    }
  }

  async function restorePersona(personaId: string) {
    setPersonas((current) => current.map((persona) => persona.id === personaId
      ? { ...persona, status: "active" }
      : persona));
  }

  async function deletePersona(personaId: string) {
    setPersonas((current) => current.filter((persona) => persona.id !== personaId));
    setFacts((current) => current.filter((fact) => fact.personaId !== personaId));
    if (activePersonaId === personaId) {
      const next = personas.find((persona) => persona.id !== personaId && persona.status === "active");
      setActivePersonaId(next?.id ?? "");
    }
  }

  async function createFact(input: CreateFactInput) {
    const id = `demo-fact-${Date.now()}`;
    const numericValue = input.dataType === "number" ? Number(input.value) : Number.NaN;
    const value = Number.isFinite(numericValue) && input.dataType === "number" ? numericValue : input.value;
    setFacts((current) => [...current, {
      id,
      personaId: input.personaId,
      category: input.category,
      factKey: input.factKey ?? input.attribute.toLowerCase().replace(/\s+/g, "_"),
      attribute: input.attribute,
      dataType: input.dataType ?? "string",
      value,
      normalizedValue: input.value,
      unit: input.unit ?? null,
      verificationStatus: input.verificationStatus ?? "unverified",
      sensitivity: input.sensitivity ?? "ordinary",
      sourceDescription: input.sourceDescription,
      validFrom: input.validFrom ?? null,
      validUntil: input.validUntil ?? null,
      isValid: true,
      hasConflict: false,
      sourcesCount: input.sourceTitle ? 1 : 0,
      createdAt: DEMO_TIME,
      updatedAt: DEMO_TIME,
    }]);
  }

  async function updateFact(input: UpdateFactInput) {
    setFacts((current) => current.map((fact) => fact.id === input.factId
      ? {
          ...fact,
          value: input.value ?? fact.value,
          normalizedValue: input.value ?? fact.normalizedValue,
          unit: input.unit ?? fact.unit,
          validUntil: input.validUntil ?? fact.validUntil,
          updatedAt: DEMO_TIME,
        }
      : fact));
  }

  async function updateFactStatus(factId: string, status: VerificationStatus) {
    setFacts((current) => current.map((fact) => fact.id === factId ? { ...fact, verificationStatus: status } : fact));
  }

  async function updateFactSensitivity(factId: string, sensitivity: Sensitivity) {
    setFacts((current) => current.map((fact) => fact.id === factId ? { ...fact, sensitivity } : fact));
  }

  async function getFactDetail(factId: string): Promise<ProfileFactDetail> {
    const fact = facts.find((item) => item.id === factId);
    if (!fact) throw new Error("Fakta demo tidak ditemukan.");
    return { ...fact, sources: [], history: [] };
  }

  async function addFactSource(input: {
    factId: string;
    sourceType: string;
    title: string;
    description: string;
    referenceUri?: string;
    observedAt?: string;
    notes?: string;
  }): Promise<FactSource> {
    const source: FactSource = {
      id: `demo-source-${Date.now()}`,
      personaId: activePersonaId,
      factId: input.factId,
      sourceType: input.sourceType,
      title: input.title,
      description: input.description,
      referenceUri: input.referenceUri ?? null,
      observedAt: input.observedAt ?? null,
      notes: input.notes ?? null,
      createdAt: DEMO_TIME,
    };
    setFacts((current) => current.map((fact) => fact.id === input.factId
      ? { ...fact, sourcesCount: fact.sourcesCount + 1 }
      : fact));
    return source;
  }

  async function deleteFact(factId: string) {
    setFacts((current) => current.filter((fact) => fact.id !== factId));
  }

  return (
    <div className="app-shell preview-shell">
      <Sidebar activePage={page} onSelectPage={setPage} />
      <main className="main-area" role="main">
        <div className="preview-mode-banner">
          <span className="preview-banner-dot" />
          <div>
            <strong>UI PREVIEW</strong>
            <span>Data sintetis · jangan masukkan data pribadi/kredensial · tanpa backend desktop · perubahan hilang saat refresh</span>
          </div>
          <span className="preview-readonly-tag">BROWSER ONLY</span>
        </div>
        <div className="preview-mobile-settings-nav">
          <button
            type="button"
            className={page === "settings" ? "active" : ""}
            aria-current={page === "settings" ? "page" : undefined}
            onClick={() => setPage("settings")}
          >
            ⚙ Pengaturan
          </button>
        </div>
        <Topbar
          title={pageTitles[page]}
          personas={personas.filter((persona) => persona.status === "active")}
          activePersona={activePersona}
          onSelectPersona={(id) => void activatePersona(id)}
        />

        {page === "home" && <HomeView activePersona={activePersona} onNavigate={setPage} />}
        {page === "profiles" && (
          <ProfilesView
            personas={personas}
            activePersona={activePersona}
            facts={activeFacts}
            conflicts={[] as ConflictWarning[]}
            profileError=""
            profileLoading={false}
            factError=""
            factLoading={false}
            onRefreshPersonas={async () => undefined}
            onRefreshFacts={async () => undefined}
            onCreatePersona={createPersona}
            onUpdatePersona={updatePersona}
            onActivatePersona={activatePersona}
            onArchivePersona={archivePersona}
            onRestorePersona={restorePersona}
            onGetDependencySummary={(personaId) => {
              const persona = personas.find((item) => item.id === personaId);
              if (!persona) throw new Error("Persona demo tidak ditemukan.");
              return Promise.resolve(makeDependencySummary(persona, facts));
            }}
            onDeletePersona={deletePersona}
            onCreateFact={createFact}
            onUpdateFact={updateFact}
            onUpdateFactStatus={updateFactStatus}
            onUpdateFactSensitivity={updateFactSensitivity}
            onGetFactDetail={getFactDetail}
            onAddFactSource={addFactSource}
            onDeleteFact={deleteFact}
          />
        )}
        {page === "copilot" && <PreviewCopilotView />}
        {page === "history" && <PreviewHistoryView />}
        {page === "settings" && <PreviewSettingsView />}
      </main>
    </div>
  );
}
