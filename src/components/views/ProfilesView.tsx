import { useState, type FormEvent } from "react";
import type { Persona, PersonaDependencySummary } from "../../types/persona";
import type { ProfileFact, VerificationStatus, Sensitivity } from "../../types/fact";

interface ProfilesViewProps {
  personas: Persona[];
  activePersona?: Persona;
  facts: ProfileFact[];
  profileError: string;
  profileLoading: boolean;
  factError: string;
  factLoading: boolean;
  onCreatePersona: (displayName: string, description?: string) => Promise<void>;
  onUpdatePersona: (personaId: string, displayName: string, description?: string) => Promise<void>;
  onActivatePersona: (personaId: string) => Promise<void>;
  onArchivePersona: (personaId: string) => Promise<void>;
  onRestorePersona: (personaId: string) => Promise<void>;
  onGetDependencySummary: (personaId: string) => Promise<PersonaDependencySummary>;
  onDeletePersona: (personaId: string) => Promise<void>;
  onCreateFact: (factData: {
    category: string;
    attribute: string;
    value: string;
    sourceDescription: string;
    verificationStatus: VerificationStatus;
    sensitivity: Sensitivity;
  }) => Promise<void>;
  onUpdateFactStatus: (factId: string, status: VerificationStatus) => Promise<void>;
  onDeleteFact: (factId: string) => Promise<void>;
}

type FilterStatus = "all" | "active" | "archived";

