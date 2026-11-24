import { useEffect, useState } from 'react';
import type { Persona } from '../../types/persona';
import type {
  CriterionInput,
  CriterionOperator,
  CriterionType,
  OpportunityDetail,
  SurveyOpportunity,
} from '../../types/opportunity';
import type {
  MatchState,
  OverallFitState,
  PersonaFitComparisonItem,
  PersonaOpportunityFitDetail,
} from '../../types/fit';
import { opportunityService } from '../../services/opportunityService';
import { captureService } from '../../services/captureService';
import { ocrService } from '../../services/ocrService';
import { fitService } from '../../services/fitService';
import { attemptService } from '../../services/attemptService';
import type { CapturePreview, WindowInfo } from '../../types/capture';
import { AppError } from '../../types/error';

interface OpportunityViewProps {
  activePersona?: Persona;
}

const CRITERION_TYPE_LABELS: Record<CriterionType, string> = {
  age: 'Usia',
  gender: 'Jenis Kelamin',
  location: 'Domisili / Lokasi',
  occupation: 'Pekerjaan',
  product_usage: 'Penggunaan Produk / Aset',
  education: 'Pendidikan',
  income: 'Pendapatan / Pengeluaran',
  marital_status: 'Status Pernikahan',
  general: 'Umum / Lainnya',
};

const OPERATOR_LABELS: Record<CriterionOperator, string> = {
  equals: 'Sama Dengan (=)',
  range: 'Rentang Nilai (Min - Maks)',
  greater_than_or_equal: 'Minimal (>=)',
  less_than_or_equal: 'Maksimal (<=)',
  in: 'Salah Satu Pilihan',
  contains: 'Mengandung / Menggunakan',
  not_equals: 'Eksklusi / Larangan (!=)',
};

const MATCH_STATE_CONFIG: Record<
  MatchState,
  { label: string; bg: string; color: string; border: string }
> = {
  match: { label: '✓ Cocok', bg: '#dcfce7', color: '#166534', border: '#86efac' },
  mismatch: { label: '✕ Tidak Cocok', bg: '#fee2e2', color: '#991b1b', border: '#f87171' },
  unknown: { label: '? Belum Diketahui', bg: '#fef3c7', color: '#92400e', border: '#fcd34d' },
  stale: { label: '⏱ Kadaluarsa', bg: '#ffedd5', color: '#9a3412', border: '#fdba74' },
  conflict: { label: '⚠ Konflik Data', bg: '#f3e8ff', color: '#6b21a8', border: '#d8b4fe' },
};

const OVERALL_FIT_CONFIG: Record<
  OverallFitState,
  { label: string; bg: string; color: string; border: string; desc: string }
> = {
  eligible: {
    label: 'Cocok Memenuhi Syarat',
    bg: '#dcfce7',
    color: '#166534',
    border: '#86efac',
    desc: 'Seluruh kriteria survei terpenuhi oleh fakta profil persona.',
  },
  ineligible: {
    label: 'Tidak Memenuhi Syarat',
    bg: '#fee2e2',
    color: '#991b1b',
    border: '#f87171',
    desc: 'Terdapat kriteria yang bertentangan langsung dengan profil persona.',
  },
  incomplete: {
    label: 'Data Belum Lengkap (Unknown)',
    bg: '#fef3c7',
    color: '#92400e',
    border: '#fcd34d',
    desc: 'Beberapa kriteria belum memiliki data fakta terkonfirmasi pada profil persona.',
  },
  stale: {
    label: 'Data Profil Kadaluarsa (Stale)',
    bg: '#ffedd5',
    color: '#9a3412',
    border: '#fdba74',
    desc: 'Fakta pendukung profil telah kadaluarsa dan perlu diperbarui.',
  },
  conflict: {
    label: 'Konflik Data Profil',
    bg: '#f3e8ff',
    color: '#6b21a8',
    border: '#d8b4fe',
    desc: 'Ditemukan pertentangan data fakta pada profil untuk kriteria ini.',
  },
  no_criteria: {
    label: 'Tidak Ada Kriteria Tertangkap',
    bg: '#f1f5f9',
    color: '#475569',
    border: '#cbd5e1',
    desc: 'Peluang survei ini tidak memiliki kriteria eksplisit tertangkap.',
  },
};

