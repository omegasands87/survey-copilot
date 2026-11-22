import { useEffect, useState } from "react";
import type { Persona } from "../../types/persona";
import type {
  SurveyQuestion,
  SurveySessionDetail,
  SurveySessionSummary,
} from "../../types/survey";
import { surveyService } from "../../services/surveyService";
import { AppError } from "../../types/error";

interface HistoryViewProps {
  activePersona?: Persona;
  onOpenSessionInCopilot?: (sessionId: string) => void;
}

export function HistoryView({ activePersona, onOpenSessionInCopilot }: HistoryViewProps) {
  const [sessions, setSessions] = useState<SurveySessionSummary[]>([]);
  const [filter, setFilter] = useState<"all" | "active" | "archived">("active");
  const [expandedSessionId, setExpandedSessionId] = useState<string | null>(null);
  const [sessionDetail, setSessionDetail] = useState<SurveySessionDetail | null>(null);

  const [loading, setLoading] = useState(false);
  const [detailLoading, setDetailLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState("");
  const [successMessage, setSuccessMessage] = useState("");

  async function loadSessions() {
    if (!activePersona) {
      setSessions([]);
      return;
    }

    try {
      setLoading(true);
      const list = await surveyService.listSessions(activePersona.id, true);
      setSessions(list);
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
      setLoading(true);
      await surveyService.archiveSession(activePersona.id, sessionId);
      setSuccessMessage("Sesi survei berhasil dipindahkan ke arsip.");
      setTimeout(() => setSuccessMessage(""), 3000);
      await loadSessions();
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setLoading(false);
    }
  }

  async function handleRestoreSession(sessionId: string) {
    if (!activePersona) return;
    try {
      setLoading(true);
      await surveyService.restoreSession(activePersona.id, sessionId);
      setSuccessMessage("Sesi survei berhasil dipulihkan ke status aktif.");
      setTimeout(() => setSuccessMessage(""), 3000);
      await loadSessions();
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setLoading(false);
    }
  }

  async function handleDeleteSession(sessionId: string, sessionTitle: string) {
    if (!activePersona) return;
    const confirmPrompt = window.prompt(
      `Peringatan: Seluruh pertanyaan, tangkapan layar, dan draf dalam sesi ini akan dihapus secara permanen.\n\nKetik "${sessionTitle}" untuk mengonfirmasi penghapusan:`
    );

    if (confirmPrompt !== sessionTitle) {
      if (confirmPrompt !== null) {
        alert("Konfirmasi nama sesi tidak cocok. Penghapusan dibatalkan.");
      }
      return;
    }

    try {
      setLoading(true);
      await surveyService.deleteSession(activePersona.id, sessionId);
      setSuccessMessage(`Sesi survei "${sessionTitle}" berhasil dihapus.`);
      setTimeout(() => setSuccessMessage(""), 3000);
      if (expandedSessionId === sessionId) {
        setExpandedSessionId(null);
        setSessionDetail(null);
      }
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
        <div className="empty-icon">◷</div>
        <h2>Pilih Profil Responden</h2>
        <p>Riwayat sesi survei dicatat terpisah untuk setiap profil responden nyata.</p>
      </section>
    );
  }

  const filteredSessions = sessions.filter((s) => {
    if (filter === "active") return s.status === "active";
    if (filter === "archived") return s.status === "archived";
    return true;
  });

  return (
    <section className="content">
      <div className="section-heading">
        <div>
          <span className="eyebrow">SURVEY MEMORY & RIWAYAT</span>
          <h2>Riwayat Sesi & Pertanyaan Survei</h2>
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

      {/* Filter Tabs */}
      <div className="filter-tabs">
        <button
          className={`filter-tab ${filter === "active" ? "active" : ""}`}
          onClick={() => setFilter("active")}
        >
          Sesi Aktif ({sessions.filter((s) => s.status === "active").length})
        </button>
        <button
          className={`filter-tab ${filter === "archived" ? "active" : ""}`}
          onClick={() => setFilter("archived")}
        >
          Diarsipkan ({sessions.filter((s) => s.status === "archived").length})
        </button>
        <button
          className={`filter-tab ${filter === "all" ? "active" : ""}`}
          onClick={() => setFilter("all")}
        >
          Semua Sesi ({sessions.length})
        </button>
      </div>

      {/* Daftar Sesi Survei */}
      {loading && sessions.length === 0 ? (
        <div className="empty-inline">Memuat riwayat sesi survei...</div>
      ) : filteredSessions.length === 0 ? (
        <div className="empty-inline" style={{ padding: "36px 20px" }}>
          Tidak ada sesi survei pada filter ini.
        </div>
      ) : (
        <div style={{ display: "grid", gap: "16px", marginTop: "16px" }}>
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
                        title="Arsipkan sesi"
                      >
                        📦
                      </button>
                    ) : (
                      <button
                        className="secondary-button"
                        onClick={() => void handleRestoreSession(s.id)}
                        style={{ padding: "8px 12px", fontSize: "11px" }}
                        title="Pulihkan sesi"
                      >
                        ↩ Pulihkan
                      </button>
                    )}

                    <button
                      className="secondary-button"
                      onClick={() => void handleDeleteSession(s.id, s.title)}
                      style={{ padding: "8px 12px", fontSize: "11px", color: "#b23a36", borderColor: "#f5c4c4" }}
                      title="Hapus sesi permanen"
                    >
                      🗑
                    </button>
                  </div>
                </div>

                {/* Expanded Session Detail: Ordered Questions List */}
                {isExpanded && (
                  <div
                    style={{
                      marginTop: "18px",
                      paddingTop: "16px",
                      borderTop: "1px dashed var(--line)",
                    }}
                  >
                    <h4 style={{ margin: "0 0 12px", fontSize: "13px", fontWeight: "800", color: "#2d4436" }}>
                      Daftar Pertanyaan & Jawaban Terurut ({sessionDetail?.questions.length || 0})
                    </h4>

                    {detailLoading ? (
                      <div className="empty-inline">Memuat detail pertanyaan...</div>
                    ) : !sessionDetail || sessionDetail.questions.length === 0 ? (
                      <div className="empty-inline">Belum ada pertanyaan pada sesi ini.</div>
                    ) : (
                      <div style={{ display: "grid", gap: "10px" }}>
                        {sessionDetail.questions.map((q: SurveyQuestion) => (
                          <div
                            key={q.id}
                            style={{
                              background: "#f9faf9",
                              border: "1px solid #e1e8e3",
                              borderRadius: "8px",
                              padding: "14px",
                            }}
                          >
                            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "6px" }}>
                              <strong style={{ fontSize: "12px", color: "var(--green-dark)" }}>
                                #{q.questionOrder}
                              </strong>
                              <span className={`survey-badge ${q.confirmationStatus}`}>
                                {q.confirmationStatus === "confirmed" ? "✓ Terkonfirmasi" : "Draf"}
                              </span>
                            </div>

                            <p style={{ margin: "0 0 8px", fontSize: "13px", fontWeight: "600", color: "var(--ink)" }}>
                              {q.questionTextOriginal}
                            </p>

                            {q.instructions && (
                              <div style={{ fontSize: "12px", color: "var(--muted)", marginBottom: "8px", fontStyle: "italic" }}>
                                Instruksi: {q.instructions}
                              </div>
                            )}

                            {q.options && q.options.length > 0 && (
                              <div style={{ fontSize: "12px", color: "#3d4e44", marginBottom: "8px" }}>
                                <strong>Pilihan ({q.options.length}):</strong> {q.options.join(", ")}
                              </div>
                            )}

                            {q.userAnswerConfirmed ? (
                              <div className="confirmed-answer-display" style={{ marginTop: "6px" }}>
                                <strong>Jawaban Dikonfirmasi:</strong> {q.userAnswerConfirmed}
                                {q.confirmedAt && (
                                  <span style={{ fontSize: "11px", marginLeft: "10px", color: "#44785b" }}>
                                    ({new Date(q.confirmedAt).toLocaleDateString("id-ID")})
                                  </span>
                                )}
                              </div>
                            ) : q.userAnswerDraft ? (
                              <div style={{ fontSize: "12px", color: "#8a6508", background: "#fdf8ec", padding: "8px 12px", borderRadius: "6px" }}>
                                <strong>Draf Jawaban:</strong> {q.userAnswerDraft}
                              </div>
                            ) : (
                              <div style={{ fontSize: "11px", color: "var(--muted)" }}>
                                Belum ada draf atau jawaban yang dikonfirmasi.
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
    </section>
  );
}
