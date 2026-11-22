import { useEffect, useState, type FormEvent } from "react";
import { invoke } from "@tauri-apps/api/core";

type Page = "home" | "profiles" | "copilot" | "history" | "settings";

type Persona = {
  id: string;
  displayName: string;
  isActive: boolean;
  status: string;
};

type ProfileFact = {
  id: string;
  category: string;
  attribute: string;
  value: string;
  verificationStatus: string;
  sensitivity: string;
  sourceDescription: string | null;
};

const pageTitles: Record<Page, string> = {
  home: "Beranda",
  profiles: "Profil",
  copilot: "Copilot",
  history: "Riwayat",
  settings: "Pengaturan",
};

const navigation: { id: Page; label: string; icon: string }[] = [
  { id: "home", label: "Beranda", icon: "⌂" },
  { id: "profiles", label: "Profil", icon: "◉" },
  { id: "copilot", label: "Copilot", icon: "✳" },
  { id: "history", label: "Riwayat", icon: "◷" },
];

export default function App() {
  const [page, setPage] = useState<Page>("home");
  const [question, setQuestion] = useState("");
  const [personas, setPersonas] = useState<Persona[]>([]);
  const [profileName, setProfileName] = useState("");
  const [profileError, setProfileError] = useState("");
  const [profileLoading, setProfileLoading] = useState(false);
  const [facts, setFacts] = useState<ProfileFact[]>([]);
  const [factCategory, setFactCategory] = useState("Umum");
  const [factAttribute, setFactAttribute] = useState("");
  const [factValue, setFactValue] = useState("");
  const [factSource, setFactSource] = useState("Dimasukkan oleh pengguna");
  const [newFactVerificationStatus, setNewFactVerificationStatus] = useState("unverified");
  const [factSensitivity, setFactSensitivity] = useState("ordinary");
  const [factError, setFactError] = useState("");
  const [factLoading, setFactLoading] = useState(false);

  async function refreshPersonas() {
    try {
      const result = await invoke<Persona[]>("persona_list");
      setPersonas(result);
    } catch (error) {
      setProfileError(String(error));
    }
  }

  useEffect(() => {
    void refreshPersonas();
  }, []);

  const activePersona = personas.find((persona) => persona.isActive);

  async function refreshFacts(personaId: string) {
    try {
      const result = await invoke<ProfileFact[]>("fact_list", { personaId });
      setFacts(result);
      setFactError("");
    } catch (error) {
      setFactError(String(error));
    }
  }

  useEffect(() => {
    if (activePersona) {
      void refreshFacts(activePersona.id);
    } else {
      setFacts([]);
    }
  }, [activePersona?.id]);

  async function createPersona(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const name = profileName.trim();
    if (!name) return;
    setProfileLoading(true);
    setProfileError("");
    try {
      await invoke<Persona>("persona_create", { displayName: name });
      setProfileName("");
      await refreshPersonas();
    } catch (error) {
      setProfileError(String(error));
    } finally {
      setProfileLoading(false);
    }
  }

  async function activatePersona(personaId: string) {
    setProfileError("");
    try {
      await invoke<Persona>("persona_set_active", { personaId });
      await refreshPersonas();
    } catch (error) {
      setProfileError(String(error));
    }
  }

  async function createFact(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!activePersona) return;
    setFactLoading(true);
    setFactError("");
    try {
      await invoke<ProfileFact>("fact_create", {
        personaId: activePersona.id,
        category: factCategory,
        attribute: factAttribute,
        value: factValue,
        sourceDescription: factSource,
        verificationStatus: newFactVerificationStatus,
        sensitivity: factSensitivity,
      });
      setFactAttribute("");
      setFactValue("");
      await refreshFacts(activePersona.id);
    } catch (error) {
      setFactError(String(error));
    } finally {
      setFactLoading(false);
    }
  }

  async function updateFactVerificationStatus(factId: string, status: string) {
    if (!activePersona) return;
    try {
      await invoke("fact_set_status", { personaId: activePersona.id, factId, status });
      await refreshFacts(activePersona.id);
    } catch (error) {
      setFactError(String(error));
    }
  }

  async function deleteFact(factId: string) {
    if (!activePersona || !window.confirm("Hapus informasi profil ini?")) return;
    try {
      await invoke("fact_delete", { personaId: activePersona.id, factId });
      await refreshFacts(activePersona.id);
    } catch (error) {
      setFactError(String(error));
    }
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">S</div>
          <div>
            <strong>Survey Copilot</strong>
            <span>Asisten survei AI</span>
          </div>
        </div>

        <nav aria-label="Navigasi utama">
          {navigation.map((item) => (
            <button
              key={item.id}
              className={page === item.id ? "nav-item active" : "nav-item"}
              onClick={() => setPage(item.id)}
            >
              <span className="nav-icon">{item.icon}</span>
              {item.label}
            </button>
          ))}
        </nav>

        <div className="sidebar-bottom">
          <span className="status-dot" />
          <span>Mode lokal</span>
          <button className="settings-link" onClick={() => setPage("settings")}>
            Pengaturan
          </button>
        </div>
      </aside>

      <main className="main-area">
        <header className="topbar">
          <div>
            <span className="eyebrow">SURVEY COPILOT</span>
            <h1>{pageTitles[page]}</h1>
          </div>
          <select
            className="persona-select"
            aria-label="Profil responden aktif"
            value={activePersona?.id ?? ""}
            disabled={personas.length === 0}
            onChange={(event) => void activatePersona(event.target.value)}
          >
            {personas.length === 0 && <option value="">Buat profil dahulu</option>}
            {personas.map((persona) => (
              <option key={persona.id} value={persona.id}>{persona.displayName}</option>
            ))}
          </select>
        </header>

        {page === "home" && (
          <section className="content home-content">
            <div className="welcome">
              <span className="eyebrow">ASISTEN SURVEI PRIBADI</span>
              <h2>
                Pahami pertanyaan.
                <br />
                <em>Temukan jawaban yang tepat.</em>
              </h2>
              <p>
                Ambil teks dari layar atau tempel pertanyaan. AI akan membantu
                memahami dan menyusun rekomendasi berdasarkan profil yang kamu pilih.
              </p>
              <button className="primary-button" onClick={() => setPage("copilot")}>
                Mulai dengan Copilot <span>→</span>
              </button>
            </div>

            <div className="quick-grid">
              <button className="quick-card" onClick={() => setPage("copilot")}>
                <span className="quick-icon">▣</span>
                <strong>Baca dari layar</strong>
                <span>Ambil screenshot pertanyaan survei</span>
              </button>
              <button className="quick-card" onClick={() => setPage("copilot")}>
                <span className="quick-icon">⌁</span>
                <strong>Tempel pertanyaan</strong>
                <span>Masukkan teks secara manual</span>
              </button>
              <button className="quick-card" onClick={() => setPage("profiles")}>
                <span className="quick-icon">◉</span>
                <strong>Kelola profil</strong>
                <span>Pisahkan informasi setiap responden</span>
              </button>
            </div>

            {!activePersona && (
              <div className="notice">
                <span>!</span>
                <p>
                  Buat profil responden terlebih dahulu agar rekomendasi AI dapat
                  menggunakan informasi yang benar.
                  <button className="inline-action" onClick={() => setPage("profiles")}>Buat profil</button>
                </p>
              </div>
            )}
            {activePersona && (
              <div className="notice">
                <span>i</span>
                <p>
                  Profil aktif: <strong>{activePersona.displayName}</strong>. AI hanya
                  akan menggunakan informasi dari profil ini.
                </p>
              </div>
            )}
          </section>
        )}

        {page === "copilot" && (
          <section className="content workspace">
            <div className="section-heading">
              <div>
                <span className="eyebrow">WORKSPACE</span>
                <h2>Masukkan pertanyaan survei</h2>
              </div>
              <span className="coming-soon">Modul analisis sedang disiapkan</span>
            </div>
            <label className="field-label" htmlFor="question">Teks pertanyaan</label>
            <textarea
              id="question"
              value={question}
              onChange={(event) => setQuestion(event.target.value)}
              placeholder="Tempel pertanyaan survei di sini..."
            />
            <div className="workspace-actions">
              <button className="secondary-button" disabled>Ambil screenshot</button>
              <button className="primary-button" disabled={!question.trim() || !activePersona}>
                Analisis pertanyaan
              </button>
            </div>
            <p className="helper-text">
              Pastikan teks dan pilihan jawaban sudah benar sebelum analisis dilakukan.
            </p>
          </section>
        )}

        {page === "profiles" && (
          <section className="content profile-content">
            <div className="section-heading">
              <div>
                <span className="eyebrow">DIGITAL PERSONA</span>
                <h2>Profil responden</h2>
              </div>
              <span className="coming-soon">Data disimpan di perangkat ini</span>
            </div>
            <form className="profile-form" onSubmit={createPersona}>
              <label className="field-label" htmlFor="profile-name">Nama profil baru</label>
              <div className="profile-form-row">
                <input
                  id="profile-name"
                  value={profileName}
                  onChange={(event) => setProfileName(event.target.value)}
                  maxLength={80}
                  placeholder="Contoh: Profil Saya"
                />
                <button className="primary-button" type="submit" disabled={!profileName.trim() || profileLoading}>
                  {profileLoading ? "Menyimpan..." : "Tambah profil"}
                </button>
              </div>
            </form>
            {profileError && <p className="error-message">{profileError}</p>}
            <div className="persona-list">
              {personas.length === 0 ? (
                <div className="empty-inline">Belum ada profil. Buat profil pertama di atas.</div>
              ) : personas.map((persona) => (
                <div className="persona-card" key={persona.id}>
                  <div>
                    <strong>{persona.displayName}</strong>
                    <span>{persona.isActive ? "Profil aktif" : "Profil tersimpan"}</span>
                  </div>
                  {persona.isActive ? (
                    <span className="active-label">Aktif</span>
                  ) : (
                    <button className="secondary-button" onClick={() => void activatePersona(persona.id)}>
                      Gunakan profil
                    </button>
                  )}
                </div>
              ))}
            </div>

            {activePersona && (
              <div className="facts-section">
                <div className="facts-heading">
                  <div>
                    <span className="eyebrow">DATA RESPONDEN</span>
                    <h3>Informasi {activePersona.displayName}</h3>
                  </div>
                </div>
                <form className="fact-form" onSubmit={createFact}>
                  <div className="fact-form-grid">
                    <label>Kelompok
                      <input value={factCategory} onChange={(event) => setFactCategory(event.target.value)} maxLength={80} required />
                    </label>
                    <label>Informasi
                      <input value={factAttribute} onChange={(event) => setFactAttribute(event.target.value)} placeholder="Contoh: Pekerjaan" maxLength={100} required />
                    </label>
                    <label className="fact-value-field">Nilai
                      <input value={factValue} onChange={(event) => setFactValue(event.target.value)} placeholder="Masukkan fakta yang benar" maxLength={4000} required />
                    </label>
                    <label>Sumber
                      <input value={factSource} onChange={(event) => setFactSource(event.target.value)} maxLength={500} required />
                    </label>
                    <label>Status
                      <select value={newFactVerificationStatus} onChange={(event) => setNewFactVerificationStatus(event.target.value)}>
                        <option value="unverified">Belum diverifikasi</option>
                        <option value="confirmed">Sudah saya pastikan</option>
                      </select>
                    </label>
                    <label>Sensitivitas
                      <select value={factSensitivity} onChange={(event) => setFactSensitivity(event.target.value)}>
                        <option value="ordinary">Biasa</option>
                        <option value="sensitive">Sensitif</option>
                        <option value="highly_sensitive">Sangat sensitif</option>
                      </select>
                    </label>
                  </div>
                  <button className="primary-button" type="submit" disabled={factLoading || !factAttribute.trim() || !factValue.trim()}>
                    {factLoading ? "Menyimpan..." : "Simpan informasi"}
                  </button>
                </form>
                {factError && <p className="error-message">{factError}</p>}
                <div className="fact-list">
                  {facts.length === 0 ? (
                    <div className="empty-inline">Belum ada informasi untuk profil ini.</div>
                  ) : facts.map((fact) => (
                    <article className="fact-card" key={fact.id}>
                      <div className="fact-card-main">
                        <span className="fact-category">{fact.category}</span>
                        <strong>{fact.attribute}</strong>
                        <p>{String(fact.value)}</p>
                        <small>Sumber: {fact.sourceDescription || "Tidak dicatat"} · {fact.sensitivity === "ordinary" ? "Biasa" : "Sensitif"}</small>
                      </div>
                      <div className="fact-card-actions">
                        <span className={fact.verificationStatus === "confirmed" ? "active-label" : "fact-unverified"}>
                          {fact.verificationStatus === "confirmed" ? "Terverifikasi" : fact.verificationStatus === "unverified" ? "Belum diverifikasi" : fact.verificationStatus}
                        </span>
                        {fact.verificationStatus !== "confirmed" && (
                          <button className="secondary-button" onClick={() => void updateFactVerificationStatus(fact.id, "confirmed")}>Verifikasi</button>
                        )}
                        <button className="text-danger" onClick={() => void deleteFact(fact.id)}>Hapus</button>
                      </div>
                    </article>
                  ))}
                </div>
              </div>
            )}
          </section>
        )}

        {page === "history" && (
          <section className="content empty-state">
            <div className="empty-icon">◷</div>
            <h2>Riwayat analisis</h2>
            <p>Pertanyaan dan rekomendasi yang kamu tinjau akan muncul di sini setelah modul riwayat tersedia.</p>
          </section>
        )}

        {page === "settings" && (
          <section className="content empty-state">
            <div className="empty-icon">⚙</div>
            <h2>Pengaturan</h2>
            <p>Pengaturan Gemini API, pool key, privasi, dan penyimpanan akan tersedia setelah modul konfigurasi dibuat.</p>
          </section>
        )}
      </main>
    </div>
  );
}