export function ProfilesView({
  personas,
  activePersona,
  facts,
  profileError,
  profileLoading,
  factError,
  factLoading,
  onCreatePersona,
  onUpdatePersona,
  onActivatePersona,
  onArchivePersona,
  onRestorePersona,
  onGetDependencySummary,
  onDeletePersona,
  onCreateFact,
  onUpdateFactStatus,
  onDeleteFact,
}: ProfilesViewProps) {
  // New persona form
  const [profileName, setProfileName] = useState("");
  const [profileDescription, setProfileDescription] = useState("");

  // Filter tab
  const [filterTab, setFilterTab] = useState<FilterStatus>("active");

  // Edit modal state
  const [editingPersona, setEditingPersona] = useState<Persona | null>(null);
  const [editName, setEditName] = useState("");
  const [editDescription, setEditDescription] = useState("");
  const [editSubmitting, setEditSubmitting] = useState(false);

  // Delete modal state
  const [deletingPersona, setDeletingPersona] = useState<Persona | null>(null);
  const [deleteSummary, setDeleteSummary] = useState<PersonaDependencySummary | null>(null);
  const [deleteLoadingSummary, setDeleteLoadingSummary] = useState(false);
  const [deleteConfirmed, setDeleteConfirmed] = useState(false);
  const [deleteSubmitting, setDeleteSubmitting] = useState(false);

  // Fact form state
  const [factCategory, setFactCategory] = useState("Umum");
  const [factAttribute, setFactAttribute] = useState("");
  const [factValue, setFactValue] = useState("");
  const [factSource, setFactSource] = useState("Dimasukkan oleh pengguna");
  const [newFactVerificationStatus, setNewFactVerificationStatus] =
    useState<VerificationStatus>("unverified");
  const [factSensitivity, setFactSensitivity] =
    useState<Sensitivity>("ordinary");

  async function handleCreatePersona(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const name = profileName.trim();
    if (!name) return;
    await onCreatePersona(name, profileDescription.trim() || undefined);
    setProfileName("");
    setProfileDescription("");
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
      await onDeletePersona(deletingPersona.id);
      setDeletingPersona(null);
    } finally {
      setDeleteSubmitting(false);
    }
  }

  async function handleCreateFact(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!activePersona) return;
    await onCreateFact({
      category: factCategory,
      attribute: factAttribute,
      value: factValue,
      sourceDescription: factSource,
      verificationStatus: newFactVerificationStatus,
      sensitivity: factSensitivity,
    });
    setFactAttribute("");
    setFactValue("");
  }

  const filteredPersonas = personas.filter((p) => {
    if (filterTab === "active") return p.status === "active";
    if (filterTab === "archived") return p.status === "archived";
    return true;
  });

  const activeCount = personas.filter((p) => p.status === "active").length;
  const archivedCount = personas.filter((p) => p.status === "archived").length;

  return (
    <section className="content profile-content">
      <div className="section-heading">
        <div>
          <span className="eyebrow">DIGITAL PERSONA</span>
          <h2>Profil Responden</h2>
        </div>
        <span className="coming-soon">Data disimpan lokal & terisolasi di perangkat ini</span>
      </div>

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
            placeholder="Nama lengkap atau panggilan responden asli (contoh: Budi Santoso)"
            required
          />
          <button
            className="primary-button"
            type="submit"
            disabled={!profileName.trim() || profileLoading}
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
      {profileError && <p className="error-message" style={{ marginTop: 8 }}>{profileError}</p>}

      {/* Filter Tabs */}
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
                    >
                      Gunakan profil
                    </button>
                  )}

                  {!isArchived && (
                    <button
                      className="text-button"
                      onClick={() => openEditModal(persona)}
                    >
                      Edit
                    </button>
                  )}

                  {isArchived ? (
                    <button
                      className="secondary-button"
                      onClick={() => void onRestorePersona(persona.id)}
                    >
                      Pulihkan
                    </button>
                  ) : (
                    <button
                      className="text-button"
                      onClick={() => void onArchivePersona(persona.id)}
                      title="Arsipkan profil ini agar tidak aktif dalam seleksi survei"
                    >
                      Arsipkan
                    </button>
                  )}

                  <button
                    className="danger-button"
                    onClick={() => void openDeleteModal(persona)}
                  >
                    Hapus
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Modal Edit Persona */}
      {editingPersona && (
        <div className="modal-backdrop">
          <div className="modal-dialog">
            <form onSubmit={handleSaveEdit}>
              <div className="modal-header">
                <h3>Edit Profil Responden</h3>
                <button
                  type="button"
                  className="text-button"
                  onClick={() => setEditingPersona(null)}
                >
                  ✕
                </button>
              </div>
              <div className="modal-body">
                <label>
                  Nama Profil (1-80 Karakter)
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

      {/* Modal Konfirmasi Hapus Persona dengan Dependency Summary */}
      {deletingPersona && (
        <div className="modal-backdrop">
          <div className="modal-dialog">
            <div className="modal-header">
              <h3 style={{ color: "#b23a36" }}>Hapus Profil Responden</h3>
              <button
                type="button"
                className="text-button"
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

      {/* Bagian Fakta / Informasi Profil Aktif */}
      {activePersona && activePersona.status === "active" && (
        <div className="facts-section" style={{ marginTop: 32 }}>
          <div className="facts-heading">
            <div>
              <span className="eyebrow">DATA RESPONDEN</span>
              <h3 style={{ margin: "4px 0 0" }}>Informasi: {activePersona.displayName}</h3>
            </div>
          </div>

          <form className="fact-form" onSubmit={handleCreateFact} style={{ marginTop: 14 }}>
            <div className="fact-form-grid" style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10 }}>
              <label style={{ fontSize: 12 }}>
                Kelompok
                <input
                  value={factCategory}
                  onChange={(event) => setFactCategory(event.target.value)}
                  maxLength={80}
                  required
                  style={{ width: "100%", padding: 8, marginTop: 4 }}
                />
              </label>
              <label style={{ fontSize: 12 }}>
                Informasi / Atribut
                <input
                  value={factAttribute}
                  onChange={(event) => setFactAttribute(event.target.value)}
                  placeholder="Contoh: Pekerjaan"
                  maxLength={80}
                  required
                  style={{ width: "100%", padding: 8, marginTop: 4 }}
                />
              </label>
            </div>

            <label style={{ display: "block", marginTop: 10, fontSize: 12 }}>
              Nilai Jawaban
              <input
                value={factValue}
                onChange={(event) => setFactValue(event.target.value)}
                placeholder="Contoh: Karyawan Swasta"
                required
                style={{ width: "100%", padding: 8, marginTop: 4 }}
              />
            </label>

            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10, marginTop: 10 }}>
              <label style={{ fontSize: 12 }}>
                Status Verifikasi
                <select
                  value={newFactVerificationStatus}
                  onChange={(event) =>
                    setNewFactVerificationStatus(
                      event.target.value as VerificationStatus
                    )
                  }
                  style={{ width: "100%", padding: 8, marginTop: 4 }}
                >
                  <option value="unverified">Belum diverifikasi</option>
                  <option value="confirmed">Terkonfirmasi</option>
                  <option value="disputed">Diperdebatkan</option>
                  <option value="outdated">Kadaluarsa</option>
                </select>
              </label>

              <label style={{ fontSize: 12 }}>
                Klasifikasi Sensitivitas
                <select
                  value={factSensitivity}
                  onChange={(event) =>
                    setFactSensitivity(event.target.value as Sensitivity)
                  }
                  style={{ width: "100%", padding: 8, marginTop: 4 }}
                >
                  <option value="ordinary">Biasa</option>
                  <option value="sensitive">Sensitif</option>
                  <option value="highly_sensitive">Sangat sensitif</option>
                </select>
              </label>
            </div>

            <label style={{ display: "block", marginTop: 10, fontSize: 12 }}>
              Keterangan Sumber Bukti
              <input
                value={factSource}
                onChange={(event) => setFactSource(event.target.value)}
                placeholder="Contoh: Dokumen KTP, Slip Gaji, Konfirmasi Mandiri"
                style={{ width: "100%", padding: 8, marginTop: 4 }}
              />
            </label>

            <button
              className="primary-button"
              type="submit"
              disabled={
                !factAttribute.trim() ||
                !factValue.trim() ||
                factLoading
              }
              style={{ marginTop: 12 }}
            >
              {factLoading ? "Menyimpan..." : "Simpan informasi"}
            </button>
          </form>

          {factError && <p className="error-message" style={{ marginTop: 8 }}>{factError}</p>}

          <div className="fact-list" style={{ marginTop: 16, display: "grid", gap: 8 }}>
            {facts.length === 0 ? (
              <div className="empty-inline">
                Belum ada informasi responden untuk profil ini. Tambahkan fakta pertama Anda.
              </div>
            ) : (
              facts.map((fact) => (
                <div
                  className="fact-item"
                  key={fact.id}
                  style={{
                    padding: "12px 16px",
                    background: "white",
                    border: "1px solid var(--line)",
                    borderRadius: 8,
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                  }}
                >
                  <div>
                    <span className="fact-category" style={{ fontSize: 10, color: "var(--muted)", textTransform: "uppercase" }}>
                      {fact.category}
                    </span>
                    <strong style={{ display: "block", fontSize: 13, marginTop: 2 }}>{fact.attribute}</strong>
                    <div className="fact-value" style={{ fontSize: 12, color: "#223028", marginTop: 2 }}>
                      {typeof fact.value === "string"
                        ? fact.value
                        : JSON.stringify(fact.value)}
                    </div>
                  </div>

                  <div className="fact-actions" style={{ display: "flex", gap: 8, alignItems: "center" }}>
                    <select
                      value={fact.verificationStatus}
                      onChange={(event) =>
                        void onUpdateFactStatus(
                          fact.id,
                          event.target.value as VerificationStatus
                        )
                      }
                      style={{ fontSize: 11, padding: "4px 8px", borderRadius: 6 }}
                    >
                      <option value="unverified">Belum terverifikasi</option>
                      <option value="confirmed">Terkonfirmasi</option>
                      <option value="disputed">Diperdebatkan</option>
                      <option value="outdated">Kadaluarsa</option>
                    </select>

                    <button
                      className="danger-button"
                      onClick={() => void onDeleteFact(fact.id)}
                    >
                      Hapus
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      )}
    </section>
  );
}
