import type { Page } from "../Sidebar";
import type { Persona } from "../../types/persona";

interface HomeViewProps {
  activePersona?: Persona;
  onNavigate: (page: Page) => void;
}

export function HomeView({ activePersona, onNavigate }: HomeViewProps) {
  return (
    <section className="content home-content" role="region" aria-label="Beranda Survey Copilot">
      <div className="welcome">
        <span className="eyebrow">ASISTEN SURVEI PRIBADI</span>
        <h2>
          Pahami pertanyaan.
          <br />
          <em>Temukan jawaban yang tepat.</em>
        </h2>
        <p>
          Ambil teks dari layar atau tempel pertanyaan. AI akan membantu
          memahami dan menyusun rekomendasi berdasarkan profil yang kamu pilih secara jujur dan berbasis bukti.
        </p>
        <button
          className="primary-button"
          onClick={() => onNavigate("copilot")}
          aria-label="Mulai bekerja dengan Copilot Survei"
        >
          Mulai dengan Copilot <span aria-hidden="true">→</span>
        </button>
      </div>

      <div className="quick-grid" role="group" aria-label="Aksi Cepat">
        <button
          className="quick-card"
          onClick={() => onNavigate("copilot")}
          aria-label="Baca dari layar: Ambil tangkapan layar pertanyaan survei"
        >
          <span className="quick-icon" aria-hidden="true">▣</span>
          <strong>Baca dari layar</strong>
          <span>Ambil screenshot pertanyaan survei</span>
        </button>
        <button
          className="quick-card"
          onClick={() => onNavigate("copilot")}
          aria-label="Tempel pertanyaan: Masukkan teks pertanyaan secara manual"
        >
          <span className="quick-icon" aria-hidden="true">⌁</span>
          <strong>Tempel pertanyaan</strong>
          <span>Masukkan teks secara manual</span>
        </button>
        <button
          className="quick-card"
          onClick={() => onNavigate("profiles")}
          aria-label="Kelola profil responden: Atur profil dan fakta responden"
        >
          <span className="quick-icon" aria-hidden="true">◉</span>
          <strong>Kelola profil</strong>
          <span>Pisahkan informasi setiap responden</span>
        </button>
      </div>

      {!activePersona && (
        <div className="notice" role="alert" aria-live="polite">
          <span aria-hidden="true">!</span>
          <p>
            Buat profil responden terlebih dahulu agar rekomendasi AI dapat
            menggunakan informasi yang benar.
            <button
              className="inline-action"
              onClick={() => onNavigate("profiles")}
              aria-label="Buka halaman profil untuk membuat profil baru"
            >
              Buat profil sekarang
            </button>
          </p>
        </div>
      )}
      {activePersona && (
        <div className="notice" role="status" aria-live="polite">
          <span aria-hidden="true">i</span>
          <p>
            Profil responden aktif: <strong>{activePersona.displayName}</strong>. Rekomendasi AI dan data survei diisolasi ketat hanya menggunakan fakta profil ini.
          </p>
        </div>
      )}
    </section>
  );
}