export function OpportunityView({ activePersona }: OpportunityViewProps) {
  // Input mode & form state
  const [rawTextInput, setRawTextInput] = useState('');
  const [platformHint, setPlatformHint] = useState('');
  const [isExtracting, setIsExtracting] = useState(false);
  const [errorMessage, setErrorMessage] = useState('');
  const [successMessage, setSuccessMessage] = useState('');

  // Screen capture & OCR for opportunity
  const [isCapturing, setIsCapturing] = useState(false);
  const [capturePreview, setCapturePreview] = useState<CapturePreview | null>(null);
  const [windowsList, setWindowsList] = useState<WindowInfo[]>([]);
  const [showWindowModal, setShowWindowModal] = useState(false);

  // Review & Edit state for extracted opportunity
  const [activeOpportunity, setActiveOpportunity] = useState<OpportunityDetail | null>(null);
  const [editTitle, setEditTitle] = useState('');
  const [editPlatform, setEditPlatform] = useState('');
  const [editTopic, setEditTopic] = useState('');
  const [editDescription, setEditDescription] = useState('');
  const [editableCriteria, setEditableCriteria] = useState<CriterionInput[]>([]);
  const [isSaving, setIsSaving] = useState(false);

  // Saved opportunities list
  const [savedOpportunities, setSavedOpportunities] = useState<SurveyOpportunity[]>([]);
  const [isLoadingList, setIsLoadingList] = useState(false);
  const [selectedSavedId, setSelectedSavedId] = useState<string | null>(null);

  // Deterministic Persona Fit state (SC-051)
  const [fitDetail, setFitDetail] = useState<PersonaOpportunityFitDetail | null>(null);
  const [isEvaluatingFit, setIsEvaluatingFit] = useState(false);
  const [personaComparison, setPersonaComparison] = useState<PersonaFitComparisonItem[] | null>(null);
  const [isComparingPersonas, setIsComparingPersonas] = useState(false);

  // Load saved opportunities on mount
  useEffect(() => {
    void loadSavedOpportunities();
  }, []);

  // Keyboard navigation: Escape key closes modal
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape' && showWindowModal) {
        setShowWindowModal(false);
      }
    }
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [showWindowModal]);

  async function loadSavedOpportunities() {
    setIsLoadingList(true);
    try {
      const list = await opportunityService.listOpportunities(50, 0);
      setSavedOpportunities(list);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsLoadingList(false);
    }
  }

  // Handle direct text extraction
  async function handleExtractFromText() {
    if (!rawTextInput.trim()) {
      setErrorMessage('Silakan masukkan teks peluang survei terlebih dahulu.');
      return;
    }

    setIsExtracting(true);
    setErrorMessage('');
    setSuccessMessage('');

    try {
      const extracted = await opportunityService.extractFromText(
        rawTextInput,
        platformHint.trim() || undefined
      );

      populateEditState(extracted);
      setSuccessMessage(
        `Berhasil mengekstrak ${extracted.criteria.length} kriteria eksplisit dari teks.`
      );
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsExtracting(false);
    }
  }

  // Handle screen capture region
  async function handleCaptureRegion() {
    setIsCapturing(true);
    setErrorMessage('');
    try {
      const preview = await captureService.captureRegion({
        sessionId: null,
        rect: null,
      });
      setCapturePreview(preview);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsCapturing(false);
    }
  }

  // Handle screen capture window
  async function handleOpenWindowModal() {
    try {
      const windows = await captureService.listWindows();
      setWindowsList(windows);
      setShowWindowModal(true);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    }
  }

  async function handleSelectWindow(window: WindowInfo) {
    setShowWindowModal(false);
    setIsCapturing(true);
    setErrorMessage('');
    try {
      const preview = await captureService.captureWindow({
        sessionId: null,
        windowId: window.id,
      });
      setCapturePreview(preview);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsCapturing(false);
    }
  }

  // Discard preview
  async function handleDiscardCapture() {
    if (!capturePreview) return;
    try {
      await captureService.discardCapture(capturePreview.captureId);
      setCapturePreview(null);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    }
  }

  // Run OCR on preview and extract opportunity criteria
  async function handleConfirmCaptureAndOcr() {
    if (!capturePreview) return;
    setIsExtracting(true);
    setErrorMessage('');
    try {
      // 1. Run OCR extraction on capture
      const ocrResult = await ocrService.extractCapture({
        captureId: capturePreview.captureId,
      });

      // 2. Extract opportunity from OCR result
      const extracted = await opportunityService.extractFromCapture(
        capturePreview.captureId,
        ocrResult.extractionId
      );

      populateEditState(extracted);
      setCapturePreview(null);
      setSuccessMessage(
        `Berhasil mengekstrak ${extracted.criteria.length} kriteria dari tangkapan layar (OCR).`
      );
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsExtracting(false);
    }
  }

  // Helper to populate editable state
  function populateEditState(detail: OpportunityDetail) {
    setActiveOpportunity(detail);
    setEditTitle(detail.opportunity.title || '');
    setEditPlatform(detail.opportunity.platformLabel || '');
    setEditTopic(detail.opportunity.topic || '');
    setEditDescription(detail.opportunity.description || '');

    setEditableCriteria(
      detail.criteria.map((c) => ({
        id: c.id,
        criterionText: c.criterionText,
        rawText: c.rawText,
        criterionType: c.criterionType,
        normalizedAttribute: c.normalizedAttribute,
        expectedValueJson: c.expectedValueJson,
        operator: c.operator,
        criterionOrder: c.criterionOrder,
      }))
    );
  }

  // Add new manual criterion
  function handleAddCriterion() {
    const newOrder = editableCriteria.length;
    setEditableCriteria([
      ...editableCriteria,
      {
        id: null,
        criterionText: '',
        rawText: 'Input manual oleh pengguna',
        criterionType: 'general',
        normalizedAttribute: null,
        expectedValueJson: null,
        operator: 'equals',
        criterionOrder: newOrder,
      },
    ]);
  }

  // Update criterion in list
  function handleUpdateCriterion(index: number, updates: Partial<CriterionInput>) {
    const copy = [...editableCriteria];
    copy[index] = { ...copy[index], ...updates };
    setEditableCriteria(copy);
  }

  // Remove criterion
  function handleRemoveCriterion(index: number) {
    const filtered = editableCriteria
      .filter((_, i) => i !== index)
      .map((c, i) => ({ ...c, criterionOrder: i }));
    setEditableCriteria(filtered);
  }

  // Save opportunity and criteria to DB
  async function handleSaveOpportunity() {
    if (!activeOpportunity) return;
    setIsSaving(true);
    setErrorMessage('');

    try {
      const validCriteria = editableCriteria.filter((c) => c.criterionText.trim().length > 0);

      const saved = await opportunityService.saveOpportunity({
        opportunity: {
          title: editTitle.trim() || null,
          platformLabel: editPlatform.trim() || null,
          topic: editTopic.trim() || null,
          description: editDescription.trim() || null,
          rawText: activeOpportunity.opportunity.rawText || rawTextInput,
          provenance: activeOpportunity.opportunity.provenance,
          sourceCaptureId: activeOpportunity.opportunity.sourceCaptureId,
          sourceOcrId: activeOpportunity.opportunity.sourceOcrId,
        },
        criteria: validCriteria,
      });

      setActiveOpportunity(saved);
      setSelectedSavedId(saved.opportunity.id);
      setSuccessMessage('Peluang survei dan kriteria berhasil disimpan ke basis data lokal.');
      await loadSavedOpportunities();
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsSaving(false);
    }
  }

  // Load a saved opportunity to inspect/edit
  async function handleSelectSavedOpportunity(id: string) {
    setSelectedSavedId(id);
    setErrorMessage('');
    setFitDetail(null);
    setPersonaComparison(null);
    try {
      const detail = await opportunityService.getOpportunity(id);
      if (detail) {
        populateEditState(detail);
        if (activePersona?.id) {
          void fitService.getOpportunityFit(id, activePersona.id).then((savedFit) => {
            if (savedFit) setFitDetail(savedFit);
          });
        }
      }
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    }
  }

  // Delete saved opportunity
  async function handleDeleteSavedOpportunity(id: string) {
    if (!confirm('Apakah Anda yakin ingin menghapus peluang survei ini?')) return;
    try {
      await opportunityService.deleteOpportunity(id);
      if (selectedSavedId === id) {
        setActiveOpportunity(null);
        setSelectedSavedId(null);
        setFitDetail(null);
        setPersonaComparison(null);
      }
      await loadSavedOpportunities();
      setSuccessMessage('Peluang survei berhasil dihapus.');
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    }
  }

  // Evaluate Persona Fit (SC-051)
  async function handleEvaluateFit() {
    if (!activeOpportunity?.opportunity.id) {
      setErrorMessage('Peluang survei harus disimpan terlebih dahulu sebelum dievaluasi kesesuaiannya.');
      return;
    }
    if (!activePersona?.id) {
      setErrorMessage('Silakan pilih persona aktif terlebih dahulu untuk mengevaluasi kesesuaian.');
      return;
    }

    setIsEvaluatingFit(true);
    setErrorMessage('');
    try {
      const result = await fitService.evaluateOpportunityFit(
        activeOpportunity.opportunity.id,
        activePersona.id
      );
      setFitDetail(result);
      setSuccessMessage(`Evaluasi kesesuaian untuk persona ${activePersona.displayName} selesai.`);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsEvaluatingFit(false);
    }
  }

  // Compare Fit Across All Personas (SC-051)
  async function handleComparePersonas() {
    if (!activeOpportunity?.opportunity.id) {
      setErrorMessage('Peluang survei harus disimpan terlebih dahulu sebelum membandingkan persona.');
      return;
    }

    setIsComparingPersonas(true);
    setErrorMessage('');
    try {
      const results = await fitService.comparePersonasFit(activeOpportunity.opportunity.id);
      setPersonaComparison(results);
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    } finally {
      setIsComparingPersonas(false);
    }
  }

  async function handleStartAttempt() {
    if (!activeOpportunity?.opportunity.id || !activePersona?.id) {
      setErrorMessage('Pilih peluang dan persona aktif untuk mencatat percobaan survei.');
      return;
    }
    try {
      await attemptService.recordAttempt({
        personaId: activePersona.id,
        opportunityId: activeOpportunity.opportunity.id,
        initialOutcome: 'pending',
        evidenceSource: 'pencatatan_dari_peluang',
        evidenceNote: `Mulai percobaan untuk peluang: ${activeOpportunity.opportunity.title || 'Survei'}`,
        confidence: 'observed',
      });
      setSuccessMessage('Percobaan survei berhasil dicatat ke Riwayat dengan status "Sedang Berjalan (Pending)". Buka tab Riwayat untuk memperbarui hasilnya.');
    } catch (err) {
      setErrorMessage(AppError.from(err).message);
    }
  }

  return (
    <div className="opportunity-container" style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      {/* Feedback Messages */}
      {errorMessage && (
        <div className="alert-error" role="alert" style={{ padding: '12px 16px', background: '#fee2e2', color: '#991b1b', borderRadius: '6px', border: '1px solid #f87171', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div>
            <strong>Peringatan: </strong> {errorMessage}
          </div>
          <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
            <button
              type="button"
              className="secondary-button"
              onClick={() => void loadSavedOpportunities()}
              aria-label="Coba muat ulang peluang survei"
              style={{ padding: '4px 8px', fontSize: '11px' }}
            >
              🔄 Coba Lagi
            </button>
            <button
              type="button"
              onClick={() => setErrorMessage('')}
              aria-label="Tutup pesan peringatan"
              style={{ background: 'transparent', border: 0, cursor: 'pointer', color: '#991b1b', fontWeight: 'bold' }}
            >
              ✕
            </button>
          </div>
        </div>
      )}
      {successMessage && (
        <div className="alert-success" role="status" aria-live="polite" style={{ padding: '12px 16px', background: '#dcfce7', color: '#166534', borderRadius: '6px', border: '1px solid #86efac', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div>
            <strong>Sukses: </strong> {successMessage}
          </div>
          <button
            type="button"
            onClick={() => setSuccessMessage('')}
            aria-label="Tutup pesan sukses"
            style={{ background: 'transparent', border: 0, cursor: 'pointer', color: '#166534', fontWeight: 'bold' }}
          >
            ✕
          </button>
        </div>
      )}

      {/* Main Grid: Left = Input & Extraction, Right = Saved List */}
      <div style={{ display: 'grid', gridTemplateColumns: 'minmax(0, 1.8fr) minmax(0, 1.2fr)', gap: '24px' }}>
        {/* Left Column: Capture / Paste & Criteria Review */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
          {/* Card: Opportunity Input */}
          <section className="card" style={{ padding: '20px', background: '#ffffff', borderRadius: '8px', border: '1px solid #e2e8f0', boxShadow: '0 1px 3px rgba(0,0,0,0.05)' }}>
            <h2 style={{ fontSize: '1.25rem', fontWeight: 600, marginBottom: '8px', color: '#0f172a' }}>
              Pencatatan & Ekstraksi Peluang Survei (SC-050)
            </h2>
            <p style={{ fontSize: '0.875rem', color: '#64748b', marginBottom: '16px' }}>
              {activePersona ? `Responden aktif: ${activePersona.displayName}. ` : ''}
              Tangkap penawaran survei dari layar atau tempel teks undangan survei. Kriteria eksplisit (usia, gender, domisili, pekerjaan, produk) akan diekstrak secara deterministik tanpa asumsi aturan tersembunyi.
            </p>

            {/* Input Action Buttons */}
            <div style={{ display: 'flex', gap: '10px', marginBottom: '16px', flexWrap: 'wrap' }}>
              <button
                type="button"
                className="btn-secondary"
                disabled={isCapturing || isExtracting}
                onClick={handleCaptureRegion}
                style={{ padding: '8px 14px', borderRadius: '6px', border: '1px solid #cbd5e1', background: '#f8fafc', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '6px' }}
              >
                <span>✂ Tangkap Area Layar</span>
              </button>
              <button
                type="button"
                className="btn-secondary"
                disabled={isCapturing || isExtracting}
                onClick={handleOpenWindowModal}
                style={{ padding: '8px 14px', borderRadius: '6px', border: '1px solid #cbd5e1', background: '#f8fafc', cursor: 'pointer', display: 'flex', alignItems: 'center', gap: '6px' }}
              >
                <span>🪟 Tangkap Jendela Aplikasi</span>
              </button>
            </div>

            {/* Screen Capture Preview Modal / Area */}
            {capturePreview && (
              <div style={{ padding: '16px', background: '#f1f5f9', borderRadius: '8px', marginBottom: '16px', border: '1px solid #cbd5e1' }}>
                <h3 style={{ fontSize: '1rem', fontWeight: 600, marginBottom: '8px', color: '#1e293b' }}>
                  Pratinjau Tangkapan Layar
                </h3>
                <img
                  src={capturePreview.previewDataUrl || ''}
                  alt="Pratinjau tangkapan peluang"
                  style={{ maxHeight: '220px', width: 'auto', borderRadius: '4px', border: '1px solid #94a3b8', display: 'block', marginBottom: '12px' }}
                />
                <div style={{ display: 'flex', gap: '10px' }}>
                  <button
                    type="button"
                    className="btn-primary"
                    disabled={isExtracting}
                    onClick={handleConfirmCaptureAndOcr}
                    style={{ padding: '8px 16px', background: '#2563eb', color: '#fff', border: 'none', borderRadius: '6px', cursor: 'pointer', fontWeight: 500 }}
                  >
                    {isExtracting ? 'Membaca OCR & Ekstraksi...' : '✓ Konfirmasi & Ekstrak Kriteria'}
                  </button>
                  <button
                    type="button"
                    className="btn-secondary"
                    disabled={isExtracting}
                    onClick={handleDiscardCapture}
                    style={{ padding: '8px 14px', background: '#fff', border: '1px solid #cbd5e1', borderRadius: '6px', cursor: 'pointer', color: '#dc2626' }}
                  >
                    Batal
                  </button>
                </div>
              </div>
            )}

            {/* Text Paste Input */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
              <div>
                <label htmlFor="opp-platform-hint" style={{ display: 'block', fontSize: '0.8125rem', fontWeight: 500, color: '#334155', marginBottom: '4px' }}>
                  Petunjuk Label Platform (Opsional, misal: YouGov, Milieu, Toluna):
                </label>
                <input
                  id="opp-platform-hint"
                  type="text"
                  placeholder="Contoh: YouGov / Toluna / Jakpat"
                  value={platformHint}
                  onChange={(e) => setPlatformHint(e.target.value)}
                  style={{ width: '100%', padding: '8px 12px', borderRadius: '6px', border: '1px solid #cbd5e1', fontSize: '0.875rem' }}
                />
              </div>

              <div>
                <label htmlFor="opp-raw-text" style={{ display: 'block', fontSize: '0.8125rem', fontWeight: 500, color: '#334155', marginBottom: '4px' }}>
                  Teks Sumber Peluang Survei:
                </label>
                <textarea
                  id="opp-raw-text"
                  rows={6}
                  placeholder="Tempel teks peluang survei di sini, misal:&#10;Survei Kepuasan Konsumen Kopi Susu&#10;Platform: YouGov&#10;Kriteria:&#10;- Pria dan Wanita usia 20-35 tahun&#10;- Domisili Jabodetabek&#10;- Mengkonsumsi kopi kekinian minimal 2x seminggu&#10;- Bukan pekerja di industri riset pasar"
                  value={rawTextInput}
                  onChange={(e) => setRawTextInput(e.target.value)}
                  style={{ width: '100%', padding: '10px 12px', borderRadius: '6px', border: '1px solid #cbd5e1', fontSize: '0.875rem', fontFamily: 'inherit' }}
                />
              </div>

              <button
                type="button"
                className="btn-primary"
                disabled={isExtracting || !rawTextInput.trim()}
                onClick={handleExtractFromText}
                style={{ alignSelf: 'flex-start', padding: '10px 18px', background: '#2563eb', color: '#fff', border: 'none', borderRadius: '6px', cursor: 'pointer', fontWeight: 500 }}
              >
                {isExtracting ? 'Mengekstrak Kriteria...' : '🔍 Ekstrak Kriteria dari Teks'}
              </button>
            </div>
          </section>

          {/* Card: Extracted Opportunity Review & Criteria Editor */}
          {activeOpportunity && (
            <section className="card" style={{ padding: '20px', background: '#ffffff', borderRadius: '8px', border: '1px solid #e2e8f0', boxShadow: '0 1px 3px rgba(0,0,0,0.05)' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                <h3 style={{ fontSize: '1.125rem', fontWeight: 600, color: '#0f172a' }}>
                  Ulas & Koreksi Kriteria (Provenance: {activeOpportunity.opportunity.provenance})
                </h3>
                <span style={{ fontSize: '0.75rem', padding: '4px 8px', borderRadius: '4px', background: '#e0f2fe', color: '#0369a1', fontWeight: 500 }}>
                  Status: {activeOpportunity.opportunity.status}
                </span>
              </div>

              {/* Opportunity Metadata Fields */}
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '12px', marginBottom: '16px' }}>
                <div>
                  <label style={{ display: 'block', fontSize: '0.8125rem', fontWeight: 500, color: '#334155', marginBottom: '4px' }}>
                    Judul Peluang:
                  </label>
                  <input
                    type="text"
                    value={editTitle}
                    onChange={(e) => setEditTitle(e.target.value)}
                    placeholder="Judul survei terlihat"
                    style={{ width: '100%', padding: '7px 10px', borderRadius: '6px', border: '1px solid #cbd5e1', fontSize: '0.875rem' }}
                  />
                </div>
                <div>
                  <label style={{ display: 'block', fontSize: '0.8125rem', fontWeight: 500, color: '#334155', marginBottom: '4px' }}>
                    Label Platform:
                  </label>
                  <input
                    type="text"
                    value={editPlatform}
                    onChange={(e) => setEditPlatform(e.target.value)}
                    placeholder="Platform (misal: YouGov)"
                    style={{ width: '100%', padding: '7px 10px', borderRadius: '6px', border: '1px solid #cbd5e1', fontSize: '0.875rem' }}
                  />
                </div>
                <div>
                  <label style={{ display: 'block', fontSize: '0.8125rem', fontWeight: 500, color: '#334155', marginBottom: '4px' }}>
                    Topik / Kategori:
                  </label>
                  <input
                    type="text"
                    value={editTopic}
                    onChange={(e) => setEditTopic(e.target.value)}
                    placeholder="Topik (misal: Makanan & Minuman)"
                    style={{ width: '100%', padding: '7px 10px', borderRadius: '6px', border: '1px solid #cbd5e1', fontSize: '0.875rem' }}
                  />
                </div>
              </div>

              <div>
                <label style={{ display: 'block', fontSize: '0.8125rem', fontWeight: 500, color: '#334155', marginBottom: '4px' }}>
                  Deskripsi / Konteks Terlihat:
                </label>
                <input
                  type="text"
                  value={editDescription}
                  onChange={(e) => setEditDescription(e.target.value)}
                  placeholder="Keterangan singkat peluang survei"
                  style={{ width: '100%', padding: '7px 10px', borderRadius: '6px', border: '1px solid #cbd5e1', fontSize: '0.875rem', marginBottom: '16px' }}
                />
              </div>

              {/* Criteria List */}
              <div style={{ marginBottom: '16px' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '10px' }}>
                  <h4 style={{ fontSize: '0.9375rem', fontWeight: 600, color: '#334155' }}>
                    Daftar Kriteria Eksplisit ({editableCriteria.length}):
                  </h4>
                  <button
                    type="button"
                    onClick={handleAddCriterion}
                    style={{ fontSize: '0.8125rem', padding: '4px 10px', borderRadius: '4px', border: '1px solid #cbd5e1', background: '#f8fafc', cursor: 'pointer' }}
                  >
                    + Tambah Kriteria Manual
                  </button>
                </div>

                {editableCriteria.length === 0 ? (
                  <p style={{ fontSize: '0.875rem', color: '#94a3b8', fontStyle: 'italic', padding: '12px', background: '#f8fafc', borderRadius: '6px' }}>
                    Tidak ada kriteria eksplisit yang terdeteksi. Silakan tambah kriteria secara manual atau periksa kembali teks sumber.
                  </p>
                ) : (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    {editableCriteria.map((c, index) => (
                      <div
                        key={c.id || `crit-${index}`}
                        style={{ padding: '12px', border: '1px solid #e2e8f0', borderRadius: '6px', background: '#f8fafc' }}
                      >
                        <div style={{ display: 'flex', gap: '10px', alignItems: 'center', marginBottom: '8px' }}>
                          <span style={{ fontSize: '0.75rem', fontWeight: 600, color: '#64748b' }}>
                            #{index + 1}
                          </span>

                          {/* Criterion Type Selector */}
                          <select
                            value={c.criterionType}
                            onChange={(e) =>
                              handleUpdateCriterion(index, {
                                criterionType: e.target.value as CriterionType,
                              })
                            }
                            style={{ padding: '4px 8px', borderRadius: '4px', border: '1px solid #cbd5e1', fontSize: '0.8125rem', background: '#fff' }}
                          >
                            {Object.entries(CRITERION_TYPE_LABELS).map(([k, label]) => (
                              <option key={k} value={k}>
                                {label}
                              </option>
                            ))}
                          </select>

                          {/* Operator Selector */}
                          <select
                            value={c.operator}
                            onChange={(e) =>
                              handleUpdateCriterion(index, {
                                operator: e.target.value as CriterionOperator,
                              })
                            }
                            style={{ padding: '4px 8px', borderRadius: '4px', border: '1px solid #cbd5e1', fontSize: '0.8125rem', background: '#fff' }}
                          >
                            {Object.entries(OPERATOR_LABELS).map(([k, label]) => (
                              <option key={k} value={k}>
                                {label}
                              </option>
                            ))}
                          </select>

                          <button
                            type="button"
                            onClick={() => handleRemoveCriterion(index)}
                            style={{ marginLeft: 'auto', background: 'transparent', border: 'none', color: '#ef4444', cursor: 'pointer', fontSize: '0.8125rem' }}
                            title="Hapus kriteria ini"
                          >
                            ✕ Hapus
                          </button>
                        </div>

                        {/* Criterion Text Editor */}
                        <div style={{ marginBottom: '6px' }}>
                          <input
                            type="text"
                            value={c.criterionText}
                            onChange={(e) =>
                              handleUpdateCriterion(index, { criterionText: e.target.value })
                            }
                            placeholder="Deskripsi kriteria yang jelas (misal: Usia 20 - 35 tahun)"
                            style={{ width: '100%', padding: '6px 10px', borderRadius: '4px', border: '1px solid #cbd5e1', fontSize: '0.875rem' }}
                          />
                        </div>

                        {/* Traceability: Original Raw Text Substring */}
                        {c.rawText && (
                          <div style={{ fontSize: '0.75rem', color: '#64748b', background: '#f1f5f9', padding: '4px 8px', borderRadius: '4px' }}>
                            <span style={{ fontWeight: 500 }}>Teks Sumber Asli: </span>
                            <em>&ldquo;{c.rawText}&rdquo;</em>
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {/* Action Buttons: Save */}
              <div style={{ display: 'flex', gap: '10px', marginTop: '16px' }}>
                <button
                  type="button"
                  className="btn-primary"
                  disabled={isSaving}
                  onClick={handleSaveOpportunity}
                  style={{ padding: '10px 20px', background: '#16a34a', color: '#fff', border: 'none', borderRadius: '6px', cursor: 'pointer', fontWeight: 600 }}
                >
                  {isSaving ? 'Menyimpan...' : '💾 Simpan Peluang Survei & Kriteria'}
                </button>
                <button
                  type="button"
                  className="btn-secondary"
                  disabled={isSaving}
                  onClick={() => setActiveOpportunity(null)}
                  style={{ padding: '10px 16px', background: '#fff', border: '1px solid #cbd5e1', borderRadius: '6px', cursor: 'pointer' }}
                >
                  Tutup Editor
                </button>
              </div>
            </section>
          )}

          {/* SC-051 Deterministic Persona Fit Section */}
          {activeOpportunity?.opportunity.id && (
            <section
              className="card"
              style={{
                padding: '20px',
                background: '#ffffff',
                borderRadius: '8px',
                border: '1px solid #e2e8f0',
                boxShadow: '0 1px 3px rgba(0,0,0,0.05)',
              }}
            >
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  marginBottom: '14px',
                  flexWrap: 'wrap',
                  gap: '8px',
                }}
              >
                <div>
                  <h3 style={{ fontSize: '1.125rem', fontWeight: 600, color: '#0f172a', margin: 0 }}>
                    Evaluasi Kesesuaian Persona (Survey Fit — SC-051)
                  </h3>
                  <p style={{ fontSize: '0.8125rem', color: '#64748b', margin: '4px 0 0 0' }}>
                    Pencocokan kriteria eksplisit secara deterministik lokal terhadap fakta profil persona nyata.
                  </p>
                </div>
                <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
                  <button
                    type="button"
                    className="btn-primary"
                    disabled={isEvaluatingFit || !activePersona?.id}
                    onClick={handleEvaluateFit}
                    style={{
                      padding: '8px 14px',
                      background: '#2563eb',
                      color: '#fff',
                      border: 'none',
                      borderRadius: '6px',
                      cursor: 'pointer',
                      fontSize: '0.8125rem',
                      fontWeight: 500,
                    }}
                  >
                    {isEvaluatingFit
                      ? 'Mengevaluasi...'
                      : `🎯 Evaluasi untuk ${activePersona?.displayName || 'Persona Aktif'}`}
                  </button>
                  <button
                    type="button"
                    className="btn-secondary"
                    disabled={isComparingPersonas}
                    onClick={handleComparePersonas}
                    style={{
                      padding: '8px 14px',
                      background: '#f8fafc',
                      color: '#0f172a',
                      border: '1px solid #cbd5e1',
                      borderRadius: '6px',
                      cursor: 'pointer',
                      fontSize: '0.8125rem',
                      fontWeight: 500,
                    }}
                  >
                    {isComparingPersonas ? 'Membandingkan...' : '👥 Bandingkan Semua Persona'}
                  </button>
                  <button
                    type="button"
                    className="btn-secondary"
                    onClick={handleStartAttempt}
                    style={{
                      padding: '8px 14px',
                      background: '#eff6ff',
                      color: '#1d4ed8',
                      border: '1px solid #93c5fd',
                      borderRadius: '6px',
                      cursor: 'pointer',
                      fontSize: '0.8125rem',
                      fontWeight: 600,
                    }}
                  >
                    📊 Mulai / Catat Percobaan (SC-052)
                  </button>
                </div>
              </div>

              {/* Overall Fit Badge & Summary */}
              {fitDetail && (
                <div style={{ marginBottom: '16px' }}>
                  <div
                    style={{
                      padding: '14px',
                      borderRadius: '8px',
                      border: `1px solid ${OVERALL_FIT_CONFIG[fitDetail.overallFit].border}`,
                      background: OVERALL_FIT_CONFIG[fitDetail.overallFit].bg,
                      color: OVERALL_FIT_CONFIG[fitDetail.overallFit].color,
                      marginBottom: '12px',
                    }}
                  >
                    <div
                      style={{
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        marginBottom: '6px',
                      }}
                    >
                      <span style={{ fontWeight: 700, fontSize: '1rem' }}>
                        Status Keseluruhan: {OVERALL_FIT_CONFIG[fitDetail.overallFit].label}
                      </span>
                      <span style={{ fontSize: '0.75rem', opacity: 0.9 }}>
                        Dievaluasi: {new Date(fitDetail.evaluatedAt).toLocaleTimeString('id-ID')}
                      </span>
                    </div>
                    <p style={{ margin: 0, fontSize: '0.875rem' }}>
                      {OVERALL_FIT_CONFIG[fitDetail.overallFit].desc}
                    </p>
                  </div>

                  {/* Metric Badges */}
                  <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', marginBottom: '14px' }}>
                    <span
                      style={{
                        fontSize: '0.75rem',
                        padding: '4px 8px',
                        borderRadius: '4px',
                        background: '#dcfce7',
                        color: '#166534',
                        fontWeight: 600,
                      }}
                    >
                      ✓ {fitDetail.matchCount} Cocok
                    </span>
                    <span
                      style={{
                        fontSize: '0.75rem',
                        padding: '4px 8px',
                        borderRadius: '4px',
                        background: '#fee2e2',
                        color: '#991b1b',
                        fontWeight: 600,
                      }}
                    >
                      ✕ {fitDetail.mismatchCount} Tidak Cocok
                    </span>
                    <span
                      style={{
                        fontSize: '0.75rem',
                        padding: '4px 8px',
                        borderRadius: '4px',
                        background: '#fef3c7',
                        color: '#92400e',
                        fontWeight: 600,
                      }}
                    >
                      ? {fitDetail.unknownCount} Belum Diketahui
                    </span>
                    {fitDetail.staleCount > 0 && (
                      <span
                        style={{
                          fontSize: '0.75rem',
                          padding: '4px 8px',
                          borderRadius: '4px',
                          background: '#ffedd5',
                          color: '#9a3412',
                          fontWeight: 600,
                        }}
                      >
                        ⏱ {fitDetail.staleCount} Kadaluarsa
                      </span>
                    )}
                    {fitDetail.conflictCount > 0 && (
                      <span
                        style={{
                          fontSize: '0.75rem',
                          padding: '4px 8px',
                          borderRadius: '4px',
                          background: '#f3e8ff',
                          color: '#6b21a8',
                          fontWeight: 600,
                        }}
                      >
                        ⚠ {fitDetail.conflictCount} Konflik Data
                      </span>
                    )}
                  </div>

                  {/* Per-Criterion Cards */}
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                    {fitDetail.criteriaResults.map((res, i) => {
                      const badge = MATCH_STATE_CONFIG[res.matchState];
                      return (
                        <div
                          key={res.criterionId || `res-${i}`}
                          style={{
                            padding: '12px',
                            borderRadius: '6px',
                            border: `1px solid ${badge.border}`,
                            background: '#ffffff',
                          }}
                        >
                          <div
                            style={{
                              display: 'flex',
                              justifyContent: 'space-between',
                              alignItems: 'flex-start',
                              marginBottom: '6px',
                            }}
                          >
                            <strong style={{ fontSize: '0.875rem', color: '#1e293b' }}>
                              #{i + 1} {res.criterionText}
                            </strong>
                            <span
                              style={{
                                fontSize: '0.75rem',
                                padding: '3px 8px',
                                borderRadius: '4px',
                                fontWeight: 600,
                                background: badge.bg,
                                color: badge.color,
                                border: `1px solid ${badge.border}`,
                                whiteSpace: 'nowrap',
                              }}
                            >
                              {badge.label}
                            </span>
                          </div>
                          <p style={{ margin: '0 0 6px 0', fontSize: '0.8125rem', color: '#475569' }}>
                            {res.explanation}
                          </p>
                          {res.supportingFactsSummary.length > 0 && (
                            <div
                              style={{
                                fontSize: '0.75rem',
                                color: '#0369a1',
                                background: '#f0f9ff',
                                padding: '4px 8px',
                                borderRadius: '4px',
                              }}
                            >
                              <strong>Fakta Terkait: </strong> {res.supportingFactsSummary.join(', ')}
                            </div>
                          )}
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}

              {/* Persona Comparison Table (Truthful Respondent Selection) */}
              {personaComparison && (
                <div style={{ marginTop: '20px', borderTop: '1px solid #e2e8f0', paddingTop: '16px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px' }}>
                    <h4 style={{ fontSize: '0.9375rem', fontWeight: 600, color: '#0f172a', margin: 0 }}>
                      Perbandingan Lintas Persona (Pemilihan Responden Jujur)
                    </h4>
                  </div>
                  <div
                    style={{
                      fontSize: '0.75rem',
                      color: '#64748b',
                      background: '#f8fafc',
                      padding: '8px 12px',
                      borderRadius: '6px',
                      border: '1px solid #e2e8f0',
                      marginBottom: '12px',
                    }}
                  >
                    💡 <strong>Prinsip Integritas:</strong> Evaluasi ini membantu memilih profil persona
                    nyata yang benar-benar memenuhi kriteria. Dilarang memalsukan identitas atau jawaban
                    demi meloloskan screener survei.
                  </div>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                    {personaComparison.map((p) => {
                      const cfg = OVERALL_FIT_CONFIG[p.overallFit];
                      const isCurrentActive = activePersona?.id === p.personaId;
                      return (
                        <div
                          key={p.personaId}
                          style={{
                            padding: '10px 14px',
                            borderRadius: '6px',
                            border: `1px solid ${cfg.border}`,
                            background: isCurrentActive ? '#f0fdf4' : '#ffffff',
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                            flexWrap: 'wrap',
                            gap: '8px',
                          }}
                        >
                          <div>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                              <strong style={{ fontSize: '0.875rem', color: '#1e293b' }}>
                                {p.personaName}
                              </strong>
                              {isCurrentActive && (
                                <span
                                  style={{
                                    fontSize: '0.6875rem',
                                    padding: '1px 6px',
                                    borderRadius: '4px',
                                    background: '#bbf7d0',
                                    color: '#14532d',
                                    fontWeight: 600,
                                  }}
                                >
                                  Aktif
                                </span>
                              )}
                            </div>
                            <div style={{ fontSize: '0.75rem', color: '#64748b', marginTop: '2px' }}>
                              Cocok: {p.matchCount}/{p.totalCriteria} kriteria{' '}
                              {p.keyReasons.length > 0 && `• ${p.keyReasons.join('; ')}`}
                            </div>
                          </div>
                          <span
                            style={{
                              fontSize: '0.75rem',
                              padding: '3px 8px',
                              borderRadius: '4px',
                              fontWeight: 600,
                              background: cfg.bg,
                              color: cfg.color,
                              border: `1px solid ${cfg.border}`,
                            }}
                          >
                            {cfg.label}
                          </span>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}
            </section>
          )}
        </div>

        {/* Right Column: Saved Opportunities List */}
        <div>
          <section className="card" style={{ padding: '20px', background: '#ffffff', borderRadius: '8px', border: '1px solid #e2e8f0', boxShadow: '0 1px 3px rgba(0,0,0,0.05)', height: '100%' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
              <h3 style={{ fontSize: '1.125rem', fontWeight: 600, color: '#0f172a' }}>
                Peluang Tersimpan ({savedOpportunities.length})
              </h3>
              <button
                type="button"
                onClick={loadSavedOpportunities}
                disabled={isLoadingList}
                style={{ fontSize: '0.75rem', padding: '4px 8px', borderRadius: '4px', border: '1px solid #cbd5e1', background: '#f8fafc', cursor: 'pointer' }}
              >
                🔄 Segarkan
              </button>
            </div>

            {isLoadingList ? (
              <p style={{ fontSize: '0.875rem', color: '#64748b' }}>Memuat daftar...</p>
            ) : savedOpportunities.length === 0 ? (
              <p style={{ fontSize: '0.875rem', color: '#94a3b8', fontStyle: 'italic' }}>
                Belum ada peluang survei yang tersimpan. Tangkap atau masukkan teks di sebelah kiri untuk mulai menyimpan.
              </p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '10px', maxHeight: '650px', overflowY: 'auto' }}>
                {savedOpportunities.map((opp) => (
                  <div
                    key={opp.id}
                    onClick={() => handleSelectSavedOpportunity(opp.id)}
                    style={{
                      padding: '12px',
                      borderRadius: '6px',
                      border: selectedSavedId === opp.id ? '2px solid #2563eb' : '1px solid #e2e8f0',
                      background: selectedSavedId === opp.id ? '#eff6ff' : '#f8fafc',
                      cursor: 'pointer',
                      transition: 'all 0.15s ease',
                    }}
                  >
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '4px' }}>
                      <strong style={{ fontSize: '0.9375rem', color: '#1e293b' }}>
                        {opp.title || 'Survei Tanpa Judul'}
                      </strong>
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          void handleDeleteSavedOpportunity(opp.id);
                        }}
                        style={{ border: 'none', background: 'transparent', color: '#ef4444', cursor: 'pointer', fontSize: '0.75rem' }}
                        title="Hapus"
                      >
                        ✕
                      </button>
                    </div>

                    <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap', marginBottom: '6px' }}>
                      {opp.platformLabel && (
                        <span style={{ fontSize: '0.6875rem', padding: '2px 6px', background: '#e2e8f0', color: '#334155', borderRadius: '4px', fontWeight: 500 }}>
                          {opp.platformLabel}
                        </span>
                      )}
                      {opp.topic && (
                        <span style={{ fontSize: '0.6875rem', padding: '2px 6px', background: '#dbeafe', color: '#1e40af', borderRadius: '4px' }}>
                          {opp.topic}
                        </span>
                      )}
                      <span style={{ fontSize: '0.6875rem', padding: '2px 6px', background: '#fef3c7', color: '#92400e', borderRadius: '4px' }}>
                        {opp.provenance}
                      </span>
                    </div>

                    <div style={{ fontSize: '0.75rem', color: '#64748b' }}>
                      Tertangkap: {new Date(opp.capturedAt).toLocaleString('id-ID')}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </section>
        </div>
      </div>

      {/* Window Select Modal */}
      {showWindowModal && (
        <div style={{ position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.5)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000 }}>
          <div role="dialog" aria-modal="true" aria-labelledby="modal-opp-window-title" style={{ background: '#fff', padding: '24px', borderRadius: '8px', maxWidth: '500px', width: '90%', maxHeight: '80vh', overflowY: 'auto' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
              <h3 id="modal-opp-window-title" style={{ fontSize: '1.125rem', fontWeight: 600, margin: 0 }}>Pilih Jendela Aplikasi</h3>
              <button
                type="button"
                onClick={() => setShowWindowModal(false)}
                aria-label="Tutup pemilih jendela aplikasi"
                style={{ background: 'transparent', border: 0, cursor: 'pointer', fontSize: '16px', fontWeight: 'bold', color: '#64748b' }}
              >
                ✕
              </button>
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '16px' }}>
              {windowsList.map((w, idx) => (
                <button
                  key={`${w.id}-${idx}`}
                  type="button"
                  onClick={() => handleSelectWindow(w)}
                  aria-label={`Pilih jendela ${w.title}`}
                  style={{ textAlign: 'left', padding: '10px', border: '1px solid #cbd5e1', borderRadius: '6px', background: '#f8fafc', cursor: 'pointer' }}
                >
                  <div style={{ fontWeight: 500, fontSize: '0.875rem' }}>{w.title}</div>
                  <div style={{ fontSize: '0.75rem', color: '#64748b' }}>{w.appName || 'Aplikasi'}</div>
                </button>
              ))}
            </div>
            <button
              type="button"
              onClick={() => setShowWindowModal(false)}
              aria-label="Tutup dialog pemilih jendela"
              style={{ width: '100%', padding: '8px', border: '1px solid #cbd5e1', borderRadius: '6px', background: '#fff', cursor: 'pointer' }}
            >
              Tutup
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
