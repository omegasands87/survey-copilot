import { useState, useEffect, type ChangeEvent, type FormEvent } from "react";
import type { Persona, PersonaDependencySummary, ProfileImportResult } from "../../types/persona";
import type { ExportPayload } from "../../types/dataManagement";
import type {
  ConflictWarning,
  CreateFactInput,
  DataType,
  FactSource,
  ProfileFact,
  ProfileFactDetail,
  Sensitivity,
  SourceType,
  UpdateFactInput,
  VerificationStatus,
} from "../../types/fact";

interface ProfilesViewProps {
  personas: Persona[];
  activePersona?: Persona;
  facts: ProfileFact[];
  conflicts: ConflictWarning[];
  profileError: string;
  profileLoading: boolean;
  factError: string;
  factLoading: boolean;
  onRefreshPersonas?: () => Promise<void>;
  onRefreshFacts?: () => Promise<void>;
  onCreatePersona: (displayName: string, description?: string) => Promise<void>;
  onUpdatePersona: (personaId: string, displayName: string, description?: string) => Promise<void>;
  onActivatePersona: (personaId: string) => Promise<void>;
  onArchivePersona: (personaId: string) => Promise<void>;
  onRestorePersona: (personaId: string) => Promise<void>;
  onGetDependencySummary: (personaId: string) => Promise<PersonaDependencySummary>;
  onDeletePersona: (personaId: string) => Promise<void>;
  onCreateFact: (input: CreateFactInput) => Promise<void>;
  onUpdateFact: (input: UpdateFactInput) => Promise<void>;
  onUpdateFactStatus: (factId: string, status: VerificationStatus) => Promise<void>;
  onUpdateFactSensitivity: (factId: string, sensitivity: Sensitivity) => Promise<void>;
  onGetFactDetail: (factId: string) => Promise<ProfileFactDetail>;
  onAddFactSource: (input: {
    factId: string;
    sourceType: string;
    title: string;
    description: string;
    referenceUri?: string;
    observedAt?: string;
    notes?: string;
  }) => Promise<FactSource>;
  onDeleteFact: (factId: string) => Promise<void>;
  onImportProfileJson: (jsonContent: string, displayName?: string) => Promise<ProfileImportResult>;
}

type FilterStatus = "all" | "active" | "archived";

interface ProfileTemplateField {
  attribute: string;
  dataType: DataType;
  unit?: string;
}

interface ProfileFieldTemplate {
  id: string;
  title: string;
  description: string;
  category: string;
  fields: ProfileTemplateField[];
}

const PROFILE_FIELD_TEMPLATES: ProfileFieldTemplate[] = [
  {
    id: "demographics",
    title: "Demografi dasar",
    description: "Field demografi yang umum muncul pada studi responden.",
    category: "Demografi",
    fields: [
      { attribute: "Usia", dataType: "number", unit: "tahun" },
      { attribute: "Domisili", dataType: "string" },
      { attribute: "Jenis kelamin", dataType: "string" },
      { attribute: "Status pernikahan", dataType: "string" },
    ],
  },
  {
    id: "work-education",
    title: "Pekerjaan & pendidikan",
    description: "Latar belakang pekerjaan dan pendidikan.",
    category: "Pekerjaan",
    fields: [
      { attribute: "Status pekerjaan", dataType: "string" },
      { attribute: "Bidang pekerjaan", dataType: "string" },
      { attribute: "Pendidikan terakhir", dataType: "string" },
      { attribute: "Jenis industri", dataType: "string" },
    ],
  },
  {
    id: "household",
    title: "Rumah tangga",
    description: "Komposisi dan kondisi rumah tangga.",
    category: "Rumah Tangga",
    fields: [
      { attribute: "Jumlah anggota rumah tangga", dataType: "number", unit: "orang" },
      { attribute: "Status tempat tinggal", dataType: "string" },
      { attribute: "Jumlah kendaraan rumah tangga", dataType: "number", unit: "unit" },
    ],
  },
  {
    id: "technology",
    title: "Teknologi & perangkat",
    description: "Perangkat dan kebiasaan penggunaan teknologi.",
    category: "Teknologi",
    fields: [
      { attribute: "Perangkat utama", dataType: "string" },
      { attribute: "Sistem operasi utama", dataType: "string" },
      { attribute: "Koneksi internet", dataType: "string" },
      { attribute: "Penggunaan aplikasi digital", dataType: "string" },
    ],
  },
  {
    id: "consumer",
    title: "Konsumsi & kebiasaan",
    description: "Kebiasaan dan preferensi konsumsi non-finansial.",
    category: "Konsumsi",
    fields: [
      { attribute: "Kanal belanja utama", dataType: "string" },
      { attribute: "Frekuensi belanja", dataType: "string" },
      { attribute: "Kategori produk yang digunakan", dataType: "string" },
      { attribute: "Merek yang digunakan", dataType: "string" },
    ],
  },
  {
    id: "mobility",
    title: "Transportasi & mobilitas",
    description: "Moda transportasi dan pola mobilitas responden.",
    category: "Transportasi",
    fields: [
      { attribute: "Moda transportasi utama", dataType: "string" },
      { attribute: "Kendaraan pribadi", dataType: "string" },
      { attribute: "Frekuensi bepergian", dataType: "string" },
    ],
  },
];

