import { useEffect, useState } from "react";
import type { Persona } from "../../types/persona";
import type {
  SurveyQuestion,
  SurveySessionDetail,
  SurveySessionSummary,
} from "../../types/survey";
import type {
  HistoricalLearningSummary,
  OutcomeConfidence,
  OutcomeLabel,
  SurveyAttemptDetail,
} from "../../types/attempt";
import { surveyService } from "../../services/surveyService";
import { attemptService } from "../../services/attemptService";
import { AppError } from "../../types/error";

interface HistoryViewProps {
  activePersona?: Persona;
  onOpenSessionInCopilot?: (sessionId: string) => void;
}

const OUTCOME_LABELS: { value: OutcomeLabel; label: string; badgeClass: string }[] = [
  { value: "completed", label: "Selesai Penuh (Completed)", badgeClass: "status-completed" },
  { value: "qualified", label: "Lolos Skrining (Qualified)", badgeClass: "status-qualified" },
  { value: "screened_out", label: "Gagal Skrining (Screened Out)", badgeClass: "status-screened" },
  { value: "quota_full", label: "Kuota Penuh (Quota Full)", badgeClass: "status-quota" },
  { value: "quality_rejected", label: "Ditolak Kualitas (Quality Rejected)", badgeClass: "status-rejected" },
  { value: "technical_failure", label: "Kegagalan Teknis (Technical Failure)", badgeClass: "status-technical" },
  { value: "abandoned_by_user", label: "Dibatalkan Pengguna (Abandoned)", badgeClass: "status-abandoned" },
  { value: "pending", label: "Sedang Berjalan (Pending)", badgeClass: "status-pending" },
  { value: "unknown", label: "Tidak Diketahui (Unknown)", badgeClass: "status-unknown" },
];

