import { useEffect, useState } from "react";
import type { Persona } from "../../types/persona";
import type {
  AllRecordCounts,
  ExportSummary,
  RetentionPolicyInfo,
  WipeReport,
} from "../../types/dataManagement";
import { dataManagementService } from "../../services/dataManagementService";
import { AppError } from "../../types/error";

type SettingsTab = "export" | "retention" | "danger";

interface SettingsViewProps {
  personas: Persona[];
  activePersona?: Persona;
  onRefreshPersonas?: () => Promise<void>;
  onDeletePersona?: (personaId: string) => Promise<void>;
}

export function SettingsView({
  personas,
  activePersona,
  onRefreshPersonas,
  onDeletePersona,
}: SettingsViewProps) {
  const [activeTab, setActiveTab] = useState<SettingsTab>("export");

  // Export State
  const [selectedPersonaId, setSelectedPersonaId] = useState<string>(activePersona ? activePersona.id : "ALL");
  const [exportSummary, setExportSummary] = useState<ExportSummary | null>(null);
  const [isExportSummaryLoading, setIsExportSummaryLoading] = useState(false);
  const [isDownloading, setIsDownloading] = useState(false);
  const [exportSuccessMessage, setExportSuccessMessage] = useState<string | null>(null);
  const [exportErrorMessage, setExportErrorMessage] = useState<string | null>(null);

  // Retention Info State
  const [retentionInfo, setRetentionInfo] = useState<RetentionPolicyInfo | null>(null);
  const [isRetentionLoading, setIsRetentionLoading] = useState(false);

  // Danger Zone / Wipe State
  const [wipeSummary, setWipeSummary] = useState<AllRecordCounts | null>(null);
  const [isWipeSummaryLoading, setIsWipeSummaryLoading] = useState(false);
  const [wipeConfirmationInput, setWipeConfirmationInput] = useState("");
  const [isConsentChecked, setIsConsentChecked] = useState(false);
  const [isWiping, setIsWiping] = useState(false);
  const [wipeReport, setWipeReport] = useState<WipeReport | null>(null);
  const [wipeErrorMessage, setWipeErrorMessage] = useState<string | null>(null);

  // Single Persona Delete in Settings
  const [personaToDeleteId, setPersonaToDeleteId] = useState<string>("");
  const [isDeletingPersona, setIsDeletingPersona] = useState(false);

  useEffect(() => {
    if (activeTab === "retention" && !retentionInfo) {
      void loadRetentionInfo();
    }
  }, [activeTab]);

  async function loadRetentionInfo() {
    setIsRetentionLoading(true);
    try {
      const info = await dataManagementService.getRetentionInfo();
      setRetentionInfo(info);
    } catch (err) {
      console.error("Gagal memuat kebijakan retensi:", err);
    } finally {
      setIsRetentionLoading(false);
    }
  }

  async function handleLoadExportSummary() {
    setIsExportSummaryLoading(true);
    setExportErrorMessage(null);
    setExportSuccessMessage(null);
    try {
      const personaId = selectedPersonaId === "ALL" ? undefined : selectedPersonaId;
      const summary = await dataManagementService.getExportSummary(personaId);
      setExportSummary(summary);
    } catch (err) {
      setExportErrorMessage(AppError.from(err).message);
    } finally {
      setIsExportSummaryLoading(false);
    }
  }

  async function handleDownloadExport() {
    setIsDownloading(true);
    setExportErrorMessage(null);
    setExportSuccessMessage(null);
    try {
      const personaId = selectedPersonaId === "ALL" ? undefined : selectedPersonaId;
      const jsonContent = await dataManagementService.generateExportJson(personaId);

      const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
      const targetLabel =
        selectedPersonaId === "ALL"
          ? "semua"
          : (personas.find((p) => p.id === selectedPersonaId)?.displayName.toLowerCase().replace(/\s+/g, "-") || "persona");
      const filename = `survey-copilot-export-${targetLabel}-${timestamp}.json`;

      dataManagementService.triggerFileDownload(jsonContent, filename);
      setExportSuccessMessage(`Berkas ekspor berhasil diunduh: ${filename}`);
    } catch (err) {
      setExportErrorMessage(AppError.from(err).message);
    } finally {
      setIsDownloading(false);
    }
  }

  async function handleLoadWipeSummary() {
    setIsWipeSummaryLoading(true);
    setWipeErrorMessage(null);
    try {
      const summary = await dataManagementService.getWipeSummary();
      setWipeSummary(summary);
    } catch (err) {
      setWipeErrorMessage(AppError.from(err).message);
    } finally {
      setIsWipeSummaryLoading(false);
    }
  }

  async function handleExecuteWipe() {
    if (wipeConfirmationInput.trim() !== "HAPUS SEMUA DATA") {
      setWipeErrorMessage("Harap ketik tepat 'HAPUS SEMUA DATA' untuk mengonfirmasi.");
      return;
    }
    if (!isConsentChecked) {
      setWipeErrorMessage("Harap centang persetujuan pemahaman konsekuensi sebelum melanjutkan.");
      return;
    }

    setIsWiping(true);
    setWipeErrorMessage(null);
    try {
      const report = await dataManagementService.wipeAllData("HAPUS SEMUA DATA");
      setWipeReport(report);
      setWipeSummary(null);
      setWipeConfirmationInput("");
      setIsConsentChecked(false);
      if (onRefreshPersonas) {
        await onRefreshPersonas();
      }
    } catch (err) {
      setWipeErrorMessage(AppError.from(err).message);
    } finally {
      setIsWiping(false);
    }
  }

  async function handleDeleteSelectedPersona() {
    if (!personaToDeleteId) return;
    if (!window.confirm("Hapus persona ini beserta seluruh fakta dan riwayatnya secara permanen?")) return;
    setIsDeletingPersona(true);
    try {
      if (onDeletePersona) {
        await onDeletePersona(personaToDeleteId);
      }
      setPersonaToDeleteId("");
      if (onRefreshPersonas) {
        await onRefreshPersonas();
      }
    } catch (err) {
      alert(AppError.from(err).message);
    } finally {
      setIsDeletingPersona(false);
    }
  }

  return (
    <div className="content settings-container">
      <div className="settings-header">
        <h2>⚙ Pengaturan Data & Privasi</h2>
        <p className="settings-subtitle">
          Kelola pencadangan data, ekspor JSON terstruktur, transparansi privasi, serta pembersihan data secara aman.
        </p>
      </div>

      <nav className="settings-nav-tabs" aria-label="Tab Pengaturan">
        <button
          className={`tab-btn ${activeTab === "export" ? "active" : ""}`}
          onClick={() => setActiveTab("export")}
        >
          ⬇ Ekspor & Cadangan Data
        </button>
        <button
          className={`tab-btn ${activeTab === "retention" ? "active" : ""}`}
          onClick={() => setActiveTab("retention")}
        >
          🛡 Kebijakan Privasi & Siklus Data
        </button>
        <button
          className={`tab-btn tab-danger ${activeTab === "danger" ? "active" : ""}`}
          onClick={() => {
            setActiveTab("danger");
            if (!wipeSummary) {
              void handleLoadWipeSummary();
            }
          }}
        >
          ⚠ Zona Bahaya: Hapus Data
        </button>
      </nav>

      {/* TAB 1: EKSPOR DATA */}
      {activeTab === "export" && (
        <section className="settings-tab-content" aria-labelledby="export-heading">
          <div className="card">
            <h3 id="export-heading">Ekspor Cadangan Data Lokal</h3>
            <p className="description-text">
              Ekspor seluruh data profil dan riwayat survei ke berkas JSON terstruktur yang aman. Sesuai arsitektur keamanan, kunci API dan kredensial penyedia AI secara mutlak <strong>dikecualikan 100%</strong> dari berkas ekspor.
            </p>

            <div className="export-scope-selector form-group">
              <label htmlFor="export-scope-select">Pilih Cakupan Ekspor:</label>
              <select
                id="export-scope-select"
                value={selectedPersonaId}
                onChange={(e) => {
                  setSelectedPersonaId(e.target.value);
                  setExportSummary(null);
                  setExportSuccessMessage(null);
                  setExportErrorMessage(null);
                }}
                className="select-input"
              >
                <option value="ALL">📦 Seluruh Data Aplikasi (Semua Persona & Riwayat)</option>
                {personas.map((p) => (
                  <option key={p.id} value={p.id}>
                    👤 Persona: {p.displayName} {p.isActive ? "(Aktif)" : ""}
                  </option>
                ))}
              </select>
            </div>

            <div className="export-actions">
              <button
                className="btn btn-secondary"
                onClick={() => void handleLoadExportSummary()}
                disabled={isExportSummaryLoading || isDownloading}
              >
                {isExportSummaryLoading ? "Memeriksa Data..." : "🔍 Periksa Ringkasan Isi Ekspor"}
              </button>
              <button
                className="btn btn-primary"
                onClick={() => void handleDownloadExport()}
                disabled={isDownloading}
              >
                {isDownloading ? "Mengunduh..." : "⬇ Unduh Berkas JSON (.json)"}
              </button>
            </div>

            {exportSuccessMessage && (
              <div className="alert alert-success" role="alert">
                ✓ {exportSuccessMessage}
              </div>
            )}

            {exportErrorMessage && (
              <div className="alert alert-error" role="alert">
                ✕ {exportErrorMessage}
              </div>
            )}

            {/* RINGKASAN PREVIEW EKSPOR */}
            {exportSummary && (
              <div className="export-summary-box">
                <h4>📋 Ringkasan Isi Berkas Ekspor</h4>
                <div className="summary-meta-grid">
                  <div className="summary-item">
                    <span className="summary-label">Cakupan Ekspor:</span>
                    <span className="summary-val">
                      {exportSummary.exportScope === "full_application"
                        ? "Seluruh Aplikasi"
                        : `Persona: ${exportSummary.targetPersonaName || "Terpilih"}`}
                    </span>
                  </div>
                  <div className="summary-item">
                    <span className="summary-label">Total Persona:</span>
                    <span className="summary-val">{exportSummary.totalPersonas}</span>
                  </div>
                  <div className="summary-item">
                    <span className="summary-label">Total Fakta Profil:</span>
                    <span className="summary-val">{exportSummary.totalFacts}</span>
                  </div>
                  <div className="summary-item">
                    <span className="summary-label">Sumber Bukti:</span>
                    <span className="summary-val">{exportSummary.totalSources}</span>
                  </div>
                  <div className="summary-item">
                    <span className="summary-label">Riwayat Perubahan:</span>
                    <span className="summary-val">{exportSummary.totalHistoryEntries}</span>
                  </div>
                  <div className="summary-item">
                    <span className="summary-label">Sesi Survei:</span>
                    <span className="summary-val">{exportSummary.totalSessions}</span>
                  </div>
                  <div className="summary-item">
                    <span className="summary-label">Pertanyaan:</span>
                    <span className="summary-val">{exportSummary.totalQuestions}</span>
                  </div>
                  <div className="summary-item">
                    <span className="summary-label">Jawaban & Rekomendasi:</span>
                    <span className="summary-val">{exportSummary.totalAnswers}</span>
                  </div>
                </div>

                {exportSummary.hasSensitiveData && (
                  <div className="alert alert-warning" role="alert">
                    <strong>⚠ Peringatan Privasi:</strong> {exportSummary.warningMessage}
                  </div>
                )}

                <div className="security-guarantee-badge">
                  🛡 <strong>Jaminan Keamanan:</strong> Kunci API dan rahasia Windows Credential Manager secara mutlak tidak disertakan dalam berkas ekspor ini.
                </div>
              </div>
            )}
          </div>
        </section>
      )}

      {/* TAB 2: KEBIJAKAN PRIVASI & RETENSI */}
      {activeTab === "retention" && (
        <section className="settings-tab-content" aria-labelledby="retention-heading">
          <div className="card">
            <h3 id="retention-heading">Kebijakan Privasi & Siklus Hidup Data</h3>
            <p className="description-text">
              Survey Copilot beroperasi secara lokal (local-first). Berikut adalah rincian cara aplikasi mengelola dan menjaga keamanan setiap kategori data:
            </p>

            {isRetentionLoading ? (
              <p>Memuat informasi kebijakan...</p>
            ) : retentionInfo ? (
              <div className="retention-grid">
                <div className="retention-card">
                  <h4>📌 Fakta Profil Aktif</h4>
                  <p>{retentionInfo.activeFactsPolicy}</p>
                </div>
                <div className="retention-card">
                  <h4>📦 Fakta Arsip & Riwayat</h4>
                  <p>{retentionInfo.archivedFactsPolicy}</p>
                </div>
                <div className="retention-card">
                  <h4>📸 Tangkapan Layar & File Sementara</h4>
                  <p>{retentionInfo.screenCapturePolicy}</p>
                </div>
                <div className="retention-card">
                  <h4>🔍 Teks & Pertanyaan OCR</h4>
                  <p>{retentionInfo.ocrDataPolicy}</p>
                </div>
                <div className="retention-card">
                  <h4>🧠 Riwayat & Survey Memory</h4>
                  <p>{retentionInfo.surveyHistoryPolicy}</p>
                </div>
                <div className="retention-card">
                  <h4>⬇ Berkas Ekspor JSON</h4>
                  <p>{retentionInfo.exportFilePolicy}</p>
                </div>
                <div className="retention-card">
                  <h4>📝 Log & Diagnostik</h4>
                  <p>{retentionInfo.logsAndDiagnosticsPolicy}</p>
                </div>
                <div className="retention-card alert-card">
                  <h4>ℹ Transparansi Batasan Penghapusan</h4>
                  <p>{retentionInfo.deletionScopeDisclaimer}</p>
                </div>
              </div>
            ) : null}
          </div>
        </section>
      )}

      {/* TAB 3: ZONA BAHAYA / DELETION */}
      {activeTab === "danger" && (
        <section className="settings-tab-content" aria-labelledby="danger-heading">
          {/* BAGIAN 1: HAPUS PERSONA TERTENTU */}
          <div className="card danger-card">
            <h3>Hapus Persona Tertentu</h3>
            <p className="description-text">
              Menghapus satu persona beserta seluruh fakta, sumber bukti, dan riwayat miliknya. Data persona lain dan pengaturan global tidak akan terpengaruh.
            </p>

            <div className="form-group">
              <label htmlFor="persona-delete-select">Pilih Persona yang Ingin Dihapus:</label>
              <select
                id="persona-delete-select"
                value={personaToDeleteId}
                onChange={(e) => setPersonaToDeleteId(e.target.value)}
                className="select-input"
              >
                <option value="">-- Pilih Persona --</option>
                {personas.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.displayName} {p.isActive ? "(Aktif)" : ""}
                  </option>
                ))}
              </select>
            </div>

            <button
              className="btn btn-danger"
              onClick={() => void handleDeleteSelectedPersona()}
              disabled={!personaToDeleteId || isDeletingPersona}
            >
              {isDeletingPersona ? "Menghapus..." : "Hapus Persona Terpilih"}
            </button>
          </div>

          {/* BAGIAN 2: RESET TOTAL SEMUA DATA */}
          <div className="card danger-card danger-card-total">
            <h3 id="danger-heading">⚠ Reset Total: Hapus Seluruh Data Aplikasi</h3>
            <p className="danger-warning">
              Tindakan ini akan menghapus <strong>seluruh persona, fakta profil, sumber bukti, riwayat pertanyaan/jawaban, dan pengaturan lokal</strong> secara permanen. Database lokal akan dipadatkan (VACUUM & TRUNCATE WAL) dan dikembalikan ke kondisi awal instalasi.
            </p>

            <div className="pre-wipe-actions">
              <button
                className="btn btn-secondary"
                onClick={() => void handleLoadWipeSummary()}
                disabled={isWipeSummaryLoading}
              >
                {isWipeSummaryLoading ? "Memeriksa Database..." : "📊 Periksa Rincian Data yang Akan Dihapus"}
              </button>
              <button
                className="btn btn-outline"
                onClick={() => {
                  setActiveTab("export");
                  setSelectedPersonaId("ALL");
                }}
              >
                📦 Ekspor Cadangan Data Terlebih Dahulu (Direkomendasikan)
              </button>
            </div>

            {wipeSummary && (
              <div className="wipe-summary-table-box">
                <h4>Rincian Record yang Akan Dihapus:</h4>
                <ul className="wipe-counts-list">
                  <li><strong>Total Persona:</strong> {wipeSummary.personas}</li>
                  <li><strong>Fakta Profil:</strong> {wipeSummary.facts}</li>
                  <li><strong>Sumber Bukti:</strong> {wipeSummary.sources}</li>
                  <li><strong>Riwayat Perubahan Fakta:</strong> {wipeSummary.history}</li>
                  <li><strong>Sesi & Pertanyaan Survei:</strong> {wipeSummary.sessions + wipeSummary.questions}</li>
                  <li><strong>Rekomendasi & Jawaban AI:</strong> {wipeSummary.recommendations + wipeSummary.answers}</li>
                  <li><strong>Konfigurasi Provider:</strong> {wipeSummary.providerConfigs}</li>
                  <li><strong>Total Keseluruhan Record:</strong> {wipeSummary.total}</li>
                </ul>
              </div>
            )}

            <div className="wipe-confirmation-container">
              <div className="consent-checkbox-group">
                <input
                  type="checkbox"
                  id="consent-wipe"
                  checked={isConsentChecked}
                  onChange={(e) => setIsConsentChecked(e.target.checked)}
                />
                <label htmlFor="consent-wipe">
                  Saya memahami bahwa tindakan ini bersifat <strong>permanen</strong> dan data yang dihapus tidak dapat dipulihkan kembali.
                </label>
              </div>

              <div className="form-group">
                <label htmlFor="wipe-confirmation-input">
                  Ketik <code>HAPUS SEMUA DATA</code> untuk mengaktifkan tombol:
                </label>
                <input
                  id="wipe-confirmation-input"
                  type="text"
                  placeholder="HAPUS SEMUA DATA"
                  value={wipeConfirmationInput}
                  onChange={(e) => setWipeConfirmationInput(e.target.value)}
                  className="text-input"
                  disabled={isWiping}
                />
              </div>

              <button
                className="btn btn-danger btn-large"
                onClick={() => void handleExecuteWipe()}
                disabled={
                  wipeConfirmationInput.trim() !== "HAPUS SEMUA DATA" ||
                  !isConsentChecked ||
                  isWiping
                }
              >
                {isWiping ? "Menghapus Seluruh Data..." : "🚨 HAPUS SEMUA DATA SEKARANG"}
              </button>
            </div>

            {wipeErrorMessage && (
              <div className="alert alert-error" role="alert">
                ✕ {wipeErrorMessage}
              </div>
            )}

            {wipeReport && (
              <div className="alert alert-success wipe-report-box" role="alert">
                <h4>✓ Seluruh Data Berhasil Dihapus</h4>
                <p>Status Pembersihan Basis Data & Disk:</p>
                <ul>
                  <li>Persona terhapus: {wipeReport.deletedPersonasCount}</li>
                  <li>Fakta profil terhapus: {wipeReport.deletedFactsCount}</li>
                  <li>Sumber bukti terhapus: {wipeReport.deletedSourcesCount}</li>
                  <li>Sesi & pertanyaan terhapus: {wipeReport.deletedSessionsCount + wipeReport.deletedQuestionsCount}</li>
                  <li>File sementara & screenshot dibersihkan: {wipeReport.deletedTempFilesCount}</li>
                  <li>Checkpoint WAL SQLite: {wipeReport.walCheckpointStatus}</li>
                  <li>Status VACUUM SQLite: {wipeReport.vacuumStatus}</li>
                </ul>
                <div className="unaffected-items-note">
                  <strong>Catatan Transparansi:</strong>
                  <ul>
                    {wipeReport.unaffectedExternalItems.map((item, idx) => (
                      <li key={idx}>{item}</li>
                    ))}
                  </ul>
                </div>
              </div>
            )}
          </div>
        </section>
      )}
    </div>
  );
}
