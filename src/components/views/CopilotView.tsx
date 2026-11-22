import { useEffect, useState } from "react";
import type { Persona } from "../../types/persona";
import type {
  AnswerFormat,
  ConfirmationStatus,
  CreateQuestionInput,
  CreateSessionInput,
  QuestionType,
  SurveyQuestion,
  SurveySession,
  SurveySessionSummary,
  UpdateQuestionInput,
} from "../../types/survey";
import { surveyService } from "../../services/surveyService";
import { AppError } from "../../types/error";

interface CopilotViewProps {
  activePersona?: Persona;
}

export function CopilotView({ activePersona }: CopilotViewProps) {
  // Session State
  const [sessions, setSessions] = useState<SurveySessionSummary[]>([]);
  const [selectedSessionId, setSelectedSessionId] = useState<string>("");
  const [currentSession, setCurrentSession] = useState<SurveySession | null>(null);
  const [isCreatingSession, setIsCreatingSession] = useState(false);

  // New Session Form State
  const [newSessionTitle, setNewSessionTitle] = useState("");
  const [newSessionPlatform, setNewSessionPlatform] = useState("");
  const [newSessionUrl, setNewSessionUrl] = useState("");
  const [newSessionNotes, setNewSessionNotes] = useState("");

  // Questions in Selected Session
  const [questions, setQuestions] = useState<SurveyQuestion[]>([]);
  const [activeQuestionId, setActiveQuestionId] = useState<string | null>(null);

  // Question Form State
  const [questionTextOriginal, setQuestionTextOriginal] = useState("");
  const [questionTextCorrected, setQuestionTextCorrected] = useState("");
  const [instructions, setInstructions] = useState("");
  const [questionType, setQuestionType] = useState<QuestionType>("single_choice");
  const [answerFormat, setAnswerFormat] = useState<AnswerFormat>("single_choice");
  const [options, setOptions] = useState<string[]>([]);
  const [newOptionInput, setNewOptionInput] = useState("");
  const [confirmationStatus, setConfirmationStatus] = useState<ConfirmationStatus>("draft");

  // User Answer Draft & Confirmed Answer
  const [userAnswerDraft, setUserAnswerDraft] = useState("");
  const [userAnswerConfirmed, setUserAnswerConfirmed] = useState<string | null>(null);
  const [confirmedAt, setConfirmedAt] = useState<string | null>(null);

  // UI Feedback States
  const [loading, setLoading] = useState(false);
  const [savingQuestion, setSavingQuestion] = useState(false);
  const [savingDraft, setSavingDraft] = useState(false);
  const [confirmingAnswer, setConfirmingAnswer] = useState(false);
  const [errorMessage, setErrorMessage] = useState("");
  const [successMessage, setSuccessMessage] = useState("");

  // Load Sessions for Active Persona
  async function loadSessions(preserveSelectedId?: string) {
    if (!activePersona) {
      setSessions([]);
      setSelectedSessionId("");
      setCurrentSession(null);
      setQuestions([]);
      return;
    }

    try {
      setLoading(true);
      const list = await surveyService.listSessions(activePersona.id, false);
      setSessions(list);
      setErrorMessage("");

      const targetId = preserveSelectedId || (list.length > 0 ? list[0].id : "");
      if (targetId) {
        setSelectedSessionId(targetId);
        await loadSessionDetails(targetId);
      } else {
        setSelectedSessionId("");
        setCurrentSession(null);
        setQuestions([]);
        resetQuestionForm();
      }
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setLoading(false);
    }
  }

  // Load Session and Questions
  async function loadSessionDetails(sessionId: string) {
    if (!activePersona || !sessionId) return;
    try {
      setLoading(true);
      const detail = await surveyService.getSessionDetail(activePersona.id, sessionId);
      setCurrentSession(detail.session);
      setQuestions(detail.questions);

      // Select first question or reset to create mode
      if (detail.questions.length > 0) {
        selectQuestionForEditing(detail.questions[0]);
      } else {
        resetQuestionForm();
      }
      setErrorMessage("");
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadSessions();
  }, [activePersona?.id]);

  function resetQuestionForm() {
    setActiveQuestionId(null);
    setQuestionTextOriginal("");
    setQuestionTextCorrected("");
    setInstructions("");
    setQuestionType("single_choice");
    setAnswerFormat("single_choice");
    setOptions([]);
    setNewOptionInput("");
    setConfirmationStatus("draft");
    setUserAnswerDraft("");
    setUserAnswerConfirmed(null);
    setConfirmedAt(null);
  }

  function selectQuestionForEditing(q: SurveyQuestion) {
    setActiveQuestionId(q.id);
    setQuestionTextOriginal(q.questionTextOriginal);
    setQuestionTextCorrected(q.questionTextCorrected || "");
    setInstructions(q.instructions || "");
    setQuestionType(q.questionType);
    setAnswerFormat(q.answerFormat);
    setOptions(q.options || []);
    setNewOptionInput("");
    setConfirmationStatus(q.confirmationStatus);
    setUserAnswerDraft(q.userAnswerDraft || "");
    setUserAnswerConfirmed(q.userAnswerConfirmed || null);
    setConfirmedAt(q.confirmedAt || null);
  }

  // Handle Session Creation
  async function handleCreateSession(e: React.FormEvent) {
    e.preventDefault();
    if (!activePersona) return;
    if (!newSessionTitle.trim()) {
      setErrorMessage("Judul survei wajib diisi");
      return;
    }

    try {
      setLoading(true);
      const input: CreateSessionInput = {
        personaId: activePersona.id,
        title: newSessionTitle.trim(),
        platformLabel: newSessionPlatform.trim() || null,
        sourceUrl: newSessionUrl.trim() || null,
        notes: newSessionNotes.trim() || null,
      };

      const created = await surveyService.createSession(input);
      setIsCreatingSession(false);
      setNewSessionTitle("");
      setNewSessionPlatform("");
      setNewSessionUrl("");
      setNewSessionNotes("");
      setSuccessMessage(`Sesi survei "${created.title}" berhasil dibuat.`);
      setTimeout(() => setSuccessMessage(""), 4000);

      await loadSessions(created.id);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setLoading(false);
    }
  }

  // Handle Option Builder
  function handleAddOption() {
    const trimmed = newOptionInput.trim();
    if (!trimmed) return;
    if (options.includes(trimmed)) {
      setErrorMessage("Pilihan jawaban ini sudah ada di daftar.");
      return;
    }
    setOptions([...options, trimmed]);
    setNewOptionInput("");
    setErrorMessage("");
  }

  function handleRemoveOption(indexToRemove: number) {
    setOptions(options.filter((_, idx) => idx !== indexToRemove));
  }

  // Save / Update Question
  async function handleSaveQuestion() {
    if (!activePersona || !currentSession) return;
    if (!questionTextOriginal.trim()) {
      setErrorMessage("Teks pertanyaan asli wajib diisi.");
      return;
    }

    try {
      setSavingQuestion(true);
      setErrorMessage("");

      if (activeQuestionId) {
        // Update existing question
        const updateInput: UpdateQuestionInput = {
          personaId: activePersona.id,
          questionId: activeQuestionId,
          questionTextOriginal: questionTextOriginal.trim(),
          questionTextCorrected: questionTextCorrected.trim() || null,
          instructions: instructions.trim() || null,
          options,
          questionType,
          answerFormat,
          confirmationStatus,
          userAnswerDraft: userAnswerDraft || null,
        };

        const updated = await surveyService.updateQuestion(updateInput);
        setSuccessMessage(`Pertanyaan #${updated.questionOrder} berhasil diperbarui.`);
        setTimeout(() => setSuccessMessage(""), 4000);

        // Refresh list
        const refreshedList = await surveyService.listQuestions(activePersona.id, currentSession.id);
        setQuestions(refreshedList);
        const refreshedQ = refreshedList.find((q) => q.id === updated.id);
        if (refreshedQ) selectQuestionForEditing(refreshedQ);
      } else {
        // Create new question
        const createInput: CreateQuestionInput = {
          personaId: activePersona.id,
          sessionId: currentSession.id,
          questionTextOriginal: questionTextOriginal.trim(),
          instructions: instructions.trim() || null,
          options,
          questionType,
          answerFormat,
          provenance: "manual_entry",
        };

        const created = await surveyService.createQuestion(createInput);
        setSuccessMessage(`Pertanyaan #${created.questionOrder} berhasil disimpan ke sesi.`);
        setTimeout(() => setSuccessMessage(""), 4000);

        // Refresh list and select the newly created question
        const refreshedList = await surveyService.listQuestions(activePersona.id, currentSession.id);
        setQuestions(refreshedList);
        const refreshedQ = refreshedList.find((q) => q.id === created.id);
        if (refreshedQ) selectQuestionForEditing(refreshedQ);
      }
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setSavingQuestion(false);
    }
  }

  // Auto-save User Draft Answer
  async function handleSaveDraftAnswer() {
    if (!activePersona || !activeQuestionId) {
      setErrorMessage("Simpan atau pilih pertanyaan terlebih dahulu sebelum menyimpan draf jawaban.");
      return;
    }

    try {
      setSavingDraft(true);
      setErrorMessage("");
      await surveyService.saveQuestionDraft({
        personaId: activePersona.id,
        questionId: activeQuestionId,
        userAnswerDraft,
      });

      setSuccessMessage("Draf jawaban pengguna berhasil disimpan ke basis data lokal.");
      setTimeout(() => setSuccessMessage(""), 3000);

      // Refresh question list to reflect changes
      if (currentSession) {
        const refreshed = await surveyService.listQuestions(activePersona.id, currentSession.id);
        setQuestions(refreshed);
      }
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setSavingDraft(false);
    }
  }

  // Confirm Actual User Answer
  async function handleConfirmAnswer() {
    if (!activePersona || !activeQuestionId) {
      setErrorMessage("Pilih pertanyaan terlebih dahulu.");
      return;
    }

    const answerToConfirm = userAnswerDraft.trim();
    if (!answerToConfirm) {
      setErrorMessage("Tuliskan jawaban yang ingin dikonfirmasi terlebih dahulu.");
      return;
    }

    try {
      setConfirmingAnswer(true);
      setErrorMessage("");
      await surveyService.confirmQuestionAnswer({
        personaId: activePersona.id,
        questionId: activeQuestionId,
        userAnswer: answerToConfirm,
      });

      setUserAnswerConfirmed(answerToConfirm);
      setConfirmationStatus("confirmed");
      setConfirmedAt(new Date().toISOString());

      setSuccessMessage("Jawaban aktual berhasil dikonfirmasi dan dicatat ke riwayat sesi.");
      setTimeout(() => setSuccessMessage(""), 4000);

      // Refresh question list
      if (currentSession) {
        const refreshed = await surveyService.listQuestions(activePersona.id, currentSession.id);
        setQuestions(refreshed);
      }
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setConfirmingAnswer(false);
    }
  }

  // Delete Question
  async function handleDeleteQuestion() {
    if (!activePersona || !activeQuestionId || !currentSession) return;
    if (!window.confirm("Apakah kamu yakin ingin menghapus pertanyaan ini dari sesi survei?")) return;

    try {
      setLoading(true);
      await surveyService.deleteQuestion(activePersona.id, activeQuestionId);
      setSuccessMessage("Pertanyaan berhasil dihapus.");
      setTimeout(() => setSuccessMessage(""), 3000);

      const refreshed = await surveyService.listQuestions(activePersona.id, currentSession.id);
      setQuestions(refreshed);
      if (refreshed.length > 0) {
        selectQuestionForEditing(refreshed[0]);
      } else {
        resetQuestionForm();
      }
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setLoading(false);
    }
  }

  // Archive Current Session
  async function handleArchiveSession() {
    if (!activePersona || !currentSession) return;
    if (!window.confirm(`Arsipkan sesi "${currentSession.title}"? Sesi akan dipindahkan ke tab Arsip di menu Riwayat.`)) return;

    try {
      setLoading(true);
      await surveyService.archiveSession(activePersona.id, currentSession.id);
      setSuccessMessage(`Sesi "${currentSession.title}" berhasil diarsipkan.`);
      setTimeout(() => setSuccessMessage(""), 3000);
      await loadSessions();
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setLoading(false);
    }
  }

  if (!activePersona) {
    return (
      <section className="content empty-state">
        <div className="empty-icon">👤</div>
        <h2>Pilih Profil Terlebih Dahulu</h2>
        <p>
          Survey Copilot bekerja berbasis profil responden nyata. Silakan pilih atau buat
          profil di menu <strong>Profil</strong> untuk memulai sesi survei.
        </p>
      </section>
    );
  }

  return (
    <section className="content workspace">
      <div className="section-heading">
        <div>
          <span className="eyebrow">WORKSPACE COPILOT</span>
          <h2>Input Pertanyaan Survei Manual & Draf</h2>
        </div>
        <span className="coming-soon">Responden: <strong>{activePersona.displayName}</strong></span>
      </div>

      {errorMessage && (
        <div className="notice" style={{ background: "#fdf2f2", color: "#a12020", marginBottom: "16px" }}>
          <span>⚠️</span>
          <p>{errorMessage}</p>
        </div>
      )}

      {successMessage && (
        <div className="notice" style={{ background: "#eef8f2", color: "#156236", marginBottom: "16px" }}>
          <span>✓</span>
          <p>{successMessage}</p>
        </div>
      )}

      {/* Sesi Survei Selector Bar */}
      <div className="survey-session-bar">
        <div className="survey-session-controls">
          <label htmlFor="session-select" style={{ fontSize: "12px", fontWeight: "700" }}>
            Sesi Survei:
          </label>
          <select
            id="session-select"
            className="survey-session-dropdown"
            value={selectedSessionId}
            onChange={(e) => {
              const val = e.target.value;
              setSelectedSessionId(val);
              if (val) void loadSessionDetails(val);
            }}
            disabled={loading || sessions.length === 0}
          >
            {sessions.length === 0 ? (
              <option value="">Belum ada sesi survei</option>
            ) : (
              sessions.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.title} ({s.questionCount} pertanyaan) {s.platformLabel ? `• ${s.platformLabel}` : ""}
                </option>
              ))
            )}
          </select>

          {currentSession && (
            <span className={`survey-badge ${currentSession.status}`}>
              {currentSession.status === "active" ? "Aktif" : currentSession.status === "archived" ? "Diarsipkan" : "Selesai"}
            </span>
          )}

          {currentSession?.platformLabel && (
            <span className="survey-badge" style={{ background: "#f0f4f1", color: "#2d4436" }}>
              🌐 {currentSession.platformLabel}
            </span>
          )}
        </div>

        <div style={{ display: "flex", gap: "8px" }}>
          <button
            className="secondary-button"
            onClick={() => setIsCreatingSession(true)}
            disabled={loading}
          >
            + Sesi Baru
          </button>
          {currentSession && (
            <button
              className="secondary-button"
              onClick={() => void handleArchiveSession()}
              disabled={loading}
              title="Pindahkan sesi ini ke arsip"
            >
              📦 Arsipkan
            </button>
          )}
        </div>
      </div>

      {/* Modal / Dialog Pembuatan Sesi Baru */}
      {isCreatingSession && (
        <div
          style={{
            background: "#fff",
            border: "1px solid #bce3cb",
            borderRadius: "12px",
            padding: "20px",
            marginBottom: "24px",
            boxShadow: "0 4px 12px rgba(0,0,0,0.06)",
          }}
        >
          <h3 style={{ margin: "0 0 14px", fontSize: "16px", fontWeight: "800" }}>
            Buat Sesi Survei Baru
          </h3>
          <form onSubmit={(e) => void handleCreateSession(e)}>
            <div style={{ display: "grid", gap: "12px", gridTemplateColumns: "1fr 1fr" }}>
              <div style={{ gridColumn: "span 2" }}>
                <label className="field-label" htmlFor="new-session-title">
                  Judul Survei *
                </label>
                <input
                  id="new-session-title"
                  type="text"
                  placeholder="Contoh: Survei Kebiasaan Belanja Online Q4 2026"
                  value={newSessionTitle}
                  onChange={(e) => setNewSessionTitle(e.target.value)}
                  style={{
                    width: "100%",
                    padding: "10px 12px",
                    border: "1px solid #dce4de",
                    borderRadius: "8px",
                  }}
                  required
                />
              </div>

              <div>
                <label className="field-label" htmlFor="new-session-platform">
                  Platform / Sumber (Opsional)
                </label>
                <input
                  id="new-session-platform"
                  type="text"
                  placeholder="Misal: Toluna, YouGov, Google Forms"
                  value={newSessionPlatform}
                  onChange={(e) => setNewSessionPlatform(e.target.value)}
                  style={{
                    width: "100%",
                    padding: "10px 12px",
                    border: "1px solid #dce4de",
                    borderRadius: "8px",
                  }}
                />
              </div>

              <div>
                <label className="field-label" htmlFor="new-session-url">
                  URL Survei (Opsional)
                </label>
                <input
                  id="new-session-url"
                  type="url"
                  placeholder="https://..."
                  value={newSessionUrl}
                  onChange={(e) => setNewSessionUrl(e.target.value)}
                  style={{
                    width: "100%",
                    padding: "10px 12px",
                    border: "1px solid #dce4de",
                    borderRadius: "8px",
                  }}
                />
              </div>

              <div style={{ gridColumn: "span 2" }}>
                <label className="field-label" htmlFor="new-session-notes">
                  Catatan Sesi (Opsional)
                </label>
                <input
                  id="new-session-notes"
                  type="text"
                  placeholder="Catatan tambahan mengenai konteks survei ini"
                  value={newSessionNotes}
                  onChange={(e) => setNewSessionNotes(e.target.value)}
                  style={{
                    width: "100%",
                    padding: "10px 12px",
                    border: "1px solid #dce4de",
                    borderRadius: "8px",
                  }}
                />
              </div>
            </div>

            <div style={{ display: "flex", gap: "10px", marginTop: "16px", justifyContent: "flex-end" }}>
              <button
                type="button"
                className="secondary-button"
                onClick={() => setIsCreatingSession(false)}
              >
                Batal
              </button>
              <button
                type="submit"
                className="primary-button"
                disabled={loading || !newSessionTitle.trim()}
                style={{ marginTop: 0 }}
              >
                {loading ? "Menyimpan..." : "Simpan Sesi"}
              </button>
            </div>
          </form>
        </div>
      )}

      {/* Main Workspace Layout */}
      {!currentSession ? (
        <div className="survey-card" style={{ textAlign: "center", padding: "48px 20px" }}>
          <div className="empty-icon" style={{ margin: "0 auto 16px" }}>📋</div>
          <h3>Belum Ada Sesi Survei Terpilih</h3>
          <p style={{ color: "var(--muted)", maxWidth: "420px", margin: "8px auto 20px" }}>
            Buat sesi survei baru untuk mulai mencatat pertanyaan, instruksi, dan pilihan jawaban survei.
          </p>
          <button
            className="primary-button"
            onClick={() => setIsCreatingSession(true)}
            style={{ marginTop: 0 }}
          >
            + Buat Sesi Survei Baru
          </button>
        </div>
      ) : (
        <div className="survey-workspace-grid">
          {/* Kolom Kiri: Form Input Pertanyaan & Jawaban */}
          <div className="survey-card">
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
              <h3 style={{ margin: 0, fontSize: "16px", fontWeight: "800" }}>
                {activeQuestionId
                  ? `Edit Pertanyaan #${questions.find((q) => q.id === activeQuestionId)?.questionOrder || ""}`
                  : "Input Pertanyaan Manual Baru"}
              </h3>
              {activeQuestionId && (
                <button
                  className="secondary-button"
                  onClick={resetQuestionForm}
                  style={{ padding: "6px 12px", fontSize: "11px" }}
                >
                  + Tambah Pertanyaan Baru
                </button>
              )}
            </div>

            {/* Teks Pertanyaan Asli */}
            <label className="field-label" htmlFor="question-original">
              Teks Pertanyaan Asli (Persis Sesuai Survei) *
            </label>
            <textarea
              id="question-original"
              value={questionTextOriginal}
              onChange={(e) => setQuestionTextOriginal(e.target.value)}
              placeholder="Ketik atau tempel teks pertanyaan survei di sini persis seperti yang tertulis..."
              style={{ minHeight: "110px", marginBottom: "16px" }}
            />

            {/* Koreksi Teks Opsional */}
            {activeQuestionId && (
              <div style={{ marginBottom: "16px" }}>
                <label className="field-label" htmlFor="question-corrected">
                  Koreksi Teks Pertanyaan (Opsional Manual)
                </label>
                <input
                  id="question-corrected"
                  type="text"
                  value={questionTextCorrected}
                  onChange={(e) => setQuestionTextCorrected(e.target.value)}
                  placeholder="Koreksi ejaan atau tata bahasa jika diperlukan"
                  style={{
                    width: "100%",
                    padding: "10px 12px",
                    border: "1px solid #dce4de",
                    borderRadius: "8px",
                  }}
                />
              </div>
            )}

            {/* Instruksi Tambahan */}
            <label className="field-label" htmlFor="question-instructions">
              Instruksi Tambahan (Opsional)
            </label>
            <input
              id="question-instructions"
              type="text"
              value={instructions}
              onChange={(e) => setInstructions(e.target.value)}
              placeholder="Contoh: Pilih 1 jawaban saja, Pilih paling banyak 3 pilihan, Tuliskan dalam angka..."
              style={{
                width: "100%",
                padding: "10px 12px",
                border: "1px solid #dce4de",
                borderRadius: "8px",
                marginBottom: "16px",
              }}
            />

            {/* Tipe & Format Pertanyaan */}
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "14px", marginBottom: "16px" }}>
              <div>
                <label className="field-label" htmlFor="question-type-select">
                  Tipe Pertanyaan
                </label>
                <select
                  id="question-type-select"
                  className="survey-session-dropdown"
                  style={{ width: "100%", maxWidth: "100%" }}
                  value={questionType}
                  onChange={(e) => setQuestionType(e.target.value as QuestionType)}
                >
                  <option value="single_choice">Pilihan Tunggal (Radio)</option>
                  <option value="multiple_choice">Pilihan Ganda (Checkbox)</option>
                  <option value="free_text">Teks Bebas / Esai</option>
                  <option value="scale">Skala Penilaian (Rating / Likert)</option>
                  <option value="matrix">Tabel / Matriks</option>
                  <option value="numeric">Angka / Numerik</option>
                  <option value="date">Tanggal</option>
                  <option value="other">Lainnya</option>
                  <option value="unknown">Tidak Diketahui</option>
                </select>
              </div>

              <div>
                <label className="field-label" htmlFor="answer-format-select">
                  Format Jawaban
                </label>
                <select
                  id="answer-format-select"
                  className="survey-session-dropdown"
                  style={{ width: "100%", maxWidth: "100%" }}
                  value={answerFormat}
                  onChange={(e) => setAnswerFormat(e.target.value as AnswerFormat)}
                >
                  <option value="single_choice">Pilihan Tunggal</option>
                  <option value="multiple_selection">Pilihan Banyak</option>
                  <option value="text">Teks Singkat</option>
                  <option value="number">Nilai Angka</option>
                  <option value="rating_scale">Skala 1 - 5 / 1 - 10</option>
                  <option value="matrix">Kisi Pilihan (Grid)</option>
                  <option value="boolean">Ya / Tidak</option>
                  <option value="unknown">Tidak Diketahui</option>
                </select>
              </div>
            </div>

            {/* Pilihan Jawaban (Options Builder) */}
            <div className="options-builder">
              <label className="field-label">
                Pilihan Jawaban yang Tersedia ({options.length})
              </label>
              <div style={{ display: "flex", gap: "8px" }}>
                <input
                  type="text"
                  placeholder="Ketik pilihan jawaban dan tekan Enter..."
                  value={newOptionInput}
                  onChange={(e) => setNewOptionInput(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault();
                      handleAddOption();
                    }
                  }}
                  style={{
                    flex: 1,
                    padding: "9px 12px",
                    border: "1px solid #dce4de",
                    borderRadius: "8px",
                  }}
                />
                <button
                  type="button"
                  className="secondary-button"
                  onClick={handleAddOption}
                  style={{ padding: "8px 14px" }}
                >
                  + Tambah
                </button>
              </div>

              {options.length > 0 && (
                <div className="options-list">
                  {options.map((opt, idx) => (
                    <div key={idx} className="option-item">
                      <span>
                        <strong style={{ color: "var(--muted)", marginRight: "8px" }}>{idx + 1}.</strong>
                        {opt}
                      </span>
                      <button
                        type="button"
                        className="option-remove-btn"
                        onClick={() => handleRemoveOption(idx)}
                        title="Hapus opsi ini"
                      >
                        ×
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Tombol Simpan Pertanyaan */}
            <div style={{ display: "flex", gap: "10px", alignItems: "center", borderTop: "1px solid var(--line)", paddingTop: "16px" }}>
              <button
                type="button"
                className="primary-button"
                onClick={() => void handleSaveQuestion()}
                disabled={savingQuestion || !questionTextOriginal.trim()}
                style={{ marginTop: 0 }}
              >
                {savingQuestion
                  ? "Menyimpan..."
                  : activeQuestionId
                  ? "Perbarui Pertanyaan"
                  : "Simpan Pertanyaan ke Sesi"}
              </button>

              {activeQuestionId && (
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => void handleDeleteQuestion()}
                  style={{ color: "#b23a36", borderColor: "#f3c2c0" }}
                  disabled={loading}
                >
                  Hapus Pertanyaan
                </button>
              )}
            </div>

            {/* Draf & Jawaban Pengguna Terpisah */}
            {activeQuestionId && (
              <div className="answer-draft-box">
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "8px" }}>
                  <label className="field-label" htmlFor="user-answer-draft" style={{ margin: 0 }}>
                    Draf Jawaban Pengguna
                  </label>
                  <span className={`survey-badge ${confirmationStatus}`}>
                    {confirmationStatus === "confirmed" ? "✓ Terkonfirmasi" : "Draf"}
                  </span>
                </div>
                <textarea
                  id="user-answer-draft"
                  value={userAnswerDraft}
                  onChange={(e) => setUserAnswerDraft(e.target.value)}
                  placeholder="Ketik atau edit draf jawaban di sini..."
                  style={{ minHeight: "80px", marginBottom: "12px" }}
                />

                <div style={{ display: "flex", gap: "10px", flexWrap: "wrap", alignItems: "center" }}>
                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => void handleSaveDraftAnswer()}
                    disabled={savingDraft}
                  >
                    {savingDraft ? "Menyimpan..." : "💾 Simpan Draf"}
                  </button>

                  <button
                    type="button"
                    className="primary-button"
                    onClick={() => void handleConfirmAnswer()}
                    disabled={confirmingAnswer || !userAnswerDraft.trim()}
                    style={{ marginTop: 0 }}
                  >
                    {confirmingAnswer ? "Mengkonfirmasi..." : "✓ Konfirmasi Jawaban Aktual"}
                  </button>
                </div>

                {userAnswerConfirmed && (
                  <div className="confirmed-answer-display">
                    <strong>Jawaban Terkonfirmasi:</strong> {userAnswerConfirmed}
                    {confirmedAt && (
                      <div style={{ fontSize: "11px", color: "#44785b", marginTop: "4px" }}>
                        Waktu konfirmasi: {new Date(confirmedAt).toLocaleString("id-ID")}
                      </div>
                    )}
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Kolom Kanan: Riwayat Pertanyaan dalam Sesi Ini */}
          <div className="question-history-panel">
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "6px" }}>
              <span style={{ fontSize: "13px", fontWeight: "800", color: "#172a20" }}>
                Riwayat Pertanyaan ({questions.length})
              </span>
              <button
                type="button"
                className="secondary-button"
                onClick={resetQuestionForm}
                style={{ padding: "4px 8px", fontSize: "11px" }}
              >
                + Baru
              </button>
            </div>

            {questions.length === 0 ? (
              <div className="empty-inline" style={{ padding: "24px 14px", background: "white" }}>
                Belum ada pertanyaan yang dicatat dalam sesi ini.
              </div>
            ) : (
              questions.map((q) => {
                const isSelected = q.id === activeQuestionId;
                return (
                  <div
                    key={q.id}
                    className={`question-item-card ${isSelected ? "selected" : ""}`}
                    onClick={() => selectQuestionForEditing(q)}
                  >
                    <div className="question-item-header">
                      <strong style={{ fontSize: "12px", color: "var(--green-dark)" }}>
                        #{q.questionOrder}
                      </strong>
                      <span className={`survey-badge ${q.confirmationStatus}`}>
                        {q.confirmationStatus === "confirmed" ? "Terkonfirmasi" : "Draf"}
                      </span>
                    </div>

                    <div
                      style={{
                        fontSize: "13px",
                        lineHeight: "1.4",
                        color: "var(--ink)",
                        display: "-webkit-box",
                        WebkitLineClamp: 2,
                        WebkitBoxOrient: "vertical",
                        overflow: "hidden",
                      }}
                    >
                      {q.questionTextOriginal}
                    </div>

                    <div style={{ display: "flex", gap: "8px", fontSize: "11px", color: "var(--muted)", marginTop: "4px" }}>
                      <span>{q.options.length} opsi</span>
                      {q.userAnswerConfirmed && (
                        <span style={{ color: "#0d6e49", fontWeight: "600" }}>✓ Terisi</span>
                      )}
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>
      )}
    </section>
  );
}
