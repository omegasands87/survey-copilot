import { useEffect, useState } from "react";
import type { Page } from "../Sidebar";
import type { Persona } from "../../types/persona";
import type {
  AnswerFormat,
  AnswerHistoryItem,
  ConfirmationStatus,
  CreateQuestionInput,
  CreateSessionInput,
  EvidencePackage,
  NextQuestionIntelligenceResult,
  QuestionCategory,
  QuestionClassificationResult,
  QuestionType,
  SurveyQuestion,
  SurveyRecognitionEvaluation,
  SurveySession,
  SurveySessionSummary,
  UpdateQuestionInput,
  ValidatedAnswerRecommendation,
} from "../../types/survey";
import { surveyService } from "../../services/surveyService";
import { captureService } from "../../services/captureService";
import { ocrService } from "../../services/ocrService";
import type { CapturePreview, WindowInfo } from "../../types/capture";
import type { ExtractedSurveyQuestion } from "../../types/ocr";
import { AppError } from "../../types/error";
import { OpportunityView } from "./OpportunityView";
import { PrivacyService } from "../../services/privacyService";
import type { OutboundDataDisclosure } from "../../types/privacy";

const CATEGORY_LABELS: Record<QuestionCategory, string> = {
  demographic: "Demografi",
  employment_education: "Pekerjaan & Pendidikan",
  financial_economic: "Kondisi Finansial & Ekonomi",
  consumption_habits: "Kebiasaan Konsumsi",
  asset_ownership: "Kepemilikan Aset",
  brand_product_usage: "Penggunaan Merek & Produk",
  experience_behavior: "Pengalaman & Perilaku",
  preference_opinion: "Preferensi & Opini",
  satisfaction_rating: "Kepuasan & Penilaian",
  multiple_choice: "Pilihan Ganda",
  matrix_grid: "Tabel / Matriks",
  numeric: "Numerik",
  date: "Tanggal",
  free_text: "Teks Bebas",
  other_unknown: "Lainnya / Tidak Diketahui",
};

interface CopilotViewProps {
  activePersona?: Persona;
  onNavigate?: (page: Page) => void;
}

