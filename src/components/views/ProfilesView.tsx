import { useState, type FormEvent } from "react";
import type { Persona } from "../../types/persona";
import type { ProfileFact, VerificationStatus, Sensitivity } from "../../types/fact";

interface ProfilesViewProps {
  personas: Persona[];
  activePersona?: Persona;
  facts: ProfileFact[];
  profileError: string;
  profileLoading: boolean;
  factError: string;
  factLoading: boolean;
  onCreatePersona: (displayName: string) => Promise<void>;
  onActivatePersona: (personaId: string) => Promise<void>;
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

export function ProfilesView({
  personas,
  activePersona,
  facts,
  profileError,
  profileLoading,
  factError,
  factLoading,
  onCreatePersona,
  onActivatePersona,
  onCreateFact,
  onUpdateFactStatus,
  onDeleteFact,
}: ProfilesViewProps) {
  const [profileName, setProfileName] = useState("");
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
    await onCreatePersona(name);
    setProfileName("");
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

  return (
    <section className="content profile-content">
      <div className="section-heading">
        <div>
          <span className="eyebrow">DIGITAL PERSONA</span>
          <h2>Profil responden</h2>
        </div>
        <span className="coming-soon">Data disimpan di perangkat ini</span>
      </div>

      <form className="profile-form" onSubmit={handleCreatePersona}>
        <label className="field-label" htmlFor="profile-name">
          Nama profil baru
        </label>
        <div className="profile-form-row">
          <input
            id="profile-name"
            value={profileName}
            onChange={(event) => setProfileName(event.target.value)}
            maxLength={80}
            placeholder="Contoh: Profil Saya"
          />
          <button
            className="primary-button"
            type="submit"
            disabled={!profileName.trim() || profileLoading}
          >
            {profileLoading ? "Menyimpan..." : "Tambah profil"}
          </button>
        </div>
      </form>
      {profileError && <p className="error-message">{profileError}</p>}

      <div className="persona-list">
        {personas.length === 0 ? (
          <div className="empty-inline">Belum ada profil. Buat profil pertama di atas.</div>
        ) : (
          personas.map((persona) => (
            <div className="persona-card" key={persona.id}>
              <div>
                <strong>{persona.displayName}</strong>
                <span>{persona.isActive ? "Profil aktif" : "Profil tersimpan"}</span>
              </div>
              {persona.isActive ? (
                <span className="active-label">Aktif</span>
              ) : (
                <button
                  className="secondary-button"
                  onClick={() => void onActivatePersona(persona.id)}
                >
                  Gunakan profil
                </button>
              )}
            </div>
          ))
        )}
      </div>

      {activePersona && (
        <div className="facts-section">
          <div className="facts-heading">
            <div>
              <span className="eyebrow">DATA RESPONDEN</span>
              <h3>Informasi {activePersona.displayName}</h3>
            </div>
          </div>

          <form className="fact-form" onSubmit={handleCreateFact}>
            <div className="fact-form-grid">
              <label>
                Kelompok
                <input
                  value={factCategory}
                  onChange={(event) => setFactCategory(event.target.value)}
                  maxLength={80}
                  required
                />
              </label>
              <label>
                Informasi
                <input
                  value={factAttribute}
                  onChange={(event) => setFactAttribute(event.target.value)}
                  placeholder="Contoh: Pekerjaan"
                  maxLength={100}
                  required
                />
              </label>
              <label className="fact-value-field">
                Nilai
                <input
                  value={factValue}
                  onChange={(event) => setFactValue(event.target.value)}
                  placeholder="Masukkan fakta yang benar"
                  maxLength={4000}
                  required
                />
              </label>
              <label>
                Sumber
                <input
                  value={factSource}
                  onChange={(event) => setFactSource(event.target.value)}
                  maxLength={500}
                  required
                />
              </label>
              <label>
                Status
                <select
                  value={newFactVerificationStatus}
                  onChange={(event) =>
                    setNewFactVerificationStatus(
                      event.target.value as VerificationStatus
                    )
                  }
                >
                  <option value="unverified">Belum diverifikasi</option>
                  <option value="confirmed">Sudah saya pastikan</option>
                </select>
              </label>
              <label>
                Sensitivitas
                <select
                  value={factSensitivity}
                  onChange={(event) =>
                    setFactSensitivity(event.target.value as Sensitivity)
                  }
                >
                  <option value="ordinary">Biasa</option>
                  <option value="sensitive">Sensitif</option>
                  <option value="highly_sensitive">Sangat sensitif</option>
                </select>
              </label>
            </div>
            <button
              className="primary-button"
              type="submit"
              disabled={factLoading || !factAttribute.trim() || !factValue.trim()}
            >
              {factLoading ? "Menyimpan..." : "Simpan informasi"}
            </button>
          </form>

          {factError && <p className="error-message">{factError}</p>}

          <div className="fact-list">
            {facts.length === 0 ? (
              <div className="empty-inline">Belum ada informasi untuk profil ini.</div>
            ) : (
              facts.map((fact) => (
                <article className="fact-card" key={fact.id}>
                  <div className="fact-card-main">
                    <span className="fact-category">{fact.category}</span>
                    <strong>{fact.attribute}</strong>
                    <p>{String(fact.value)}</p>
                    <small>
                      Sumber: {fact.sourceDescription || "Tidak dicatat"} ·{" "}
                      {fact.sensitivity === "ordinary" ? "Biasa" : "Sensitif"}
                    </small>
                  </div>
                  <div className="fact-card-actions">
                    <span
                      className={
                        fact.verificationStatus === "confirmed"
                          ? "active-label"
                          : "fact-unverified"
                      }
                    >
                      {fact.verificationStatus === "confirmed"
                        ? "Terverifikasi"
                        : fact.verificationStatus === "unverified"
                        ? "Belum diverifikasi"
                        : fact.verificationStatus}
                    </span>
                    {fact.verificationStatus !== "confirmed" && (
                      <button
                        className="secondary-button"
                        onClick={() =>
                          void onUpdateFactStatus(fact.id, "confirmed")
                        }
                      >
                        Verifikasi
                      </button>
                    )}
                    <button
                      className="text-danger"
                      onClick={() => void onDeleteFact(fact.id)}
                    >
                      Hapus
                    </button>
                  </div>
                </article>
              ))
            )}
          </div>
        </div>
      )}
    </section>
  );
}