export function ProfilesView({
  personas,
  activePersona,
  facts,
  conflicts,
  profileError,
  profileLoading,
  factError,
  factLoading,
  onRefreshPersonas,
  onRefreshFacts,
  onCreatePersona,
  onUpdatePersona,
  onActivatePersona,
  onArchivePersona,
  onRestorePersona,
  onGetDependencySummary,
  onDeletePersona,
  onCreateFact,
  onUpdateFact,
  onUpdateFactStatus,
  onUpdateFactSensitivity,
  onGetFactDetail,
  onAddFactSource,
  onDeleteFact,
  onImportProfileJson,
}: ProfilesViewProps) {
  // Success state notification
  const [successMessage, setSuccessMessage] = useState("");
  function showSuccess(msg: string) {
    setSuccessMessage(msg);
    setTimeout(() => setSuccessMessage(""), 4000);
  }

  // New persona form
  const [profileName, setProfileName] = useState("");
  const [profileDescription, setProfileDescription] = useState("");

  // Import JSON hanya menerima export satu persona; isi file tetap lokal pada desktop.
  const [profileImportJson, setProfileImportJson] = useState("");
  const [profileImportPayload, setProfileImportPayload] = useState<ExportPayload | null>(null);
  const [profileImportName, setProfileImportName] = useState("");
  const [profileImportFileName, setProfileImportFileName] = useState("");
  const [profileImportError, setProfileImportError] = useState("");
  const [profileImportSubmitting, setProfileImportSubmitting] = useState(false);

  // Filter tab for personas
  const [filterTab, setFilterTab] = useState<FilterStatus>("active");

  // Edit persona modal
  const [editingPersona, setEditingPersona] = useState<Persona | null>(null);
  const [editName, setEditName] = useState("");
  const [editDescription, setEditDescription] = useState("");
  const [editSubmitting, setEditSubmitting] = useState(false);

  // Delete persona modal
  const [deletingPersona, setDeletingPersona] = useState<Persona | null>(null);
  const [deleteSummary, setDeleteSummary] = useState<PersonaDependencySummary | null>(null);
  const [deleteLoadingSummary, setDeleteLoadingSummary] = useState(false);
  const [deleteConfirmed, setDeleteConfirmed] = useState(false);
  const [deleteSubmitting, setDeleteSubmitting] = useState(false);

  // Conflict modal
  const [showConflictModal, setShowConflictModal] = useState(false);

  // Fact edit & detail modal
  const [editingFact, setEditingFact] = useState<ProfileFact | null>(null);
  const [editFactValue, setEditFactValue] = useState("");
  const [editFactUnit, setEditFactUnit] = useState("");
  const [editFactValidUntil, setEditFactValidUntil] = useState("");
  const [editFactReason, setEditFactReason] = useState("");
  const [editFactSubmitting, setEditFactSubmitting] = useState(false);

  const [selectedFactDetail, setSelectedFactDetail] = useState<ProfileFactDetail | null>(null);
  const [addingSourceFactId, setAddingSourceFactId] = useState<string | null>(null);
  const [newSourceType, setNewSourceType] = useState<SourceType>("manual_entry");
  const [newSourceTitle, setNewSourceTitle] = useState("");
  const [newSourceDesc, setNewSourceDesc] = useState("");
  const [newSourceRef, setNewSourceRef] = useState("");
  const [newSourceSubmitting, setNewSourceSubmitting] = useState(false);

  // Keyboard navigation & accessibility: Escape key closes active modal
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        if (editingPersona) setEditingPersona(null);
        else if (deletingPersona) setDeletingPersona(null);
        else if (editingFact) setEditingFact(null);
        else if (addingSourceFactId) setAddingSourceFactId(null);
        else if (selectedFactDetail) setSelectedFactDetail(null);
        else if (showConflictModal) setShowConflictModal(false);
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    editingPersona,
    deletingPersona,
    editingFact,
    addingSourceFactId,
    selectedFactDetail,
    showConflictModal,
  ]);

  // Fact creation form
  const [factCategory, setFactCategory] = useState("Demografi");
  const [factAttribute, setFactAttribute] = useState("");
  const [factKey, setFactKey] = useState("");
  const [factDataType, setFactDataType] = useState<DataType>("string");
  const [factValue, setFactValue] = useState("");
  const [factUnit, setFactUnit] = useState("");
  const [factSourceType, setFactSourceType] = useState<SourceType>("manual_entry");
  const [factSourceTitle, setFactSourceTitle] = useState("Input Manual");
  const [factSourceDesc, setFactSourceDesc] = useState("Dimasukkan langsung oleh pengguna");
  const [factValidFrom, setFactValidFrom] = useState("");
  const [factValidUntil, setFactValidUntil] = useState("");
  const [factNotes, setFactNotes] = useState("");
  const [newFactStatus, setNewFactStatus] = useState<VerificationStatus>("unverified");
  const [factSensitivity, setFactSensitivity] = useState<Sensitivity>("ordinary");

  // Handlers for Persona
  async function handleCreatePersona(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const name = profileName.trim();
    if (!name) return;
    await onCreatePersona(name, profileDescription.trim() || undefined);
    setProfileName("");
    setProfileDescription("");
    showSuccess(`Profil responden "${name}" berhasil dibuat.`);
  }

  async function handleProfileImportFile(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    setProfileImportError("");
    setProfileImportJson("");
    setProfileImportPayload(null);
    setProfileImportFileName("");
    setProfileImportName("");

    if (!file) return;
    if (!file.name.toLowerCase().endsWith(".json")) {
      setProfileImportError("Pilih file JSON ekspor satu profil Survey Copilot.");
      event.target.value = "";
      return;
    }
    if (file.size > 5 * 1024 * 1024) {
      setProfileImportError("Ukuran file maksimal 5 MB.");
      event.target.value = "";
      return;
    }

    try {
      const content = await file.text();
      const parsed: unknown = JSON.parse(content);
      if (!parsed || typeof parsed !== "object") {
        throw new Error("Struktur file JSON tidak valid.");
      }
      const candidate = parsed as Partial<ExportPayload>;
      const data = candidate.data;
      if (
        candidate.schemaVersion !== "1.0.0" ||
        candidate.exportScope !== "single_persona" ||
        candidate.summary?.exportScope !== "single_persona" ||
        !data ||
        !Array.isArray(data.personas) ||
        data.personas.length !== 1 ||
        !Array.isArray(data.facts) ||
        !Array.isArray(data.factSources)
      ) {
        throw new Error("File harus merupakan ekspor satu profil Survey Copilot versi 1.0.0, bukan ekspor seluruh aplikasi.");
      }

      const importedPersona = data.personas[0];
      if (
        !importedPersona ||
        typeof importedPersona.id !== "string" ||
        typeof importedPersona.displayName !== "string" ||
        importedPersona.id !== candidate.targetPersonaId ||
        !data.facts.every((fact) => fact.personaId === importedPersona.id) ||
        !data.factSources.every((source) => source.personaId === importedPersona.id)
      ) {
        throw new Error("Profil, fakta, atau sumber bukti pada file tidak konsisten.");
      }

      setProfileImportJson(content);
      setProfileImportPayload(candidate as ExportPayload);
      setProfileImportFileName(file.name);
      setProfileImportName(importedPersona.displayName);
    } catch (error) {
      setProfileImportError(
        error instanceof Error ? error.message : "File JSON tidak dapat dibaca.",
      );
      event.target.value = "";
    }
  }

  async function handleImportProfile() {
    if (!profileImportPayload || !profileImportJson || !profileImportName.trim()) return;
    setProfileImportSubmitting(true);
    setProfileImportError("");
    try {
      const result = await onImportProfileJson(profileImportJson, profileImportName.trim());
      await onRefreshPersonas?.();
      showSuccess(
        `Profil "${result.persona.displayName}" diimpor: ${result.importedFacts} fakta dan ${result.importedSources} sumber bukti.`,
      );
      setProfileImportJson("");
      setProfileImportPayload(null);
      setProfileImportFileName("");
      setProfileImportName("");
    } catch (error) {
      setProfileImportError(error instanceof Error ? error.message : "Import profil gagal.");
    } finally {
      setProfileImportSubmitting(false);
    }
  }

  function openEditModal(persona: Persona) {
    setEditingPersona(persona);
    setEditName(persona.displayName);
    setEditDescription(persona.description || "");
  }

  async function handleSaveEdit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!editingPersona) return;
    const name = editName.trim();
    if (!name) return;
    setEditSubmitting(true);
    try {
      await onUpdatePersona(editingPersona.id, name, editDescription.trim() || undefined);
      setEditingPersona(null);
      showSuccess(`Profil responden "${name}" berhasil diperbarui.`);
    } finally {
      setEditSubmitting(false);
    }
  }

  async function openDeleteModal(persona: Persona) {
    setDeletingPersona(persona);
    setDeleteConfirmed(false);
    setDeleteLoadingSummary(true);
    setDeleteSummary(null);
    try {
      const summary = await onGetDependencySummary(persona.id);
      setDeleteSummary(summary);
    } finally {
      setDeleteLoadingSummary(false);
    }
  }

  async function handleConfirmDelete() {
    if (!deletingPersona) return;
    setDeleteSubmitting(true);
    try {
      const name = deletingPersona.displayName;
      await onDeletePersona(deletingPersona.id);
      setDeletingPersona(null);
      showSuccess(`Profil responden "${name}" beserta seluruh data berhasil dihapus.`);
    } finally {
      setDeleteSubmitting(false);
    }
  }

  // Handlers for Facts
  async function handleCreateFact(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!activePersona) return;
    await onCreateFact({
      personaId: activePersona.id,
      category: factCategory,
      factKey: factKey.trim() || undefined,
      attribute: factAttribute.trim(),
      dataType: factDataType,
      value: factValue.trim(),
      unit: factUnit.trim() || undefined,
      sourceType: factSourceType,
      sourceTitle: factSourceTitle.trim() || undefined,
      sourceDescription: factSourceDesc.trim(),
      validFrom: factValidFrom.trim() || undefined,
      validUntil: factValidUntil.trim() || undefined,
      notes: factNotes.trim() || undefined,
      verificationStatus: newFactStatus,
      sensitivity: factSensitivity,
    });
    setFactAttribute("");
    setFactKey("");
    setFactValue("");
    setFactUnit("");
    setFactNotes("");
    setFactValidFrom("");
    setFactValidUntil("");
  }

  async function openFactDetail(factId: string) {
    const detail = await onGetFactDetail(factId);
    setSelectedFactDetail(detail);
  }

  function openEditFact(fact: ProfileFact) {
    setEditingFact(fact);
    setEditFactValue(typeof fact.value === "string" ? fact.value : JSON.stringify(fact.value));
    setEditFactUnit(fact.unit || "");
    setEditFactValidUntil(fact.validUntil || "");
    setEditFactReason("");
  }

  async function handleSaveFactEdit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!activePersona || !editingFact) return;
    setEditFactSubmitting(true);
    try {
      await onUpdateFact({
        personaId: activePersona.id,
        factId: editingFact.id,
        value: editFactValue.trim(),
        unit: editFactUnit.trim() || undefined,
        validUntil: editFactValidUntil.trim() || undefined,
        changeReason: editFactReason.trim() || undefined,
      });
      setEditingFact(null);
    } finally {
      setEditFactSubmitting(false);
    }
  }

  async function handleAddSourceSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!addingSourceFactId) return;
    setNewSourceSubmitting(true);
    try {
      await onAddFactSource({
        factId: addingSourceFactId,
        sourceType: newSourceType,
        title: newSourceTitle.trim(),
        description: newSourceDesc.trim(),
        referenceUri: newSourceRef.trim() || undefined,
      });
      setAddingSourceFactId(null);
      setNewSourceTitle("");
      setNewSourceDesc("");
      setNewSourceRef("");
      if (selectedFactDetail && selectedFactDetail.id === addingSourceFactId) {
        await openFactDetail(addingSourceFactId);
      }
    } finally {
      setNewSourceSubmitting(false);
    }
  }

  const filteredPersonas = personas.filter((p) => {
    if (filterTab === "active") return p.status === "active";
    if (filterTab === "archived") return p.status === "archived";
    return true;
  });

  const activeCount = personas.filter((p) => p.status === "active").length;
  const archivedCount = personas.filter((p) => p.status === "archived").length;

  return (
    <section className="content profile-content" role="region" aria-label="Kelola Profil Responden">
      <div className="section-heading">
        <div>
          <span className="eyebrow">DIGITAL PERSONA</span>
          <h2>Profil Responden</h2>
        </div>
        <span className="coming-soon">Data disimpan lokal & terisolasi di perangkat ini</span>
      </div>

      {/* Notifikasi Global Profil */}
      {successMessage && (
        <div className="notice" role="status" aria-live="polite" style={{ background: "#eef8f2", color: "#156236", marginBottom: "16px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <span aria-hidden="true">✓</span>
            <p style={{ margin: 0 }}>{successMessage}</p>
          </div>
          <button
            type="button"
            onClick={() => setSuccessMessage("")}
            aria-label="Tutup notifikasi sukses"
            style={{ background: "transparent", border: 0, cursor: "pointer", color: "#156236", fontWeight: "bold" }}
          >
            ✕
          </button>
        </div>
      )}

      {profileError && (
        <div className="notice" role="alert" style={{ background: "#fdf2f2", color: "#a12020", marginBottom: "16px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <span aria-hidden="true">⚠️</span>
            <p style={{ margin: 0 }}>{profileError}</p>
          </div>
          {onRefreshPersonas && (
            <button
              type="button"
              className="secondary-button"
              onClick={() => void onRefreshPersonas()}
              aria-label="Coba muat ulang daftar profil"
              style={{ padding: "4px 10px", fontSize: "11px" }}
            >
              🔄 Coba Lagi
            </button>
          )}
        </div>
      )}

      {/* Tambah Profil Baru */}
      <form className="profile-form" onSubmit={handleCreatePersona}>
        <label className="field-label" htmlFor="profile-name">
          Tambah Profil Responden Baru
        </label>
        <div className="profile-form-row">
          <input
            id="profile-name"
            value={profileName}
            onChange={(event) => setProfileName(event.target.value)}
            maxLength={80}
            placeholder="Nama lengkap responden asli (contoh: Budi Santoso)"
            required
            aria-required="true"
          />
          <button
            className="primary-button"
            type="submit"
            disabled={!profileName.trim() || profileLoading}
            aria-label="Simpan profil responden baru"
          >
            {profileLoading ? "Menyimpan..." : "Tambah profil"}
          </button>
        </div>
        <div style={{ marginTop: 8 }}>
          <input
            value={profileDescription}
            onChange={(event) => setProfileDescription(event.target.value)}
            maxLength={500}
            placeholder="Deskripsi singkat atau catatan responden (opsional)"
            aria-label="Deskripsi singkat profil (opsional)"
            style={{
              width: "100%",
              border: "1px solid #dce4de",
              borderRadius: 8,
              padding: "8px 12px",
              fontSize: 12,
              outline: "none",
            }}
          />
        </div>
      </form>

      {/* Filter Tabs Persona */}
      <div className="filter-tabs">
        <button
          className={`filter-tab ${filterTab === "active" ? "active" : ""}`}
          onClick={() => setFilterTab("active")}
        >
          Profil Aktif ({activeCount})
        </button>
        <button
          className={`filter-tab ${filterTab === "archived" ? "active" : ""}`}
          onClick={() => setFilterTab("archived")}
        >
          Diarsipkan ({archivedCount})
        </button>
        <button
          className={`filter-tab ${filterTab === "all" ? "active" : ""}`}
          onClick={() => setFilterTab("all")}
        >
          Semua ({personas.length})
        </button>
      </div>

      {/* Daftar Persona */}
      <div className="persona-list">
        {filteredPersonas.length === 0 ? (
          <div className="empty-inline">
            {filterTab === "archived"
              ? "Tidak ada profil yang sedang diarsipkan."
              : "Belum ada profil responden. Buat profil pertama menggunakan formulir di atas."}
          </div>
        ) : (
          filteredPersonas.map((persona) => {
            const isArchived = persona.status === "archived";
            const isCurrentlyActive = persona.isActive;

            return (
              <div
                className={`persona-card ${isCurrentlyActive ? "card-active" : ""} ${
                  isArchived ? "card-archived" : ""
                }`}
                key={persona.id}
              >
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div className="persona-card-header">
                    <strong>{persona.displayName}</strong>
                    {isCurrentlyActive && (
                      <span className="badge badge-active">Sedang Digunakan</span>
                    )}
                    {isArchived && (
                      <span className="badge badge-archived">Diarsipkan</span>
                    )}
                  </div>

                  {persona.description && (
                    <div className="persona-desc">{persona.description}</div>
                  )}

                  <div className="persona-meta">
                    <span>{persona.factsCount ?? 0} fakta tersimpan</span>
                    <span>{persona.sessionsCount ?? 0} sesi survei</span>
                  </div>
                </div>

                <div className="card-actions">
                  {!isCurrentlyActive && !isArchived && (
                    <button
                      className="secondary-button"
                      onClick={() => void onActivatePersona(persona.id)}
                      title="Gunakan profil ini sebagai responden aktif untuk rekomendasi survei"
                      aria-label={`Jadikan profil ${persona.displayName} sebagai responden aktif`}
                    >
                      Jadikan Aktif
                    </button>
                  )}

                  {!isArchived && (
                    <button
                      className="text-button"
                      onClick={() => openEditModal(persona)}
                      title="Ubah nama dan deskripsi profil"
                      aria-label={`Ubah nama dan deskripsi profil ${persona.displayName}`}
                    >
                      Ubah
                    </button>
                  )}

                  {isArchived ? (
                    <button
                      className="text-button"
                      onClick={() => void onRestorePersona(persona.id)}
                      title="Pulihkan profil ke daftar aktif"
                      aria-label={`Pulihkan profil ${persona.displayName} ke daftar aktif`}
                    >
                      Pulihkan
                    </button>
                  ) : (
                    <button
                      className="text-button"
                      onClick={() => void onArchivePersona(persona.id)}
                      title="Arsipkan profil ini"
                      aria-label={`Arsipkan profil ${persona.displayName}`}
                    >
                      Arsipkan
                    </button>
                  )}

                  <button
                    className="danger-button"
                    onClick={() => void openDeleteModal(persona)}
                    title="Hapus profil ini beserta seluruh datanya secara permanen"
                    aria-label={`Hapus profil ${persona.displayName} secara permanen`}
                  >
                    Hapus
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Bagian Fakta / Informasi Profil Aktif */}
      {activePersona && activePersona.status === "active" && (
        <div className="facts-section" style={{ marginTop: 36 }}>
          <div className="facts-heading">
            <div>
              <span className="eyebrow">BUKTI & FAKTA RESPONDEN</span>
              <h3 style={{ margin: "4px 0 0" }}>Fakta Profil: {activePersona.displayName}</h3>
            </div>
            <span style={{ fontSize: 11, color: "var(--muted)" }}>
              Data terikat strictly pada profil ini. AI hanya boleh menggunakan fakta yang berstatus Terkonfirmasi.
            </span>
          </div>

          {/* Banner Peringatan Konflik */}
          {conflicts.length > 0 && (
            <div className="conflict-banner" style={{ marginTop: 14 }}>
              <h4>
                <span>⚠️</span>
                <span>Peringatan Konflik Data ({conflicts.length} konflik terdeteksi)</span>
              </h4>
              <p>
                Terdapat fakta yang saling bertentangan pada profil ini. Fakta yang berkonflik otomatis ditangguhkan dari rekomendasi AI untuk mencegah inkonsistensi jawaban.
              </p>
              <button
                type="button"
                className="secondary-button"
                style={{ fontSize: 11, padding: "6px 12px", background: "white", borderColor: "#e4b977" }}
                onClick={() => setShowConflictModal(true)}
              >
                Tinjau & Selesaikan Konflik
              </button>
            </div>
          )}

          {/* Formulir Penambahan Fakta Baru (Typed Facts) */}
          <form className="profile-form" onSubmit={handleCreateFact} style={{ marginTop: 16 }}>
            <h4 style={{ margin: "0 0 12px", fontSize: 13, fontWeight: 800 }}>Tambah Fakta Baru</h4>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 10 }}>
              <label style={{ fontSize: 12 }}>
                Kategori
                <select
                  value={factCategory}
                  onChange={(e) => setFactCategory(e.target.value)}
                  style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                >
                  <option value="Demografi">Demografi</option>
                  <option value="Pekerjaan">Pekerjaan</option>
                  <option value="Finansial">Finansial</option>
                  <option value="Rumah Tangga">Rumah Tangga</option>
                  <option value="Transportasi">Transportasi</option>
                  <option value="Teknologi">Teknologi</option>
                  <option value="Kesehatan">Kesehatan</option>
                  <option value="Hobi & Minat">Hobi & Minat</option>
                  <option value="Lainnya">Lainnya</option>
                </select>
              </label>

              <label style={{ fontSize: 12 }}>
                Nama Fakta / Atribut
                <input
                  value={factAttribute}
                  onChange={(e) => setFactAttribute(e.target.value)}
                  placeholder="Contoh: Penghasilan Bulanan, Usia, dll"
                  maxLength={100}
                  required
                  style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                />
              </label>

              <label style={{ fontSize: 12 }}>
                Tipe Data
                <select
                  value={factDataType}
                  onChange={(e) => setFactDataType(e.target.value as DataType)}
                  style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                >
                  <option value="string">Teks (String)</option>
                  <option value="number">Angka / Finansial (Number)</option>
                  <option value="boolean">Ya / Tidak (Boolean)</option>
                  <option value="date">Tanggal (Date: YYYY-MM-DD)</option>
                  <option value="json">Struktur JSON</option>
                </select>
              </label>
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "2fr 1fr", gap: 10, marginTop: 10 }}>
              <label style={{ fontSize: 12 }}>
                Nilai Fakta
                {factDataType === "boolean" ? (
                  <select
                    value={factValue}
                    onChange={(e) => setFactValue(e.target.value)}
                    required
                    style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                  >
                    <option value="">-- Pilih Nilai --</option>
                    <option value="ya">Ya / Benar (True)</option>
                    <option value="tidak">Tidak / Salah (False)</option>
                  </select>
                ) : factDataType === "date" ? (
                  <input
                    type="date"
                    value={factValue}
                    onChange={(e) => setFactValue(e.target.value)}
                    required
                    style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                  />
                ) : (
                  <input
                    value={factValue}
                    onChange={(e) => setFactValue(e.target.value)}
                    placeholder={
                      factDataType === "number"
                        ? "Contoh: 15000000 atau 28"
                        : factDataType === "json"
                        ? '{"merk": ["Toyota", "Honda"]}'
                        : "Contoh: Karyawan Swasta"
                    }
                    required
                    style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                  />
                )}
              </label>

              <label style={{ fontSize: 12 }}>
                Satuan / Unit (Opsional)
                <input
                  value={factUnit}
                  onChange={(e) => setFactUnit(e.target.value)}
                  placeholder="Contoh: IDR, tahun, bulan, unit"
                  maxLength={30}
                  style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                />
              </label>
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 10, marginTop: 10 }}>
              <label style={{ fontSize: 12 }}>
                Status Verifikasi
                <select
                  value={newFactStatus}
                  onChange={(e) => setNewFactStatus(e.target.value as VerificationStatus)}
                  style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                >
                  <option value="unverified">Belum Diverifikasi (Default)</option>
                  <option value="confirmed">Terkonfirmasi (Pengguna)</option>
                  <option value="disputed">Diperdebatkan</option>
                  <option value="outdated">Kadaluarsa</option>
                </select>
              </label>

              <label style={{ fontSize: 12 }}>
                Klasifikasi Sensitivitas
                <select
                  value={factSensitivity}
                  onChange={(e) => setFactSensitivity(e.target.value as Sensitivity)}
                  style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                >
                  <option value="ordinary">Biasa (Non-Sensitif)</option>
                  <option value="sensitive">Sensitif (Perlu Izin AI)</option>
                  <option value="highly_sensitive">Sangat Sensitif (Ketat)</option>
                </select>
              </label>

              <label style={{ fontSize: 12 }}>
                Jenis Sumber Bukti
                <select
                  value={factSourceType}
                  onChange={(e) => setFactSourceType(e.target.value as SourceType)}
                  style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                >
                  <option value="manual_entry">Input Manual Pengguna</option>
                  <option value="user_declaration">Pernyataan Tertulis</option>
                  <option value="uploaded_document">Dokumen Resmi / Bukti</option>
                  <option value="observed_action">Observasi Terkonfirmasi</option>
                  <option value="external_record">Catatan Bukti Lain</option>
                </select>
              </label>
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 2fr", gap: 10, marginTop: 10 }}>
              <label style={{ fontSize: 12 }}>
                Judul Sumber Bukti
                <input
                  value={factSourceTitle}
                  onChange={(e) => setFactSourceTitle(e.target.value)}
                  placeholder="Contoh: KTP, Slip Gaji, dll"
                  maxLength={80}
                  style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                />
              </label>
              <label style={{ fontSize: 12 }}>
                Deskripsi Bukti / Dasar Fakta
                <input
                  value={factSourceDesc}
                  onChange={(e) => setFactSourceDesc(e.target.value)}
                  placeholder="Keterangan bagaimana fakta ini diperoleh atau diverifikasi"
                  required
                  maxLength={500}
                  style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                />
              </label>
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10, marginTop: 10 }}>
              <label style={{ fontSize: 12 }}>
                Berlaku Mulai (Opsional)
                <input
                  type="date"
                  value={factValidFrom}
                  onChange={(e) => setFactValidFrom(e.target.value)}
                  style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                />
              </label>
              <label style={{ fontSize: 12 }}>
                Berlaku Hingga (Opsional)
                <input
                  type="date"
                  value={factValidUntil}
                  onChange={(e) => setFactValidUntil(e.target.value)}
                  style={{ width: "100%", padding: 8, marginTop: 4, borderRadius: 6, border: "1px solid #dce4de" }}
                />
              </label>
            </div>

            <button
              className="primary-button"
              type="submit"
              disabled={!factAttribute.trim() || !factValue.trim() || !factSourceDesc.trim() || factLoading}
              style={{ marginTop: 14 }}
            >
              {factLoading ? "Menyimpan Fakta..." : "Simpan Fakta & Sumber"}
            </button>
          </form>

          {factError && (
            <div className="notice" role="alert" style={{ background: "#fdf2f2", color: "#a12020", marginTop: "12px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                <span aria-hidden="true">⚠️</span>
                <p style={{ margin: 0 }}>{factError}</p>
              </div>
              {onRefreshFacts && (
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => void onRefreshFacts()}
                  aria-label="Coba muat ulang fakta profil"
                  style={{ padding: "4px 10px", fontSize: "11px" }}
                >
                  🔄 Coba Lagi
                </button>
              )}
            </div>
          )}

          {/* Daftar Kartu Fakta */}
          <div style={{ marginTop: 20, display: "grid", gap: 12 }}>
            {facts.length === 0 ? (
              <div className="empty-inline" role="status">
                Belum ada fakta profil tersimpan untuk {activePersona.displayName}. Gunakan formulir di atas untuk mencatat fakta nyata responden.
              </div>
            ) : (
              facts.map((fact) => {
                const isConfirmed = fact.verificationStatus === "confirmed";
                const isUnverified = fact.verificationStatus === "unverified";
                const isDisputed = fact.verificationStatus === "disputed";

                const isOrdinary = fact.sensitivity === "ordinary";
                const isSensitive = fact.sensitivity === "sensitive";

                return (
                  <div
                    key={fact.id}
                    className={`fact-card ${fact.hasConflict ? "has-conflict" : ""}`}
                  >
                    <div className="fact-card-header">
                      <div className="fact-title-row">
                        <span style={{ fontSize: 11, fontWeight: 700, color: "var(--muted)", textTransform: "uppercase" }}>
                          {fact.category}
                        </span>
                        <strong style={{ fontSize: 14 }}>{fact.attribute}</strong>
                        <span className="badge-datatype">{fact.dataType}</span>
                        {fact.unit && <span className="badge-unit">{fact.unit}</span>}
                      </div>

                      <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
                        {/* Verification Status Badge */}
                        <span
                          className={`badge ${
                            isConfirmed
                              ? "badge-confirmed"
                              : isUnverified
                              ? "badge-unverified"
                              : isDisputed
                              ? "badge-disputed"
                              : "badge-outdated"
                          }`}
                        >
                          {isConfirmed
                            ? "Terkonfirmasi"
                            : isUnverified
                            ? "Belum Terverifikasi"
                            : isDisputed
                            ? "Diperdebatkan"
                            : "Kadaluarsa"}
                        </span>

                        {/* Sensitivity Badge */}
                        <span
                          className={`badge ${
                            isOrdinary
                              ? "badge-ordinary"
                              : isSensitive
                              ? "badge-sensitive"
                              : "badge-highly-sensitive"
                          }`}
                        >
                          {isOrdinary ? "Biasa" : isSensitive ? "Sensitif" : "Sangat Sensitif"}
                        </span>
                      </div>
                    </div>

                    <div className="fact-value-box">
                      <span>
                        {fact.dataType === "boolean"
                          ? fact.normalizedValue === "true"
                            ? "Ya / Benar"
                            : "Tidak / Salah"
                          : typeof fact.value === "string"
                          ? fact.value
                          : JSON.stringify(fact.value)}
                      </span>
                      {fact.unit && <span style={{ fontSize: 12, fontWeight: 600, color: "#54665c" }}>{fact.unit}</span>}
                    </div>

                    <div className="fact-meta-row">
                      {fact.sourceDescription && (
                        <span>Sumber: <strong>{fact.sourceDescription}</strong></span>
                      )}
                      <span>({fact.sourcesCount} sumber bukti)</span>
                      {fact.validUntil && (
                        <span style={{ color: fact.isValid ? "#3d5246" : "#b23a36" }}>
                          {fact.isValid ? `Berlaku s/d: ${fact.validUntil}` : `Sudah Expired (${fact.validUntil})`}
                        </span>
                      )}
                      {fact.hasConflict && (
                        <span style={{ color: "#a85400", fontWeight: 700 }}>⚠️ Ada pertentangan nilai</span>
                      )}
                    </div>

                    {/* Actions Row */}
                    <div className="fact-card-actions">
                      <select
                        value={fact.verificationStatus}
                        onChange={(e) => void onUpdateFactStatus(fact.id, e.target.value as VerificationStatus)}
                        style={{ fontSize: 11, padding: "5px 8px", borderRadius: 6, border: "1px solid #cfdad2" }}
                        title="Ubah status verifikasi"
                      >
                        <option value="unverified">Belum Terverifikasi</option>
                        <option value="confirmed">Terkonfirmasi</option>
                        <option value="disputed">Diperdebatkan</option>
                        <option value="outdated">Kadaluarsa</option>
                      </select>

                      <select
                        value={fact.sensitivity}
                        onChange={(e) => void onUpdateFactSensitivity(fact.id, e.target.value as Sensitivity)}
                        style={{ fontSize: 11, padding: "5px 8px", borderRadius: 6, border: "1px solid #cfdad2" }}
                        title="Ubah klasifikasi sensitivitas"
                      >
                        <option value="ordinary">Sensitivitas: Biasa</option>
                        <option value="sensitive">Sensitivitas: Sensitif</option>
                        <option value="highly_sensitive">Sensitivitas: Sangat Sensitif</option>
                      </select>

                      <button
                        type="button"
                        className="secondary-button"
                        style={{ fontSize: 11, padding: "5px 10px" }}
                        onClick={() => openEditFact(fact)}
                        aria-label={`Ubah nilai fakta ${fact.attribute}`}
                      >
                        Ubah Nilai
                      </button>

                      <button
                        type="button"
                        className="secondary-button"
                        style={{ fontSize: 11, padding: "5px 10px" }}
                        onClick={() => openFactDetail(fact.id)}
                        aria-label={`Detail dan riwayat sumber bukti fakta ${fact.attribute} (${fact.sourcesCount} sumber)`}
                      >
                        Detail & Riwayat ({fact.sourcesCount})
                      </button>

                      <button
                        type="button"
                        className="text-button"
                        style={{ color: "#8a5814" }}
                        onClick={() => void onUpdateFactStatus(fact.id, "outdated")}
                        aria-label={`Tandai fakta ${fact.attribute} sebagai kadaluarsa atau usang`}
                      >
                        Tandai Usang (Outdated)
                      </button>

                      <button
                        type="button"
                        className="danger-button"
                        style={{ marginLeft: "auto", padding: "5px 10px" }}
                        onClick={() => void onDeleteFact(fact.id)}
                        aria-label={`Hapus fakta ${fact.attribute}`}
                      >
                        Hapus
                      </button>
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>
      )}

      {/* Modal Edit Persona */}
      {editingPersona && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-edit-persona-title">
            <div className="modal-header">
              <h3 id="modal-edit-persona-title">Edit Profil Responden</h3>
              <button
                type="button"
                className="text-button"
                aria-label="Tutup dialog edit profil"
                onClick={() => setEditingPersona(null)}
              >
                ✕
              </button>
            </div>
            <form onSubmit={handleSaveEdit}>
              <div className="modal-body">
                <label>
                  Nama Profil / Responden (1 - 80 Karakter)
                  <input
                    value={editName}
                    onChange={(e) => setEditName(e.target.value)}
                    maxLength={80}
                    required
                  />
                </label>
                <label>
                  Deskripsi / Catatan Responden (Maksimal 500 Karakter)
                  <textarea
                    value={editDescription}
                    onChange={(e) => setEditDescription(e.target.value)}
                    maxLength={500}
                    rows={3}
                  />
                </label>
              </div>
              <div className="modal-footer">
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => setEditingPersona(null)}
                  disabled={editSubmitting}
                >
                  Batal
                </button>
                <button
                  type="submit"
                  className="primary-button"
                  style={{ marginTop: 0 }}
                  disabled={!editName.trim() || editSubmitting}
                >
                  {editSubmitting ? "Menyimpan..." : "Simpan Perubahan"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Modal Konfirmasi Hapus Persona */}
      {deletingPersona && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="alertdialog" aria-modal="true" aria-labelledby="modal-delete-persona-title">
            <div className="modal-header">
              <h3 id="modal-delete-persona-title" style={{ color: "#b23a36" }}>Hapus Profil Responden</h3>
              <button
                type="button"
                className="text-button"
                aria-label="Tutup dialog konfirmasi hapus"
                onClick={() => setDeletingPersona(null)}
              >
                ✕
              </button>
            </div>
            <div className="modal-body">
              <div className="danger-callout">
                <h4>Peringatan: Penghapusan Permanen</h4>
                <p>
                  Anda akan menghapus profil responden{" "}
                  <strong>{deletingPersona.displayName}</strong>. Tindakan ini tidak
                  dapat dibatalkan.
                </p>

                {deleteLoadingSummary ? (
                  <p>Memuat ringkasan data terkait...</p>
                ) : deleteSummary ? (
                  <div>
                    <span style={{ fontSize: 11, fontWeight: 700, color: "#6e2927" }}>
                      DATA TERKAIT YANG AKAN DIHAPUS:
                    </span>
                    <div className="impact-grid">
                      <div className="impact-item">
                        <span>Fakta Profil:</span>
                        <span className={deleteSummary.factsCount > 0 ? "impact-count" : "impact-zero"}>
                          {deleteSummary.factsCount}
                        </span>
                      </div>
                      <div className="impact-item">
                        <span>Sumber Bukti:</span>
                        <span className={deleteSummary.sourcesCount > 0 ? "impact-count" : "impact-zero"}>
                          {deleteSummary.sourcesCount}
                        </span>
                      </div>
                      <div className="impact-item">
                        <span>Sesi Survei:</span>
                        <span className={deleteSummary.sessionsCount > 0 ? "impact-count" : "impact-zero"}>
                          {deleteSummary.sessionsCount}
                        </span>
                      </div>
                      <div className="impact-item">
                        <span>Pertanyaan & Rekomendasi:</span>
                        <span className={deleteSummary.questionsCount > 0 ? "impact-count" : "impact-zero"}>
                          {deleteSummary.questionsCount}
                        </span>
                      </div>
                    </div>
                    <div style={{ fontSize: 11, color: "#617066", marginTop: 8 }}>
                      * Kredensial API AI dan data milik profil responden lain tidak akan terhapus.
                    </div>
                  </div>
                ) : null}
              </div>

              <label style={{ display: "flex", alignItems: "center", gap: 8, cursor: "pointer" }}>
                <input
                  type="checkbox"
                  checked={deleteConfirmed}
                  onChange={(e) => setDeleteConfirmed(e.target.checked)}
                />
                <span style={{ fontSize: 12, fontWeight: 600 }}>
                  Saya mengonfirmasi untuk menghapus seluruh data profil ini secara permanen.
                </span>
              </label>
            </div>
            <div className="modal-footer">
              <button
                type="button"
                className="secondary-button"
                onClick={() => setDeletingPersona(null)}
                disabled={deleteSubmitting}
              >
                Batal
              </button>
              <button
                type="button"
                className="danger-button"
                style={{ padding: "9px 16px" }}
                disabled={!deleteConfirmed || deleteSubmitting || deleteLoadingSummary}
                onClick={handleConfirmDelete}
              >
                {deleteSubmitting ? "Menghapus..." : "Hapus Permanen"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Modal Edit Fakta (SC-011) */}
      {editingFact && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-edit-fact-title">
            <div className="modal-header">
              <h3 id="modal-edit-fact-title">Edit Fakta: {editingFact.attribute}</h3>
              <button
                type="button"
                className="text-button"
                aria-label="Tutup dialog edit fakta"
                onClick={() => setEditingFact(null)}
              >
                ✕
              </button>
            </div>
            <form onSubmit={handleSaveFactEdit}>
              <div className="modal-body">
                <label>
                  Nilai Baru
                  <input
                    value={editFactValue}
                    onChange={(e) => setEditFactValue(e.target.value)}
                    required
                  />
                </label>
                <label>
                  Satuan / Unit (Opsional)
                  <input
                    value={editFactUnit}
                    onChange={(e) => setEditFactUnit(e.target.value)}
                    placeholder="Contoh: IDR, tahun, bulan"
                  />
                </label>
                <label>
                  Berlaku Hingga (Opsional)
                  <input
                    type="date"
                    value={editFactValidUntil}
                    onChange={(e) => setEditFactValidUntil(e.target.value)}
                  />
                </label>
                <label>
                  Alasan / Catatan Perubahan (Terekam di Riwayat)
                  <input
                    value={editFactReason}
                    onChange={(e) => setEditFactReason(e.target.value)}
                    placeholder="Contoh: Kenaikan gaji, pindah domisili, update tahun baru"
                    required
                  />
                </label>
              </div>
              <div className="modal-footer">
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => setEditingFact(null)}
                  disabled={editFactSubmitting}
                >
                  Batal
                </button>
                <button
                  type="submit"
                  className="primary-button"
                  style={{ marginTop: 0 }}
                  disabled={!editFactValue.trim() || !editFactReason.trim() || editFactSubmitting}
                >
                  {editFactSubmitting ? "Menyimpan..." : "Simpan Perubahan & Audit Trail"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Modal Detail Fakta, Sumber Bukti, dan Riwayat (SC-011) */}
      {selectedFactDetail && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-detail-fact-title" style={{ maxWidth: 650 }}>
            <div className="modal-header">
              <div>
                <h3 id="modal-detail-fact-title" style={{ margin: 0 }}>Detail Fakta: {selectedFactDetail.attribute}</h3>
                <span style={{ fontSize: 11, color: "var(--muted)" }}>
                  Kategori: {selectedFactDetail.category} | Key: {selectedFactDetail.factKey}
                </span>
              </div>
              <button
                type="button"
                className="text-button"
                aria-label="Tutup dialog detail fakta"
                onClick={() => setSelectedFactDetail(null)}
              >
                ✕
              </button>
            </div>
            <div className="modal-body" style={{ maxHeight: "70vh", overflowY: "auto" }}>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10, background: "#f8faf8", padding: 12, borderRadius: 8 }}>
                <div>
                  <span style={{ fontSize: 11, color: "var(--muted)" }}>Nilai Tersimpan:</span>
                  <div style={{ fontSize: 14, fontWeight: 800 }}>
                    {typeof selectedFactDetail.value === "string" ? selectedFactDetail.value : JSON.stringify(selectedFactDetail.value)}
                    {selectedFactDetail.unit && ` (${selectedFactDetail.unit})`}
                  </div>
                </div>
                <div>
                  <span style={{ fontSize: 11, color: "var(--muted)" }}>Nilai Normalisasi:</span>
                  <div style={{ fontSize: 12, fontFamily: "monospace", color: "#365946" }}>
                    {selectedFactDetail.normalizedValue}
                  </div>
                </div>
              </div>

              {/* Sumber Bukti Section */}
              <div>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
                  <h4 style={{ margin: 0, fontSize: 13, fontWeight: 800 }}>
                    Sumber Bukti ({selectedFactDetail.sources.length})
                  </h4>
                  <button
                    type="button"
                    className="secondary-button"
                    style={{ fontSize: 11, padding: "4px 8px" }}
                    onClick={() => {
                      setAddingSourceFactId(selectedFactDetail.id);
                      setNewSourceTitle("");
                      setNewSourceDesc("");
                    }}
                  >
                    + Tambah Sumber
                  </button>
                </div>

                <div style={{ display: "grid", gap: 8 }}>
                  {selectedFactDetail.sources.map((src) => (
                    <div key={src.id} className="source-item">
                      <h5>{src.title}</h5>
                      <div style={{ color: "#46594e", fontSize: 12 }}>{src.description}</div>
                      <div style={{ marginTop: 4, fontSize: 10, color: "var(--muted)", display: "flex", gap: 10 }}>
                        <span>Tipe: {src.sourceType}</span>
                        <span>Dibuat: {new Date(src.createdAt).toLocaleString("id-ID")}</span>
                        {src.referenceUri && <span>Ref: {src.referenceUri}</span>}
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Riwayat Perubahan (Audit Trail) */}
              <div style={{ marginTop: 8 }}>
                <h4 style={{ margin: "0 0 8px", fontSize: 13, fontWeight: 800 }}>
                  Riwayat Perubahan & Audit Trail ({selectedFactDetail.history.length})
                </h4>
                <div className="timeline-list">
                  {selectedFactDetail.history.map((hist) => (
                    <div key={hist.id} className="timeline-item">
                      <div className="timeline-date">
                        {new Date(hist.changedAt).toLocaleString("id-ID")}
                      </div>
                      <div>
                        {hist.previousValueJson ? (
                          <span>
                            Nilai: <s>{hist.previousValueJson}</s> → <strong>{hist.newValueJson}</strong>
                          </span>
                        ) : (
                          <span>Nilai awal: <strong>{hist.newValueJson}</strong></span>
                        )}
                        {hist.previousStatus && hist.previousStatus !== hist.newStatus && (
                          <span style={{ marginLeft: 8 }}>
                            (Status: {hist.previousStatus} → {hist.newStatus})
                          </span>
                        )}
                      </div>
                      {hist.changeReason && (
                        <div className="timeline-reason">Alasan: {hist.changeReason}</div>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            </div>
            <div className="modal-footer">
              <button
                type="button"
                className="secondary-button"
                onClick={() => setSelectedFactDetail(null)}
              >
                Tutup
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Modal Tambah Sumber Bukti Tambahan */}
      {addingSourceFactId && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-add-source-title">
            <div className="modal-header">
              <h3 id="modal-add-source-title">Tambah Sumber Bukti</h3>
              <button
                type="button"
                className="text-button"
                aria-label="Tutup dialog tambah sumber"
                onClick={() => setAddingSourceFactId(null)}
              >
                ✕
              </button>
            </div>
            <form onSubmit={handleAddSourceSubmit}>
              <div className="modal-body">
                <label>
                  Jenis Sumber
                  <select
                    value={newSourceType}
                    onChange={(e) => setNewSourceType(e.target.value as SourceType)}
                  >
                    <option value="uploaded_document">Dokumen Resmi / Unggahan</option>
                    <option value="user_declaration">Pernyataan Tertulis Pengguna</option>
                    <option value="manual_entry">Input Manual Pengguna</option>
                    <option value="observed_action">Observasi Terkonfirmasi</option>
                    <option value="external_record">Catatan Bukti Lain</option>
                  </select>
                </label>
                <label>
                  Judul Sumber
                  <input
                    value={newSourceTitle}
                    onChange={(e) => setNewSourceTitle(e.target.value)}
                    placeholder="Contoh: KTP, Slip Gaji Terbaru, Surat Kontrak"
                    required
                  />
                </label>
                <label>
                  Keterangan Bukti
                  <textarea
                    value={newSourceDesc}
                    onChange={(e) => setNewSourceDesc(e.target.value)}
                    placeholder="Jelaskan detail bukti atau verifikasi ini"
                    rows={2}
                    required
                  />
                </label>
                <label>
                  Referensi Dokumen / URI (Opsional)
                  <input
                    value={newSourceRef}
                    onChange={(e) => setNewSourceRef(e.target.value)}
                    placeholder="Contoh: /path/to/dokumen.pdf atau nomor dokumen"
                  />
                </label>
              </div>
              <div className="modal-footer">
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => setAddingSourceFactId(null)}
                  disabled={newSourceSubmitting}
                >
                  Batal
                </button>
                <button
                  type="submit"
                  className="primary-button"
                  style={{ marginTop: 0 }}
                  disabled={!newSourceTitle.trim() || !newSourceDesc.trim() || newSourceSubmitting}
                >
                  {newSourceSubmitting ? "Menyimpan..." : "Simpan Sumber"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Modal Tinjau & Selesaikan Konflik Fakta */}
      {showConflictModal && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-conflict-title" style={{ maxWidth: 640 }}>
            <div className="modal-header">
              <h3 id="modal-conflict-title" style={{ color: "#8a5308" }}>Resolusi Konflik Data</h3>
              <button
                type="button"
                className="text-button"
                aria-label="Tutup dialog resolusi konflik"
                onClick={() => setShowConflictModal(false)}
              >
                ✕
              </button>
            </div>
            <div className="modal-body">
              <p style={{ margin: "0 0 12px", fontSize: 13, color: "#4f3b1b" }}>
                Sistem tidak memilih fakta secara otomatis untuk mencegah jawaban yang salah. Silakan tandai fakta yang tidak lagi berlaku sebagai <strong>Kadaluarsa (Outdated)</strong> atau hapus fakta yang keliru.
              </p>

              <div style={{ display: "grid", gap: 14 }}>
                {conflicts.map((conf, idx) => (
                  <div key={idx} style={{ background: "#fffbf2", border: "1px solid #fae4be", borderRadius: 8, padding: 14 }}>
                    <div style={{ fontSize: 13, fontWeight: 800, color: "#774202", marginBottom: 6 }}>
                      Atribut: {conf.existingDisplayName} (Key: {conf.factKey})
                    </div>
                    <div style={{ fontSize: 12, color: "#543d1a", marginBottom: 12 }}>{conf.message}</div>

                    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10 }}>
                      <div style={{ background: "white", padding: 10, borderRadius: 6, border: "1px solid #ebd3ab" }}>
                        <span style={{ fontSize: 11, color: "var(--muted)" }}>Fakta A:</span>
                        <div style={{ fontSize: 13, fontWeight: 700, margin: "2px 0 6px" }}>{conf.existingValue}</div>
                        <button
                          type="button"
                          className="secondary-button"
                          style={{ fontSize: 11, padding: "4px 8px", width: "100%" }}
                          onClick={async () => {
                            await onUpdateFactStatus(conf.existingFactId, "outdated");
                            setShowConflictModal(false);
                          }}
                        >
                          Tandai Fakta A Outdated
                        </button>
                      </div>

                      <div style={{ background: "white", padding: 10, borderRadius: 6, border: "1px solid #ebd3ab" }}>
                        <span style={{ fontSize: 11, color: "var(--muted)" }}>Fakta B:</span>
                        <div style={{ fontSize: 13, fontWeight: 700, margin: "2px 0 6px" }}>{conf.conflictingValue}</div>
                        <button
                          type="button"
                          className="secondary-button"
                          style={{ fontSize: 11, padding: "4px 8px", width: "100%" }}
                          onClick={async () => {
                            await onUpdateFactStatus(conf.conflictingFactId, "outdated");
                            setShowConflictModal(false);
                          }}
                        >
                          Tandai Fakta B Outdated
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
            <div className="modal-footer">
              <button
                type="button"
                className="secondary-button"
                onClick={() => setShowConflictModal(false)}
              >
                Tutup
              </button>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}
