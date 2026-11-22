import type { Page } from "../Sidebar";
import type { Persona } from "../../types/persona";

interface HomeViewProps {
  activePersona?: Persona;
  onNavigate: (page: Page) => void;
}

export function HomeView({ activePersona, onNavigate }: HomeViewProps) {
  return (
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
        <button className="primary-button" onClick={() => onNavigate("copilot")}>
          Mulai dengan Copilot <span>→</span>
        </button>
      </div>

      <div className="quick-grid">
        <button className="quick-card" onClick={() => onNavigate("copilot")}>
          <span className="quick-icon">▣</span>
          <strong>Baca dari layar</strong>
          <span>Ambil screenshot pertanyaan survei</span>
        </button>
        <button className="quick-card" onClick={() => onNavigate("copilot")}>
          <span className="quick-icon">⌁</span>
          <strong>Tempel pertanyaan</strong>
          <span>Masukkan teks secara manual</span>
        </button>
        <button className="quick-card" onClick={() => onNavigate("profiles")}>
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
            <button
              className="inline-action"
              onClick={() => onNavigate("profiles")}
            >
              Buat profil
            </button>
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
  );
}
