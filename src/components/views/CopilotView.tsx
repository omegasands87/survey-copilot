import type { Persona } from "../../types/persona";

interface CopilotViewProps {
  question: string;
  activePersona?: Persona;
  onChangeQuestion: (text: string) => void;
}

export function CopilotView({
  question,
  activePersona,
  onChangeQuestion,
}: CopilotViewProps) {
  return (
    <section className="content workspace">
      <div className="section-heading">
        <div>
          <span className="eyebrow">WORKSPACE</span>
          <h2>Masukkan pertanyaan survei</h2>
        </div>
        <span className="coming-soon">Modul analisis sedang disiapkan</span>
      </div>
      <label className="field-label" htmlFor="question">
        Teks pertanyaan
      </label>
      <textarea
        id="question"
        value={question}
        onChange={(event) => onChangeQuestion(event.target.value)}
        placeholder="Tempel pertanyaan survei di sini..."
      />
      <div className="workspace-actions">
        <button className="secondary-button" disabled>
          Ambil screenshot
        </button>
        <button
          className="primary-button"
          disabled={!question.trim() || !activePersona}
        >
          Analisis pertanyaan
        </button>
      </div>
      <p className="helper-text">
        Pastikan teks dan pilihan jawaban sudah benar sebelum analisis dilakukan.
      </p>
    </section>
  );
}