export function CopilotView({ activePersona, onNavigate }: CopilotViewProps) {
  // Mode Copilot: Pertanyaan Sesi vs Peluang Survei (SC-050)
  const [copilotMode, setCopilotMode] = useState<"questions" | "opportunities">("questions");

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

  // User Answer Draft & Confirmed Answer (SC-034: Permanent Separation from AI Draft)
  const [userAnswerDraft, setUserAnswerDraft] = useState("");
  const [userAnswerConfirmed, setUserAnswerConfirmed] = useState<string | null>(null);
  const [confirmedAt, setConfirmedAt] = useState<string | null>(null);
  const [editReasonInput, setEditReasonInput] = useState("");

  // UI Feedback States
  const [loading, setLoading] = useState(false);
  const [savingQuestion, setSavingQuestion] = useState(false);
  const [savingDraft, setSavingDraft] = useState(false);
  const [confirmingAnswer, setConfirmingAnswer] = useState(false);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [classificationResult, setClassificationResult] = useState<QuestionClassificationResult | null>(null);
  const [isEvaluatingEvidence, setIsEvaluatingEvidence] = useState(false);
  const [evidencePackage, setEvidencePackage] = useState<EvidencePackage | null>(null);
  const [allowSensitiveConsent, setAllowSensitiveConsent] = useState(false);
  const [allowHighlySensitiveConsent, setAllowHighlySensitiveConsent] = useState(false);
  const [allowExternalVisionConsent, setAllowExternalVisionConsent] = useState(false);
  const [isInspectingOutbound, setIsInspectingOutbound] = useState(false);
  const [outboundDisclosure, setOutboundDisclosure] = useState<OutboundDataDisclosure | null>(null);
  const [showOutboundModal, setShowOutboundModal] = useState(false);
  const [isGeneratingAnswer, setIsGeneratingAnswer] = useState(false);
  const [aiRecommendation, setAiRecommendation] = useState<ValidatedAnswerRecommendation | null>(null);
  const [copiedToClipboard, setCopiedToClipboard] = useState(false);
  const [errorMessage, setErrorMessage] = useState("");
  const [successMessage, setSuccessMessage] = useState("");

  // SC-034: Review Actions & History States
  const [answerHistory, setAnswerHistory] = useState<AnswerHistoryItem[]>([]);
  const [recommendationRevisions, setRecommendationRevisions] = useState<ValidatedAnswerRecommendation[]>([]);
  const [loadingHistory, setLoadingHistory] = useState(false);
  const [historyActionFilter, setHistoryActionFilter] = useState<string>("all");
  const [activeTab, setActiveTab] = useState<"workspace" | "history" | "revisions">("workspace");

  // Reject Modal State
  const [showRejectModal, setShowRejectModal] = useState(false);
  const [rejectReasonInput, setRejectReasonInput] = useState("");
  const [isRejecting, setIsRejecting] = useState(false);

  // Regenerate Modal State
  const [showRegenerateModal, setShowRegenerateModal] = useState(false);
  const [regenerateReasonInput, setRegenerateReasonInput] = useState("");
  const [isRegenerating, setIsRegenerating] = useState(false);

  // SC-040: Screen Capture States
  const [isCapturing, setIsCapturing] = useState(false);
  const [showWindowPicker, setShowWindowPicker] = useState(false);
  const [availableWindows, setAvailableWindows] = useState<WindowInfo[]>([]);
  const [loadingWindows, setLoadingWindows] = useState(false);
  const [activeCapturePreview, setActiveCapturePreview] = useState<CapturePreview | null>(null);
  const [showCapturePreviewModal, setShowCapturePreviewModal] = useState(false);
  const [activeCaptureId, setActiveCaptureId] = useState<string | null>(null);

  // SC-041: OCR Extraction & Review States
  const [isExtractingOcr, setIsExtractingOcr] = useState(false);
  const [activeOcrExtraction, setActiveOcrExtraction] = useState<ExtractedSurveyQuestion | null>(null);
  const [showOcrReviewModal, setShowOcrReviewModal] = useState(false);
  const [ocrCorrectedText, setOcrCorrectedText] = useState("");
  const [ocrCorrectedInstructions, setOcrCorrectedInstructions] = useState("");
  const [ocrCorrectedOptions, setOcrCorrectedOptions] = useState<string[]>([]);
  const [ocrError, setOcrError] = useState("");

  // SC-053: Survey Memory & Recognition States
  const [recognitionEvaluation, setRecognitionEvaluation] = useState<SurveyRecognitionEvaluation | null>(null);
  const [isEvaluatingRecognition, setIsEvaluatingRecognition] = useState(false);
  const [showRecognitionModal, setShowRecognitionModal] = useState(false);

  // SC-054: Next Question Intelligence & Branch Prediction States
  const [nqiResult, setNqiResult] = useState<NextQuestionIntelligenceResult | null>(null);
  const [isLoadingNqi, setIsLoadingNqi] = useState(false);

  // Keyboard navigation & accessibility: Escape key closes active modal
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        if (showRejectModal) setShowRejectModal(false);
        else if (showRegenerateModal) setShowRegenerateModal(false);
        else if (showOutboundModal) setShowOutboundModal(false);
        else if (showRecognitionModal) setShowRecognitionModal(false);
        else if (showOcrReviewModal) setShowOcrReviewModal(false);
        else if (showCapturePreviewModal) setShowCapturePreviewModal(false);
        else if (showWindowPicker) setShowWindowPicker(false);
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [
    showRejectModal,
    showRegenerateModal,
    showOutboundModal,
    showRecognitionModal,
    showOcrReviewModal,
    showCapturePreviewModal,
    showWindowPicker,
  ]);

  async function evaluateNextQuestionPredictions(questionId?: string, currentAnswer?: string | null) {
    const targetQId = questionId || activeQuestionId;
    if (!activePersona || !selectedSessionId || !targetQId) {
      setNqiResult(null);
      return;
    }
    setIsLoadingNqi(true);
    try {
      const result = await surveyService.predictNextQuestions({
        personaId: activePersona.id,
        sessionId: selectedSessionId,
        currentQuestionId: targetQId,
        currentAnswer: currentAnswer || undefined,
      });
      setNqiResult(result);
    } catch (err) {
      console.warn("Gagal mengevaluasi prediksi NQI:", err);
    } finally {
      setIsLoadingNqi(false);
    }
  }

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

      if (detail.questions.length > 0) {
        selectQuestionForEditing(detail.questions[0]);
      } else {
        resetQuestionForm();
      }
      setErrorMessage("");

      // Evaluasi pengenalan survei historis (SC-053)
      void evaluateRecognition(sessionId);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setLoading(false);
    }
  }

  // SC-053: Evaluasi pengenalan survei historis terhadap memori
  async function evaluateRecognition(sessionId?: string) {
    const sId = sessionId || currentSession?.id;
    if (!activePersona || !sId) return;
    try {
      setIsEvaluatingRecognition(true);
      const result = await surveyService.recognizeSurvey({
        personaId: activePersona.id,
        sessionId: sId,
      });
      setRecognitionEvaluation(result);
    } catch (err) {
      console.warn("Gagal mengevaluasi pengenalan survei:", err);
    } finally {
      setIsEvaluatingRecognition(false);
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
    setEditReasonInput("");
    setClassificationResult(null);
    setEvidencePackage(null);
    setAiRecommendation(null);
    setAnswerHistory([]);
    setRecommendationRevisions([]);
    setIsAnalyzing(false);
    setIsEvaluatingEvidence(false);
    setIsGeneratingAnswer(false);
    setShowRejectModal(false);
    setShowRegenerateModal(false);
    setActiveCaptureId(null);
    setActiveCapturePreview(null);
    setShowCapturePreviewModal(false);
    setShowWindowPicker(false);
    setNqiResult(null);
    setIsLoadingNqi(false);
  }

  // Muat riwayat dan revisi untuk pertanyaan aktif (SC-034)
  async function loadQuestionHistoryAndRevisions(questionId: string) {
    if (!activePersona || !questionId) return;
    try {
      setLoadingHistory(true);
      const [history, revisions] = await Promise.all([
        surveyService.listAnswerHistory({
          personaId: activePersona.id,
          questionId,
        }),
        surveyService.listRecommendationRevisions(activePersona.id, questionId),
      ]);
      setAnswerHistory(history);
      setRecommendationRevisions(revisions);
    } catch (err) {
      console.error("Gagal memuat riwayat pertanyaan:", err);
    } finally {
      setLoadingHistory(false);
    }
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
    setEditReasonInput("");
    setEvidencePackage(null);
    setAiRecommendation(null);

    if (q.captureId) {
      setActiveCaptureId(q.captureId);
      void captureService
        .getPreview(q.captureId)
        .then((prev) => setActiveCapturePreview(prev))
        .catch(() => setActiveCapturePreview(null));
    } else {
      setActiveCaptureId(null);
      setActiveCapturePreview(null);
    }

    if (activePersona && q.id) {
      void surveyService.getLatestAnswerRecommendation(activePersona.id, q.id)
        .then((rec) => {
          if (rec) setAiRecommendation(rec);
        })
        .catch(() => {});

      void loadQuestionHistoryAndRevisions(q.id);
    }

    if (q.structureJson && typeof q.structureJson === "object") {
      setClassificationResult(q.structureJson as unknown as QuestionClassificationResult);
    } else {
      setClassificationResult(null);
    }

    void evaluateNextQuestionPredictions(q.id, q.userAnswerConfirmed || q.userAnswerDraft || null);
  }

  // SC-040/041: one explicit capture action starts preview and local OCR.
  // OCR remains local and only prepares a draft for review; it never starts AI analysis.
  async function handleStartRegionCapture() {
    if (!activePersona) return;
    const sessionId = currentSession?.id || selectedSessionId;
    if (!sessionId) {
      setErrorMessage("Buat atau pilih sesi survei sebelum mengambil tangkapan layar.");
      return;
    }

    try {
      setIsCapturing(true);
      setOcrError("");
      setErrorMessage("");
      const preview = await captureService.captureRegion({
        sessionId,
        rect: null,
      });
      setActiveCapturePreview(preview);
      setActiveCaptureId(preview.captureId);
      setActiveOcrExtraction(null);
      setShowCapturePreviewModal(true);
      setShowOcrReviewModal(false);
      setSuccessMessage("Tangkapan layar berhasil. OCR lokal dimulai otomatis; hasil akan muncul untuk ditinjau.");
      setTimeout(() => setSuccessMessage(""), 4000);
      await handleExtractOcr(preview.captureId);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsCapturing(false);
    }
  }

  async function handleOpenWindowPicker() {
    if (!activePersona) return;
    if (!currentSession?.id && !selectedSessionId) {
      setErrorMessage("Buat atau pilih sesi survei sebelum mengambil tangkapan layar.");
      return;
    }
    try {
      setLoadingWindows(true);
      setErrorMessage("");
      const windows = await captureService.listWindows();
      setAvailableWindows(windows);
      setShowWindowPicker(true);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setLoadingWindows(false);
    }
  }

  async function handleSelectWindowToCapture(windowId: number) {
    const sessionId = currentSession?.id || selectedSessionId;
    if (!activePersona || !sessionId) {
      setShowWindowPicker(false);
      setErrorMessage("Buat atau pilih sesi survei sebelum mengambil tangkapan layar.");
      return;
    }

    try {
      setIsCapturing(true);
      setShowWindowPicker(false);
      setOcrError("");
      setErrorMessage("");
      const preview = await captureService.captureWindow({
        sessionId,
        windowId,
      });
      setActiveCapturePreview(preview);
      setActiveCaptureId(preview.captureId);
      setShowCapturePreviewModal(true);
      setShowOcrReviewModal(false);
      setSuccessMessage("Jendela berhasil ditangkap. OCR lokal dimulai otomatis; hasil akan muncul untuk ditinjau.");
      setTimeout(() => setSuccessMessage(""), 4000);
      await handleExtractOcr(preview.captureId);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsCapturing(false);
    }
  }

  async function handleDiscardCapture() {
    if (!activeCapturePreview) return;
    try {
      setErrorMessage("");
      await captureService.discardCapture(activeCapturePreview.captureId);
      setActiveCapturePreview(null);
      setActiveCaptureId(null);
      setShowCapturePreviewModal(false);
      setSuccessMessage("Tangkapan layar dibatalkan dan berkas sementara telah dihapus permanen dari disk.");
      setTimeout(() => setSuccessMessage(""), 3500);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    }
  }

  function handleConfirmCaptureUse() {
    if (!activeCapturePreview) return;
    setActiveCaptureId(activeCapturePreview.captureId);
    setShowCapturePreviewModal(false);
    setSuccessMessage("Tangkapan layar berhasil ditautkan ke pertanyaan survei.");
    setTimeout(() => setSuccessMessage(""), 3500);
  }

  async function handleExtractOcr(targetCaptureId?: string) {
    const cid = targetCaptureId || activeCapturePreview?.captureId || activeCaptureId;
    if (!cid) {
      setErrorMessage("Tidak ada tangkapan layar aktif untuk diekstraksi.");
      return;
    }

    try {
      setIsExtractingOcr(true);
      setOcrError("");
      setErrorMessage("");
      setActiveCaptureId(cid);

      const extraction = await ocrService.extractCapture({
        captureId: cid,
        sessionId: currentSession?.id || selectedSessionId || undefined,
        personaId: activePersona?.id,
      });

      setActiveOcrExtraction(extraction);
      setOcrCorrectedText(extraction.questionTextOriginal);
      setOcrCorrectedInstructions(extraction.instructions || "");
      setOcrCorrectedOptions([...extraction.answerOptions]);
      setShowOcrReviewModal(true);
      setShowCapturePreviewModal(false);
      setSuccessMessage("Ekstraksi OCR dan rekonstruksi layout berhasil. Silakan tinjau dan koreksi hasil jika diperlukan.");
      setTimeout(() => setSuccessMessage(""), 4000);
    } catch (err) {
      const appErr = AppError.from(err);
      setOcrError(appErr.message);
      setErrorMessage(`Gagal melakukan OCR: ${appErr.message}. Anda tetap dapat menggunakan input teks manual (AT-053).`);
    } finally {
      setIsExtractingOcr(false);
    }
  }

  async function handleConfirmOcrUse(recordToSurvey: boolean) {
    if (!activeOcrExtraction) return;

    if (!recordToSurvey) {
      setQuestionTextOriginal(activeOcrExtraction.questionTextOriginal);
      setQuestionTextCorrected(ocrCorrectedText);
      setInstructions(ocrCorrectedInstructions);
      setOptions([...ocrCorrectedOptions]);
      setQuestionType(activeOcrExtraction.questionType);
      setAnswerFormat(activeOcrExtraction.answerFormat);
      if (activeCapturePreview) {
        setActiveCaptureId(activeCapturePreview.captureId);
      }
      setShowOcrReviewModal(false);
      setSuccessMessage("Hasil ekstraksi OCR berhasil diterapkan ke editor formulir pertanyaan.");
      setTimeout(() => setSuccessMessage(""), 4000);
      return;
    }

    if (!selectedSessionId || !activePersona) {
      setErrorMessage("Pilih sesi survei aktif terlebih dahulu untuk merekam pertanyaan.");
      return;
    }

    try {
      setLoading(true);
      setErrorMessage("");

      const confirmedQuestion = await ocrService.confirmExtraction({
        extractionId: activeOcrExtraction.extractionId,
        sessionId: selectedSessionId,
        personaId: activePersona.id,
        captureId: activeOcrExtraction.captureId,
        questionTextCorrected: ocrCorrectedText.trim(),
        instructions: ocrCorrectedInstructions.trim() || undefined,
        options: ocrCorrectedOptions.filter((o) => o.trim().length > 0),
        questionType: activeOcrExtraction.questionType,
        answerFormat: activeOcrExtraction.answerFormat,
        scaleMin: activeOcrExtraction.scale?.minValue,
        scaleMax: activeOcrExtraction.scale?.maxValue,
        scaleLabels: activeOcrExtraction.scale?.allLabels,
        matrixRows: activeOcrExtraction.matrix?.rows,
        matrixColumns: activeOcrExtraction.matrix?.columns,
        visibleUnits: activeOcrExtraction.visibleUnits,
      });

      setShowOcrReviewModal(false);
      await loadSessionDetails(selectedSessionId);
      selectQuestionForEditing(confirmedQuestion);
      setSuccessMessage("Pertanyaan survei hasil OCR berhasil dikonfirmasi dan direkam ke sesi aktif.");
      setTimeout(() => setSuccessMessage(""), 4000);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setLoading(false);
    }
  }

  async function handleEvaluateEvidence() {
    if (!activePersona) return;
    if (!questionTextOriginal.trim()) {
      setErrorMessage("Ketikkan teks pertanyaan sebelum mencari bukti fakta profil.");
      return;
    }

    try {
      setIsEvaluatingEvidence(true);
      setErrorMessage("");
      const pkg = await surveyService.evaluateEvidence({
        personaId: activePersona.id,
        sessionId: currentSession?.id || null,
        questionId: activeQuestionId || null,
        questionText: questionTextOriginal.trim(),
        instructions: instructions.trim() || null,
        options: options.length > 0 ? options : null,
        classification: classificationResult,
        consent: {
          allowSensitive: allowSensitiveConsent,
          allowHighlySensitive: allowHighlySensitiveConsent,
          allowExternalVision: allowExternalVisionConsent,
        },
      });

      setEvidencePackage(pkg);
      setSuccessMessage("Pencarian bukti profil & evaluasi konsistensi berhasil.");
      setTimeout(() => setSuccessMessage(""), 3000);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsEvaluatingEvidence(false);
    }
  }

  async function handleInspectOutbound() {
    if (!activePersona) return;
    const trimmed = questionTextOriginal.trim();
    if (!trimmed) {
      setErrorMessage("Ketikkan teks pertanyaan sebelum melakukan inspeksi transmisi data.");
      return;
    }
    try {
      setIsInspectingOutbound(true);
      setErrorMessage("");
      const disclosure = await PrivacyService.inspectOutboundData({
        personaId: activePersona.id,
        questionText: trimmed,
        instructions: instructions.trim() || undefined,
        options: options.length > 0 ? options : undefined,
        allowSensitive: allowSensitiveConsent,
        allowHighlySensitive: allowHighlySensitiveConsent,
        allowExternalVision: allowExternalVisionConsent,
        hasImageAttachment: Boolean(activeCapturePreview),
      });
      setOutboundDisclosure(disclosure);
      setShowOutboundModal(true);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsInspectingOutbound(false);
    }
  }

  async function handleGenerateAnswer() {
    if (!activePersona) return;
    const trimmed = questionTextOriginal.trim();
    if (!trimmed) {
      setErrorMessage("Ketikkan teks pertanyaan sebelum menghasilkan rekomendasi AI.");
      return;
    }

    try {
      setIsGeneratingAnswer(true);
      setErrorMessage("");
      const rec = await surveyService.generateAnswer({
        personaId: activePersona.id,
        sessionId: currentSession?.id || null,
        questionId: activeQuestionId || null,
        questionText: trimmed,
        instructions: instructions.trim() || null,
        options: options.length > 0 ? options : null,
        consent: {
          allowSensitive: allowSensitiveConsent,
          allowHighlySensitive: allowHighlySensitiveConsent,
          allowExternalVision: allowExternalVisionConsent,
        },
      });

      setAiRecommendation(rec);
      setSuccessMessage("Rekomendasi AI terstruktur berhasil dirumuskan.");
      setTimeout(() => setSuccessMessage(""), 3000);

      if (activeQuestionId) {
        await loadQuestionHistoryAndRevisions(activeQuestionId);
      }
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsGeneratingAnswer(false);
    }
  }

  function handleApplyDraftToUserAnswer(draft: string) {
    setUserAnswerDraft(draft);
    setSuccessMessage("Draf rekomendasi disalin ke kolom jawaban aktual pengguna.");
    setTimeout(() => setSuccessMessage(""), 3000);
  }

  // SC-034 Review Action: Copy Answer
  async function handleCopyRecommendationToClipboard(text: string) {
    if (!text.trim()) return;
    try {
      await navigator.clipboard.writeText(text);
      setCopiedToClipboard(true);
      setTimeout(() => setCopiedToClipboard(false), 2000);

      // Catat aksi copy ke audit history
      if (activePersona && activeQuestionId) {
        await surveyService.copyAnswer({
          personaId: activePersona.id,
          questionId: activeQuestionId,
          recommendationId: aiRecommendation?.id || null,
          copiedText: text,
        });
        await loadQuestionHistoryAndRevisions(activeQuestionId);
      }

      setSuccessMessage("Jawaban berhasil disalin ke clipboard dan dicatat di riwayat.");
      setTimeout(() => setSuccessMessage(""), 3000);
    } catch {
      setErrorMessage("Gagal menyalin ke clipboard.");
    }
  }

  // SC-034 Review Action: Edit Answer
  async function handleEditAnswer() {
    if (!activePersona || !activeQuestionId) {
      setErrorMessage("Pilih pertanyaan terlebih dahulu sebelum menyimpan draf jawaban.");
      return;
    }

    const trimmed = userAnswerDraft.trim();
    if (!trimmed) {
      setErrorMessage("Tuliskan jawaban aktual yang ingin disimpan.");
      return;
    }

    try {
      setSavingDraft(true);
      setErrorMessage("");

      await surveyService.editAnswer({
        personaId: activePersona.id,
        questionId: activeQuestionId,
        recommendationId: aiRecommendation?.id || null,
        actualAnswer: trimmed,
        reason: editReasonInput.trim() || null,
      });

      setSuccessMessage("Hasil suntingan jawaban aktual pengguna berhasil disimpan ke riwayat.");
      setTimeout(() => setSuccessMessage(""), 3500);

      // Refresh riwayat & pertanyaan
      await loadQuestionHistoryAndRevisions(activeQuestionId);
      if (currentSession) {
        const refreshed = await surveyService.listQuestions(activePersona.id, currentSession.id);
        setQuestions(refreshed);
      }
      void evaluateNextQuestionPredictions(activeQuestionId, userAnswerDraft);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setSavingDraft(false);
    }
  }

  // SC-034 Review Action: Confirm Answer
  async function handleConfirmAnswer() {
    if (!activePersona || !activeQuestionId) {
      setErrorMessage("Pilih pertanyaan terlebih dahulu.");
      return;
    }

    const answerToConfirm = userAnswerDraft.trim();
    if (!answerToConfirm) {
      setErrorMessage("Tuliskan jawaban aktual yang ingin dikonfirmasi.");
      return;
    }

    try {
      setConfirmingAnswer(true);
      setErrorMessage("");

      await surveyService.confirmAnswer({
        personaId: activePersona.id,
        questionId: activeQuestionId,
        recommendationId: aiRecommendation?.id || null,
        actualAnswer: answerToConfirm,
      });

      setUserAnswerConfirmed(answerToConfirm);
      setConfirmationStatus("confirmed");
      setConfirmedAt(new Date().toISOString());

      setSuccessMessage("Persetujuan draf jawaban berhasil dicatat secara internal.");
      setTimeout(() => setSuccessMessage(""), 4000);

      await loadQuestionHistoryAndRevisions(activeQuestionId);
      if (currentSession) {
        const refreshed = await surveyService.listQuestions(activePersona.id, currentSession.id);
        setQuestions(refreshed);
      }
      void evaluateNextQuestionPredictions(activeQuestionId, answerToConfirm);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setConfirmingAnswer(false);
    }
  }

  // SC-034 Review Action: Reject Answer
  async function handleRejectAnswer() {
    if (!activePersona || !activeQuestionId) return;

    try {
      setIsRejecting(true);
      setErrorMessage("");

      await surveyService.rejectAnswer({
        personaId: activePersona.id,
        questionId: activeQuestionId,
        recommendationId: aiRecommendation?.id || null,
        reason: rejectReasonInput.trim() || null,
      });

      setConfirmationStatus("rejected");
      setShowRejectModal(false);
      setRejectReasonInput("");
      setSuccessMessage("Rekomendasi AI berhasil ditolak dan dicatat di riwayat.");
      setTimeout(() => setSuccessMessage(""), 4000);

      await loadQuestionHistoryAndRevisions(activeQuestionId);
      if (currentSession) {
        const refreshed = await surveyService.listQuestions(activePersona.id, currentSession.id);
        setQuestions(refreshed);
      }
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsRejecting(false);
    }
  }

  // SC-034 Review Action: Regenerate Answer
  async function handleRegenerateAnswer() {
    if (!activePersona || !activeQuestionId) return;
    const trimmed = questionTextOriginal.trim();
    if (!trimmed) {
      setErrorMessage("Teks pertanyaan tidak boleh kosong.");
      return;
    }

    try {
      setIsRegenerating(true);
      setErrorMessage("");

      const newRec = await surveyService.regenerateAnswer({
        personaId: activePersona.id,
        sessionId: currentSession?.id || null,
        questionId: activeQuestionId,
        questionText: trimmed,
        instructions: instructions.trim() || null,
        options: options.length > 0 ? options : null,
        consent: {
          allowSensitive: allowSensitiveConsent,
          allowHighlySensitive: allowHighlySensitiveConsent,
          allowExternalVision: allowExternalVisionConsent,
        },
        reason: regenerateReasonInput.trim() || null,
      });

      setAiRecommendation(newRec);
      setShowRegenerateModal(false);
      setRegenerateReasonInput("");
      setSuccessMessage("Revisi rekomendasi baru berhasil dirumuskan tanpa menghilangkan riwayat sebelumnya.");
      setTimeout(() => setSuccessMessage(""), 4000);

      await loadQuestionHistoryAndRevisions(activeQuestionId);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsRegenerating(false);
    }
  }

  async function handleAnalyzeQuestion() {
    if (!activePersona) return;
    if (!questionTextOriginal.trim()) {
      setErrorMessage("Ketikkan teks pertanyaan sebelum melakukan analisis kategori & batasan.");
      return;
    }

    try {
      setIsAnalyzing(true);
      setErrorMessage("");
      const result = await surveyService.classifyQuestion({
        personaId: activePersona.id,
        sessionId: currentSession?.id || null,
        questionId: activeQuestionId || null,
        questionText: questionTextOriginal.trim(),
        instructions: instructions.trim() || null,
        options: options.length > 0 ? options : null,
      });

      setClassificationResult(result);
      setQuestionType(result.questionType);
      setAnswerFormat(result.answerFormat);
      setSuccessMessage("Analisis kategori dan batasan berhasil dieksekusi.");
      setTimeout(() => setSuccessMessage(""), 3000);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsAnalyzing(false);
    }
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

        const refreshedList = await surveyService.listQuestions(activePersona.id, currentSession.id);
        setQuestions(refreshedList);
        const refreshedQ = refreshedList.find((q) => q.id === updated.id);
        if (refreshedQ) selectQuestionForEditing(refreshedQ);
        void evaluateRecognition(currentSession.id);
      } else {
        const createInput: CreateQuestionInput = {
          personaId: activePersona.id,
          sessionId: currentSession.id,
          captureId: activeCaptureId || null,
          questionTextOriginal: questionTextOriginal.trim(),
          instructions: instructions.trim() || null,
          options,
          questionType,
          answerFormat,
          provenance: activeCaptureId ? "ocr_detected" : "manual_entry",
        };

        const created = await surveyService.createQuestion(createInput);
        setSuccessMessage(`Pertanyaan #${created.questionOrder} berhasil disimpan ke sesi.`);
        setTimeout(() => setSuccessMessage(""), 4000);

        const refreshedList = await surveyService.listQuestions(activePersona.id, currentSession.id);
        setQuestions(refreshedList);
        const refreshedQ = refreshedList.find((q) => q.id === created.id);
        if (refreshedQ) selectQuestionForEditing(refreshedQ);
        void evaluateRecognition(currentSession.id);
      }
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setSavingQuestion(false);
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
      void evaluateRecognition(currentSession.id);
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
    if (!window.confirm(`Arsipkan sesi "${currentSession.title}"?`)) return;

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

  const filteredHistory = answerHistory.filter((item) => {
    if (historyActionFilter === "all") return true;
    return item.action === historyActionFilter;
  });

  if (!activePersona) {
    return (
      <section className="content empty-state" role="region" aria-label="Profil Responden Belum Dipilih">
        <div className="empty-icon" aria-hidden="true">👤</div>
        <h2>Pilih Profil Terlebih Dahulu</h2>
        <p>
          Survey Copilot bekerja berbasis profil responden nyata. Silakan pilih atau buat
          profil di menu <strong>Profil</strong> untuk memulai sesi survei.
        </p>
        {onNavigate && (
          <button
            type="button"
            className="primary-button"
            onClick={() => onNavigate("profiles")}
            aria-label="Buka menu profil responden"
            style={{ marginTop: "16px" }}
          >
            Buka Menu Profil
          </button>
        )}
      </section>
    );
  }

  return (
    <section className="content workspace">
      <div className="section-heading">
        <div>
          <span className="eyebrow">WORKSPACE COPILOT</span>
          <h2>Peninjauan Jawaban & Riwayat Sesi (SC-034)</h2>
        </div>
        <span className="coming-soon">Responden: <strong>{activePersona.displayName}</strong></span>
      </div>

      {/* Mode Switcher: Pertanyaan Sesi vs Peluang Survei (SC-050) */}
      <div style={{ display: "flex", gap: "10px", marginBottom: "20px", borderBottom: "1px solid #e2e8f0", paddingBottom: "12px" }}>
        <button
          type="button"
          onClick={() => setCopilotMode("questions")}
          style={{
            padding: "8px 16px",
            borderRadius: "6px",
            border: copilotMode === "questions" ? "2px solid #2563eb" : "1px solid #cbd5e1",
            background: copilotMode === "questions" ? "#eff6ff" : "#ffffff",
            color: copilotMode === "questions" ? "#1e40af" : "#475569",
            fontWeight: copilotMode === "questions" ? 700 : 500,
            cursor: "pointer",
          }}
        >
          📝 Asisten Pertanyaan Survei
        </button>
        <button
          type="button"
          onClick={() => setCopilotMode("opportunities")}
          style={{
            padding: "8px 16px",
            borderRadius: "6px",
            border: copilotMode === "opportunities" ? "2px solid #2563eb" : "1px solid #cbd5e1",
            background: copilotMode === "opportunities" ? "#eff6ff" : "#ffffff",
            color: copilotMode === "opportunities" ? "#1e40af" : "#475569",
            fontWeight: copilotMode === "opportunities" ? 700 : 500,
            cursor: "pointer",
          }}
        >
          🎯 Peluang Survei & Kriteria (SC-050)
        </button>
      </div>

      {copilotMode === "opportunities" ? (
        <OpportunityView activePersona={activePersona} />
      ) : (
        <>
      {errorMessage && (
        <div className="notice" role="alert" style={{ background: "#fdf2f2", color: "#a12020", marginBottom: "16px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
            <span aria-hidden="true">⚠️</span>
            <p style={{ margin: 0 }}>{errorMessage}</p>
          </div>
          <button
            type="button"
            onClick={() => setErrorMessage("")}
            aria-label="Tutup pesan peringatan"
            style={{ background: "transparent", border: 0, cursor: "pointer", color: "#a12020", fontWeight: "bold" }}
          >
            ✕
          </button>
        </div>
      )}

      {successMessage && (
        <div className="notice" role="status" aria-live="polite" style={{ background: "#eef8f2", color: "#156236", marginBottom: "16px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
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

      {/* Modal Pembuatan Sesi Baru */}
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
                  placeholder="Contoh: Survei Gaya Hidup & Konsumsi 2026"
                  value={newSessionTitle}
                  onChange={(e) => setNewSessionTitle(e.target.value)}
                  style={{ width: "100%", padding: "10px 12px", border: "1px solid #dce4de", borderRadius: "8px" }}
                  required
                />
              </div>

              <div>
                <label className="field-label" htmlFor="new-session-platform">
                  Platform / Mitra (Opsional)
                </label>
                <input
                  id="new-session-platform"
                  type="text"
                  placeholder="Contoh: YouGov, Toluna, Milieu"
                  value={newSessionPlatform}
                  onChange={(e) => setNewSessionPlatform(e.target.value)}
                  style={{ width: "100%", padding: "10px 12px", border: "1px solid #dce4de", borderRadius: "8px" }}
                />
              </div>

              <div>
                <label className="field-label" htmlFor="new-session-url">
                  URL Survei (Opsional Referensi)
                </label>
                <input
                  id="new-session-url"
                  type="text"
                  placeholder="https://..."
                  value={newSessionUrl}
                  onChange={(e) => setNewSessionUrl(e.target.value)}
                  style={{ width: "100%", padding: "10px 12px", border: "1px solid #dce4de", borderRadius: "8px" }}
                />
              </div>

              <div style={{ gridColumn: "span 2" }}>
                <label className="field-label" htmlFor="new-session-notes">
                  Catatan Sesi (Opsional)
                </label>
                <textarea
                  id="new-session-notes"
                  placeholder="Catatan topik atau kriteria khusus..."
                  value={newSessionNotes}
                  onChange={(e) => setNewSessionNotes(e.target.value)}
                  style={{ minHeight: "60px" }}
                />
              </div>
            </div>

            <div style={{ display: "flex", gap: "10px", marginTop: "14px" }}>
              <button type="submit" className="primary-button" disabled={loading} style={{ marginTop: 0 }}>
                {loading ? "Menyimpan..." : "Simpan Sesi"}
              </button>
              <button
                type="button"
                className="secondary-button"
                onClick={() => setIsCreatingSession(false)}
                disabled={loading}
              >
                Batal
              </button>
            </div>
          </form>
        </div>
      )}

      {/* Konten Utama Workspace */}
      {!currentSession ? (
        <div className="empty-inline" style={{ padding: "40px 20px" }}>
          <p>Belum ada sesi survei yang dipilih. Silakan buat atau pilih sesi survei di atas.</p>
        </div>
      ) : (
        <>
          {/* SC-053: Survey Memory & Recognition Banner */}
          <div
            style={{
              marginBottom: "16px",
              padding: "16px 20px",
              borderRadius: "12px",
              border: "1px solid #d1fae5",
              background: "#f0fdf4",
              boxShadow: "0 2px 6px rgba(0,0,0,0.02)",
            }}
          >
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                flexWrap: "wrap",
                gap: "12px",
              }}
            >
              <div>
                <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "6px" }}>
                  <span style={{ fontSize: "14px", fontWeight: "800", color: "#065f46" }}>
                    🧠 Survey Memory (SC-053)
                  </span>
                  <span
                    className="survey-badge"
                    style={{ background: "#dcfce7", color: "#166534", fontWeight: "700" }}
                  >
                    Versi {currentSession.version}
                  </span>
                  {currentSession.sourceDomain && (
                    <span className="survey-badge" style={{ background: "#e0e7ff", color: "#3730a3" }}>
                      🔗 {currentSession.sourceDomain}
                    </span>
                  )}
                  {currentSession.fingerprint && (
                    <span
                      className="survey-badge"
                      style={{ background: "#f1f5f9", color: "#475569", fontFamily: "monospace", fontSize: "11px" }}
                      title={`Sidik Jari Struktural: ${currentSession.fingerprint}`}
                    >
                      🧬 {currentSession.fingerprint}
                    </span>
                  )}
                </div>
                <p style={{ margin: 0, fontSize: "13px", color: "#166534" }}>
                  {recognitionEvaluation
                    ? recognitionEvaluation.explanation
                    : "Menganalisis pengenalan survei terhadap riwayat responden..."}
                </p>
              </div>

              <div style={{ display: "flex", gap: "8px", alignItems: "center", flexWrap: "wrap" }}>
                {recognitionEvaluation && (
                  <span
                    style={{
                      padding: "4px 10px",
                      borderRadius: "6px",
                      fontSize: "12px",
                      fontWeight: "700",
                      background:
                        recognitionEvaluation.matchStatus === "exact_match"
                          ? "#16a34a"
                          : recognitionEvaluation.matchStatus === "version_changed"
                          ? "#0284c7"
                          : recognitionEvaluation.matchStatus === "likely_match"
                          ? "#2563eb"
                          : recognitionEvaluation.matchStatus === "possible_match"
                          ? "#d97706"
                          : "#64748b",
                      color: "#ffffff",
                    }}
                  >
                    {recognitionEvaluation.matchStatus === "exact_match"
                      ? "🎯 100% Sama Persis"
                      : recognitionEvaluation.matchStatus === "version_changed"
                      ? `🔄 Versi Diperbarui (${(recognitionEvaluation.confidenceScore * 100).toFixed(0)}%)`
                      : recognitionEvaluation.matchStatus === "likely_match"
                      ? `💡 Kemiripan Kuat (${(recognitionEvaluation.confidenceScore * 100).toFixed(0)}%)`
                      : recognitionEvaluation.matchStatus === "possible_match"
                      ? `🔍 Kemiripan Sebagian (${(recognitionEvaluation.confidenceScore * 100).toFixed(0)}%)`
                      : "🆕 Survei Baru"}
                  </span>
                )}
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => void evaluateRecognition()}
                  disabled={isEvaluatingRecognition}
                  style={{ padding: "6px 12px", fontSize: "11px", marginTop: 0 }}
                  title="Jalankan ulang pencocokan sidik jari survei"
                >
                  {isEvaluatingRecognition ? "⏳ Menganalisis..." : "🔄 Cek Pengenalan"}
                </button>
                {recognitionEvaluation && recognitionEvaluation.matchedSessionId && (
                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => setShowRecognitionModal(true)}
                    style={{ padding: "6px 12px", fontSize: "11px", marginTop: 0, borderColor: "#86efac", color: "#15803d", fontWeight: "700" }}
                  >
                    🔍 Rincian ({recognitionEvaluation.recognizedAnswers.length} Jawaban Tersimpan)
                  </button>
                )}
              </div>
            </div>
          </div>

          <div className="survey-layout-grid">
          {/* Kolom Kiri: Pertanyaan & Peninjauan Rekomendasi */}
          <div className="question-editor-card">
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
              <h3 style={{ margin: 0, fontSize: "16px", fontWeight: "800" }}>
                {activeQuestionId ? `Pertanyaan #${questions.find((q) => q.id === activeQuestionId)?.questionOrder || ""}` : "Input Pertanyaan Survei"}
              </h3>
              {activeQuestionId && (
                <button
                  type="button"
                  className="secondary-button"
                  onClick={resetQuestionForm}
                  style={{ padding: "6px 12px", fontSize: "11px" }}
                >
                  + Tambah Pertanyaan Baru
                </button>
              )}
            </div>

            {/* Tab Navigasi: Workspace vs Riwayat vs Revisi */}
            {activeQuestionId && (
              <div style={{ display: "flex", gap: "8px", borderBottom: "1px solid var(--line)", marginBottom: "16px", paddingBottom: "8px" }}>
                <button
                  type="button"
                  className={`secondary-button ${activeTab === "workspace" ? "primary-button" : ""}`}
                  onClick={() => setActiveTab("workspace")}
                  style={{ padding: "6px 14px", fontSize: "12px", marginTop: 0 }}
                >
                  📝 Peninjauan & Editor
                </button>
                <button
                  type="button"
                  className={`secondary-button ${activeTab === "history" ? "primary-button" : ""}`}
                  onClick={() => setActiveTab("history")}
                  style={{ padding: "6px 14px", fontSize: "12px", marginTop: 0 }}
                >
                  📜 Riwayat Perubahan ({answerHistory.length})
                </button>
                <button
                  type="button"
                  className={`secondary-button ${activeTab === "revisions" ? "primary-button" : ""}`}
                  onClick={() => setActiveTab("revisions")}
                  style={{ padding: "6px 14px", fontSize: "12px", marginTop: 0 }}
                >
                  🔄 Revisi Draf AI ({recommendationRevisions.length})
                </button>
              </div>
            )}

            {activeTab === "workspace" && (
              <>
                {/* SC-040: Screen Capture Action Bar */}
                <div className="capture-action-bar">
                  <div className="capture-btn-group">
                    <button
                      type="button"
                      className="capture-btn primary"
                      onClick={() => void handleStartRegionCapture()}
                      disabled={isCapturing}
                      title="Tangkap seluruh tampilan layar atau area survei"
                    >
                      <span>📸</span>
                      <span>{isCapturing ? "Mengambil Tangkapan..." : "Tangkap Area Layar"}</span>
                    </button>

                    <button
                      type="button"
                      className="capture-btn"
                      onClick={() => void handleOpenWindowPicker()}
                      disabled={isCapturing || loadingWindows}
                      title="Pilih jendela browser atau aplikasi survei aktif"
                    >
                      <span>🪟</span>
                      <span>{loadingWindows ? "Membaca Jendela..." : "Tangkap Jendela Aplikasi"}</span>
                    </button>
                  </div>

                  <span className="capture-boundary-pill" title="Batasan privasi SC-040: Screenshot hanya ditangkap atas tindakan eksplisit pengguna. Tidak ada perekaman latar belakang tersembunyi.">
                    🔒 Tindakan Eksplisit Pengguna
                  </span>
                </div>

                {/* SC-040: Active Capture Preview Banner */}
                {activeCapturePreview && (
                  <div className="capture-preview-banner">
                    {activeCapturePreview.previewDataUrl ? (
                      <img
                        src={activeCapturePreview.previewDataUrl}
                        alt="Tangkapan Layar"
                        className="capture-thumbnail"
                        onClick={() => setShowCapturePreviewModal(true)}
                        title="Klik untuk memperbesar pratinjau"
                      />
                    ) : (
                      <div className="capture-thumbnail" style={{ display: "grid", placeItems: "center", color: "#94a3b8", fontSize: "20px" }}>
                        🖼️
                      </div>
                    )}

                    <div className="capture-preview-details">
                      <span className="capture-preview-title">
                        {activeCapturePreview.targetLabel || (activeCapturePreview.sourceMode === "window" ? "Tangkapan Jendela" : "Tangkapan Layar")}
                      </span>
                      <span className="capture-preview-meta">
                        {activeCapturePreview.width > 0 ? `${activeCapturePreview.width} × ${activeCapturePreview.height} px • ` : ""}
                        {(activeCapturePreview.fileSizeBytes / 1024).toFixed(1)} KB • Disimpan di folder aman aplikasi
                      </span>
                    </div>

                    <div style={{ display: "flex", gap: "6px" }}>
                      <button
                        type="button"
                        className="primary-button"
                        style={{ padding: "6px 12px", fontSize: "11px", marginTop: 0 }}
                        onClick={() => void handleExtractOcr()}
                        disabled={isExtractingOcr}
                        title="Ekstrak teks, pilihan, skala, atau matriks dari gambar (SC-041)"
                      >
                        {isExtractingOcr ? "⏳ Mengekstrak..." : "⚡ Ekstrak Teks (OCR)"}
                      </button>
                      <button
                        type="button"
                        className="secondary-button"
                        style={{ padding: "6px 10px", fontSize: "11px", marginTop: 0 }}
                        onClick={() => setShowCapturePreviewModal(true)}
                      >
                        🔍 Lihat Pratinjau
                      </button>
                      <button
                        type="button"
                        className="secondary-button"
                        style={{ padding: "6px 10px", fontSize: "11px", marginTop: 0, color: "#9c2b28", borderColor: "#f6d2d0" }}
                        onClick={() => void handleDiscardCapture()}
                        title="Batalkan dan hapus berkas tangkapan sementara"
                      >
                        🗑️ Batalkan
                      </button>
                    </div>
                  </div>
                )}

                {/* Teks Pertanyaan Asli */}
                <label className="field-label" htmlFor="question-original">
                  Teks Pertanyaan Asli (Persis Sesuai Survei) *
                </label>
                <textarea
                  id="question-original"
                  value={questionTextOriginal}
                  onChange={(e) => setQuestionTextOriginal(e.target.value)}
                  placeholder="Ketik atau tempel teks pertanyaan survei di sini..."
                  style={{ minHeight: "90px", marginBottom: "14px" }}
                />

                {/* Koreksi Teks Opsional */}
                {activeQuestionId && (
                  <div style={{ marginBottom: "14px" }}>
                    <label className="field-label" htmlFor="question-corrected">
                      Koreksi Teks Pertanyaan (Opsional Manual)
                    </label>
                    <input
                      id="question-corrected"
                      type="text"
                      value={questionTextCorrected}
                      onChange={(e) => setQuestionTextCorrected(e.target.value)}
                      placeholder="Koreksi ejaan jika diperlukan"
                      style={{ width: "100%", padding: "9px 12px", border: "1px solid #dce4de", borderRadius: "8px" }}
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
                  placeholder="Contoh: Pilih 1 jawaban saja, Tuliskan dalam angka..."
                  style={{ width: "100%", padding: "9px 12px", border: "1px solid #dce4de", borderRadius: "8px", marginBottom: "14px" }}
                />

                {/* Tipe & Format Pertanyaan */}
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "14px", marginBottom: "14px" }}>
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
                      style={{ flex: 1, padding: "8px 12px", border: "1px solid #dce4de", borderRadius: "8px" }}
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

                {/* Action Buttons: Analisis & Rekomendasi */}
                <div style={{ marginTop: "14px", marginBottom: "16px", display: "flex", gap: "10px", alignItems: "center", flexWrap: "wrap" }}>
                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => void handleAnalyzeQuestion()}
                    disabled={isAnalyzing || !questionTextOriginal.trim()}
                    style={{ background: "#f0f7f3", borderColor: "#a9d5bc", color: "#135d36" }}
                  >
                    {isAnalyzing ? "Menganalisis..." : "🔍 Analisis Kategori & Batasan (SC-031)"}
                  </button>

                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => void handleEvaluateEvidence()}
                    disabled={isEvaluatingEvidence || !questionTextOriginal.trim()}
                    style={{ background: "#f5f8ff", borderColor: "#adc6ea", color: "#194d82" }}
                  >
                    {isEvaluatingEvidence ? "Mengevaluasi Bukti..." : "🎯 Cari Bukti Profil (SC-032)"}
                  </button>

                  <button
                    type="button"
                    className="primary-button"
                    onClick={() => void handleGenerateAnswer()}
                    disabled={isGeneratingAnswer || !questionTextOriginal.trim()}
                    style={{ marginTop: 0, background: "#175438", borderColor: "#175438" }}
                  >
                    {isGeneratingAnswer ? "🤖 Merumuskan..." : "🤖 Rekomendasi Jawaban AI (SC-033)"}
                  </button>

                  <label style={{ fontSize: "11px", display: "flex", alignItems: "center", gap: "4px", color: "var(--muted)", cursor: "pointer" }}>
                    <input
                      type="checkbox"
                      checked={allowSensitiveConsent}
                      onChange={(e) => setAllowSensitiveConsent(e.target.checked)}
                    />
                    Izinkan Sensitif
                  </label>

                  <label style={{ fontSize: "11px", display: "flex", alignItems: "center", gap: "4px", color: "var(--muted)", cursor: "pointer" }}>
                    <input
                      type="checkbox"
                      checked={allowHighlySensitiveConsent}
                      onChange={(e) => setAllowHighlySensitiveConsent(e.target.checked)}
                    />
                    Izinkan Sangat Sensitif
                  </label>

                  <label style={{ fontSize: "11px", display: "flex", alignItems: "center", gap: "4px", color: "var(--muted)", cursor: "pointer" }}>
                    <input
                      type="checkbox"
                      checked={allowExternalVisionConsent}
                      onChange={(e) => setAllowExternalVisionConsent(e.target.checked)}
                    />
                    Izinkan External Vision
                  </label>

                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => void handleInspectOutbound()}
                    disabled={isInspectingOutbound || !questionTextOriginal.trim()}
                    style={{ background: "#f0fbf6", borderColor: "#a3dbbf", color: "#0c5a38", fontSize: "11px", padding: "5px 10px" }}
                    title="Tinjau data apa saja yang dikirim ke provider AI sebelum diproses"
                  >
                    {isInspectingOutbound ? "Memeriksa..." : "🔍 Inspeksi Data Keluar (SC-060)"}
                  </button>
                </div>

                {/* Panel Hasil Analisis Klasifikasi (SC-031) */}
                {classificationResult && (
                  <div className="classification-result-box" style={{ marginBottom: "16px" }}>
                    <div className="classification-header">
                      <div>
                        <strong style={{ fontSize: "14px", color: "#1b4d32" }}>
                          Kategori: {CATEGORY_LABELS[classificationResult.primaryCategory] || classificationResult.primaryCategory}
                        </strong>
                        <p style={{ margin: "2px 0 0", fontSize: "12px", color: "var(--muted)" }}>
                          Format Jawaban: <strong>{classificationResult.answerFormat}</strong>
                        </p>
                      </div>
                    </div>
                  </div>
                )}

                {/* Panel Bukti Profil (SC-032) */}
                {evidencePackage && (
                  <div className="evidence-result-box" style={{ marginBottom: "16px" }}>
                    <div className="evidence-header">
                      <div>
                        <strong style={{ fontSize: "14px", color: "#154228" }}>
                          Bukti Profil Responden ({evidencePackage.retrievedEvidence.length} Fakta)
                        </strong>
                        <p style={{ margin: "3px 0 0", fontSize: "11px", color: "var(--muted)" }}>
                          {evidencePackage.evaluationSummary}
                        </p>
                      </div>
                      <span className={`evidence-badge ${evidencePackage.recommendation}`}>
                        {evidencePackage.recommendation.replace("_", " ").toUpperCase()}
                      </span>
                    </div>

                    {evidencePackage.retrievedEvidence.length > 0 && (
                      <div className="evidence-item-list" style={{ marginTop: "8px" }}>
                        {evidencePackage.retrievedEvidence.map((item) => (
                          <div key={item.factId} className="evidence-item-card">
                            <div className="evidence-item-info">
                              <strong>{item.attribute}</strong>: {item.normalizedValue}
                            </div>
                            <span className="evidence-score-pill">
                              {Math.round(item.relevanceScore * 100)}%
                            </span>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                )}

                {/* ========================================================= */}
                {/* PANEL REKOMENDASI AI TERSTRUKTUR (draft_answer) (SC-033/SC-034) */}
                {/* ========================================================= */}
                {aiRecommendation && (
                  <div className="ai-recommendation-box" style={{ marginBottom: "20px" }}>
                    <div className="ai-recommendation-header">
                      <div>
                        <strong style={{ fontSize: "14px", color: "#163a2b" }}>
                          🤖 Rekomendasi Jawaban AI Terstruktur (SC-033)
                        </strong>
                        <p style={{ margin: "3px 0 0", fontSize: "11px", color: "var(--muted)" }}>
                          Interpretasi: {aiRecommendation.interpretation}
                        </p>
                      </div>
                      <div style={{ display: "flex", gap: "6px", alignItems: "center" }}>
                        <span className={`evidence-badge ${aiRecommendation.status}`}>
                          Status: {aiRecommendation.status.replace("_", " ").toUpperCase()}
                        </span>
                        <span className={`survey-badge ${aiRecommendation.confidence}`}>
                          Keyakinan: {aiRecommendation.confidence.toUpperCase()}
                        </span>
                      </div>
                    </div>

                    {/* Peringatan Sitasi Ditolak (AT-043) */}
                    {aiRecommendation.rejectedCitations.length > 0 && (
                      <div className="evidence-conflict-alert" style={{ background: "#fff5f5", borderColor: "#feb2b2", marginTop: "10px" }}>
                        <div className="evidence-conflict-title" style={{ color: "#c53030" }}>
                          <span>🛡️</span> Peringatan Keamanan Allowlist (AT-043): Sitasi ID Ditolak
                        </div>
                        <p style={{ margin: "4px 0 0", fontSize: "12px", color: "#742a2a" }}>
                          Sitasi tidak dikenal ditolak: <strong>{aiRecommendation.rejectedCitations.join(", ")}</strong>.
                        </p>
                      </div>
                    )}

                    {/* Draf Jawaban AI Asli */}
                    {aiRecommendation.draftAnswer ? (
                      <div className="ai-draft-card" style={{ marginTop: "12px" }}>
                        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "6px" }}>
                          <span style={{ fontSize: "12px", fontWeight: "700", color: "#175438" }}>
                            Draf Rekomendasi Asli AI (draft_answer):
                          </span>
                          <span style={{ fontSize: "11px", color: "var(--muted)" }}>
                            Format: {aiRecommendation.answerFormat} • Prompt: v{aiRecommendation.promptVersion}
                          </span>
                        </div>

                        <div className="ai-draft-text-display">
                          {aiRecommendation.draftAnswer}
                        </div>

                        {/* Aksi Cepat pada Rekomendasi AI (SC-034) */}
                        <div style={{ display: "flex", gap: "8px", flexWrap: "wrap", marginTop: "12px" }}>
                          <button
                            type="button"
                            className="secondary-button"
                            onClick={() => void handleCopyRecommendationToClipboard(aiRecommendation.draftAnswer!)}
                            style={{ background: "#f0fdf4", borderColor: "#86efac", color: "#166534" }}
                          >
                            {copiedToClipboard ? "✓ Tersalin!" : "📋 Salin Jawaban"}
                          </button>

                          <button
                            type="button"
                            className="secondary-button"
                            onClick={() => handleApplyDraftToUserAnswer(aiRecommendation.draftAnswer!)}
                            style={{ background: "#eff6ff", borderColor: "#bfdbfe", color: "#1e40af" }}
                          >
                            ✏️ Gunakan di Draf Pengguna
                          </button>

                          <button
                            type="button"
                            className="secondary-button"
                            onClick={() => setShowRegenerateModal(true)}
                            style={{ background: "#fdf4ff", borderColor: "#e9d5ff", color: "#6b21a8" }}
                          >
                            🔄 Regenerasi Jawaban...
                          </button>

                          <button
                            type="button"
                            className="secondary-button"
                            onClick={() => setShowRejectModal(true)}
                            style={{ background: "#fff1f2", borderColor: "#fecdd3", color: "#be123c" }}
                          >
                            ❌ Tolak Rekomendasi...
                          </button>
                        </div>
                      </div>
                    ) : (
                      /* Abstention Box */
                      <div className="ai-abstention-card" style={{ marginTop: "12px" }}>
                        <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "6px" }}>
                          <span style={{ fontSize: "16px" }}>🛑</span>
                          <strong style={{ color: "#92400e" }}>
                            Sistem Abstain: Tidak Memberikan Draf Jawaban Faktual
                          </strong>
                        </div>

                        {aiRecommendation.rationale && (
                          <p style={{ margin: "4px 0", fontSize: "12px", color: "#78350f" }}>
                            {aiRecommendation.rationale}
                          </p>
                        )}

                        {aiRecommendation.clarifyingQuestion && (
                          <div className="ai-clarification-prompt-box">
                            <span style={{ fontWeight: "700", color: "#1e3a8a" }}>💡 Pertanyaan Klarifikasi Terfokus:</span>
                            <p style={{ margin: "4px 0 0", color: "#1e40af" }}>
                              {aiRecommendation.clarifyingQuestion}
                            </p>
                          </div>
                        )}

                        <div style={{ display: "flex", gap: "8px", marginTop: "10px" }}>
                          <button
                            type="button"
                            className="secondary-button"
                            onClick={() => setShowRegenerateModal(true)}
                            style={{ background: "#fdf4ff", borderColor: "#e9d5ff", color: "#6b21a8" }}
                          >
                            🔄 Coba Regenerasi...
                          </button>
                          <button
                            type="button"
                            className="secondary-button"
                            onClick={() => setShowRejectModal(true)}
                            style={{ background: "#fff1f2", borderColor: "#fecdd3", color: "#be123c" }}
                          >
                            ❌ Tolak Rekomendasi...
                          </button>
                        </div>
                      </div>
                    )}

                    {/* Fakta Bukti yang Digunakan */}
                    {aiRecommendation.supportingFacts.length > 0 && (
                      <div style={{ marginTop: "12px", borderTop: "1px dashed var(--line)", paddingTop: "8px" }}>
                        <span style={{ fontSize: "11px", fontWeight: "700", color: "#166534" }}>
                          Fakta Profil Terverifikasi yang Digunakan ({aiRecommendation.supportingFacts.length}):
                        </span>
                        <div style={{ display: "flex", gap: "6px", flexWrap: "wrap", marginTop: "4px" }}>
                          {aiRecommendation.supportingFacts.map((fact) => (
                            <span key={fact.factId} className="evidence-fact-chip">
                              {fact.attribute}: <strong>{fact.normalizedValue}</strong>
                            </span>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                )}

                {/* ========================================================= */}
                {/* PANEL JAWABAN AKTUAL PENGGUNA (actual_answer) (SC-034) */}
                {/* ========================================================= */}
                {activeQuestionId && (
                  <div className="answer-draft-box" style={{ marginBottom: "20px" }}>
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "8px" }}>
                      <label className="field-label" htmlFor="user-answer-draft" style={{ margin: 0, fontSize: "13px", fontWeight: "800" }}>
                        ✍️ Jawaban Aktual Pengguna (actual_answer)
                      </label>
                      <span className={`survey-badge ${confirmationStatus}`}>
                        {confirmationStatus === "confirmed" ? "✓ Terkonfirmasi" : confirmationStatus === "rejected" ? "❌ Ditolak" : "Draf"}
                      </span>
                    </div>

                    <textarea
                      id="user-answer-draft"
                      value={userAnswerDraft}
                      onChange={(e) => setUserAnswerDraft(e.target.value)}
                      placeholder="Ketik atau sesuaikan jawaban yang akan Anda masukkan ke formulir survei..."
                      style={{ minHeight: "85px", marginBottom: "10px" }}
                    />

                    <div style={{ display: "flex", gap: "10px", alignItems: "center", marginBottom: "12px" }}>
                      <input
                        type="text"
                        placeholder="Catatan / alasan perubahan (opsional manual)..."
                        value={editReasonInput}
                        onChange={(e) => setEditReasonInput(e.target.value)}
                        style={{ flex: 1, padding: "8px 12px", fontSize: "12px", border: "1px solid #dce4de", borderRadius: "6px" }}
                      />
                      <button
                        type="button"
                        className="secondary-button"
                        onClick={() => void handleEditAnswer()}
                        disabled={savingDraft || !userAnswerDraft.trim()}
                        style={{ padding: "8px 14px" }}
                      >
                        {savingDraft ? "Menyimpan..." : "💾 Simpan Edit (SC-034)"}
                      </button>
                    </div>

                    <div style={{ display: "flex", gap: "10px", flexWrap: "wrap", alignItems: "center" }}>
                      <button
                        type="button"
                        className="primary-button"
                        onClick={() => void handleConfirmAnswer()}
                        disabled={confirmingAnswer || !userAnswerDraft.trim()}
                        style={{ marginTop: 0, background: "#166534", borderColor: "#166534" }}
                      >
                        {confirmingAnswer ? "Mengkonfirmasi..." : "✓ Konfirmasi Jawaban Aktual (SC-034)"}
                      </button>

                      <button
                        type="button"
                        className="secondary-button"
                        onClick={() => void handleCopyRecommendationToClipboard(userAnswerDraft)}
                        disabled={!userAnswerDraft.trim()}
                      >
                        📋 Salin Jawaban Aktual
                      </button>
                    </div>

                    {/* Batasan Non-Negosiasi: Konfirmasi adalah internal approval semata */}
                    <div style={{ background: "#f8fafc", border: "1px solid #e2e8f0", borderRadius: "8px", padding: "10px", marginTop: "12px", fontSize: "11px", color: "#475569", lineHeight: "1.5" }}>
                      <strong>🛡️ Batasan Privasi & Keamanan:</strong> Konfirmasi adalah pencatatan persetujuan internal Anda. Survey Copilot <strong>TIDAK PERNAH</strong> melakukan auto-fill, auto-click, atau auto-submit ke situs survei. Jawaban survei juga <strong>TIDAK PERNAH</strong> otomatis dipromosikan menjadi fakta profil responden.
                    </div>

                    {userAnswerConfirmed && (
                      <div className="confirmed-answer-display" style={{ marginTop: "12px" }}>
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

                {/* SC-054: Next Question Intelligence & Branch Prediction Panel */}
                {activeQuestionId && (
                  <div
                    className="nqi-panel-card"
                    style={{
                      background: "#faf5ff",
                      border: "1px solid #d8b4fe",
                      borderRadius: "10px",
                      padding: "16px",
                      marginTop: "16px",
                      marginBottom: "16px",
                    }}
                  >
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "8px", flexWrap: "wrap", gap: "8px" }}>
                      <div style={{ display: "flex", alignItems: "center", gap: "8px", flexWrap: "wrap" }}>
                        <span style={{ fontSize: "16px" }}>🔮</span>
                        <h4 style={{ margin: 0, fontSize: "14px", fontWeight: "800", color: "#581c87" }}>
                          Next Question Intelligence & Branch Prediction (SC-054)
                        </h4>
                        {nqiResult && nqiResult.status === "predicted_not_detected" && (
                          <span
                            style={{
                              display: "inline-block",
                              background: "#7e22ce",
                              color: "#ffffff",
                              fontSize: "10px",
                              fontWeight: "700",
                              padding: "2px 8px",
                              borderRadius: "12px",
                              textTransform: "uppercase",
                              letterSpacing: "0.5px",
                            }}
                          >
                            Predicted — not yet detected
                          </span>
                        )}
                        {nqiResult && nqiResult.status === "insufficient_evidence" && (
                          <span
                            style={{
                              display: "inline-block",
                              background: "#64748b",
                              color: "#ffffff",
                              fontSize: "10px",
                              fontWeight: "700",
                              padding: "2px 8px",
                              borderRadius: "12px",
                            }}
                          >
                            Insufficient Evidence
                          </span>
                        )}
                      </div>

                      <button
                        type="button"
                        className="secondary-button"
                        onClick={() => void evaluateNextQuestionPredictions(activeQuestionId, userAnswerConfirmed || userAnswerDraft || null)}
                        disabled={isLoadingNqi}
                        style={{ padding: "4px 10px", fontSize: "11px", marginTop: 0 }}
                      >
                        {isLoadingNqi ? "Menganalisis..." : "🔄 Perbarui Prediksi"}
                      </button>
                    </div>

                    <p style={{ margin: "0 0 12px 0", fontSize: "12px", color: "#6b21a8", lineHeight: "1.4" }}>
                      {nqiResult ? nqiResult.explanation : "Analisis kemungkinan pertanyaan berikutnya berdasarkan riwayat transisi percabangan teramati."}
                    </p>

                    {isLoadingNqi && (
                      <div style={{ fontSize: "12px", color: "#7e22ce", padding: "8px 0" }}>
                        ⏳ Menganalisis alur percabangan dan riwayat transisi...
                      </div>
                    )}

                    {!isLoadingNqi && nqiResult && nqiResult.predictions.length === 0 && (
                      <div
                        style={{
                          background: "#ffffff",
                          border: "1px dashed #cbd5e1",
                          borderRadius: "8px",
                          padding: "12px",
                          fontSize: "12px",
                          color: "#64748b",
                        }}
                      >
                        ℹ️ <strong>Bukti Riwayat Tidak Cukup:</strong> Belum ada riwayat transisi dari pertanyaan ini pada sesi historis. Sesuai prinsip kehati-hatian, sistem tidak membuat prediksi spekulatif sampai pertanyaan berikutnya diamati secara nyata.
                      </div>
                    )}

                    {!isLoadingNqi && nqiResult && nqiResult.predictions.length > 0 && (
                      <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
                        {nqiResult.predictions.map((pred, idx) => (
                          <div
                            key={pred.id || idx}
                            style={{
                              background: "#ffffff",
                              border: "1px solid #e9d5ff",
                              borderRadius: "8px",
                              padding: "12px",
                              boxShadow: "0 1px 3px rgba(0,0,0,0.04)",
                            }}
                          >
                            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", gap: "8px", marginBottom: "6px", flexWrap: "wrap" }}>
                              <div>
                                <span
                                  style={{
                                    display: "inline-block",
                                    background: "#f3e8ff",
                                    color: "#6b21a8",
                                    fontSize: "10px",
                                    fontWeight: "800",
                                    padding: "2px 6px",
                                    borderRadius: "4px",
                                    marginRight: "6px",
                                  }}
                                >
                                  Kandidat #{idx + 1}
                                </span>
                                <span
                                  style={{
                                    display: "inline-block",
                                    background: "#fef3c7",
                                    color: "#92400e",
                                    fontSize: "10px",
                                    fontWeight: "700",
                                    padding: "2px 6px",
                                    borderRadius: "4px",
                                    marginRight: "6px",
                                  }}
                                >
                                  Predicted — not yet detected
                                </span>
                                {pred.leadingCondition && (
                                  <span
                                    style={{
                                      display: "inline-block",
                                      background: "#e0f2fe",
                                      color: "#0369a1",
                                      fontSize: "10px",
                                      fontWeight: "700",
                                      padding: "2px 6px",
                                      borderRadius: "4px",
                                    }}
                                  >
                                    {pred.leadingCondition}
                                  </span>
                                )}
                              </div>
                              <span
                                style={{
                                  fontSize: "11px",
                                  fontWeight: "800",
                                  color: pred.confidence >= 0.8 ? "#15803d" : pred.confidence >= 0.5 ? "#b45309" : "#64748b",
                                  background: pred.confidence >= 0.8 ? "#dcfce7" : pred.confidence >= 0.5 ? "#fef3c7" : "#f1f5f9",
                                  padding: "2px 8px",
                                  borderRadius: "10px",
                                  whiteSpace: "nowrap",
                                }}
                              >
                                Keyakinan: {(pred.confidence * 100).toFixed(0)}%
                              </span>
                            </div>

                            <div style={{ fontSize: "13px", fontWeight: "700", color: "#1e1b4b", marginBottom: "4px" }}>
                              {pred.candidateQuestionText}
                            </div>

                            <div style={{ fontSize: "11px", color: "#6b7280", marginBottom: "6px" }}>
                              💡 <em>{pred.explanation}</em> (Sumber bukti: <code>{pred.evidenceSource}</code>)
                            </div>

                            {pred.options && pred.options.length > 0 && (
                              <div style={{ marginTop: "6px", paddingTop: "6px", borderTop: "1px dashed #f3e8ff" }}>
                                <span style={{ fontSize: "11px", fontWeight: "600", color: "#4b5563" }}>Pilihan Teramati:</span>
                                <div style={{ display: "flex", flexWrap: "wrap", gap: "4px", marginTop: "4px" }}>
                                  {pred.options.map((opt, optIdx) => (
                                    <span
                                      key={optIdx}
                                      style={{
                                        fontSize: "11px",
                                        background: "#f5f3ff",
                                        color: "#5b21b6",
                                        padding: "2px 6px",
                                        borderRadius: "4px",
                                        border: "1px solid #ddd6fe",
                                      }}
                                    >
                                      {opt}
                                    </span>
                                  ))}
                                </div>
                              </div>
                            )}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                )}

                {/* Tombol Simpan & Hapus Pertanyaan */}
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
                      ? "Perbarui Teks Pertanyaan"
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
              </>
            )}

            {/* TAB RIWAYAT PERUBAHAN JAWABAN (SC-034 AUDIT TIMELINE) */}
            {activeTab === "history" && (
              <div className="history-card-container">
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
                  <h4 style={{ margin: 0, fontSize: "14px", fontWeight: "800", color: "#163a2b" }}>
                    📜 Riwayat Perubahan Jawaban ({filteredHistory.length})
                  </h4>
                  <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
                    <label htmlFor="history-filter-select" style={{ fontSize: "12px", color: "var(--muted)" }}>
                      Filter Aksi:
                    </label>
                    <select
                      id="history-filter-select"
                      className="survey-session-dropdown"
                      style={{ padding: "4px 8px", fontSize: "12px" }}
                      value={historyActionFilter}
                      onChange={(e) => setHistoryActionFilter(e.target.value)}
                    >
                      <option value="all">Semua Aksi</option>
                      <option value="edited">Edited (Disunting)</option>
                      <option value="confirmed">Confirmed (Dikonfirmasi)</option>
                      <option value="copied">Copied (Disalin)</option>
                      <option value="rejected">Rejected (Ditolak)</option>
                      <option value="regenerated">Regenerated (Diregenerasi)</option>
                    </select>
                  </div>
                </div>

                {loadingHistory ? (
                  <p style={{ fontSize: "12px", color: "var(--muted)" }}>Memuat riwayat perubahan...</p>
                ) : filteredHistory.length === 0 ? (
                  <div className="empty-inline" style={{ padding: "20px" }}>
                    Belum ada riwayat perubahan yang dicatat untuk pertanyaan ini.
                  </div>
                ) : (
                  <div className="history-timeline-list">
                    {filteredHistory.map((item) => (
                      <div key={item.id} className="history-item-row">
                        <div className="history-item-header">
                          <span className={`review-action-badge ${item.action}`}>
                            {item.action === "edited" && "✏️ Disunting"}
                            {item.action === "confirmed" && "✓ Dikonfirmasi"}
                            {item.action === "copied" && "📋 Disalin"}
                            {item.action === "rejected" && "❌ Ditolak"}
                            {item.action === "regenerated" && "🔄 Diregenerasi"}
                          </span>
                          <span style={{ fontSize: "11px", color: "var(--muted)" }}>
                            {new Date(item.createdAt).toLocaleString("id-ID")}
                          </span>
                        </div>

                        {item.reason && (
                          <div style={{ fontSize: "12px", color: "#78350f", margin: "4px 0", fontStyle: "italic" }}>
                            Alasan / Catatan: {item.reason}
                          </div>
                        )}

                        <div className="history-comparison-box">
                          <div className="history-draft-col">
                            <strong>Draf AI:</strong>
                            <p style={{ margin: "4px 0 0", color: "#334155" }}>
                              {item.draftAnswer || "— (Abstain / Tidak ada draf)"}
                            </p>
                          </div>
                          <div className="history-actual-col">
                            <strong>Jawaban Pengguna (Actual):</strong>
                            <p style={{ margin: "4px 0 0", color: "#14532d" }}>
                              {item.actualAnswer || "—"}
                            </p>
                          </div>
                        </div>

                        <div style={{ display: "flex", gap: "10px", fontSize: "11px", color: "var(--muted)", marginTop: "6px" }}>
                          {item.providerId && <span>Provider: {item.providerId}</span>}
                          {item.modelId && <span>Model: {item.modelId}</span>}
                          {item.promptVersion && <span>Prompt: v{item.promptVersion}</span>}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* TAB DAFTAR REVISI REKOMENDASI AI (SC-034 REVISIONS) */}
            {activeTab === "revisions" && (
              <div className="history-card-container">
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
                  <h4 style={{ margin: 0, fontSize: "14px", fontWeight: "800", color: "#163a2b" }}>
                    🔄 Daftar Revisi Rekomendasi AI ({recommendationRevisions.length})
                  </h4>
                  <button
                    type="button"
                    className="secondary-button"
                    onClick={() => setShowRegenerateModal(true)}
                    style={{ padding: "4px 10px", fontSize: "12px" }}
                  >
                    + Regenerasi Baru
                  </button>
                </div>

                {recommendationRevisions.length === 0 ? (
                  <div className="empty-inline" style={{ padding: "20px" }}>
                    Belum ada rekomendasi AI yang dirumuskan untuk pertanyaan ini.
                  </div>
                ) : (
                  <div className="revisions-list-grid">
                    {recommendationRevisions.map((rev, index) => {
                      const isLatest = index === 0;
                      return (
                        <div key={rev.id} className={`revision-card ${isLatest ? "current" : ""}`}>
                          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                            <strong style={{ fontSize: "12px", color: "#154228" }}>
                              Revisi #{recommendationRevisions.length - index} {isLatest && "(Terbaru)"}
                            </strong>
                            <span className={`evidence-badge ${rev.status}`}>
                              {rev.status.replace("_", " ").toUpperCase()}
                            </span>
                          </div>

                          <div style={{ fontSize: "12px", color: "var(--ink)", margin: "4px 0", maxHeight: "80px", overflow: "hidden" }}>
                            {rev.draftAnswer ? (
                              <span><strong>Draf:</strong> {rev.draftAnswer}</span>
                            ) : (
                              <em style={{ color: "var(--muted)" }}>Abstain: {rev.rationale || "Tanpa draf faktual"}</em>
                            )}
                          </div>

                          <div style={{ fontSize: "11px", color: "var(--muted)" }}>
                            Dibuat: {new Date(rev.createdAt).toLocaleString("id-ID")}
                          </div>

                          <div style={{ display: "flex", gap: "6px", marginTop: "6px" }}>
                            <button
                              type="button"
                              className="secondary-button"
                              onClick={() => {
                                setAiRecommendation(rev);
                                setActiveTab("workspace");
                              }}
                              style={{ padding: "4px 8px", fontSize: "11px" }}
                            >
                              Lihat Detail
                            </button>
                            {rev.draftAnswer && (
                              <button
                                type="button"
                                className="secondary-button"
                                onClick={() => {
                                  handleApplyDraftToUserAnswer(rev.draftAnswer!);
                                  setActiveTab("workspace");
                                }}
                                style={{ padding: "4px 8px", fontSize: "11px", background: "#eff6ff", color: "#1d4ed8" }}
                              >
                                Gunakan Draf
                              </button>
                            )}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Kolom Kanan: Riwayat Pertanyaan dalam Sesi Ini */}
          <div className="question-history-panel">
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "6px" }}>
              <span style={{ fontSize: "13px", fontWeight: "800", color: "#172a20" }}>
                Daftar Pertanyaan ({questions.length})
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
                        {q.confirmationStatus === "confirmed" ? "✓ Terkonfirmasi" : q.confirmationStatus === "rejected" ? "❌ Ditolak" : "Draf"}
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

                    <div style={{ display: "flex", gap: "6px", alignItems: "center", flexWrap: "wrap", fontSize: "11px", color: "var(--muted)", marginTop: "4px" }}>
                      <span>{q.options.length} opsi</span>
                      <span style={{ background: "#f8fafc", color: "#64748b", padding: "1px 6px", borderRadius: "4px", fontSize: "10px" }}>
                        {q.provenance === "manual_entry" ? "Manual" : q.provenance === "ocr_detected" ? "OCR" : "Konfirmasi"}
                      </span>
                      {q.structureJson && typeof q.structureJson === "object" && "primary_category" in q.structureJson && (
                        <span style={{ background: "#edf5f0", color: "#1e603c", padding: "1px 6px", borderRadius: "4px", fontWeight: "600" }}>
                          {CATEGORY_LABELS[(q.structureJson as { primary_category: QuestionCategory }).primary_category] || (q.structureJson as { primary_category: string }).primary_category}
                        </span>
                      )}
                      {q.userAnswerConfirmed ? (
                        <span style={{ color: "#0d6e49", fontWeight: "700", background: "#dcfce7", padding: "1px 6px", borderRadius: "4px" }}>
                          ✓ Jawaban: {q.userAnswerConfirmed}
                        </span>
                      ) : q.userAnswerDraft ? (
                        <span style={{ color: "#b45309", fontWeight: "600", background: "#fef3c7", padding: "1px 6px", borderRadius: "4px" }}>
                          📝 Draf: {q.userAnswerDraft}
                        </span>
                      ) : null}
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>
        </>
      )}

      {/* Modal Tolak Rekomendasi (SC-034) */}
      {showRejectModal && (
        <div
          style={{
            position: "fixed",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: "rgba(0,0,0,0.4)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
          }}
        >
          <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="modal-reject-title"
            style={{
              background: "white",
              borderRadius: "12px",
              padding: "24px",
              width: "100%",
              maxWidth: "450px",
              boxShadow: "0 8px 24px rgba(0,0,0,0.15)",
            }}
          >
            <h3 id="modal-reject-title" style={{ margin: "0 0 10px", fontSize: "16px", fontWeight: "800", color: "#991b1b" }}>
              ❌ Tolak Rekomendasi Jawaban
            </h3>
            <p style={{ margin: "0 0 14px", fontSize: "13px", color: "var(--muted)" }}>
              Berikan alasan penolakan draf rekomendasi ini (opsional). Alasan akan disimpan dalam riwayat audit.
            </p>
            <textarea
              placeholder="Contoh: Jawaban tidak mencerminkan preferensi saya, atau opsi kurang tepat..."
              value={rejectReasonInput}
              onChange={(e) => setRejectReasonInput(e.target.value)}
              style={{ width: "100%", minHeight: "80px", marginBottom: "16px" }}
            />
            <div style={{ display: "flex", gap: "10px", justifyContent: "flex-end" }}>
              <button
                type="button"
                className="secondary-button"
                onClick={() => {
                  setShowRejectModal(false);
                  setRejectReasonInput("");
                }}
                disabled={isRejecting}
              >
                Batal
              </button>
              <button
                type="button"
                className="primary-button"
                onClick={() => void handleRejectAnswer()}
                disabled={isRejecting}
                style={{ marginTop: 0, background: "#dc2626", borderColor: "#dc2626" }}
              >
                {isRejecting ? "Menolak..." : "Konfirmasi Tolak"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Modal Regenerasi Jawaban (SC-034) */}
      {showRegenerateModal && (
        <div
          style={{
            position: "fixed",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: "rgba(0,0,0,0.4)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
          }}
        >
          <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="modal-regen-title"
            style={{
              background: "white",
              borderRadius: "12px",
              padding: "24px",
              width: "100%",
              maxWidth: "480px",
              boxShadow: "0 8px 24px rgba(0,0,0,0.15)",
            }}
          >
            <h3 id="modal-regen-title" style={{ margin: "0 0 10px", fontSize: "16px", fontWeight: "800", color: "#166534" }}>
              🔄 Regenerasi Rekomendasi Jawaban AI
            </h3>
            <p style={{ margin: "0 0 14px", fontSize: "13px", color: "var(--muted)" }}>
              Rekomendasi baru akan dibuat tanpa menghilangkan draf dan riwayat sebelumnya. Anda dapat memberikan petunjuk tambahan untuk proses generasi baru.
            </p>
            <textarea
              placeholder="Petunjuk / alasan regenerasi (contoh: Periksa opsi lain yang lebih relevan)..."
              value={regenerateReasonInput}
              onChange={(e) => setRegenerateReasonInput(e.target.value)}
              style={{ width: "100%", minHeight: "80px", marginBottom: "16px" }}
            />
            <div style={{ display: "flex", gap: "10px", justifyContent: "flex-end" }}>
              <button
                type="button"
                className="secondary-button"
                onClick={() => {
                  setShowRegenerateModal(false);
                  setRegenerateReasonInput("");
                }}
                disabled={isRegenerating}
              >
                Batal
              </button>
              <button
                type="button"
                className="primary-button"
                onClick={() => void handleRegenerateAnswer()}
                disabled={isRegenerating}
                style={{ marginTop: 0 }}
              >
                {isRegenerating ? "Memproses..." : "Regenerasi Sekarang"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* SC-040: Modal Pemilih Jendela Aplikasi (Window Picker) */}
      {showWindowPicker && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-window-picker-title" style={{ maxWidth: "560px" }}>
            <div className="modal-header">
              <h3 id="modal-window-picker-title">🪟 Pilih Jendela Aplikasi untuk Ditangkap</h3>
              <button
                type="button"
                className="secondary-button"
                style={{ padding: "4px 8px", fontSize: "12px", marginTop: 0 }}
                aria-label="Tutup pemilih jendela aplikasi"
                onClick={() => setShowWindowPicker(false)}
              >
                ✕
              </button>
            </div>
            <div className="modal-body">
              <p style={{ margin: 0, fontSize: "13px", color: "var(--muted)" }}>
                Pilih jendela aplikasi browser atau survei yang sedang aktif di Windows untuk mengambil tangkapan layar beresolusi penuh:
              </p>

              {availableWindows.length === 0 ? (
                <div className="empty-inline" style={{ padding: "24px 12px" }}>
                  <p>Tidak ada jendela aplikasi yang terdeteksi aktif atau seluruh jendela sedang terminimalkan.</p>
                </div>
              ) : (
                <div className="window-picker-list">
                  {availableWindows.map((win) => (
                    <div
                      key={win.id}
                      className="window-item-card"
                    >
                      <div className="window-item-info">
                        <span className="window-item-title">{win.title}</span>
                        <span className="window-item-meta">
                          Ukuran: {win.width} × {win.height} px {win.isMinimized ? "• (Terminimalkan)" : ""}
                        </span>
                      </div>
                      <button
                        type="button"
                        className="capture-btn primary"
                        style={{ padding: "6px 12px", fontSize: "11px", whiteSpace: "nowrap" }}
                        onClick={() => void handleSelectWindowToCapture(win.id)}
                      >
                        Tangkap
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
            <div className="modal-footer">
              <button
                type="button"
                className="secondary-button"
                onClick={() => setShowWindowPicker(false)}
              >
                Tutup
              </button>
            </div>
          </div>
        </div>
      )}

      {/* SC-040: Modal Pratinjau Tangkapan Layar (Capture Preview) */}
      {showCapturePreviewModal && activeCapturePreview && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-capture-preview-title" style={{ maxWidth: "680px" }}>
            <div className="modal-header">
              <div>
                <h3 id="modal-capture-preview-title" style={{ margin: 0, fontSize: "16px", fontWeight: "800" }}>
                  📸 Pratinjau Tangkapan Layar (SC-040)
                </h3>
                <span style={{ fontSize: "12px", color: "var(--muted)" }}>
                  {activeCapturePreview.targetLabel || "Tangkapan Layar"}
                </span>
              </div>
              <button
                type="button"
                className="secondary-button"
                style={{ padding: "4px 8px", fontSize: "12px", marginTop: 0 }}
                aria-label="Tutup pratinjau tangkapan layar"
                onClick={() => setShowCapturePreviewModal(false)}
                disabled={isExtractingOcr}
              >
                ✕
              </button>
            </div>
            <div className="modal-body" style={{ gap: "12px" }}>
              <div className="capture-modal-img-container">
                {activeCapturePreview.previewDataUrl ? (
                  <img
                    src={activeCapturePreview.previewDataUrl}
                    alt="Pratinjau tangkapan layar yang dipilih"
                    className="capture-modal-img"
                  />
                ) : (
                  <div style={{ color: "#94a3b8", padding: "40px" }}>
                    Pratinjau gambar tidak tersedia
                  </div>
                )}
              </div>

              {/* Rincian Teknis & Metadata Tangkapan */}
              <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "8px", background: "#f8fafc", padding: "10px 12px", borderRadius: "8px", fontSize: "11px", border: "1px solid #e2e8f0" }}>
                <div>
                  <strong style={{ color: "#64748b", display: "block" }}>Mode Tangkapan:</strong>
                  <span style={{ fontWeight: "700", textTransform: "capitalize" }}>{activeCapturePreview.sourceMode}</span>
                </div>
                <div>
                  <strong style={{ color: "#64748b", display: "block" }}>Dimensi Piksel:</strong>
                  <span style={{ fontWeight: "700" }}>{activeCapturePreview.width} × {activeCapturePreview.height} px</span>
                </div>
                <div>
                  <strong style={{ color: "#64748b", display: "block" }}>Ukuran Berkas:</strong>
                  <span style={{ fontWeight: "700" }}>{(activeCapturePreview.fileSizeBytes / 1024).toFixed(1)} KB</span>
                </div>
              </div>

              <div className="capture-local-ocr-notice" role="status">
                <strong>🔒 Capture dan OCR lokal</strong>
                <span>Gambar diproses oleh OCR lokal secara otomatis. Screenshot dan teks belum dikirim ke provider AI. Analisis AI hanya dapat dimulai setelah Anda meninjau hasil dan memilih tindakan berikutnya.</span>
              </div>

              {isExtractingOcr && (
                <div className="capture-ocr-progress" role="status" aria-live="polite">
                  <span className="capture-ocr-spinner" aria-hidden="true" />
                  <div>
                    <strong>Membaca pertanyaan secara lokal…</strong>
                    <span>Rekonstruksi teks dan opsi sedang berjalan. Tidak ada request AI eksternal.</span>
                  </div>
                </div>
              )}

              {ocrError && (
                <div style={{ background: "#fef2f2", border: "1px solid #fecaca", borderRadius: "8px", padding: "10px 12px", fontSize: "12px", color: "#991b1b" }}>
                  ⚠️ {ocrError}
                </div>
              )}
            </div>
            <div className="modal-footer" style={{ justifyContent: "space-between" }}>
              <button
                type="button"
                className="secondary-button"
                style={{ color: "#991b1b", borderColor: "#fecaca" }}
                onClick={() => void handleDiscardCapture()}
                title="Hapus berkas gambar sementara dari disk"
                disabled={isExtractingOcr}
              >
                🗑️ Batalkan & Hapus Gambar
              </button>

              <div style={{ display: "flex", gap: "8px" }}>
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => setShowCapturePreviewModal(false)}
                  disabled={isExtractingOcr}
                >
                  Tutup Pratinjau
                </button>
                {ocrError && (
                  <button
                    type="button"
                    className="secondary-button"
                    style={{ marginTop: 0 }}
                    onClick={() => void handleExtractOcr()}
                    disabled={isExtractingOcr}
                  >
                    {isExtractingOcr ? "⏳ Membaca lokal..." : "↻ Coba OCR lokal lagi"}
                  </button>
                )}
                <button
                  type="button"
                  className="secondary-button"
                  style={{ marginTop: 0 }}
                  onClick={handleConfirmCaptureUse}
                  title="Gunakan tangkapan layar untuk diketik manual"
                  disabled={isExtractingOcr}
                >
                  ✓ Tautkan Manual
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* SC-041: Modal Review & Konfirmasi Hasil Ekstraksi OCR */}
      {showOcrReviewModal && activeOcrExtraction && (
        <div className="modal-backdrop">
          <div className="modal-dialog ocr-review-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-ocr-review-title">
            <div className="modal-header">
              <div>
                <h3 id="modal-ocr-review-title" style={{ margin: 0, fontSize: "16px", fontWeight: "800" }}>
                  📝 Review & Konfirmasi Hasil OCR (SC-041)
                </h3>
                <span style={{ fontSize: "12px", color: "var(--muted)" }}>
                  Tinjau dan koreksi hasil ekstraksi sebelum digunakan dalam analisis AI
                </span>
              </div>
              <button
                type="button"
                className="secondary-button"
                style={{ padding: "4px 8px", fontSize: "12px", marginTop: 0 }}
                aria-label="Tutup ulasan hasil OCR"
                onClick={() => setShowOcrReviewModal(false)}
              >
                ✕
              </button>
            </div>

            <div className="modal-body" style={{ gap: "14px", maxHeight: "65vh", overflowY: "auto" }}>
              {/* Quality & Metadata Badges */}
              <div className="ocr-badge-row">
                <span
                  className={`ocr-badge ${
                    activeOcrExtraction.overallConfidence <= 0
                      ? "info"
                      : activeOcrExtraction.overallConfidence >= 0.85
                      ? "success"
                      : activeOcrExtraction.overallConfidence >= 0.70
                      ? "warning"
                      : "danger"
                  }`}
                  title={
                    activeOcrExtraction.overallConfidence <= 0
                      ? "Engine OCR tidak menyediakan skor confidence numerik"
                      : "Tingkat keyakinan agregat dari skor OCR yang tersedia"
                  }
                >
                  🎯 {activeOcrExtraction.overallConfidence <= 0
                    ? "Keyakinan: tidak tersedia"
                    : `Keyakinan: ${(activeOcrExtraction.overallConfidence * 100).toFixed(0)}%`}
                </span>

                <span className="ocr-badge info" title="Tipe format pertanyaan yang terdeteksi">
                  📋 {CATEGORY_LABELS[activeOcrExtraction.questionType as QuestionCategory] || activeOcrExtraction.questionType}
                </span>

                <span className="ocr-badge info" title="Format jawaban survei">
                  ⚙️ {activeOcrExtraction.answerFormat}
                </span>

                {activeOcrExtraction.isRequired && (
                  <span className="ocr-badge danger">
                    * Wajib Diisi
                  </span>
                )}

                {activeOcrExtraction.visibleUnits && (
                  <span className="ocr-badge warning">
                    📐 Satuan: {activeOcrExtraction.visibleUnits}
                  </span>
                )}

                <span className="ocr-badge" style={{ background: "#f1f5f9", color: "#475569" }}>
                  Engine: {activeOcrExtraction.ocrEngine}
                </span>
              </div>

              {/* Warnings & Ambiguities Alert (AT-052) */}
              {(activeOcrExtraction.warnings.length > 0 || activeOcrExtraction.uncertainRegions.length > 0) && (
                <div className="ocr-warnings-box">
                  <strong>⚠️ Peringatan Kualitas & Ambiguitas (AT-052):</strong>
                  {activeOcrExtraction.warnings.map((w, idx) => (
                    <div key={idx} style={{ paddingLeft: "8px" }}>• {w}</div>
                  ))}
                  {activeOcrExtraction.uncertainRegions.filter(u => u.reason === "likely_clipped").length > 0 && (
                    <div style={{ paddingLeft: "8px", fontWeight: "600", color: "#b45309" }}>
                      • Ada teks di tepi gambar yang terpotong. Harap lengkapi teks pertanyaan di bawah.
                    </div>
                  )}
                </div>
              )}

              {/* Teks Pertanyaan Asli OCR vs Teks Koreksi (Immutability Preservation) */}
              <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                <label className="field-label" htmlFor="ocr-corrected-prompt" style={{ fontWeight: "700" }}>
                  Teks Pertanyaan (Dapat Dikoreksi) *
                </label>
                <textarea
                  id="ocr-corrected-prompt"
                  value={ocrCorrectedText}
                  onChange={(e) => setOcrCorrectedText(e.target.value)}
                  rows={3}
                  className="input-textarea"
                  style={{ width: "100%", fontSize: "13px" }}
                  placeholder="Koreksi teks pertanyaan jika ada kesalahan OCR..."
                />
                <span style={{ fontSize: "11px", color: "#64748b" }}>
                  🔒 <em>Teks Asli OCR:</em> "{activeOcrExtraction.questionTextOriginal}"
                </span>
              </div>

              {/* Instruksi Pengisian */}
              <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                <label className="field-label" htmlFor="ocr-corrected-inst" style={{ fontWeight: "700" }}>
                  Instruksi Pengisian Survei
                </label>
                <input
                  id="ocr-corrected-inst"
                  type="text"
                  value={ocrCorrectedInstructions}
                  onChange={(e) => setOcrCorrectedInstructions(e.target.value)}
                  className="input-text"
                  style={{ width: "100%", fontSize: "12px" }}
                  placeholder="Contoh: (Pilih satu jawaban), (Centang semua yang sesuai)..."
                />
              </div>

              {/* Struktur Pilihan Jawaban (Jika Ada) */}
              {ocrCorrectedOptions.length > 0 && (
                <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
                  <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                    <label className="field-label" style={{ fontWeight: "700", margin: 0 }}>
                      Pilihan Jawaban ({ocrCorrectedOptions.length} opsi terdeteksi)
                    </label>
                    <button
                      type="button"
                      className="secondary-button"
                      style={{ padding: "2px 8px", fontSize: "11px", marginTop: 0 }}
                      onClick={() => setOcrCorrectedOptions([...ocrCorrectedOptions, `Opsi Baru ${ocrCorrectedOptions.length + 1}`])}
                    >
                      + Tambah Opsi
                    </button>
                  </div>

                  <div className="ocr-options-editor">
                    {ocrCorrectedOptions.map((opt, optIdx) => (
                      <div key={optIdx} className="ocr-option-row">
                        <span style={{ fontSize: "12px", color: "#64748b", width: "24px", fontWeight: "600" }}>
                          {String.fromCharCode(65 + optIdx)}.
                        </span>
                        <input
                          type="text"
                          value={opt}
                          onChange={(e) => {
                            const updated = [...ocrCorrectedOptions];
                            updated[optIdx] = e.target.value;
                            setOcrCorrectedOptions(updated);
                          }}
                          className="ocr-option-input"
                        />
                        <button
                          type="button"
                          className="secondary-button"
                          style={{ padding: "4px 8px", fontSize: "11px", color: "#991b1b", borderColor: "#fecaca" }}
                          onClick={() => setOcrCorrectedOptions(ocrCorrectedOptions.filter((_, i) => i !== optIdx))}
                          title="Hapus opsi ini"
                        >
                          ✕
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Struktur Skala Penilaian (Jika Ada) */}
              {activeOcrExtraction.scale && (
                <div style={{ background: "#f8fafc", border: "1px solid #e2e8f0", borderRadius: "8px", padding: "10px 14px" }}>
                  <strong style={{ fontSize: "12px", color: "#334155", display: "block", marginBottom: "6px" }}>
                    ⭐ Parameter Skala Penilaian Terdeteksi:
                  </strong>
                  <div style={{ display: "flex", gap: "16px", fontSize: "12px" }}>
                    <div>
                      <span style={{ color: "#64748b" }}>Nilai Min: </span>
                      <strong>{activeOcrExtraction.scale.minValue}</strong>
                      {activeOcrExtraction.scale.minLabel && ` (${activeOcrExtraction.scale.minLabel})`}
                    </div>
                    <div>
                      <span style={{ color: "#64748b" }}>Nilai Max: </span>
                      <strong>{activeOcrExtraction.scale.maxValue}</strong>
                      {activeOcrExtraction.scale.maxLabel && ` (${activeOcrExtraction.scale.maxLabel})`}
                    </div>
                  </div>
                </div>
              )}

              {/* Struktur Tabel Matriks / Grid (Jika Ada) */}
              {activeOcrExtraction.matrix && (
                <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
                  <strong style={{ fontSize: "12px", color: "#334155" }}>
                    📊 Struktur Tabel Matriks (Grid Table Terpetakan):
                  </strong>
                  <div style={{ overflowX: "auto" }}>
                    <table className="ocr-matrix-table">
                      <thead>
                        <tr>
                          <th>Pernyataan / Aspek</th>
                          {activeOcrExtraction.matrix.columns.map((col, cIdx) => (
                            <th key={cIdx}>{col}</th>
                          ))}
                        </tr>
                      </thead>
                      <tbody>
                        {activeOcrExtraction.matrix.rows.map((row, rIdx) => (
                          <tr key={rIdx}>
                            <td style={{ fontWeight: "600", color: "#1e293b" }}>{row}</td>
                            {activeOcrExtraction.matrix!.columns.map((_, cIdx) => (
                              <td key={cIdx} style={{ color: "#94a3b8" }}>
                                {activeOcrExtraction.matrix!.isMultiSelect ? "☐" : "○"}
                              </td>
                            ))}
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}
            </div>

            <div className="modal-footer" style={{ justifyContent: "space-between" }}>
              <button
                type="button"
                className="secondary-button"
                onClick={() => setShowOcrReviewModal(false)}
              >
                Batal / Tutup
              </button>

              <div style={{ display: "flex", gap: "8px" }}>
                <button
                  type="button"
                  className="secondary-button"
                  onClick={() => void handleConfirmOcrUse(false)}
                  title="Salin hasil koreksi ke editor pertanyaan tanpa langsung merekam"
                >
                  📋 Terapkan ke Editor
                </button>
                <button
                  type="button"
                  className="primary-button"
                  style={{ marginTop: 0 }}
                  onClick={() => void handleConfirmOcrUse(true)}
                  disabled={loading || !selectedSessionId}
                >
                  {loading ? "Menyimpan..." : "✓ Konfirmasi & Rekam ke Survei"}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* SC-053: Modal Rincian Pengenalan Survei & Survey Memory */}
      {showRecognitionModal && recognitionEvaluation && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-recognition-title" style={{ maxWidth: "700px" }}>
            <div className="modal-header">
              <h3 id="modal-recognition-title">🧠 Rincian Pengenalan Survei (SC-053)</h3>
              <button
                type="button"
                className="secondary-button"
                style={{ padding: "4px 8px", fontSize: "12px", marginTop: 0 }}
                aria-label="Tutup rincian pengenalan survei"
                onClick={() => setShowRecognitionModal(false)}
              >
                ✕
              </button>
            </div>
            <div className="modal-body">
              <div
                style={{
                  background: "#f0fdf4",
                  border: "1px solid #bbf7d0",
                  borderRadius: "8px",
                  padding: "14px",
                  marginBottom: "16px",
                }}
              >
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "6px" }}>
                  <strong style={{ fontSize: "14px", color: "#166534" }}>
                    {recognitionEvaluation.matchedSessionTitle || "Sesi Historis"}
                  </strong>
                  <span
                    style={{
                      padding: "2px 8px",
                      borderRadius: "4px",
                      fontSize: "11px",
                      fontWeight: "700",
                      background: "#16a34a",
                      color: "white",
                    }}
                  >
                    Tingkat Keyakinan: {(recognitionEvaluation.confidenceScore * 100).toFixed(0)}%
                  </span>
                </div>
                <p style={{ margin: "0 0 8px", fontSize: "12px", color: "#166534" }}>
                  {recognitionEvaluation.explanation}
                </p>
                <div style={{ display: "flex", gap: "12px", fontSize: "11px", color: "#374151" }}>
                  <span>🧬 Kemiripan Struktur: {(recognitionEvaluation.structuralSimilarity * 100).toFixed(0)}%</span>
                  <span>🏷️ Kemiripan Judul: {(recognitionEvaluation.titleSimilarity * 100).toFixed(0)}%</span>
                  <span>📊 Pertanyaan Cocok: {recognitionEvaluation.matchedQuestionCount} dari {recognitionEvaluation.totalQuestionCount}</span>
                </div>
              </div>

              {/* Rincian Perbedaan jika Ada */}
              {recognitionEvaluation.differences.length > 0 && (
                <div style={{ marginBottom: "16px" }}>
                  <h4 style={{ margin: "0 0 6px", fontSize: "12px", fontWeight: "700", textTransform: "uppercase", color: "#475569" }}>
                    Rincian Perbedaan / Perubahan Versi ({recognitionEvaluation.differences.length}):
                  </h4>
                  <ul style={{ margin: 0, paddingLeft: "18px", fontSize: "12px", color: "#475569" }}>
                    {recognitionEvaluation.differences.map((diff, idx) => (
                      <li key={idx} style={{ marginBottom: "3px" }}>{diff}</li>
                    ))}
                  </ul>
                </div>
              )}

              {/* Daftar Jawaban Historis Nyata yang Tersimpan */}
              <div>
                <h4 style={{ margin: "0 0 8px", fontSize: "13px", fontWeight: "700", color: "#1e293b" }}>
                  Jawaban Responden Nyata Terkonfirmasi pada Survei Sebelumnya ({recognitionEvaluation.recognizedAnswers.length}):
                </h4>
                {recognitionEvaluation.recognizedAnswers.length === 0 ? (
                  <p style={{ fontSize: "12px", color: "var(--muted)", margin: 0 }}>
                    Belum ada jawaban yang dikonfirmasi pada sesi sebelumnya yang cocok dengan pertanyaan saat ini.
                  </p>
                ) : (
                  <div style={{ display: "grid", gap: "8px", maxHeight: "250px", overflowY: "auto" }}>
                    {recognitionEvaluation.recognizedAnswers.map((ans, idx) => (
                      <div
                        key={idx}
                        style={{
                          background: "#ffffff",
                          border: "1px solid #e2e8f0",
                          borderRadius: "8px",
                          padding: "10px 14px",
                          display: "flex",
                          justifyContent: "space-between",
                          alignItems: "center",
                          gap: "10px",
                        }}
                      >
                        <div>
                          <div style={{ fontSize: "12px", fontWeight: "600", color: "#1e293b", marginBottom: "2px" }}>
                            #{ans.questionOrder}: {ans.questionText}
                          </div>
                          <div style={{ fontSize: "12px", color: "#065f46", fontWeight: "700" }}>
                            ✓ Jawaban Terkonfirmasi: {ans.confirmedAnswer}
                          </div>
                          <div style={{ fontSize: "10px", color: "#94a3b8" }}>
                            Dikonfirmasi: {ans.confirmedAt ? new Date(ans.confirmedAt).toLocaleDateString("id-ID") : "Sebelumnya"}
                          </div>
                        </div>

                        {activeQuestionId && (
                          <button
                            type="button"
                            className="secondary-button"
                            onClick={() => {
                              setUserAnswerDraft(ans.confirmedAnswer);
                              setSuccessMessage("Jawaban dari memori disalin ke draf pertanyaan aktif.");
                              setTimeout(() => setSuccessMessage(""), 3000);
                            }}
                            style={{ padding: "4px 8px", fontSize: "11px", whiteSpace: "nowrap" }}
                            title="Salin jawaban historis ini ke draf jawaban pertanyaan aktif"
                          >
                            Salin ke Draf
                          </button>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
            <div className="modal-footer">
              <button
                type="button"
                className="secondary-button"
                onClick={() => setShowRecognitionModal(false)}
              >
                Tutup
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Modal Inspeksi Data Keluar (SC-060: What Data Leaves The Device) */}
      {showOutboundModal && outboundDisclosure && (
        <div className="modal-backdrop">
          <div className="modal-content" role="dialog" aria-modal="true" aria-labelledby="modal-outbound-title" style={{ maxWidth: "700px", maxHeight: "88vh", overflowY: "auto" }}>
            <div className="modal-header">
              <h3 id="modal-outbound-title" style={{ margin: 0, color: "#0c5a38", display: "flex", alignItems: "center", gap: "8px" }}>
                🔍 Transparansi Transmisi Data (SC-060)
              </h3>
              <button
                type="button"
                className="close-button"
                aria-label="Tutup transparansi data keluar"
                onClick={() => setShowOutboundModal(false)}
              >
                ✕
              </button>
            </div>

            <div className="modal-body" style={{ display: "grid", gap: "16px", paddingTop: "8px" }}>
              <p style={{ margin: 0, fontSize: "13px", color: "var(--muted)", lineHeight: 1.5 }}>
                Berikut adalah rincian data yang akan dikirim keluar perangkat ke provider AI untuk pertanyaan aktif.
                Hanya data yang secara eksplisit relevan dan diizinkan yang akan ditransmisikan.
              </p>

              {/* Destination Endpoint Card */}
              <div style={{ background: "#f8fafc", border: "1px solid #e2e8f0", borderRadius: "8px", padding: "12px 16px" }}>
                <div style={{ fontSize: "11px", fontWeight: "700", color: "#64748b", textTransform: "uppercase", letterSpacing: "0.5px" }}>
                  Provider & Endpoint Tujuan
                </div>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginTop: "4px" }}>
                  <div>
                    <strong style={{ fontSize: "14px", color: "#0f172a" }}>{outboundDisclosure.providerName}</strong>
                    <div style={{ fontSize: "12px", color: "#64748b" }}>Model: <code>{outboundDisclosure.modelId}</code></div>
                  </div>
                  <span style={{ fontSize: "11px", background: "#ecfdf5", color: "#047857", padding: "3px 8px", borderRadius: "4px", fontWeight: "600" }}>
                    HTTPS Terverifikasi
                  </span>
                </div>
                <div style={{ marginTop: "6px", fontSize: "11px", color: "#94a3b8", wordBreak: "break-all" }}>
                  Tujuan: <code>{outboundDisclosure.destinationEndpoint}</code>
                </div>
              </div>

              {/* Included Facts */}
              <div>
                <h4 style={{ margin: "0 0 6px", fontSize: "13px", color: "#1e293b", fontWeight: "700" }}>
                  Fakta Profil yang Disertakan ({outboundDisclosure.includedFacts.length}):
                </h4>
                {outboundDisclosure.includedFacts.length === 0 ? (
                  <div style={{ fontSize: "12px", color: "#94a3b8", background: "#f8fafc", padding: "10px", borderRadius: "6px" }}>
                    Tidak ada fakta profil yang relevan atau memenuhi izin untuk disertakan.
                  </div>
                ) : (
                  <div style={{ display: "grid", gap: "6px", maxHeight: "150px", overflowY: "auto" }}>
                    {outboundDisclosure.includedFacts.map((fact) => (
                      <div
                        key={fact.factId}
                        style={{
                          background: "#ffffff",
                          border: "1px solid #cbd5e1",
                          borderRadius: "6px",
                          padding: "8px 12px",
                          display: "flex",
                          justifyContent: "space-between",
                          alignItems: "center",
                        }}
                      >
                        <div>
                          <span style={{ fontSize: "11px", color: "#64748b", textTransform: "uppercase", fontWeight: "600" }}>
                            [{fact.category}] {fact.attribute}:
                          </span>{" "}
                          <strong style={{ fontSize: "12px", color: "#0f172a" }}>{fact.valuePreview}</strong>
                        </div>
                        <span
                          style={{
                            fontSize: "10px",
                            padding: "2px 6px",
                            borderRadius: "4px",
                            fontWeight: "700",
                            background: fact.isSensitive ? "#fef3c7" : "#f1f5f9",
                            color: fact.isSensitive ? "#92400e" : "#475569",
                          }}
                        >
                          {fact.isSensitive ? "SENSITIF" : "BIASA"}
                        </span>
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* Excluded Facts */}
              {outboundDisclosure.excludedFacts.length > 0 && (
                <div>
                  <h4 style={{ margin: "0 0 6px", fontSize: "13px", color: "#64748b", fontWeight: "700" }}>
                    Fakta yang Dikecualikan / Tidak Ditransmisikan ({outboundDisclosure.excludedFacts.length}):
                  </h4>
                  <div style={{ display: "grid", gap: "6px", maxHeight: "120px", overflowY: "auto" }}>
                    {outboundDisclosure.excludedFacts.map((fact) => (
                      <div
                        key={fact.factId}
                        style={{
                          background: "#f8fafc",
                          border: "1px dashed #cbd5e1",
                          borderRadius: "6px",
                          padding: "6px 12px",
                          fontSize: "12px",
                          color: "#64748b",
                          display: "flex",
                          justifyContent: "space-between",
                        }}
                      >
                        <span>[{fact.category}] {fact.attribute}</span>
                        <span style={{ fontSize: "11px", color: "#dc2626" }}>
                          {fact.exclusionReason || "Dikecualikan"}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* External Vision Status */}
              <div style={{ background: "#f0fdf4", border: "1px solid #bbf7d0", borderRadius: "8px", padding: "10px 14px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <div>
                  <div style={{ fontSize: "12px", fontWeight: "700", color: "#166534" }}>
                    Analisis Visual Eksternal (Vision)
                  </div>
                  <div style={{ fontSize: "11px", color: "#15803d" }}>
                    {outboundDisclosure.includesExternalVision
                      ? outboundDisclosure.visionConsentGranted
                        ? "✓ Gambar tangkapan layar diizinkan dikirim atas persetujuan pengguna"
                        : "⚠ Gambar dilampirkan namun izin eksternal vision belum dicentang"
                      : "Tidak ada gambar tangkapan layar yang dikirimkan ke model AI"}
                  </div>
                </div>
                <label style={{ fontSize: "11px", display: "flex", alignItems: "center", gap: "4px", color: "#166534", cursor: "pointer", fontWeight: "600" }}>
                  <input
                    type="checkbox"
                    checked={allowExternalVisionConsent}
                    onChange={(e) => setAllowExternalVisionConsent(e.target.checked)}
                  />
                  Izin Vision
                </label>
              </div>

              {/* Jaminan Keamanan Data Lokal */}
              <div style={{ background: "#f8fafc", border: "1px solid #e2e8f0", borderRadius: "8px", padding: "12px 14px" }}>
                <div style={{ fontSize: "12px", fontWeight: "700", color: "#334155", marginBottom: "6px" }}>
                  🛡 Jaminan Keamanan & Privasi Lokal:
                </div>
                <ul style={{ margin: 0, paddingLeft: "18px", fontSize: "11px", color: "#64748b", lineHeight: 1.6 }}>
                  {outboundDisclosure.notTransmittedGuarantees.map((g, idx) => (
                    <li key={idx}>{g}</li>
                  ))}
                </ul>
              </div>
            </div>

            <div className="modal-footer" style={{ display: "flex", justifyContent: "space-between" }}>
              <button
                type="button"
                className="secondary-button"
                onClick={() => setShowOutboundModal(false)}
              >
                Tutup Pratinjau
              </button>

              <button
                type="button"
                className="primary-button"
                onClick={() => {
                  setShowOutboundModal(false);
                  void handleGenerateAnswer();
                }}
                style={{ background: "#175438", borderColor: "#175438" }}
              >
                Lanjutkan ke Rekomendasi AI ➔
              </button>
            </div>
          </div>
        </div>
      )}
        </>
      )}
    </section>
  );
}