export function HistoryView({ activePersona, onOpenSessionInCopilot }: HistoryViewProps) {
  // Mode Tab: Hasil Percobaan (SC-052) vs Sesi Survei (Survey Sessions)
  const [activeTab, setActiveTab] = useState<"attempts" | "sessions">("attempts");

  // State untuk Hasil Percobaan Survei (SC-052)
  const [attempts, setAttempts] = useState<SurveyAttemptDetail[]>([]);
  const [learningSummary, setLearningSummary] = useState<HistoricalLearningSummary | null>(null);
  const [filterCategory, setFilterCategory] = useState<string>("");
  const [filterPlatform, setFilterPlatform] = useState<string>("");
  const [filterStatus, setFilterStatus] = useState<string>("");
  const [attemptsLoading, setAttemptsLoading] = useState(false);

  // Modal Catat Percobaan Baru
  const [showRecordModal, setShowRecordModal] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newPlatform, setNewPlatform] = useState("");
  const [newCategory, setNewCategory] = useState("");
  const [newTargetUrl, setNewTargetUrl] = useState("");
  const [newRawText, setNewRawText] = useState("");
  const [newInitialOutcome, setNewInitialOutcome] = useState<OutcomeLabel>("pending");
  const [newEvidenceSource, setNewEvidenceSource] = useState("pencatatan_manual_pengguna");
  const [newEvidenceNote, setNewEvidenceNote] = useState("");
  const [newConfidence, setNewConfidence] = useState<OutcomeConfidence>("observed");
  const [recordSubmitting, setRecordSubmitting] = useState(false);

  // Modal Update Event Percobaan
  const [updatingAttempt, setUpdatingAttempt] = useState<SurveyAttemptDetail | null>(null);
  const [updateOutcome, setUpdateOutcome] = useState<OutcomeLabel>("completed");
  const [updateEvidenceSource, setUpdateEvidenceSource] = useState("layar_akhir_survei");
  const [updateEvidenceNote, setUpdateEvidenceNote] = useState("");
  const [updateConfidence, setUpdateConfidence] = useState<OutcomeConfidence>("observed");
  const [updateSubmitting, setUpdateSubmitting] = useState(false);

  // State untuk Sesi Survei (Existing sessions)
  const [sessions, setSessions] = useState<SurveySessionSummary[]>([]);
  const [sessionFilter, setSessionFilter] = useState<"all" | "active" | "archived">("active");
  const [expandedSessionId, setExpandedSessionId] = useState<string | null>(null);
  const [sessionDetail, setSessionDetail] = useState<SurveySessionDetail | null>(null);
  const [sessionsLoading, setSessionsLoading] = useState(false);
  const [detailLoading, setDetailLoading] = useState(false);

  const [errorMessage, setErrorMessage] = useState("");
  const [successMessage, setSuccessMessage] = useState("");

  // Muat data percobaan & ringkasan pembelajaran historis
  async function loadAttemptsAndLearning() {
    if (!activePersona) {
      setAttempts([]);
      setLearningSummary(null);
      return;
    }

    try {
      setAttemptsLoading(true);
      const [attemptList, summary] = await Promise.all([
        attemptService.listAttempts({
          personaId: activePersona.id,
          category: filterCategory ? filterCategory : undefined,
          sourcePlatform: filterPlatform ? filterPlatform : undefined,
          status: filterStatus ? (filterStatus as OutcomeLabel) : undefined,
        }),
        attemptService.getLearningSummary(
          activePersona.id,
          filterCategory ? filterCategory : undefined,
          filterPlatform ? filterPlatform : undefined
        ),
      ]);
      setAttempts(attemptList);
      setLearningSummary(summary);
      setErrorMessage("");
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setAttemptsLoading(false);
    }
  }

  // Muat data sesi survei
  async function loadSessions() {
    if (!activePersona) {
      setSessions([]);
      return;
    }

    try {
      setSessionsLoading(true);
      const list = await surveyService.listSessions(activePersona.id, true);
      setSessions(list);
      setErrorMessage("");
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setSessionsLoading(false);
    }
  }

  useEffect(() => {
    if (activeTab === "attempts") {
      void loadAttemptsAndLearning();
    } else {
      void loadSessions();
    }
  }, [activePersona?.id, activeTab, filterCategory, filterPlatform, filterStatus]);

  // Keyboard navigation: Escape closes active modals
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        if (showRecordModal) setShowRecordModal(false);
        else if (updatingAttempt) setUpdatingAttempt(null);
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [showRecordModal, updatingAttempt]);

  // Handler simpan percobaan baru
  async function handleCreateAttempt(e: React.FormEvent) {
    e.preventDefault();
    if (!activePersona) return;

    try {
      setRecordSubmitting(true);
      await attemptService.recordAttempt({
        personaId: activePersona.id,
        opportunityTitle: newTitle.trim() || "Survei Mandiri",
        sourcePlatform: newPlatform.trim() || "Manual",
        category: newCategory.trim() || undefined,
        targetUrl: newTargetUrl.trim() || undefined,
        rawText: newRawText.trim() || undefined,
        initialOutcome: newInitialOutcome,
        evidenceSource: newEvidenceSource.trim() || "pencatatan_manual_pengguna",
        evidenceNote: newEvidenceNote.trim() || undefined,
        confidence: newConfidence,
      });

      setSuccessMessage("Hasil percobaan survei berhasil dicatat ke dalam riwayat observasi.");
      setTimeout(() => setSuccessMessage(""), 3500);
      setShowRecordModal(false);
      resetRecordForm();
      await loadAttemptsAndLearning();
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setRecordSubmitting(false);
    }
  }

  function resetRecordForm() {
    setNewTitle("");
    setNewPlatform("");
    setNewCategory("");
    setNewTargetUrl("");
    setNewRawText("");
    setNewInitialOutcome("pending");
    setNewEvidenceSource("pencatatan_manual_pengguna");
    setNewEvidenceNote("");
    setNewConfidence("observed");
  }

  // Handler perbarui peristiwa hasil survei
  async function handleRecordOutcomeEvent(e: React.FormEvent) {
    e.preventDefault();
    if (!activePersona || !updatingAttempt) return;

    try {
      setUpdateSubmitting(true);
      await attemptService.recordOutcomeEvent({
        attemptId: updatingAttempt.id,
        personaId: activePersona.id,
        outcomeLabel: updateOutcome,
        evidenceSource: updateEvidenceSource.trim() || "pembaruan_status",
        evidenceNote: updateEvidenceNote.trim() || undefined,
        confidence: updateConfidence,
      });

      setSuccessMessage("Status percobaan survei berhasil diperbarui dengan peristiwa baru.");
      setTimeout(() => setSuccessMessage(""), 3500);
      setUpdatingAttempt(null);
      setUpdateEvidenceNote("");
      await loadAttemptsAndLearning();
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setUpdateSubmitting(false);
    }
  }

  // Handler hapus percobaan
  async function handleDeleteAttempt(attemptId: string, title: string) {
    if (!activePersona) return;
    const confirmed = window.confirm(
      `Apakah Anda yakin ingin menghapus catatan percobaan survei "${title}"?\n\nTindakan ini permanen dan tidak dapat dibatalkan.`
    );
    if (!confirmed) return;

    try {
      setAttemptsLoading(true);
      await attemptService.deleteAttempt(attemptId, activePersona.id);
      setSuccessMessage("Catatan percobaan survei berhasil dihapus.");
      setTimeout(() => setSuccessMessage(""), 3000);
      await loadAttemptsAndLearning();
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setAttemptsLoading(false);
    }
  }

  // Sesi handlers
  async function toggleExpandSession(sessionId: string) {
    if (expandedSessionId === sessionId) {
      setExpandedSessionId(null);
      setSessionDetail(null);
      return;
    }

    if (!activePersona) return;
    try {
      setExpandedSessionId(sessionId);
      setDetailLoading(true);
      const detail = await surveyService.getSessionDetail(activePersona.id, sessionId);
      setSessionDetail(detail);
      setErrorMessage("");
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setDetailLoading(false);
    }
  }

  async function handleArchiveSession(sessionId: string) {
    if (!activePersona) return;
    try {
      setSessionsLoading(true);
      await surveyService.archiveSession(activePersona.id, sessionId);
      setSuccessMessage("Sesi survei berhasil dipindahkan ke arsip.");
      setTimeout(() => setSuccessMessage(""), 3000);
      await loadSessions();
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setSessionsLoading(false);
    }
  }

  async function handleRestoreSession(sessionId: string) {
    if (!activePersona) return;
    try {
      setSessionsLoading(true);
      await surveyService.restoreSession(activePersona.id, sessionId);
      setSuccessMessage("Sesi survei berhasil dipulihkan ke status aktif.");
      setTimeout(() => setSuccessMessage(""), 3000);
      await loadSessions();
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setSessionsLoading(false);
    }
  }

  async function handleDeleteSession(sessionId: string, sessionTitle: string) {
    if (!activePersona) return;
    const confirmed = window.confirm(
      `Apakah Anda yakin ingin menghapus sesi survei "${sessionTitle}" beserta seluruh pertanyaan dan rekomendasinya?\n\nTindakan ini permanen dan tidak dapat dibatalkan.`
    );
    if (!confirmed) return;

    try {
      setSessionsLoading(true);
      await surveyService.deleteSession(activePersona.id, sessionId);
      setSuccessMessage("Sesi survei berhasil dihapus.");
      setTimeout(() => setSuccessMessage(""), 3000);
      if (expandedSessionId === sessionId) {
        setExpandedSessionId(null);
        setSessionDetail(null);
      }
      await loadSessions();
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setSessionsLoading(false);
    }
  }

  if (!activePersona) {
    return (
      <div className="card" style={{ padding: "40px 24px", textAlign: "center" }}>
        <p className="eyebrow">HISTORICAL OUTCOME LEARNING & SURVEY MEMORY</p>
        <h2>Tidak Ada Profil Responden Aktif</h2>
        <p className="lead" style={{ maxWidth: "540px", margin: "12px auto" }}>
          Pilih atau aktifkan profil responden terlebih dahulu di menu <strong>Profil</strong> untuk melihat riwayat hasil percobaan dan analisis pembelajaran historis.
        </p>
      </div>
    );
  }

  const filteredSessions = sessions.filter((s) => {
    if (sessionFilter === "all") return true;
    return s.status === sessionFilter;
  });

  return (
    <div style={{ display: "grid", gap: "20px" }}>
      {/* Top Header */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: "16px", flexWrap: "wrap" }}>
        <div>
          <span className="eyebrow">SURVEY HISTORY & HISTORICAL OUTCOMES (SC-052)</span>
          <h2 style={{ margin: "4px 0" }}>Riwayat Observasi & Pembelajaran Hasil Survei</h2>
          <p style={{ margin: 0, fontSize: "13px", color: "var(--muted)" }}>
            Responden Aktif: <strong>{activePersona.displayName}</strong> | Pencatatan observasi empiris terpisah tegas dari analisis AI.
          </p>
        </div>

        {activeTab === "attempts" && (
          <button
            className="primary-button"
            onClick={() => setShowRecordModal(true)}
            style={{ padding: "10px 16px", fontSize: "13px", fontWeight: "700" }}
          >
            + Catat Hasil Percobaan Baru
          </button>
        )}
      </div>

      {/* Notifikasi Global */}
      {errorMessage && (
        <div className="notice" role="alert" style={{ background: "#fdf2f2", color: "#a12020", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <span aria-hidden="true">⚠️</span>
            <p style={{ margin: 0 }}>{errorMessage}</p>
          </div>
          <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
            <button
              type="button"
              className="secondary-button"
              onClick={() => {
                if (activeTab === "attempts") void loadAttemptsAndLearning();
                else void loadSessions();
              }}
              aria-label="Coba muat ulang data riwayat"
              style={{ padding: "4px 10px", fontSize: "11px" }}
            >
              🔄 Coba Lagi
            </button>
            <button
              type="button"
              onClick={() => setErrorMessage("")}
              aria-label="Tutup pesan peringatan"
              style={{ background: "transparent", border: 0, cursor: "pointer", color: "#a12020", fontWeight: "bold" }}
            >
              ✕
            </button>
          </div>
        </div>
      )}

      {successMessage && (
        <div className="notice" role="status" aria-live="polite" style={{ background: "#eef8f2", color: "#156236", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <span aria-hidden="true">✓</span>
            <p style={{ margin: 0 }}>{successMessage}</p>
          </div>
          <button
            type="button"
            onClick={() => setSuccessMessage("")}
            aria-label="Tutup pesan sukses"
            style={{ background: "transparent", border: 0, cursor: "pointer", color: "#156236", fontWeight: "bold" }}
          >
            ✕
          </button>
        </div>
      )}

      {/* Main Tab Switcher */}
      <div className="filter-tabs" style={{ borderBottom: "2px solid var(--line)", paddingBottom: "2px" }}>
        <button
          className={`filter-tab ${activeTab === "attempts" ? "active" : ""}`}
          onClick={() => setActiveTab("attempts")}
          style={{ fontSize: "13px", fontWeight: "700" }}
        >
          📊 Hasil Percobaan & Pembelajaran Historis (SC-052)
        </button>
        <button
          className={`filter-tab ${activeTab === "sessions" ? "active" : ""}`}
          onClick={() => setActiveTab("sessions")}
          style={{ fontSize: "13px", fontWeight: "700" }}
        >
          📝 Sesi Pertanyaan & Rekomendasi Jawaban ({sessions.length})
        </button>
      </div>

      {/* TAB 1: HASIL PERCOBAAN SURVEI & PEMBELAJARAN HISTORIS (SC-052) */}
      {activeTab === "attempts" && (
        <div style={{ display: "grid", gap: "20px" }}>
          {/* Card Analisis Pembelajaran Historis (AI / Statistical Learning) - Terpisah dari Observasi */}
          <div
            style={{
              background: learningSummary?.status === "insufficient_data" ? "#fffcf0" : "#f4fbf7",
              border: `1px solid ${learningSummary?.status === "insufficient_data" ? "#eed98b" : "#9ed9b8"}`,
              borderRadius: "12px",
              padding: "20px",
            }}
          >
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: "12px", flexWrap: "wrap", marginBottom: "12px" }}>
              <div>
                <span
                  style={{
                    fontSize: "11px",
                    fontWeight: "800",
                    textTransform: "uppercase",
                    letterSpacing: "0.5px",
                    color: learningSummary?.status === "insufficient_data" ? "#8a6d1c" : "#1a6e3d",
                  }}
                >
                  ANALISIS PEMBELAJARAN HISTORIS (TERPISAH DARI OBSERVASI FAKTUAL)
                </span>
                <h3 style={{ margin: "4px 0 0", fontSize: "16px", fontWeight: "800" }}>
                  {learningSummary?.status === "insufficient_data"
                    ? "Data Historis Belum Cukup (Insufficient Data)"
                    : "Model Pembelajaran Historis Terkalibrasi (Sufficient Data)"}
                </h3>
              </div>

              <span
                style={{
                  padding: "4px 10px",
                  borderRadius: "20px",
                  fontSize: "12px",
                  fontWeight: "800",
                  background: learningSummary?.status === "insufficient_data" ? "#faecd0" : "#d8f3e5",
                  color: learningSummary?.status === "insufficient_data" ? "#82530d" : "#0d6837",
                }}
              >
                {learningSummary?.status === "insufficient_data" ? "⚠️ Insufficient Data (AT-063)" : "✓ Terkalibrasi (Holdout Valid)"}
              </span>
            </div>

            <p style={{ margin: "0 0 14px", fontSize: "13px", lineHeight: "1.5", color: "var(--text)" }}>
              {learningSummary?.message || "Memuat ringkasan pembelajaran..."}
            </p>

            {/* Statistik Jika Cukup Data vs Insufficient Data */}
            {learningSummary?.status === "sufficient_data" ? (
              <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(160px, 1fr))", gap: "12px", marginBottom: "14px" }}>
                <div style={{ background: "white", padding: "12px", borderRadius: "8px", border: "1px solid #d4eedf" }}>
                  <div style={{ fontSize: "11px", color: "var(--muted)", fontWeight: "600" }}>Ukuran Sampel</div>
                  <div style={{ fontSize: "20px", fontWeight: "800", color: "#14532d" }}>{learningSummary.sampleSize}</div>
                  <div style={{ fontSize: "10px", color: "var(--muted)" }}>Minimal: {learningSummary.minimumRequiredSample}</div>
                </div>

                <div style={{ background: "white", padding: "12px", borderRadius: "8px", border: "1px solid #d4eedf" }}>
                  <div style={{ fontSize: "11px", color: "var(--muted)", fontWeight: "600" }}>Tingkat Kelulusan Empiris</div>
                  <div style={{ fontSize: "20px", fontWeight: "800", color: "#1d4ed8" }}>
                    {learningSummary.qualificationRate !== null && learningSummary.qualificationRate !== undefined
                      ? `${(learningSummary.qualificationRate * 100).toFixed(1)}%`
                      : "-"}
                  </div>
                  <div style={{ fontSize: "10px", color: "var(--muted)" }}>Lolos + Selesai</div>
                </div>

                <div style={{ background: "white", padding: "12px", borderRadius: "8px", border: "1px solid #d4eedf" }}>
                  <div style={{ fontSize: "11px", color: "var(--muted)", fontWeight: "600" }}>Tingkat Selesai Penuh</div>
                  <div style={{ fontSize: "20px", fontWeight: "800", color: "#15803d" }}>
                    {learningSummary.completionRate !== null && learningSummary.completionRate !== undefined
                      ? `${(learningSummary.completionRate * 100).toFixed(1)}%`
                      : "-"}
                  </div>
                  <div style={{ fontSize: "10px", color: "var(--muted)" }}>Completed</div>
                </div>

                <div style={{ background: "white", padding: "12px", borderRadius: "8px", border: "1px solid #d4eedf" }}>
                  <div style={{ fontSize: "11px", color: "var(--muted)", fontWeight: "600" }}>Konsistensi Holdout</div>
                  <div style={{ fontSize: "20px", fontWeight: "800", color: "#7c3aed" }}>
                    {learningSummary.holdoutConsistencyRate !== null && learningSummary.holdoutConsistencyRate !== undefined
                      ? `${(learningSummary.holdoutConsistencyRate * 100).toFixed(1)}%`
                      : "-"}
                  </div>
                  <div style={{ fontSize: "10px", color: "var(--muted)" }}>Uji {learningSummary.holdoutSampleSize} sampel kronologis</div>
                </div>
              </div>
            ) : (
              <div
                style={{
                  background: "white",
                  padding: "12px 16px",
                  borderRadius: "8px",
                  border: "1px dashed #e6c86e",
                  fontSize: "12px",
                  color: "#6b4f0c",
                  marginBottom: "14px",
                }}
              >
                🔒 <strong>Prinsip Anti-Fabrikasi (AT-063):</strong> Karena sampel baru tercatat {learningSummary?.sampleSize || 0} dari minimal {learningSummary?.minimumRequiredSample || 10} observasi, sistem dilarang mengarang persentase probabilitas kelulusan atau membuat prediksi kelayakan prematur.
              </div>
            )}

            {/* Pengungkapan Ketidakpastian & Disclaimers */}
            <div style={{ fontSize: "11px", color: "var(--muted)", borderTop: "1px solid rgba(0,0,0,0.06)", paddingTop: "10px" }}>
              <div style={{ fontWeight: "700", marginBottom: "4px", color: "var(--text)" }}>
                ℹ️ {learningSummary?.uncertaintyDisclosure}
              </div>
              <ul style={{ margin: "4px 0 0", paddingLeft: "18px", lineHeight: "1.4" }}>
                {learningSummary?.disclaimers.map((d, idx) => (
                  <li key={idx}>{d}</li>
                ))}
              </ul>
            </div>
          </div>

          {/* Filter Bar Hasil Percobaan */}
          <div
            style={{
              background: "white",
              border: "1px solid var(--line)",
              borderRadius: "12px",
              padding: "16px",
              display: "flex",
              flexWrap: "wrap",
              gap: "12px",
              alignItems: "flex-end",
            }}
          >
            <div style={{ flex: "1 1 180px" }}>
              <label style={{ display: "block", fontSize: "11px", fontWeight: "700", marginBottom: "4px" }}>
                Kategori / Topik
              </label>
              <input
                type="text"
                placeholder="Semua kategori..."
                value={filterCategory}
                onChange={(e) => setFilterCategory(e.target.value)}
                style={{ width: "100%", padding: "8px 10px", fontSize: "12px", borderRadius: "6px", border: "1px solid var(--line)" }}
              />
            </div>

            <div style={{ flex: "1 1 180px" }}>
              <label style={{ display: "block", fontSize: "11px", fontWeight: "700", marginBottom: "4px" }}>
                Sumber Platform
              </label>
              <input
                type="text"
                placeholder="Semua platform..."
                value={filterPlatform}
                onChange={(e) => setFilterPlatform(e.target.value)}
                style={{ width: "100%", padding: "8px 10px", fontSize: "12px", borderRadius: "6px", border: "1px solid var(--line)" }}
              />
            </div>

            <div style={{ flex: "1 1 200px" }}>
              <label style={{ display: "block", fontSize: "11px", fontWeight: "700", marginBottom: "4px" }}>
                Status Hasil
              </label>
              <select
                value={filterStatus}
                onChange={(e) => setFilterStatus(e.target.value)}
                style={{ width: "100%", padding: "8px 10px", fontSize: "12px", borderRadius: "6px", border: "1px solid var(--line)" }}
              >
                <option value="">Semua Status Hasil ({OUTCOME_LABELS.length})</option>
                {OUTCOME_LABELS.map((o) => (
                  <option key={o.value} value={o.value}>
                    {o.label}
                  </option>
                ))}
              </select>
            </div>

            {(filterCategory || filterPlatform || filterStatus) && (
              <button
                className="secondary-button"
                onClick={() => {
                  setFilterCategory("");
                  setFilterPlatform("");
                  setFilterStatus("");
                }}
                style={{ padding: "8px 14px", fontSize: "12px" }}
              >
                Reset Filter
              </button>
            )}
          </div>

          {/* Daftar Percobaan Survei Teramati */}
          <div>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
              <h3 style={{ margin: 0, fontSize: "15px", fontWeight: "800" }}>
                Buku Observasi Faktual Teramati ({attempts.length} Percobaan)
              </h3>
              <span style={{ fontSize: "12px", color: "var(--muted)" }}>
                Mencatat fakta hasil yang dialami responden asli
              </span>
            </div>

            {attemptsLoading && attempts.length === 0 ? (
              <div className="empty-inline">Memuat riwayat observasi hasil survei...</div>
            ) : attempts.length === 0 ? (
              <div
                className="card"
                style={{
                  padding: "40px 20px",
                  textAlign: "center",
                  background: "white",
                  border: "1px dashed var(--line)",
                  borderRadius: "12px",
                }}
              >
                <p style={{ margin: 0, fontWeight: "700", fontSize: "14px" }}>
                  Belum ada catatan percobaan survei untuk profil ini.
                </p>
                <p style={{ margin: "6px 0 16px", fontSize: "12px", color: "var(--muted)" }}>
                  Setelah mencoba mengisi survei secara manual di browser, catat hasilnya di sini untuk membangun histori observasi yang jujur.
                </p>
                <button
                  className="primary-button"
                  onClick={() => setShowRecordModal(true)}
                  style={{ padding: "8px 16px", fontSize: "12px" }}
                >
                  + Catat Percobaan Survei Sekarang
                </button>
              </div>
            ) : (
              <div style={{ display: "grid", gap: "16px" }}>
                {attempts.map((att) => {
                  const statusMeta =
                    OUTCOME_LABELS.find((o) => o.value === att.currentOutcome) || OUTCOME_LABELS[8];

                  return (
                    <div
                      key={att.id}
                      style={{
                        background: "white",
                        border: "1px solid var(--line)",
                        borderRadius: "12px",
                        padding: "18px 20px",
                        boxShadow: "0 1px 3px rgba(0,0,0,0.03)",
                      }}
                    >
                      {/* Baris Atas Kartu */}
                      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: "12px", flexWrap: "wrap", marginBottom: "10px" }}>
                        <div>
                          <div style={{ display: "flex", alignItems: "center", gap: "8px", flexWrap: "wrap" }}>
                            <h4 style={{ margin: 0, fontSize: "15px", fontWeight: "800" }}>
                              {att.opportunityTitle}
                            </h4>
                            <span
                              style={{
                                padding: "3px 8px",
                                borderRadius: "6px",
                                fontSize: "11px",
                                fontWeight: "700",
                                ...getStatusBadgeStyle(att.currentOutcome),
                              }}
                            >
                              {statusMeta.label}
                            </span>
                            <span
                              style={{
                                padding: "2px 8px",
                                borderRadius: "4px",
                                fontSize: "11px",
                                background: "#f0f4f8",
                                color: "#1e3a8a",
                                fontWeight: "600",
                              }}
                            >
                              🌐 {att.sourcePlatform}
                            </span>
                            {att.category && (
                              <span
                                style={{
                                  padding: "2px 8px",
                                  borderRadius: "4px",
                                  fontSize: "11px",
                                  background: "#fdf4ff",
                                  color: "#86198f",
                                  fontWeight: "600",
                                }}
                              >
                                🏷️ {att.category}
                              </span>
                            )}
                          </div>

                          <div style={{ fontSize: "12px", color: "var(--muted)", marginTop: "4px", display: "flex", gap: "12px", flexWrap: "wrap" }}>
                            <span>Mulai: {new Date(att.startedAt).toLocaleString("id-ID")}</span>
                            {att.endedAt && <span>Selesai: {new Date(att.endedAt).toLocaleString("id-ID")}</span>}
                            <span>Events: {att.events.length}</span>
                          </div>
                        </div>

                        {/* Aksi */}
                        <div style={{ display: "flex", gap: "8px" }}>
                          <button
                            className="secondary-button"
                            onClick={() => {
                              setUpdatingAttempt(att);
                              setUpdateOutcome(att.currentOutcome);
                            }}
                            style={{ padding: "6px 12px", fontSize: "12px", fontWeight: "600" }}
                          >
                            + Perbarui Status
                          </button>
                          <button
                            className="danger-button"
                            onClick={() => void handleDeleteAttempt(att.id, att.opportunityTitle)}
                            style={{ padding: "6px 12px", fontSize: "12px" }}
                          >
                            Hapus
                          </button>
                        </div>
                      </div>

                      {/* Kriteria Teramati (jika ada) */}
                      {att.criteria && att.criteria.length > 0 && (
                        <div style={{ background: "#f9fafb", borderRadius: "8px", padding: "10px 14px", marginTop: "10px", fontSize: "12px" }}>
                          <div style={{ fontWeight: "700", marginBottom: "6px", color: "#374151" }}>
                            Kriteria Kelayakan Teramati ({att.criteria.length}):
                          </div>
                          <div style={{ display: "flex", flexWrap: "wrap", gap: "6px" }}>
                            {att.criteria.map((c) => (
                              <span
                                key={c.id}
                                style={{
                                  background: "white",
                                  border: "1px solid #e5e7eb",
                                  padding: "3px 8px",
                                  borderRadius: "4px",
                                  fontSize: "11px",
                                }}
                              >
                                📌 {c.criterionText}
                              </span>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Timeline Events Teramati */}
                      <div style={{ marginTop: "12px", borderTop: "1px solid #f3f4f6", paddingTop: "10px" }}>
                        <div style={{ fontSize: "11px", fontWeight: "700", textTransform: "uppercase", color: "var(--muted)", marginBottom: "6px" }}>
                          Kronologi Bukti & Kejadian ({att.events.length}):
                        </div>
                        <div style={{ display: "grid", gap: "6px" }}>
                          {att.events.map((ev, evIdx) => (
                            <div
                              key={ev.id}
                              style={{
                                display: "flex",
                                alignItems: "center",
                                gap: "10px",
                                fontSize: "12px",
                                padding: "4px 8px",
                                background: evIdx === att.events.length - 1 ? "#fafafa" : "transparent",
                                borderRadius: "4px",
                              }}
                            >
                              <span style={{ color: "var(--muted)", fontSize: "11px", minWidth: "120px" }}>
                                {new Date(ev.observedAt).toLocaleTimeString("id-ID")}
                              </span>
                              <span
                                style={{
                                  fontWeight: "700",
                                  fontSize: "11px",
                                  padding: "2px 6px",
                                  borderRadius: "4px",
                                  ...getStatusBadgeStyle(ev.outcomeLabel),
                                }}
                              >
                                {ev.outcomeLabel}
                              </span>
                              <span style={{ color: "var(--text)" }}>
                                Sumber: <strong>{ev.evidenceSource}</strong>
                              </span>
                              {ev.evidenceNote && (
                                <span style={{ color: "var(--muted)", fontStyle: "italic" }}>
                                  — "{ev.evidenceNote}"
                                </span>
                              )}
                              <span style={{ fontSize: "10px", color: "#6b7280", marginLeft: "auto" }}>
                                [{ev.confidence}]
                              </span>
                            </div>
                          ))}
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </div>
      )}

      {/* TAB 2: SESI SURVEI (SURVEY SESSIONS) */}
      {activeTab === "sessions" && (
        <div style={{ display: "grid", gap: "16px" }}>
          {/* Filter Tabs Sesi */}
          <div className="filter-tabs">
            <button
              className={`filter-tab ${sessionFilter === "active" ? "active" : ""}`}
              onClick={() => setSessionFilter("active")}
            >
              Sesi Aktif ({sessions.filter((s) => s.status === "active").length})
            </button>
            <button
              className={`filter-tab ${sessionFilter === "archived" ? "active" : ""}`}
              onClick={() => setSessionFilter("archived")}
            >
              Diarsipkan ({sessions.filter((s) => s.status === "archived").length})
            </button>
            <button
              className={`filter-tab ${sessionFilter === "all" ? "active" : ""}`}
              onClick={() => setSessionFilter("all")}
            >
              Semua Sesi ({sessions.length})
            </button>
          </div>

          {sessionsLoading && sessions.length === 0 ? (
            <div className="empty-inline">Memuat riwayat sesi survei...</div>
          ) : filteredSessions.length === 0 ? (
            <div className="empty-inline" style={{ padding: "36px 20px" }}>
              Tidak ada sesi survei pada filter ini.
            </div>
          ) : (
            <div style={{ display: "grid", gap: "16px" }}>
              {filteredSessions.map((s) => {
                const isExpanded = expandedSessionId === s.id;
                return (
                  <div
                    key={s.id}
                    style={{
                      background: "white",
                      border: "1px solid var(--line)",
                      borderRadius: "12px",
                      padding: "20px",
                      transition: "border-color .15s ease",
                    }}
                  >
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: "12px", flexWrap: "wrap" }}>
                      <div>
                        <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "6px" }}>
                          <h3 style={{ margin: 0, fontSize: "16px", fontWeight: "800" }}>{s.title}</h3>
                          <span className={`survey-badge ${s.status}`}>
                            {s.status === "active" ? "Aktif" : s.status === "archived" ? "Diarsipkan" : "Selesai"}
                          </span>
                          {s.platformLabel && (
                            <span className="survey-badge" style={{ background: "#f0f4f1", color: "#2d4436" }}>
                              🌐 {s.platformLabel}
                            </span>
                          )}
                        </div>

                        <div style={{ fontSize: "12px", color: "var(--muted)", display: "flex", gap: "14px", flexWrap: "wrap" }}>
                          <span>📝 {s.questionCount} Pertanyaan</span>
                          <span>✓ {s.confirmedAnswerCount} Terkonfirmasi</span>
                          <span>Dibuat: {new Date(s.createdAt).toLocaleDateString("id-ID")}</span>
                          <span>Diperbarui: {new Date(s.updatedAt).toLocaleTimeString("id-ID")}</span>
                        </div>
                      </div>

                      <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
                        {onOpenSessionInCopilot && s.status === "active" && (
                          <button
                            className="secondary-button"
                            onClick={() => onOpenSessionInCopilot(s.id)}
                            style={{ padding: "8px 12px", fontSize: "11px" }}
                          >
                            Buka di Copilot
                          </button>
                        )}

                        <button
                          className="secondary-button"
                          onClick={() => void toggleExpandSession(s.id)}
                          style={{ padding: "8px 12px", fontSize: "11px" }}
                        >
                          {isExpanded ? "Tutup Detail" : "Lihat Pertanyaan"}
                        </button>

                        {s.status === "active" ? (
                          <button
                            className="secondary-button"
                            onClick={() => void handleArchiveSession(s.id)}
                            style={{ padding: "8px 12px", fontSize: "11px" }}
                          >
                            Arsipkan
                          </button>
                        ) : (
                          <button
                            className="secondary-button"
                            onClick={() => void handleRestoreSession(s.id)}
                            style={{ padding: "8px 12px", fontSize: "11px" }}
                          >
                            Pulihkan
                          </button>
                        )}

                        <button
                          className="danger-button"
                          onClick={() => void handleDeleteSession(s.id, s.title)}
                          style={{ padding: "8px 12px", fontSize: "11px" }}
                        >
                          Hapus
                        </button>
                      </div>
                    </div>

                    {isExpanded && (
                      <div style={{ marginTop: "16px", borderTop: "1px solid var(--line)", paddingTop: "16px" }}>
                        {detailLoading ? (
                          <div className="empty-inline" style={{ padding: "16px" }}>Memuat daftar pertanyaan sesi...</div>
                        ) : !sessionDetail || sessionDetail.questions.length === 0 ? (
                          <div className="empty-inline" style={{ padding: "16px" }}>
                            Belum ada pertanyaan yang dicatat dalam sesi survei ini.
                          </div>
                        ) : (
                          <div style={{ display: "grid", gap: "12px" }}>
                            <div style={{ fontSize: "12px", fontWeight: "700", color: "var(--muted)" }}>
                              Daftar Pertanyaan ({sessionDetail.questions.length}):
                            </div>
                            {sessionDetail.questions.map((q: SurveyQuestion, idx: number) => (
                              <div
                                key={q.id}
                                style={{
                                  background: "#f9fafb",
                                  border: "1px solid #e5e7eb",
                                  borderRadius: "8px",
                                  padding: "12px",
                                  fontSize: "13px",
                                }}
                              >
                                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: "8px", marginBottom: "4px" }}>
                                  <span style={{ fontWeight: "700" }}>#{idx + 1}. {q.questionTextOriginal}</span>
                                  <span className={`survey-badge ${q.confirmationStatus}`}>
                                    {q.confirmationStatus === "confirmed"
                                      ? "Terkonfirmasi"
                                      : q.confirmationStatus === "rejected"
                                      ? "Ditolak"
                                      : q.confirmationStatus === "skipped"
                                      ? "Dilewati"
                                      : "Draf"}
                                  </span>
                                </div>

                                {q.userAnswerConfirmed && (
                                  <div style={{ marginTop: "6px", color: "#166534", fontSize: "12px" }}>
                                    <strong>Jawaban Terkonfirmasi:</strong> {q.userAnswerConfirmed}
                                  </div>
                                )}
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}

      {/* MODAL 1: CATAT PERCOBAAN SURVEI BARU */}
      {showRecordModal && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-record-attempt-title" style={{ maxWidth: "580px", maxHeight: "90vh", overflowY: "auto" }}>
            <div className="modal-header">
              <h3 id="modal-record-attempt-title">Catat Hasil Percobaan Survei Baru</h3>
              <button className="icon-button" aria-label="Tutup dialog catat percobaan" onClick={() => setShowRecordModal(false)}>✕</button>
            </div>

            <form onSubmit={handleCreateAttempt} style={{ display: "grid", gap: "14px", padding: "16px" }}>
              <div style={{ display: "grid", gap: "6px" }}>
                <label style={{ fontSize: "12px", fontWeight: "700" }}>Judul Survei *</label>
                <input
                  type="text"
                  required
                  placeholder="Contoh: Survei Kebiasaan Belanja Mingguan"
                  value={newTitle}
                  onChange={(e) => setNewTitle(e.target.value)}
                  style={{ width: "100%", padding: "8px 10px", fontSize: "13px", borderRadius: "6px", border: "1px solid var(--line)" }}
                />
              </div>

              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px" }}>
                <div style={{ display: "grid", gap: "6px" }}>
                  <label style={{ fontSize: "12px", fontWeight: "700" }}>Platform Survei *</label>
                  <input
                    type="text"
                    required
                    placeholder="Contoh: YouGov, Milieu, Toluna"
                    value={newPlatform}
                    onChange={(e) => setNewPlatform(e.target.value)}
                    style={{ width: "100%", padding: "8px 10px", fontSize: "13px", borderRadius: "6px", border: "1px solid var(--line)" }}
                  />
                </div>

                <div style={{ display: "grid", gap: "6px" }}>
                  <label style={{ fontSize: "12px", fontWeight: "700" }}>Kategori / Topik</label>
                  <input
                    type="text"
                    placeholder="Contoh: Konsumsi, Teknologi, Finansial"
                    value={newCategory}
                    onChange={(e) => setNewCategory(e.target.value)}
                    style={{ width: "100%", padding: "8px 10px", fontSize: "13px", borderRadius: "6px", border: "1px solid var(--line)" }}
                  />
                </div>
              </div>

              <div style={{ display: "grid", gap: "6px" }}>
                <label style={{ fontSize: "12px", fontWeight: "700" }}>URL / Tautan Survei (Opsional)</label>
                <input
                  type="text"
                  placeholder="https://..."
                  value={newTargetUrl}
                  onChange={(e) => setNewTargetUrl(e.target.value)}
                  style={{ width: "100%", padding: "8px 10px", fontSize: "13px", borderRadius: "6px", border: "1px solid var(--line)" }}
                />
              </div>

              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px" }}>
                <div style={{ display: "grid", gap: "6px" }}>
                  <label style={{ fontSize: "12px", fontWeight: "700" }}>Status Hasil Awal *</label>
                  <select
                    value={newInitialOutcome}
                    onChange={(e) => setNewInitialOutcome(e.target.value as OutcomeLabel)}
                    style={{ width: "100%", padding: "8px 10px", fontSize: "13px", borderRadius: "6px", border: "1px solid var(--line)" }}
                  >
                    {OUTCOME_LABELS.map((o) => (
                      <option key={o.value} value={o.value}>
                        {o.label}
                      </option>
                    ))}
                  </select>
                </div>

                <div style={{ display: "grid", gap: "6px" }}>
                  <label style={{ fontSize: "12px", fontWeight: "700" }}>Derajat Keyakinan Bukti</label>
                  <select
                    value={newConfidence}
                    onChange={(e) => setNewConfidence(e.target.value as OutcomeConfidence)}
                    style={{ width: "100%", padding: "8px 10px", fontSize: "13px", borderRadius: "6px", border: "1px solid var(--line)" }}
                  >
                    <option value="observed">Teramati Langsung</option>
                    <option value="user_confirmed">Dikonfirmasi Pengguna</option>
                    <option value="uncertain">Belum Pasti</option>
                  </select>
                </div>
              </div>

              <div style={{ display: "grid", gap: "6px" }}>
                <label style={{ fontSize: "12px", fontWeight: "700" }}>Sumber Bukti</label>
                <input
                  type="text"
                  value={newEvidenceSource}
                  onChange={(e) => setNewEvidenceSource(e.target.value)}
                  placeholder="Contoh: layar_akhir_survei, pesan_kuota_penuh, laporan_manual"
                  style={{ width: "100%", padding: "8px 10px", fontSize: "13px", borderRadius: "6px", border: "1px solid var(--line)" }}
                />
              </div>

              <div style={{ display: "grid", gap: "6px" }}>
                <label style={{ fontSize: "12px", fontWeight: "700" }}>Catatan Kejadian (Opsional)</label>
                <textarea
                  rows={2}
                  placeholder="Catatan observasi mengapa lolos/gagal..."
                  value={newEvidenceNote}
                  onChange={(e) => setNewEvidenceNote(e.target.value)}
                  style={{ width: "100%", padding: "8px 10px", fontSize: "13px", borderRadius: "6px", border: "1px solid var(--line)", resize: "vertical" }}
                />
              </div>

              <div style={{ display: "flex", justifyContent: "flex-end", gap: "10px", marginTop: "10px" }}>
                <button type="button" className="secondary-button" onClick={() => setShowRecordModal(false)}>
                  Batal
                </button>
                <button type="submit" className="primary-button" disabled={recordSubmitting}>
                  {recordSubmitting ? "Menyimpan..." : "Simpan Percobaan"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL 2: PERBARUI STATUS PERCOBAAN */}
      {updatingAttempt && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-update-attempt-title" style={{ maxWidth: "520px" }}>
            <div className="modal-header">
              <h3 id="modal-update-attempt-title">Perbarui Status Percobaan Survei</h3>
              <button className="icon-button" aria-label="Tutup dialog perbarui status percobaan" onClick={() => setUpdatingAttempt(null)}>✕</button>
            </div>

            <form onSubmit={handleRecordOutcomeEvent} style={{ display: "grid", gap: "14px", padding: "16px" }}>
              <div style={{ fontSize: "13px", color: "var(--muted)" }}>
                Survei: <strong>{updatingAttempt.opportunityTitle}</strong> ({updatingAttempt.sourcePlatform})
              </div>

              <div style={{ display: "grid", gap: "6px" }}>
                <label style={{ fontSize: "12px", fontWeight: "700" }}>Status Hasil Baru *</label>
                <select
                  value={updateOutcome}
                  onChange={(e) => setUpdateOutcome(e.target.value as OutcomeLabel)}
                  style={{ width: "100%", padding: "8px 10px", fontSize: "13px", borderRadius: "6px", border: "1px solid var(--line)" }}
                >
                  {OUTCOME_LABELS.map((o) => (
                    <option key={o.value} value={o.value}>
                      {o.label}
                    </option>
                  ))}
                </select>
              </div>

              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px" }}>
                <div style={{ display: "grid", gap: "6px" }}>
                  <label style={{ fontSize: "12px", fontWeight: "700" }}>Sumber Bukti</label>
                  <input
                    type="text"
                    required
                    value={updateEvidenceSource}
                    onChange={(e) => setUpdateEvidenceSource(e.target.value)}
                    style={{ width: "100%", padding: "8px 10px", fontSize: "13px", borderRadius: "6px", border: "1px solid var(--line)" }}
                  />
                </div>

                <div style={{ display: "grid", gap: "6px" }}>
                  <label style={{ fontSize: "12px", fontWeight: "700" }}>Derajat Keyakinan</label>
                  <select
                    value={updateConfidence}
                    onChange={(e) => setUpdateConfidence(e.target.value as OutcomeConfidence)}
                    style={{ width: "100%", padding: "8px 10px", fontSize: "13px", borderRadius: "6px", border: "1px solid var(--line)" }}
                  >
                    <option value="observed">Teramati Langsung</option>
                    <option value="user_confirmed">Dikonfirmasi Pengguna</option>
                    <option value="uncertain">Belum Pasti</option>
                  </select>
                </div>
              </div>

              <div style={{ display: "grid", gap: "6px" }}>
                <label style={{ fontSize: "12px", fontWeight: "700" }}>Catatan Kejadian</label>
                <textarea
                  rows={2}
                  placeholder="Misal: Mencapai halaman penutupan survei..."
                  value={updateEvidenceNote}
                  onChange={(e) => setUpdateEvidenceNote(e.target.value)}
                  style={{ width: "100%", padding: "8px 10px", fontSize: "13px", borderRadius: "6px", border: "1px solid var(--line)" }}
                />
              </div>

              <div style={{ display: "flex", justifyContent: "flex-end", gap: "10px", marginTop: "10px" }}>
                <button type="button" className="secondary-button" onClick={() => setUpdatingAttempt(null)}>
                  Batal
                </button>
                <button type="submit" className="primary-button" disabled={updateSubmitting}>
                  {updateSubmitting ? "Memperbarui..." : "Perbarui Status"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}

function getStatusBadgeStyle(outcome: OutcomeLabel): React.CSSProperties {
  switch (outcome) {
    case "completed":
      return { background: "#dcfce7", color: "#15803d" };
    case "qualified":
      return { background: "#dbeafe", color: "#1d4ed8" };
    case "screened_out":
      return { background: "#fee2e2", color: "#b91c1c" };
    case "quota_full":
      return { background: "#ffedd5", color: "#c2410c" };
    case "quality_rejected":
      return { background: "#f3e8ff", color: "#7e22ce" };
    case "technical_failure":
      return { background: "#fef2f2", color: "#991b1b" };
    case "abandoned_by_user":
      return { background: "#f3f4f6", color: "#4b5563" };
    case "pending":
      return { background: "#fef9c3", color: "#854d0e" };
    case "unknown":
    default:
      return { background: "#f1f5f9", color: "#64748b" };
  }
}
