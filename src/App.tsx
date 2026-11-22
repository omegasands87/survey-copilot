import { useState } from "react";

type Page = "home" | "profiles" | "copilot" | "history" | "settings";

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
          <span>Data tersimpan lokal</span>
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
          <div className="persona-pill" title="Pemilihan profil aktif akan tersedia setelah modul profil dibuat">
            <span className="avatar">P</span>
            Profil belum dibuat
          </div>
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

            <div className="notice">
              <span>i</span>
              <p>
                Rekomendasi AI bukan jawaban yang dikirim otomatis. Kamu tetap
                memeriksa, mengisi, dan mengirim survei sendiri.
              </p>
            </div>
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
              <button className="primary-button" disabled={!question.trim()}>
                Analisis pertanyaan
              </button>
            </div>
            <p className="helper-text">
              Pastikan teks dan pilihan jawaban sudah benar sebelum analisis dilakukan.
            </p>
          </section>
        )}

        {page === "profiles" && (
          <section className="content empty-state">
            <div className="empty-icon">◉</div>
            <h2>Profil responden</h2>
            <p>Modul untuk membuat beberapa profil responden yang terpisah sedang disiapkan.</p>
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
