import { useEffect, useState } from "react";
import type { Persona } from "../../types/persona";
import type {
  AllRecordCounts,
  ExportSummary,
  RetentionPolicyInfo,
  WipeReport,
} from "../../types/dataManagement";
import type {
  CredentialSummary,
  ProviderType,
  TestCredentialResult,
} from "../../types/credential";
import type {
  AdapterType,
  ConnectionTestResult,
  ModelOption,
  ProviderConfigSummary,
} from "../../types/provider";
import type {
  PoolSettings,
  PoolStrategy,
  ProviderHealthInfo,
} from "../../types/providerPool";
import { dataManagementService } from "../../services/dataManagementService";
import { CredentialService } from "../../services/credentialService";
import { ProviderService } from "../../services/providerService";
import { ProviderPoolService } from "../../services/providerPoolService";
import { PrivacyService } from "../../services/privacyService";
import type {
  DiagnosticsLogEntry,
  PrivacySettings,
  RetentionCleanupResult,
  RetentionStatus,
  SystemDiagnosticsInfo,
} from "../../types/privacy";
import { AppError } from "../../types/error";

type SettingsTab = "credentials" | "privacy" | "export" | "retention" | "danger";

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
  const [activeTab, setActiveTab] = useState<SettingsTab>("credentials");

  // Credential Management State (SC-020)
  const [credentials, setCredentials] = useState<CredentialSummary[]>([]);
  const [isCredentialsLoading, setIsCredentialsLoading] = useState(false);
  const [credErrorMessage, setCredErrorMessage] = useState<string | null>(null);
  const [credSuccessMessage, setCredSuccessMessage] = useState<string | null>(null);

  // New Credential Form State
  const [newProviderType, setNewProviderType] = useState<ProviderType>("gemini");
  const [newLabel, setNewLabel] = useState("");
  const [newSecret, setNewSecret] = useState("");
  const [isSavingCred, setIsSavingCred] = useState(false);

  // Testing & Status State
  const [testResults, setTestResults] = useState<Record<string, TestCredentialResult>>({});
  const [testingCredId, setTestingCredId] = useState<string | null>(null);

  // Edit / Update Credential Modal State
  const [editingCred, setEditingCred] = useState<CredentialSummary | null>(null);
  const [updateLabel, setUpdateLabel] = useState("");
  const [updateSecret, setUpdateSecret] = useState("");
  const [isUpdatingCred, setIsUpdatingCred] = useState(false);

  // Delete Shared Credential Modal State
  const [deletingCred, setDeletingCred] = useState<CredentialSummary | null>(null);
  const [forceDeleteConsent, setForceDeleteConsent] = useState(false);
  const [isDeletingCred, setIsDeletingCred] = useState(false);

  // Provider Configuration State (SC-021)
  const [providerConfigs, setProviderConfigs] = useState<ProviderConfigSummary[]>([]);
  const [isProviderConfigsLoading, setIsProviderConfigsLoading] = useState(false);
  const [providerErrorMessage, setProviderErrorMessage] = useState<string | null>(null);
  const [providerSuccessMessage, setProviderSuccessMessage] = useState<string | null>(null);

  // New Provider Config Form State
  const [newPConfigDisplayName, setNewPConfigDisplayName] = useState("");
  const [newPConfigProviderType, setNewPConfigProviderType] = useState<ProviderType>("gemini");
  const [newPConfigAdapterType, setNewPConfigAdapterType] = useState<AdapterType>("gemini");
  const [newPConfigModelId, setNewPConfigModelId] = useState("gemini-2.5-flash");
  const [newPConfigCredentialRef, setNewPConfigCredentialRef] = useState("");
  const [newPConfigBaseUrl, setNewPConfigBaseUrl] = useState("");
  const [newPConfigPriority, setNewPConfigPriority] = useState(100);
  const [newPConfigTimeoutMs, setNewPConfigTimeoutMs] = useState(30000);
  const [isCreatingPConfig, setIsCreatingPConfig] = useState(false);

  // Recommended Models State
  const [recommendedModels, setRecommendedModels] = useState<ModelOption[]>([]);

  // Provider Connection Testing State
  const [providerTestResults, setProviderTestResults] = useState<Record<string, ConnectionTestResult>>({});
  const [testingProviderId, setTestingProviderId] = useState<string | null>(null);

  // Edit Provider Config Modal State
  const [editingProviderConfig, setEditingProviderConfig] = useState<ProviderConfigSummary | null>(null);
  const [editPConfigDisplayName, setEditPConfigDisplayName] = useState("");
  const [editPConfigModelId, setEditPConfigModelId] = useState("");
  const [editPConfigBaseUrl, setEditPConfigBaseUrl] = useState("");
  const [editPConfigPriority, setEditPConfigPriority] = useState(100);
  const [editPConfigTimeoutMs, setEditPConfigTimeoutMs] = useState(30000);
  const [editPConfigCredentialRef, setEditPConfigCredentialRef] = useState("");
  const [isUpdatingPConfig, setIsUpdatingPConfig] = useState(false);

  // Export State (SC-012)
  const [selectedPersonaId, setSelectedPersonaId] = useState<string>(
    activePersona ? activePersona.id : "ALL"
  );
  const [exportSummary, setExportSummary] = useState<ExportSummary | null>(null);
  const [isExportSummaryLoading, setIsExportSummaryLoading] = useState(false);
  const [isDownloading, setIsDownloading] = useState(false);
  const [exportSuccessMessage, setExportSuccessMessage] = useState<string | null>(null);
  const [exportErrorMessage, setExportErrorMessage] = useState<string | null>(null);

  // Retention Info State (SC-012)
  const [retentionInfo, setRetentionInfo] = useState<RetentionPolicyInfo | null>(null);
  const [isRetentionLoading, setIsRetentionLoading] = useState(false);

  // Danger Zone / Wipe State (SC-012)
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

  // Provider Pool State (SC-022)
  const [poolSettings, setPoolSettings] = useState<PoolSettings | null>(null);
  const [poolStrategy, setPoolStrategy] = useState<PoolStrategy>("adaptive");
  const [poolMaxAttempts, setPoolMaxAttempts] = useState(3);
  const [poolAutoRetry, setPoolAutoRetry] = useState(true);
  const [poolRetryBaseDelayMs, setPoolRetryBaseDelayMs] = useState(1000);
  const [poolMaxRetryDelayMs, setPoolMaxRetryDelayMs] = useState(15000);
  const [isSavingPoolSettings, setIsSavingPoolSettings] = useState(false);
  const [providerHealthList, setProviderHealthList] = useState<ProviderHealthInfo[]>([]);

  // SC-060: Privacy, Retention & Diagnostics States
  const [privacySettings, setPrivacySettings] = useState<PrivacySettings | null>(null);
  const [isPrivacyLoading, setIsPrivacyLoading] = useState(false);
  const [privacyErrorMessage, setPrivacyErrorMessage] = useState<string | null>(null);
  const [privacySuccessMessage, setPrivacySuccessMessage] = useState<string | null>(null);
  const [systemDiag, setSystemDiag] = useState<SystemDiagnosticsInfo | null>(null);
  const [retentionStatus, setRetentionStatus] = useState<RetentionStatus | null>(null);
  const [diagLogs, setDiagLogs] = useState<DiagnosticsLogEntry[]>([]);
  const [logFilterLevel, setLogFilterLevel] = useState<string>("ALL");
  const [logFilterCategory, setLogFilterCategory] = useState<string>("ALL");
  const [isCleaningTemp, setIsCleaningTemp] = useState(false);
  const [cleanResult, setCleanResult] = useState<RetentionCleanupResult | null>(null);
  const [isClearingLogs, setIsClearingLogs] = useState(false);
  const [isSavingPrivacy, setIsSavingPrivacy] = useState(false);

  // Privacy Form States
  const [privAutoRedact, setPrivAutoRedact] = useState(true);
  const [privRequireSensitiveConsent, setPrivRequireSensitiveConsent] = useState(true);
  const [privRequireVisionConsent, setPrivRequireVisionConsent] = useState(true);
  const [privRetentionPolicy, setPrivRetentionPolicy] = useState<string>("immediate_delete");
  const [privAutoCleanupOnExit, setPrivAutoCleanupOnExit] = useState(true);
  const [privLogLevel, setPrivLogLevel] = useState<string>("INFO");

  useEffect(() => {
    if (activeTab === "credentials") {
      void loadCredentials();
      void loadProviderConfigs();
      void loadRecommendedModels(newPConfigProviderType);
      void loadPoolData();
    } else if (activeTab === "privacy") {
      void loadPrivacyData();
    } else if (activeTab === "retention" && !retentionInfo) {
      void loadRetentionInfo();
    }
  }, [activeTab]);

  // Keyboard navigation: Escape closes active modals
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        if (editingProviderConfig) setEditingProviderConfig(null);
        else if (editingCred) setEditingCred(null);
        else if (deletingCred) setDeletingCred(null);
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [editingProviderConfig, editingCred, deletingCred]);

  async function loadPoolData() {
    try {
      const [settings, health] = await Promise.all([
        ProviderPoolService.getSettings(),
        ProviderPoolService.listHealth(),
      ]);
      setPoolSettings(settings);
      setPoolStrategy(settings.strategy);
      setPoolMaxAttempts(settings.maxAttempts);
      setPoolAutoRetry(settings.autoRetry);
      setPoolRetryBaseDelayMs(settings.retryBaseDelayMs);
      setPoolMaxRetryDelayMs(settings.maxRetryDelayMs);
      setProviderHealthList(health);
    } catch (err) {
      console.error("Gagal memuat data provider pool:", err);
    }
  }

  async function handleSavePoolSettings(e: React.FormEvent) {
    e.preventDefault();
    setIsSavingPoolSettings(true);
    setProviderErrorMessage(null);
    setProviderSuccessMessage(null);
    try {
      const updated = await ProviderPoolService.updateSettings({
        strategy: poolStrategy,
        maxAttempts: Number(poolMaxAttempts),
        autoRetry: poolAutoRetry,
        retryBaseDelayMs: Number(poolRetryBaseDelayMs),
        maxRetryDelayMs: Number(poolMaxRetryDelayMs),
      });
      setPoolSettings(updated);
      setProviderSuccessMessage("Pengaturan Provider Pool berhasil disimpan.");
      await loadPoolData();
    } catch (err) {
      setProviderErrorMessage(AppError.from(err).message);
    } finally {
      setIsSavingPoolSettings(false);
    }
  }

  // SC-060: Privacy & Diagnostics Handlers
  async function loadPrivacyData(level = logFilterLevel, category = logFilterCategory) {
    setIsPrivacyLoading(true);
    setPrivacyErrorMessage(null);
    try {
      const [settings, diag, retStatus, logs] = await Promise.all([
        PrivacyService.getSettings(),
        PrivacyService.getSystemDiagnostics(),
        PrivacyService.getRetentionStatus(),
        PrivacyService.getDiagnosticLogs(
          100,
          level === "ALL" ? undefined : level,
          category === "ALL" ? undefined : category
        ),
      ]);
      setPrivacySettings(settings);
      setPrivAutoRedact(settings.autoRedactLogs);
      setPrivRequireSensitiveConsent(settings.requireSensitiveConsentPrompt);
      setPrivRequireVisionConsent(settings.requireVisionConsentPrompt);
      setPrivRetentionPolicy(settings.screenshotRetentionPolicy);
      setPrivAutoCleanupOnExit(settings.autoCleanupTempOnExit);
      setPrivLogLevel(settings.logLevel);

      setSystemDiag(diag);
      setRetentionStatus(retStatus);
      setDiagLogs(logs);
    } catch (err) {
      setPrivacyErrorMessage(AppError.from(err).message);
    } finally {
      setIsPrivacyLoading(false);
    }
  }

  async function handleSavePrivacySettings(e: React.FormEvent) {
    e.preventDefault();
    setIsSavingPrivacy(true);
    setPrivacyErrorMessage(null);
    setPrivacySuccessMessage(null);
    try {
      const updated = await PrivacyService.updateSettings({
        autoRedactLogs: privAutoRedact,
        requireSensitiveConsentPrompt: privRequireSensitiveConsent,
        requireVisionConsentPrompt: privRequireVisionConsent,
        screenshotRetentionPolicy: privRetentionPolicy,
        autoCleanupTempOnExit: privAutoCleanupOnExit,
        logLevel: privLogLevel,
      });
      setPrivacySettings(updated);
      setPrivacySuccessMessage("Pengaturan privasi dan retensi berhasil disimpan.");
      await loadPrivacyData();
      setTimeout(() => setPrivacySuccessMessage(null), 4000);
    } catch (err) {
      setPrivacyErrorMessage(AppError.from(err).message);
    } finally {
      setIsSavingPrivacy(false);
    }
  }

  async function handleCleanupTempFiles() {
    setIsCleaningTemp(true);
    setPrivacyErrorMessage(null);
    try {
      const res = await PrivacyService.cleanupTempFiles({ maxAgeSeconds: 0 });
      setCleanResult(res);
      setPrivacySuccessMessage(res.message);
      const [ret, diag] = await Promise.all([
        PrivacyService.getRetentionStatus(),
        PrivacyService.getSystemDiagnostics(),
      ]);
      setRetentionStatus(ret);
      setSystemDiag(diag);
      setTimeout(() => setPrivacySuccessMessage(null), 4000);
    } catch (err) {
      setPrivacyErrorMessage(AppError.from(err).message);
    } finally {
      setIsCleaningTemp(false);
    }
  }

  async function handleClearDiagnosticLogs() {
    setIsClearingLogs(true);
    setPrivacyErrorMessage(null);
    try {
      const count = await PrivacyService.clearDiagnosticLogs();
      setDiagLogs([]);
      setPrivacySuccessMessage(`Berhasil mengosongkan ${count} entri log diagnostik.`);
      setTimeout(() => setPrivacySuccessMessage(null), 4000);
    } catch (err) {
      setPrivacyErrorMessage(AppError.from(err).message);
    } finally {
      setIsClearingLogs(false);
    }
  }

  async function loadCredentials() {
    setIsCredentialsLoading(true);
    setCredErrorMessage(null);
    try {
      const list = await CredentialService.listCredentials();
      setCredentials(list);
      // Jika form provider belum memilih kredensial, otomatis pilih kredensial pertama yang cocok
      if (list.length > 0 && !newPConfigCredentialRef) {
        setNewPConfigCredentialRef(list[0].credentialRef);
      }
    } catch (err) {
      setCredErrorMessage(AppError.from(err).message);
    } finally {
      setIsCredentialsLoading(false);
    }
  }

  async function loadProviderConfigs() {
    setIsProviderConfigsLoading(true);
    setProviderErrorMessage(null);
    try {
      const list = await ProviderService.listProviderConfigs();
      setProviderConfigs(list);
    } catch (err) {
      setProviderErrorMessage(AppError.from(err).message);
    } finally {
      setIsProviderConfigsLoading(false);
    }
  }

  async function loadRecommendedModels(pType: string) {
    try {
      const models = await ProviderService.getRecommendedModels(pType);
      setRecommendedModels(models);
      if (models.length > 0) {
        setNewPConfigModelId(models[0].id);
      }
    } catch (err) {
      console.error("Gagal memuat model rekomendasi:", err);
    }
  }

  async function handleCreateProviderConfig(e: React.FormEvent) {
    e.preventDefault();
    if (!newPConfigDisplayName.trim()) {
      setProviderErrorMessage("Nama konfigurasi provider wajib diisi.");
      return;
    }
    if (!newPConfigModelId.trim()) {
      setProviderErrorMessage("Model ID wajib dipilih atau diisi.");
      return;
    }
    if (!newPConfigCredentialRef.trim()) {
      setProviderErrorMessage("Pilih kunci API kredensial tersimpan untuk konfigurasi ini.");
      return;
    }

    setIsCreatingPConfig(true);
    setProviderErrorMessage(null);
    setProviderSuccessMessage(null);

    try {
      await ProviderService.createProviderConfig({
        displayName: newPConfigDisplayName.trim(),
        providerType: newPConfigProviderType,
        adapterType: newPConfigAdapterType,
        modelId: newPConfigModelId.trim(),
        credentialRef: newPConfigCredentialRef.trim(),
        baseUrl: newPConfigBaseUrl.trim() ? newPConfigBaseUrl.trim() : null,
        priority: newPConfigPriority,
        timeoutMs: newPConfigTimeoutMs,
      });

      setNewPConfigDisplayName("");
      setNewPConfigBaseUrl("");
      setProviderSuccessMessage(`Konfigurasi provider '${newPConfigDisplayName.trim()}' berhasil dibuat.`);
      await loadProviderConfigs();
      await loadCredentials();
    } catch (err) {
      setProviderErrorMessage(AppError.from(err).message);
    } finally {
      setIsCreatingPConfig(false);
    }
  }

  async function handleTestProviderConnection(id: string) {
    setTestingProviderId(id);
    setProviderErrorMessage(null);
    try {
      const res = await ProviderService.testConnection(id);
      setProviderTestResults((prev) => ({ ...prev, [id]: res }));
      setProviderSuccessMessage(`Uji koneksi berhasil (${res.latencyMs} ms): ${res.message}`);
      await loadProviderConfigs();
    } catch (err) {
      setProviderErrorMessage(AppError.from(err).message);
    } finally {
      setTestingProviderId(null);
    }
  }

  async function handleToggleProviderEnabled(config: ProviderConfigSummary) {
    try {
      await ProviderService.updateProviderConfig({
        id: config.id,
        enabled: !config.enabled,
      });
      await loadProviderConfigs();
    } catch (err) {
      setProviderErrorMessage(AppError.from(err).message);
    }
  }

  async function handleDeleteProviderConfig(id: string, name: string) {
    if (!window.confirm(`Yakin ingin menghapus konfigurasi provider '${name}'?`)) {
      return;
    }
    try {
      await ProviderService.deleteProviderConfig(id);
      setProviderSuccessMessage(`Konfigurasi provider '${name}' berhasil dihapus.`);
      await loadProviderConfigs();
      await loadCredentials();
    } catch (err) {
      setProviderErrorMessage(AppError.from(err).message);
    }
  }

  function handleOpenEditProviderModal(config: ProviderConfigSummary) {
    setEditingProviderConfig(config);
    setEditPConfigDisplayName(config.displayName);
    setEditPConfigModelId(config.modelId);
    setEditPConfigBaseUrl(config.baseUrl || "");
    setEditPConfigPriority(config.priority);
    setEditPConfigTimeoutMs(config.timeoutMs);
    setEditPConfigCredentialRef(config.credentialRef);
    setProviderErrorMessage(null);
  }

  async function handleSaveUpdateProvider(e: React.FormEvent) {
    e.preventDefault();
    if (!editingProviderConfig) return;

    setIsUpdatingPConfig(true);
    setProviderErrorMessage(null);
    try {
      await ProviderService.updateProviderConfig({
        id: editingProviderConfig.id,
        displayName: editPConfigDisplayName.trim(),
        modelId: editPConfigModelId.trim(),
        baseUrl: editPConfigBaseUrl.trim() ? editPConfigBaseUrl.trim() : null,
        priority: editPConfigPriority,
        timeoutMs: editPConfigTimeoutMs,
        credentialRef: editPConfigCredentialRef,
      });

      setEditingProviderConfig(null);
      setProviderSuccessMessage("Konfigurasi provider berhasil diperbarui.");
      await loadProviderConfigs();
    } catch (err) {
      setProviderErrorMessage(AppError.from(err).message);
    } finally {
      setIsUpdatingPConfig(false);
    }
  }

  async function handleSaveCredential(e: React.FormEvent) {
    e.preventDefault();
    if (!newLabel.trim()) {
      setCredErrorMessage("Label kredensial wajib diisi.");
      return;
    }
    if (!newSecret.trim() || newSecret.trim().length < 8) {
      setCredErrorMessage("Kunci API rahasia wajib diisi dan minimal 8 karakter.");
      return;
    }

    setIsSavingCred(true);
    setCredErrorMessage(null);
    setCredSuccessMessage(null);

    try {
      await CredentialService.saveCredential({
        providerType: newProviderType,
        label: newLabel.trim(),
        secret: newSecret.trim(),
      });

      // BERSIHKAN INPUT STATE SEKETIKA DARI MEMORI FRONTEND
      setNewSecret("");
      setNewLabel("");
      setCredSuccessMessage("Kredensial berhasil disimpan dengan aman di Windows Credential Manager.");
      await loadCredentials();
    } catch (err) {
      setCredErrorMessage(AppError.from(err).message);
    } finally {
      setIsSavingCred(false);
    }
  }

  async function handleTestCredential(id: string) {
    setTestingCredId(id);
    setCredErrorMessage(null);
    try {
      const res = await CredentialService.testCredential(id);
      setTestResults((prev) => ({ ...prev, [id]: res }));
      await loadCredentials();
    } catch (err) {
      setCredErrorMessage(AppError.from(err).message);
    } finally {
      setTestingCredId(null);
    }
  }

  function handleOpenUpdateModal(cred: CredentialSummary) {
    setEditingCred(cred);
    setUpdateLabel(cred.label);
    setUpdateSecret("");
    setCredErrorMessage(null);
  }

  async function handleSaveUpdate(e: React.FormEvent) {
    e.preventDefault();
    if (!editingCred) return;

    if (!updateLabel.trim()) {
      setCredErrorMessage("Label kredensial tidak boleh kosong.");
      return;
    }

    setIsUpdatingCred(true);
    setCredErrorMessage(null);
    try {
      await CredentialService.updateCredential({
        id: editingCred.id,
        label: updateLabel.trim(),
        secret: updateSecret.trim() ? updateSecret.trim() : undefined,
      });

      // BERSIHKAN INPUT STATE SEKETIKA
      setUpdateSecret("");
      setEditingCred(null);
      setCredSuccessMessage(`Kredensial '${updateLabel.trim()}' berhasil diperbarui.`);
      await loadCredentials();
    } catch (err) {
      setCredErrorMessage(AppError.from(err).message);
    } finally {
      setIsUpdatingCred(false);
    }
  }

  async function handleDeleteCredentialClick(cred: CredentialSummary) {
    if (cred.usageCount > 0) {
      // Memerlukan konfirmasi khusus karena kredensial digunakan bersama
      setDeletingCred(cred);
      setForceDeleteConsent(false);
      setCredErrorMessage(null);
    } else {
      if (!window.confirm(`Yakin ingin menghapus kredensial '${cred.label}' dari Windows Credential Manager?`)) {
        return;
      }
      try {
        await CredentialService.deleteCredential(cred.id, false);
        setCredSuccessMessage(`Kredensial '${cred.label}' berhasil dihapus.`);
        await loadCredentials();
      } catch (err) {
        setCredErrorMessage(AppError.from(err).message);
      }
    }
  }

  async function handleExecuteForceDelete() {
    if (!deletingCred) return;

    setIsDeletingCred(true);
    setCredErrorMessage(null);
    try {
      await CredentialService.deleteCredential(deletingCred.id, forceDeleteConsent);
      setCredSuccessMessage(`Kredensial '${deletingCred.label}' berhasil dihapus secara bersih.`);
      setDeletingCred(null);
      await loadCredentials();
    } catch (err) {
      setCredErrorMessage(AppError.from(err).message);
    } finally {
      setIsDeletingCred(false);
    }
  }

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

      const blob = new Blob([jsonContent], { type: "application/json;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = filename;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);

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
    if (wipeConfirmationInput.trim() !== "HAPUS SEMUA DATA" || !isConsentChecked) {
      return;
    }

    setIsWiping(true);
    setWipeErrorMessage(null);
    setWipeReport(null);
    try {
      const report = await dataManagementService.wipeAllData(wipeConfirmationInput.trim());
      setWipeReport(report);
      setWipeConfirmationInput("");
      setIsConsentChecked(false);
      setWipeSummary(null);

      if (onRefreshPersonas) {
        await onRefreshPersonas();
      }
    } catch (err) {
      setWipeErrorMessage(AppError.from(err).message);
    } finally {
      setIsWiping(false);
    }
  }

  async function handleExecuteDeleteSinglePersona() {
    if (!personaToDeleteId) return;
    const target = personas.find((p) => p.id === personaToDeleteId);
    if (!target) return;

    if (!window.confirm(`Yakin ingin menghapus persona '${target.displayName}' beserta seluruh data miliknya?`)) {
      return;
    }

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
      alert(`Gagal menghapus persona: ${AppError.from(err).message}`);
    } finally {
      setIsDeletingPersona(false);
    }
  }

  return (
    <div className="content settings-container">
      <div className="settings-header">
        <h2>⚙ Pengaturan Data, Privasi & AI</h2>
        <p className="settings-subtitle">
          Kelola kredensial provider AI secara aman di Windows Credential Manager, ekspor cadangan data, dan siklus privasi.
        </p>
      </div>

      <nav className="settings-nav-tabs" aria-label="Tab Pengaturan">
        <button
          className={`tab-btn ${activeTab === "credentials" ? "active" : ""}`}
          onClick={() => setActiveTab("credentials")}
        >
          🔑 Kredensial & Provider AI
        </button>
        <button
          className={`tab-btn ${activeTab === "privacy" ? "active" : ""}`}
          onClick={() => setActiveTab("privacy")}
        >
          🛡 Privasi & Diagnostik (SC-060)
        </button>
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

      {/* TAB 1: KREDENSIAL & PROVIDER AI (SC-020) */}
      {activeTab === "credentials" && (
        <section className="settings-tab-content" aria-labelledby="cred-heading">
          <div className="card">
            <h3 id="cred-heading">Penyimpanan Kredensial AI di Windows Credential Manager</h3>
            <p className="description-text">
              Kunci API disimpan secara native di <strong>Windows Credential Manager</strong> pada sistem operasi Windows 11 Anda.
              Basis data lokal SQLite hanya menyimpan referensi buram (<code>credential_ref</code>) dan tidak pernah menyimpan secret asli.
              Kunci API tidak pernah bocor ke antarmuka, berkas ekspor, atau log diagnostik.
            </p>

            <div className="security-guarantee-badge">
              🛡 <strong>Keamanan Kunci Terjamin:</strong> Kunci API hanya diproses di backend saat melakukan panggilan ke provider AI.
              Setelah Anda menyimpannya, nilai rahasia tidak pernah ditampilkan kembali dalam bentuk teks mentah.
            </div>

            {credSuccessMessage && (
              <div className="alert alert-success" role="alert" style={{ marginTop: "14px" }}>
                ✓ {credSuccessMessage}
              </div>
            )}

            {credErrorMessage && (
              <div className="alert alert-error" role="alert" style={{ marginTop: "14px" }}>
                ✕ {credErrorMessage}
              </div>
            )}

            {/* FORM TAMBAH KREDENSIAL */}
            <form onSubmit={handleSaveCredential} className="cred-form" style={{ marginTop: "18px" }}>
              <h4>Tambah Kunci API Provider Baru</h4>

              <div className="form-group">
                <label htmlFor="cred-provider-type">Tipe Provider AI:</label>
                <select
                  id="cred-provider-type"
                  value={newProviderType}
                  onChange={(e) => setNewProviderType(e.target.value as ProviderType)}
                  className="select-input"
                >
                  <option value="gemini">Google Gemini (Utama / Rekomendasi)</option>
                  <option value="openai_compatible">OpenAI / Kompatibel HTTP Endpoint</option>
                  <option value="anthropic_compatible">Anthropic Claude Kompatibel</option>
                  <option value="custom_http">Custom HTTP Provider (Local / Private)</option>
                </select>
              </div>

              <div className="form-group">
                <label htmlFor="cred-label">Nama / Label Pengenal:</label>
                <input
                  id="cred-label"
                  type="text"
                  placeholder="Contoh: Gemini 1.5 Flash Utama"
                  value={newLabel}
                  onChange={(e) => setNewLabel(e.target.value)}
                  maxLength={100}
                  required
                />
              </div>

              <div className="form-group">
                <label htmlFor="cred-secret">Kunci API Rahasia (API Key):</label>
                <input
                  id="cred-secret"
                  type="password"
                  placeholder="Tempel kunci API rahasia (contoh: AIzaSy...)"
                  value={newSecret}
                  onChange={(e) => setNewSecret(e.target.value)}
                  autoComplete="new-password"
                  required
                />
              </div>

              <p className="cred-security-hint">
                🔒 Nilai kunci akan disimpan langsung ke Windows Credential Manager dan segera dihapus dari memori formulir peramban.
              </p>

              <div>
                <button
                  type="submit"
                  className="btn btn-primary"
                  disabled={isSavingCred || !newLabel.trim() || !newSecret.trim()}
                >
                  {isSavingCred ? "Menyimpan ke Windows Credential Manager..." : "💾 Simpan Kredensial ke OS"}
                </button>
              </div>
            </form>

            <hr style={{ margin: "24px 0", border: "0", borderTop: "1px solid var(--line)" }} />

            {/* DAFTAR KREDENSIAL TERSIMPAN */}
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <h4>Daftar Kredensial yang Dikelola ({credentials.length})</h4>
              <button
                className="btn btn-secondary"
                style={{ padding: "6px 12px", fontSize: "11px" }}
                onClick={() => void loadCredentials()}
                disabled={isCredentialsLoading}
              >
                {isCredentialsLoading ? "Menyegarkan..." : "🔄 Segarkan Status"}
              </button>
            </div>

            {credentials.length === 0 ? (
              <div className="empty-inline" style={{ marginTop: "14px" }}>
                Belum ada kredensial API yang tersimpan di Windows Credential Manager. Silakan tambahkan kunci API di atas.
              </div>
            ) : (
              <div className="cred-grid">
                {credentials.map((cred) => {
                  const testRes = testResults[cred.id];
                  return (
                    <div key={cred.id} className="cred-card">
                      <div className="cred-card-header">
                        <div>
                          <h5 className="cred-card-title">{cred.label}</h5>
                          <span className="badge" style={{ marginTop: "4px" }}>
                            {cred.providerType === "gemini"
                              ? "Google Gemini"
                              : cred.providerType === "openai_compatible"
                              ? "OpenAI Compatible"
                              : cred.providerType === "anthropic_compatible"
                              ? "Anthropic Compatible"
                              : "Custom HTTP"}
                          </span>
                        </div>
                        <span className={`badge badge-status-${cred.status}`}>
                          {cred.status === "configured"
                            ? "✓ Tersimpan di OS"
                            : cred.status === "unconfigured"
                            ? "✕ Belum di OS"
                            : cred.status === "needs_update"
                            ? "⚠ Perlu Update"
                            : "⚠ Akses Ditolak"}
                        </span>
                      </div>

                      <div>
                        <div style={{ fontSize: "11px", color: "var(--muted)", marginBottom: "3px" }}>
                          Referensi Buram (SQLite):
                        </div>
                        <code className="cred-ref-code">{cred.credentialRef}</code>
                      </div>

                      <div className="cred-meta-row">
                        <span>Digunakan: <strong>{cred.usageCount}</strong> provider config</span>
                        <span>Diperbarui: {new Date(cred.updatedAt).toLocaleDateString("id-ID")}</span>
                      </div>

                      {testRes && (
                        <div
                          className={`alert ${testRes.isValid ? "alert-success" : "alert-error"}`}
                          style={{ padding: "8px 10px", fontSize: "11px", margin: "4px 0" }}
                        >
                          {testRes.message}
                        </div>
                      )}

                      <div className="card-actions" style={{ marginTop: "8px", borderTop: "1px solid #f2f5f3", paddingTop: "8px" }}>
                        <button
                          className="btn btn-secondary"
                          style={{ padding: "6px 10px", fontSize: "11px" }}
                          onClick={() => void handleTestCredential(cred.id)}
                          disabled={testingCredId === cred.id}
                        >
                          {testingCredId === cred.id ? "Memeriksa..." : "🔍 Uji Akses"}
                        </button>
                        <button
                          className="btn btn-secondary"
                          style={{ padding: "6px 10px", fontSize: "11px" }}
                          onClick={() => handleOpenUpdateModal(cred)}
                        >
                          ✏ Ganti Kunci
                        </button>
                        <button
                          className="danger-button"
                          style={{ padding: "6px 10px", fontSize: "11px" }}
                          onClick={() => void handleDeleteCredentialClick(cred)}
                        >
                          🗑 Hapus
                        </button>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>

          {/* BAGIAN 2: KONFIGURASI MODEL & PROVIDER AI (SC-021) */}
          <div className="card" style={{ marginTop: "24px" }}>
            <h3>🤖 Konfigurasi Model & Provider AI</h3>
            <p className="description-text">
              Atur model AI yang digunakan oleh Survey Copilot (misalnya <strong>Google Gemini 2.5 Flash</strong>), kaitkan dengan kunci API aman yang tersimpan di Windows Credential Manager, dan lakukan uji koneksi langsung.
            </p>

            <div className="security-guarantee-badge">
              ⚡ <strong>Multi-Provider & Tanpa Kebocoran Data:</strong> Konfigurasi provider hanya mereferensikan kunci API melalui pengenal buram. Anda dapat mengatur beberapa model untuk ketahanan operasional tanpa membeberkan kunci rahasia.
            </div>

            {providerSuccessMessage && (
              <div className="alert alert-success" role="alert" style={{ marginTop: "14px" }}>
                ✓ {providerSuccessMessage}
              </div>
            )}

            {providerErrorMessage && (
              <div className="alert alert-error" role="alert" style={{ marginTop: "14px" }}>
                ✕ {providerErrorMessage}
              </div>
            )}

            {/* FORM TAMBAH KONFIGURASI PROVIDER */}
            <form onSubmit={handleCreateProviderConfig} className="cred-form" style={{ marginTop: "18px", maxWidth: "100%" }}>
              <h4>Tambah Konfigurasi Provider AI Baru</h4>

              <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))", gap: "16px" }}>
                <div className="form-group">
                  <label htmlFor="pconfig-name">Nama Konfigurasi:</label>
                  <input
                    id="pconfig-name"
                    type="text"
                    placeholder="Contoh: Gemini 2.5 Flash Utama"
                    value={newPConfigDisplayName}
                    onChange={(e) => setNewPConfigDisplayName(e.target.value)}
                    maxLength={100}
                    required
                  />
                </div>

                <div className="form-group">
                  <label htmlFor="pconfig-type">Tipe Provider & Protokol:</label>
                  <select
                    id="pconfig-type"
                    value={newPConfigProviderType}
                    onChange={(e) => {
                      const t = e.target.value as ProviderType;
                      setNewPConfigProviderType(t);
                      setNewPConfigAdapterType(t === "openai_compatible" ? "openai_compatible" : "gemini");
                      void loadRecommendedModels(t);
                    }}
                    className="select-input"
                  >
                    <option value="gemini">Google Gemini (Utama / Rekomendasi)</option>
                    <option value="openai_compatible">OpenAI-Compatible (Lokal Ollama / vLLM / OpenAI)</option>
                  </select>
                </div>

                <div className="form-group">
                  <label htmlFor="pconfig-cred">Kredensial Kunci API Tersimpan:</label>
                  <select
                    id="pconfig-cred"
                    value={newPConfigCredentialRef}
                    onChange={(e) => setNewPConfigCredentialRef(e.target.value)}
                    className="select-input"
                    required
                  >
                    {credentials.length === 0 ? (
                      <option value="">-- Belum ada kredensial tersimpan (Tambahkan di atas) --</option>
                    ) : (
                      credentials.map((c) => (
                        <option key={c.credentialRef} value={c.credentialRef}>
                          {c.label} ({c.providerType})
                        </option>
                      ))
                    )}
                  </select>
                </div>

                <div className="form-group">
                  <label htmlFor="pconfig-model-select">Model AI:</label>
                  <div style={{ display: "flex", gap: "8px" }}>
                    <select
                      id="pconfig-model-select"
                      value={newPConfigModelId}
                      onChange={(e) => setNewPConfigModelId(e.target.value)}
                      className="select-input"
                      style={{ flex: "1" }}
                    >
                      {recommendedModels.map((m) => (
                        <option key={m.id} value={m.id}>
                          {m.displayName}
                        </option>
                      ))}
                      <option value="custom">Model Lain (Ketik manual...)</option>
                    </select>
                    {newPConfigModelId === "custom" && (
                      <input
                        type="text"
                        placeholder="ID Model (mis: gemini-2.5-flash)"
                        onChange={(e) => setNewPConfigModelId(e.target.value)}
                        style={{ flex: "1" }}
                        required
                      />
                    )}
                  </div>
                </div>

                <div className="form-group">
                  <label htmlFor="pconfig-url">Custom Endpoint Base URL (Opsional):</label>
                  <input
                    id="pconfig-url"
                    type="text"
                    placeholder="Biarkan kosong untuk default Google API (atau http://localhost:11434)"
                    value={newPConfigBaseUrl}
                    onChange={(e) => setNewPConfigBaseUrl(e.target.value)}
                  />
                  <small style={{ color: "var(--muted)", fontSize: "11px" }}>
                    🔒 Wajib HTTPS untuk endpoint internet. HTTP diizinkan khusus untuk pengujian lokal (localhost).
                  </small>
                </div>

                <div className="form-group" style={{ display: "flex", gap: "12px" }}>
                  <div style={{ flex: "1" }}>
                    <label htmlFor="pconfig-priority">Prioritas (1 = Tertinggi):</label>
                    <input
                      id="pconfig-priority"
                      type="number"
                      min={1}
                      max={1000}
                      value={newPConfigPriority}
                      onChange={(e) => setNewPConfigPriority(Number(e.target.value))}
                    />
                  </div>
                  <div style={{ flex: "1" }}>
                    <label htmlFor="pconfig-timeout">Timeout (ms):</label>
                    <input
                      id="pconfig-timeout"
                      type="number"
                      min={1000}
                      max={120000}
                      step={1000}
                      value={newPConfigTimeoutMs}
                      onChange={(e) => setNewPConfigTimeoutMs(Number(e.target.value))}
                    />
                  </div>
                </div>
              </div>

              <div style={{ marginTop: "14px" }}>
                <button
                  type="submit"
                  className="btn btn-primary"
                  disabled={isCreatingPConfig || credentials.length === 0}
                >
                  {isCreatingPConfig ? "Menyimpan Konfigurasi..." : "➕ Tambah Konfigurasi Provider"}
                </button>
              </div>
            </form>

            <hr style={{ margin: "24px 0", border: "0", borderTop: "1px solid var(--line)" }} />

            {/* PENGATURAN PROVIDER POOL & KEBIJAKAN RETRY (SC-022) */}
            <form onSubmit={handleSavePoolSettings} className="cred-form" style={{ marginTop: "18px", maxWidth: "100%", background: "#f8fbf9" }}>
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <h4>⚙️ Pengaturan Provider Pool & Kebijakan Retry (SC-022)</h4>
                {poolSettings && (
                  <span style={{ fontSize: "11px", color: "var(--muted)" }}>
                    Pembaruan: {new Date(poolSettings.updatedAt).toLocaleTimeString()}
                  </span>
                )}
              </div>
              <p style={{ fontSize: "12px", color: "var(--muted)", margin: "4px 0 14px 0" }}>
                Atur strategi pemilihan provider, kuota konkurensi, dan toleransi retry. Pool menjamin ketahanan operasional tanpa memanipulasi kuota atau ketentuan penyedia AI.
              </p>

              <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))", gap: "16px" }}>
                <div className="form-group">
                  <label htmlFor="pool-strategy">Strategi Pemilihan Provider:</label>
                  <select
                    id="pool-strategy"
                    value={poolStrategy}
                    onChange={(e) => setPoolStrategy(e.target.value as PoolStrategy)}
                  >
                    <option value="adaptive">Adaptive Availability-Aware (Cerdas & Seimbang)</option>
                    <option value="priority">Priority (Utamakan Prioritas Tertinggi)</option>
                    <option value="round_robin">Round-Robin (Rotasi Bergantian)</option>
                    <option value="weighted">Weighted (Berdasarkan Bobot Relatif)</option>
                  </select>
                </div>

                <div className="form-group">
                  <label htmlFor="pool-max-attempts">Batas Percobaan Maksimal (Attempts):</label>
                  <input
                    id="pool-max-attempts"
                    type="number"
                    min={1}
                    max={10}
                    value={poolMaxAttempts}
                    onChange={(e) => setPoolMaxAttempts(Number(e.target.value))}
                  />
                </div>

                <div className="form-group">
                  <label htmlFor="pool-base-delay">Jeda Awal Retry (ms):</label>
                  <input
                    id="pool-base-delay"
                    type="number"
                    min={100}
                    max={60000}
                    step={500}
                    value={poolRetryBaseDelayMs}
                    onChange={(e) => setPoolRetryBaseDelayMs(Number(e.target.value))}
                  />
                </div>

                <div className="form-group">
                  <label htmlFor="pool-max-delay">Batas Maksimal Delay Retry (ms):</label>
                  <input
                    id="pool-max-delay"
                    type="number"
                    min={1000}
                    max={120000}
                    step={1000}
                    value={poolMaxRetryDelayMs}
                    onChange={(e) => setPoolMaxRetryDelayMs(Number(e.target.value))}
                  />
                </div>
              </div>

              <div style={{ marginTop: "10px", display: "flex", alignItems: "center", gap: "8px" }}>
                <label style={{ display: "flex", alignItems: "center", gap: "8px", fontSize: "13px", cursor: "pointer", fontWeight: "normal" }}>
                  <input
                    type="checkbox"
                    checked={poolAutoRetry}
                    onChange={(e) => setPoolAutoRetry(e.target.checked)}
                  />
                  Aktifkan Auto-Retry otomatis pada kegagalan sementara (Rate Limit 429 & Network Error)
                </label>
              </div>

              <div className="info-callout" style={{ marginTop: "12px", fontSize: "11px", color: "var(--muted)" }}>
                <strong>Kepatuhan Kuota & Privasi:</strong> Error pembatasan kuota tingkat akun/project Google Cloud (<code>QuotaExhausted</code>) akan langsung menghentikan retry pada seluruh pool. Dilarang menggunakan pool untuk menghindari batas free-tier atau ketentuan penyedia AI.
              </div>

              <div style={{ marginTop: "14px" }}>
                <button type="submit" className="btn btn-primary" disabled={isSavingPoolSettings}>
                  {isSavingPoolSettings ? "Menyimpan Pengaturan..." : "💾 Simpan Pengaturan Pool"}
                </button>
              </div>
            </form>

            <hr style={{ margin: "24px 0", border: "0", borderTop: "1px solid var(--line)" }} />

            {/* DAFTAR KONFIGURASI PROVIDER */}
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <h4>Daftar Konfigurasi Provider AI ({providerConfigs.length})</h4>
              <button
                className="btn btn-secondary"
                style={{ padding: "6px 12px", fontSize: "11px" }}
                onClick={() => {
                  void loadProviderConfigs();
                  void loadPoolData();
                }}
                disabled={isProviderConfigsLoading}
              >
                {isProviderConfigsLoading ? "Menyegarkan..." : "🔄 Segarkan Provider & Health"}
              </button>
            </div>

            {providerConfigs.length === 0 ? (
              <div className="empty-inline" style={{ marginTop: "14px" }}>
                Belum ada konfigurasi provider AI yang dibuat. Tambahkan konfigurasi di atas untuk mengaktifkan mesin jawaban.
              </div>
            ) : (
              <div className="provider-grid">
                {providerConfigs.map((config) => {
                  const testRes = providerTestResults[config.id];
                  const isTesting = testingProviderId === config.id;
                  const healthInfo = providerHealthList.find((h) => h.providerConfigId === config.id);
                  const stateKey = !config.enabled ? "disabled" : (healthInfo?.state || "ready");
                  const stateLabels: Record<string, string> = {
                    ready: "🟢 Siap",
                    cooling_down: `⏳ Cooldown${healthInfo?.cooldownUntil ? ` (${new Date(healthInfo.cooldownUntil).toLocaleTimeString()})` : ""}`,
                    quota_exhausted: "🛑 Batas Kuota",
                    needs_attention: "⚠️ Perlu Perhatian",
                    disabled: "⚪ Nonaktif",
                  };

                  return (
                    <div key={config.id} className={`provider-card ${!config.enabled ? "disabled" : ""}`}>
                      <div className="cred-card-header">
                        <div>
                          <h5 className="cred-card-title">{config.displayName}</h5>
                          <div style={{ display: "flex", gap: "6px", alignItems: "center", marginTop: "4px", flexWrap: "wrap" }}>
                            <span className="badge">
                              {config.providerType === "gemini" ? "Google Gemini" : "OpenAI Compatible"}
                            </span>
                            <span className={`pool-health-badge ${stateKey}`}>
                              {stateLabels[stateKey] || stateKey}
                            </span>
                          </div>
                        </div>
                        <button
                          className={`btn ${config.enabled ? "btn-secondary" : "btn-primary"}`}
                          style={{ padding: "3px 8px", fontSize: "11px" }}
                          onClick={() => void handleToggleProviderEnabled(config)}
                        >
                          {config.enabled ? "✓ Aktif" : "Nonaktif"}
                        </button>
                      </div>

                      <div style={{ display: "flex", alignItems: "center", gap: "8px", flexWrap: "wrap" }}>
                        <span className="provider-model-pill">Model: {config.modelId}</span>
                        <span className="badge" style={{ fontSize: "10px" }}>Prioritas: {config.priority}</span>
                        <span className="badge" style={{ fontSize: "10px" }}>Bobot: {config.weight}</span>
                        <span className="badge" style={{ fontSize: "10px" }}>
                          Konkurensi: {healthInfo?.currentConcurrency || 0} / {config.maxConcurrency}
                        </span>
                        <span className="badge" style={{ fontSize: "10px" }}>Timeout: {config.timeoutMs}ms</span>
                      </div>

                      <div style={{ fontSize: "11px", color: "var(--muted)" }}>
                        <div>Kredensial: <strong>{config.credentialLabel || config.credentialRef}</strong></div>
                        {config.baseUrl && <div>Endpoint: <code>{config.baseUrl}</code></div>}
                        {healthInfo?.recentLatencyMs != null && (
                          <div>Latensi Terkini: <strong>{healthInfo.recentLatencyMs} ms</strong></div>
                        )}
                      </div>

                      {testRes && (
                        <div className={`provider-test-status ${testRes.success ? "success" : "error"}`}>
                          <div>
                            <strong>{testRes.success ? "✓ Terhubung" : "✕ Gagal"}</strong> ({testRes.latencyMs} ms)
                          </div>
                          <div>{testRes.message}</div>
                        </div>
                      )}

                      <div className="card-actions" style={{ marginTop: "auto", borderTop: "1px solid #f2f5f3", paddingTop: "10px" }}>
                        <button
                          className="btn btn-primary"
                          style={{ padding: "6px 12px", fontSize: "11px" }}
                          onClick={() => void handleTestProviderConnection(config.id)}
                          disabled={isTesting}
                        >
                          {isTesting ? "Menguji Koneksi..." : "⚡ Uji Koneksi Model"}
                        </button>
                        <button
                          className="btn btn-secondary"
                          style={{ padding: "6px 10px", fontSize: "11px" }}
                          onClick={() => handleOpenEditProviderModal(config)}
                        >
                          ✏ Ubah
                        </button>
                        <button
                          className="danger-button"
                          style={{ padding: "6px 10px", fontSize: "11px" }}
                          onClick={() => void handleDeleteProviderConfig(config.id, config.displayName)}
                        >
                          🗑 Hapus
                        </button>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </section>
      )}

      {/* MODAL EDIT KONFIGURASI PROVIDER (SC-021) */}
      {editingProviderConfig && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-edit-provider-title">
            <div className="modal-header">
              <h3 id="modal-edit-provider-title">Ubah Konfigurasi: {editingProviderConfig.displayName}</h3>
              <button className="text-button" aria-label="Tutup dialog ubah konfigurasi provider" onClick={() => setEditingProviderConfig(null)}>✕</button>
            </div>
            <form onSubmit={handleSaveUpdateProvider}>
              <div className="modal-body">
                <label>
                  Nama Konfigurasi Provider:
                  <input
                    type="text"
                    value={editPConfigDisplayName}
                    onChange={(e) => setEditPConfigDisplayName(e.target.value)}
                    required
                  />
                </label>
                <label>
                  Model ID:
                  <input
                    type="text"
                    value={editPConfigModelId}
                    onChange={(e) => setEditPConfigModelId(e.target.value)}
                    required
                  />
                </label>
                <label>
                  Custom Endpoint Base URL (Opsional):
                  <input
                    type="text"
                    placeholder="Contoh: https://api.openai.com/v1"
                    value={editPConfigBaseUrl}
                    onChange={(e) => setEditPConfigBaseUrl(e.target.value)}
                  />
                </label>
                <label>
                  Prioritas (1 = Tertinggi):
                  <input
                    type="number"
                    min={1}
                    max={1000}
                    value={editPConfigPriority}
                    onChange={(e) => setEditPConfigPriority(Number(e.target.value))}
                  />
                </label>
                <label>
                  Timeout (ms):
                  <input
                    type="number"
                    min={1000}
                    max={120000}
                    step={1000}
                    value={editPConfigTimeoutMs}
                    onChange={(e) => setEditPConfigTimeoutMs(Number(e.target.value))}
                  />
                </label>
                <label>
                  Kredensial Terkait:
                  <select
                    value={editPConfigCredentialRef}
                    onChange={(e) => setEditPConfigCredentialRef(e.target.value)}
                    className="select-input"
                  >
                    {credentials.map((c) => (
                      <option key={c.credentialRef} value={c.credentialRef}>
                        {c.label} ({c.providerType})
                      </option>
                    ))}
                  </select>
                </label>
              </div>
              <div className="modal-footer">
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => setEditingProviderConfig(null)}
                  disabled={isUpdatingPConfig}
                >
                  Batal
                </button>
                <button
                  type="submit"
                  className="btn btn-primary"
                  disabled={isUpdatingPConfig}
                >
                  {isUpdatingPConfig ? "Menyimpan..." : "Simpan Perubahan"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL GANTI KUNCI / UPDATE */}
      {editingCred && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="dialog" aria-modal="true" aria-labelledby="modal-update-cred-title">
            <div className="modal-header">
              <h3 id="modal-update-cred-title">Perbarui Kredensial: {editingCred.label}</h3>
              <button className="text-button" aria-label="Tutup dialog perbarui kredensial" onClick={() => setEditingCred(null)}>✕</button>
            </div>
            <form onSubmit={handleSaveUpdate}>
              <div className="modal-body">
                <label>
                  Label Kredensial:
                  <input
                    type="text"
                    value={updateLabel}
                    onChange={(e) => setUpdateLabel(e.target.value)}
                    required
                  />
                </label>
                <label>
                  Kunci API Baru (Kosongkan jika hanya ingin mengubah label):
                  <input
                    type="password"
                    placeholder="Tempel kunci API rahasia baru (opsional)"
                    value={updateSecret}
                    onChange={(e) => setUpdateSecret(e.target.value)}
                    autoComplete="new-password"
                  />
                </label>
                <p className="cred-security-hint">
                  🔒 Kunci API baru akan menimpa data di Windows Credential Manager secara aman.
                </p>
              </div>
              <div className="modal-footer">
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => setEditingCred(null)}
                  disabled={isUpdatingCred}
                >
                  Batal
                </button>
                <button
                  type="submit"
                  className="btn btn-primary"
                  disabled={isUpdatingCred}
                >
                  {isUpdatingCred ? "Menyimpan..." : "Simpan Pembaruan"}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* MODAL KONFIRMASI HAPUS KREDENSIAL BERSAMA */}
      {deletingCred && (
        <div className="modal-backdrop">
          <div className="modal-dialog" role="alertdialog" aria-modal="true" aria-labelledby="modal-del-cred-title">
            <div className="modal-header">
              <h3 id="modal-del-cred-title" style={{ color: "#9c2b28" }}>⚠ Kredensial Masih Digunakan Bersama</h3>
              <button className="text-button" aria-label="Tutup dialog peringatan kredensial bersama" onClick={() => setDeletingCred(null)}>✕</button>
            </div>
            <div className="modal-body">
              <p>
                Kredensial <strong>'{deletingCred.label}'</strong> saat ini sedang direferensikan oleh{" "}
                <strong>{deletingCred.usageCount} konfigurasi provider AI</strong>.
              </p>
              <div className="danger-callout">
                <h4>Peringatan Kredensial Bersama:</h4>
                <p>
                  Jika Anda menghapus kredensial ini dari Windows Credential Manager, seluruh konfigurasi provider yang menggunakannya tidak akan dapat melakukan otentikasi ke AI.
                </p>
              </div>
              <label style={{ display: "flex", alignItems: "flex-start", gap: "8px", fontWeight: "normal" }}>
                <input
                  type="checkbox"
                  checked={forceDeleteConsent}
                  onChange={(e) => setForceDeleteConsent(e.target.checked)}
                />
                <span>Saya memahami dampaknya dan tetap ingin menghapus kunci ini secara permanen dari OS.</span>
              </label>
            </div>
            <div className="modal-footer">
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => setDeletingCred(null)}
                disabled={isDeletingCred}
              >
                Batal
              </button>
              <button
                type="button"
                className="btn btn-danger"
                onClick={() => void handleExecuteForceDelete()}
                disabled={!forceDeleteConsent || isDeletingCred}
              >
                {isDeletingCred ? "Menghapus..." : "Tetap Hapus Kredensial"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* TAB PRIVASI & DIAGNOSTIK (SC-060) */}
      {activeTab === "privacy" && (
        <section className="settings-tab-content" aria-labelledby="privacy-heading">
          {privacyErrorMessage && (
            <div className="alert alert-danger" role="alert" style={{ marginBottom: "16px" }}>
              {privacyErrorMessage}
            </div>
          )}

          {privacySuccessMessage && (
            <div className="alert alert-success" role="alert" style={{ marginBottom: "16px" }}>
              {privacySuccessMessage}
            </div>
          )}

          {/* KARTU 1: PENGATURAN PRIVASI & PERSETUJUAN */}
          <div className="card" style={{ marginBottom: "20px" }}>
            <h3 id="privacy-heading">🛡 Kebijakan Privasi & Persetujuan Eksplisit (SC-060)</h3>
            <p className="description-text">
              Atur batas privasi transmisi data keluar ke model AI, kebijakan persetujuan eksplisit per-permintaan, dan retensi tangkapan layar.
            </p>
            {privacySettings && (
              <div style={{ fontSize: "11px", color: "#64748b", marginBottom: "12px" }}>
                Terakhir diperbarui: {new Date(privacySettings.updatedAt).toLocaleString("id-ID")}
              </div>
            )}

            <form onSubmit={(e) => void handleSavePrivacySettings(e)} style={{ display: "grid", gap: "16px" }}>
              <div style={{ background: "#f8fafc", border: "1px solid #e2e8f0", borderRadius: "8px", padding: "14px 16px" }}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <div>
                    <strong style={{ fontSize: "13px", color: "#0f172a" }}>🔒 Redaksi Otomatis Log & Diagnostik</strong>
                    <div style={{ fontSize: "12px", color: "#64748b", marginTop: "2px" }}>
                      Seluruh kunci API, Bearer token, password, NIK KTP 16 digit, alamat email, dan nomor telepon otomatis disensor sebelum dicatat.
                    </div>
                  </div>
                  <span style={{ fontSize: "11px", fontWeight: "700", background: "#dcfce7", color: "#15803d", padding: "4px 10px", borderRadius: "20px" }}>
                    ✓ Aktif & Terlindungi
                  </span>
                </div>
              </div>

              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "16px" }}>
                <div className="form-group">
                  <label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer", fontWeight: "600", fontSize: "13px" }}>
                    <input
                      type="checkbox"
                      checked={privRequireSensitiveConsent}
                      onChange={(e) => setPrivRequireSensitiveConsent(e.target.checked)}
                    />
                    Wajibkan Izin Fakta Profil Sensitif
                  </label>
                  <p style={{ margin: "4px 0 0", fontSize: "11px", color: "#64748b" }}>
                    Model AI tidak akan menerima data berlabel sensitif (pendapatan, agama, identitas) tanpa izin per-permintaan.
                  </p>
                </div>

                <div className="form-group">
                  <label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer", fontWeight: "600", fontSize: "13px" }}>
                    <input
                      type="checkbox"
                      checked={privRequireVisionConsent}
                      onChange={(e) => setPrivRequireVisionConsent(e.target.checked)}
                    />
                    Wajibkan Izin External Vision
                  </label>
                  <p style={{ margin: "4px 0 0", fontSize: "11px", color: "#64748b" }}>
                    Gambar tangkapan layar dilarang dikirimkan ke model visual eksternal tanpa persetujuan eksplisit per-permintaan.
                  </p>
                </div>
              </div>

              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "16px" }}>
                <div className="form-group">
                  <label style={{ fontWeight: "600", fontSize: "13px", color: "#334155" }}>
                    Kebijakan Retensi Screenshot Sementara:
                  </label>
                  <select
                    value={privRetentionPolicy}
                    onChange={(e) => setPrivRetentionPolicy(e.target.value)}
                    className="select-input"
                    style={{ marginTop: "4px" }}
                  >
                    <option value="immediate_delete">Hapus Segera Setelah OCR Selesai (Rekomendasi Privasi)</option>
                    <option value="retain_1_hour">Simpan Maksimal 1 Jam</option>
                    <option value="retain_24_hours">Simpan Maksimal 24 Jam</option>
                    <option value="retain_manual_only">Pertahankan Hingga Pembersihan Manual</option>
                  </select>
                </div>

                <div className="form-group">
                  <label style={{ fontWeight: "600", fontSize: "13px", color: "#334155" }}>
                    Level Logging Diagnostik:
                  </label>
                  <select
                    value={privLogLevel}
                    onChange={(e) => setPrivLogLevel(e.target.value)}
                    className="select-input"
                    style={{ marginTop: "4px" }}
                  >
                    <option value="INFO">INFO (Normal)</option>
                    <option value="WARN">WARN (Peringatan & Error)</option>
                    <option value="ERROR">ERROR (Hanya Error)</option>
                    <option value="DEBUG">DEBUG (Diagnostik Mendalam)</option>
                  </select>
                </div>
              </div>

              <div className="form-group">
                <label style={{ display: "flex", alignItems: "center", gap: "8px", cursor: "pointer", fontWeight: "600", fontSize: "13px" }}>
                  <input
                    type="checkbox"
                    checked={privAutoCleanupOnExit}
                    onChange={(e) => setPrivAutoCleanupOnExit(e.target.checked)}
                  />
                  Bersihkan Seluruh Berkas Sementara saat Aplikasi Ditutup (Auto-cleanup on Exit)
                </label>
              </div>

              <div style={{ display: "flex", justifyContent: "flex-end", marginTop: "8px" }}>
                <button
                  type="submit"
                  className="btn btn-primary"
                  disabled={isSavingPrivacy}
                  style={{ background: "#0f6642", borderColor: "#0f6642" }}
                >
                  {isSavingPrivacy ? "Menyimpan..." : "💾 Simpan Pengaturan Privasi"}
                </button>
              </div>
            </form>
          </div>

          {/* KARTU 2: PEMBERSIHAN BERKAS SEMENTARA */}
          <div className="card" style={{ marginBottom: "20px" }}>
            <h3>🧹 Penyimpanan Sementara & Pembersihan Screenshot</h3>
            <p className="description-text">
              Kelola penggunaan disk untuk gambar tangkapan layar sementara di folder <code>temp_captures</code>.
            </p>

            <div style={{ background: "#f8fafc", border: "1px solid #e2e8f0", borderRadius: "8px", padding: "14px 16px", marginBottom: "16px" }}>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px", fontSize: "13px" }}>
                <div>
                  <span style={{ color: "#64748b" }}>Direktori Cache Sementara:</span>
                  <div style={{ fontWeight: "600", wordBreak: "break-all", marginTop: "2px" }}>
                    <code>{retentionStatus?.tempDirectoryPath || "app_data/temp_captures"}</code>
                  </div>
                </div>
                <div>
                  <span style={{ color: "#64748b" }}>Berkas Screenshot Tersimpan:</span>
                  <div style={{ fontWeight: "700", fontSize: "15px", color: "#0f172a", marginTop: "2px" }}>
                    {retentionStatus?.tempFileCount || 0} berkas ({((retentionStatus?.tempTotalBytes || 0) / 1024).toFixed(1)} KB)
                  </div>
                </div>
              </div>
            </div>

            <div style={{ display: "flex", gap: "12px", alignItems: "center" }}>
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => void handleCleanupTempFiles()}
                disabled={isCleaningTemp}
              >
                {isCleaningTemp ? "Membersihkan..." : "🧹 Bersihkan Berkas Sementara Sekarang"}
              </button>
              {cleanResult && (
                <span style={{ fontSize: "12px", color: "#059669", fontWeight: "600" }}>
                  ✓ {cleanResult.message}
                </span>
              )}
            </div>
          </div>

          {/* KARTU 3: DIAGNOSTIK SISTEM & KESEHATAN APLIKASI */}
          <div className="card" style={{ marginBottom: "20px" }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
              <h3 style={{ margin: 0 }}>📊 Diagnostik Sistem & Basis Data</h3>
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => void loadPrivacyData()}
                disabled={isPrivacyLoading}
                style={{ fontSize: "12px", padding: "4px 12px" }}
              >
                {isPrivacyLoading ? "Memuat..." : "🔄 Segarkan Diagnostik"}
              </button>
            </div>

            <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", gap: "12px" }}>
              <div style={{ background: "#ffffff", border: "1px solid #e2e8f0", borderRadius: "8px", padding: "12px" }}>
                <div style={{ fontSize: "11px", color: "#64748b" }}>Versi Aplikasi & Platform</div>
                <strong style={{ fontSize: "14px", color: "#0f172a" }}>v{systemDiag?.appVersion || "0.1.0"}</strong>
                <div style={{ fontSize: "11px", color: "#94a3b8" }}>OS: {systemDiag?.osPlatform || "windows"}</div>
              </div>

              <div style={{ background: "#ffffff", border: "1px solid #e2e8f0", borderRadius: "8px", padding: "12px" }}>
                <div style={{ fontSize: "11px", color: "#64748b" }}>SQLite & Skema Migrasi</div>
                <strong style={{ fontSize: "14px", color: "#0f172a" }}>v{systemDiag?.sqliteVersion || "3.x"}</strong>
                <div style={{ fontSize: "11px", color: "#059669" }}>Migrasi: v{systemDiag?.dbSchemaVersion || 13}</div>
              </div>

              <div style={{ background: "#ffffff", border: "1px solid #e2e8f0", borderRadius: "8px", padding: "12px" }}>
                <div style={{ fontSize: "11px", color: "#64748b" }}>Ukuran Basis Data & WAL</div>
                <strong style={{ fontSize: "14px", color: "#0f172a" }}>
                  {((systemDiag?.totalDbSizeBytes || 0) / 1024).toFixed(1)} KB
                </strong>
                <div style={{ fontSize: "11px", color: "#94a3b8" }}>
                  WAL: {((systemDiag?.walSizeBytes || 0) / 1024).toFixed(1)} KB
                </div>
              </div>

              <div style={{ background: "#ffffff", border: "1px solid #e2e8f0", borderRadius: "8px", padding: "12px" }}>
                <div style={{ fontSize: "11px", color: "#64748b" }}>Penyimpanan Kredensial</div>
                <strong style={{ fontSize: "13px", color: "#0f172a" }}>
                  {systemDiag?.credentialStoreType || "Windows Credential Manager"}
                </strong>
                <div style={{ fontSize: "11px", color: "#059669" }}>✓ Native OS Vault</div>
              </div>
            </div>
          </div>

          {/* KARTU 4: PENAMPIL LOG DIAGNOSTIK TEREDAKSI */}
          <div className="card">
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "12px" }}>
              <div>
                <h3 style={{ margin: 0 }}>📝 Penampil Log Diagnostik Aman (Redacted Logs)</h3>
                <p style={{ margin: "2px 0 0", fontSize: "12px", color: "#64748b" }}>
                  Seluruh log disimpan secara lokal dan telah melalui penyensoran otomatis mutlak (anti-leak PII & Secret).
                </p>
              </div>

              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => void handleClearDiagnosticLogs()}
                disabled={isClearingLogs || diagLogs.length === 0}
                style={{ fontSize: "12px", padding: "4px 12px" }}
              >
                {isClearingLogs ? "Mengosongkan..." : "🗑 Kosongkan Log"}
              </button>
            </div>

            <div style={{ display: "flex", gap: "12px", marginBottom: "12px" }}>
              <select
                value={logFilterLevel}
                onChange={(e) => {
                  setLogFilterLevel(e.target.value);
                  void loadPrivacyData(e.target.value, logFilterCategory);
                }}
                className="select-input"
                style={{ maxWidth: "160px", fontSize: "12px" }}
              >
                <option value="ALL">Semua Level</option>
                <option value="INFO">INFO</option>
                <option value="WARN">WARN</option>
                <option value="ERROR">ERROR</option>
                <option value="DEBUG">DEBUG</option>
              </select>

              <select
                value={logFilterCategory}
                onChange={(e) => {
                  setLogFilterCategory(e.target.value);
                  void loadPrivacyData(logFilterLevel, e.target.value);
                }}
                className="select-input"
                style={{ maxWidth: "200px", fontSize: "12px" }}
              >
                <option value="ALL">Semua Kategori</option>
                <option value="AI_PROVIDER">AI_PROVIDER</option>
                <option value="SCREEN_CAPTURE">SCREEN_CAPTURE</option>
                <option value="DATA_RETENTION">DATA_RETENTION</option>
                <option value="SECURITY">SECURITY</option>
                <option value="SYSTEM">SYSTEM</option>
              </select>
            </div>

            {diagLogs.length === 0 ? (
              <p style={{ fontSize: "12px", color: "#94a3b8", fontStyle: "italic", margin: 0 }}>
                Belum ada rekaman log diagnostik pada filter ini.
              </p>
            ) : (
              <div style={{ maxHeight: "300px", overflowY: "auto", border: "1px solid #e2e8f0", borderRadius: "6px" }}>
                <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "12px" }}>
                  <thead>
                    <tr style={{ background: "#f8fafc", borderBottom: "1px solid #e2e8f0", textAlign: "left" }}>
                      <th style={{ padding: "8px 12px", width: "160px" }}>Waktu (UTC)</th>
                      <th style={{ padding: "8px 12px", width: "80px" }}>Level</th>
                      <th style={{ padding: "8px 12px", width: "130px" }}>Kategori</th>
                      <th style={{ padding: "8px 12px" }}>Pesan Log (Teredaksi)</th>
                    </tr>
                  </thead>
                  <tbody>
                    {diagLogs.map((log) => (
                      <tr key={log.id} style={{ borderBottom: "1px solid #f1f5f9" }}>
                        <td style={{ padding: "6px 12px", color: "#64748b", fontSize: "11px" }}>
                          {new Date(log.timestamp).toLocaleString("id-ID")}
                        </td>
                        <td style={{ padding: "6px 12px" }}>
                          <span
                            style={{
                              padding: "2px 6px",
                              borderRadius: "4px",
                              fontSize: "10px",
                              fontWeight: "700",
                              background:
                                log.level === "ERROR"
                                  ? "#fee2e2"
                                  : log.level === "WARN"
                                  ? "#fef3c7"
                                  : "#e0f2fe",
                              color:
                                log.level === "ERROR"
                                  ? "#991b1b"
                                  : log.level === "WARN"
                                  ? "#92400e"
                                  : "#0369a1",
                            }}
                          >
                            {log.level}
                          </span>
                        </td>
                        <td style={{ padding: "6px 12px", fontWeight: "600", color: "#334155" }}>
                          {log.category}
                        </td>
                        <td style={{ padding: "6px 12px", color: "#0f172a" }}>
                          {log.message}
                          {log.metadataJson && (
                            <div style={{ fontSize: "10px", color: "#94a3b8", marginTop: "2px", fontFamily: "monospace" }}>
                              {log.metadataJson}
                            </div>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </section>
      )}

      {/* TAB 2: EKSPOR DATA (SC-012) */}
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

      {/* TAB 3: RETENSI & KEBIJAKAN PRIVASI (SC-012) */}
      {activeTab === "retention" && (
        <section className="settings-tab-content" aria-labelledby="retention-heading">
          <div className="card">
            <h3 id="retention-heading">Kebijakan Privasi & Siklus Hidup Data</h3>
            <p className="description-text">
              Transparansi siklus retensi data aplikasi Survey Copilot. Seluruh data responden disimpan secara lokal di mesin Anda dan tidak pernah dibagikan tanpa tindakan eksplisit.
            </p>

            {isRetentionLoading ? (
              <p>Memuat kebijakan retensi...</p>
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

      {/* TAB 4: ZONA BAHAYA (SC-012) */}
      {activeTab === "danger" && (
        <section className="settings-tab-content" aria-labelledby="danger-heading">
          {/* HAPUS SINGLE PERSONA */}
          <div className="card danger-card">
            <h3>Hapus Persona Spesifik</h3>
            <p className="description-text">
              Pilih satu persona untuk dihapus secara tuntas beserta seluruh fakta profil, riwayat, dan sesi survei terkait.
            </p>
            <div className="form-group" style={{ maxWidth: "400px", marginBottom: "14px" }}>
              <select
                value={personaToDeleteId}
                onChange={(e) => setPersonaToDeleteId(e.target.value)}
                className="select-input"
              >
                <option value="">-- Pilih persona untuk dihapus --</option>
                {personas.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.displayName} {p.isActive ? "(Aktif)" : ""}
                  </option>
                ))}
              </select>
            </div>
            <button
              className="btn btn-danger"
              onClick={() => void handleExecuteDeleteSinglePersona()}
              disabled={!personaToDeleteId || isDeletingPersona}
            >
              {isDeletingPersona ? "Menghapus..." : "🗑 Hapus Persona Terpilih"}
            </button>
          </div>

          {/* WIPE SELURUH DATA APLIKASI */}
          <div className="card danger-card danger-card-total">
            <h3 id="danger-heading">🚨 Pembersihan Total: Hapus Seluruh Data Aplikasi</h3>
            <p className="danger-warning">
              Tindakan ini akan <strong>MENGHAPUS PERMANEN</strong> seluruh persona responden, fakta profil, bukti sumber, riwayat jawaban, riwayat sesi survei, dan file tangkapan layar sementara.
              Basis data akan dipadatkan (VACUUM) dan checkpoint WAL akan di-truncate.
            </p>

            <div className="pre-wipe-actions">
              <button
                className="btn btn-secondary"
                onClick={() => void handleLoadWipeSummary()}
                disabled={isWipeSummaryLoading || isWiping}
              >
                {isWipeSummaryLoading ? "Menghitung Record..." : "📊 Hitung Jumlah Data yang Akan Terhapus"}
              </button>
            </div>

            {wipeSummary && (
              <div className="wipe-summary-table-box">
                <h4>Data yang Akan Dihapus Bersih:</h4>
                <ul className="wipe-counts-list">
                  <li>Total Persona: <strong>{wipeSummary.personas}</strong></li>
                  <li>Fakta Profil: <strong>{wipeSummary.facts}</strong></li>
                  <li>Kredensial OS: <strong>{wipeSummary.credentials}</strong></li>
                  <li>Ekstraksi OCR: <strong>{wipeSummary.ocrExtractions}</strong></li>
                  <li>Survey recognition: <strong>{wipeSummary.recognitions}</strong></li>
                  <li>Branch transition NQI: <strong>{wipeSummary.branchTransitions}</strong></li>
                  <li>Prediksi pertanyaan NQI: <strong>{wipeSummary.questionPredictions}</strong></li>
                  <li>Log diagnostik: <strong>{wipeSummary.diagnosticLogs}</strong></li>
                  <li>Sumber rekomendasi: <strong>{wipeSummary.recommendationSources}</strong></li>
                  <li>Riwayat health provider: <strong>{wipeSummary.providerHealth}</strong></li>
                  <li>Sumber Bukti: <strong>{wipeSummary.sources}</strong></li>
                  <li>Riwayat Perubahan: <strong>{wipeSummary.history}</strong></li>
                  <li>Preferensi: <strong>{wipeSummary.preferences}</strong></li>
                  <li>Sesi Survei: <strong>{wipeSummary.sessions}</strong></li>
                  <li>Pertanyaan: <strong>{wipeSummary.questions}</strong></li>
                  <li>Rekomendasi Jawaban: <strong>{wipeSummary.recommendations}</strong></li>
                  <li>Riwayat Jawaban: <strong>{wipeSummary.answers}</strong></li>
                  <li>Hasil Kesesuaian Survei: <strong>{wipeSummary.fitResults}</strong></li>
                  <li>Percobaan Survei: <strong>{wipeSummary.attempts}</strong></li>
                </ul>
              </div>
            )}

            <div className="wipe-confirmation-container">
              <label className="consent-checkbox-group">
                <input
                  type="checkbox"
                  checked={isConsentChecked}
                  onChange={(e) => setIsConsentChecked(e.target.checked)}
                />
                <span>
                  Saya secara sadar mengonfirmasi bahwa tindakan pembersihan total ini bersifat destruktif permanen dan tidak dapat dibatalkan.
                </span>
              </label>

              <div className="form-group">
                <label htmlFor="wipe-confirmation-text">
                  Ketik <strong>HAPUS SEMUA DATA</strong> di bawah ini untuk mengonfirmasi:
                </label>
                <input
                  id="wipe-confirmation-text"
                  type="text"
                  placeholder="Ketik persis: HAPUS SEMUA DATA"
                  value={wipeConfirmationInput}
                  onChange={(e) => setWipeConfirmationInput(e.target.value)}
                  className="select-input"
                  autoComplete="off"
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
              <div
                className={`alert ${wipeReport.failedTempFilesCount > 0 ? "alert-error" : "alert-success"} wipe-report-box`}
                role="alert"
              >
                <h4>
                  {wipeReport.failedTempFilesCount > 0
                    ? "Database dan credential telah dihapus; pembersihan file perlu diperiksa"
                    : "✓ Seluruh Data Terdaftar Berhasil Dihapus"}
                </h4>
                <p>Status Pembersihan Basis Data, Credential OS & Disk:</p>
                <ul>
                  <li>Persona terhapus: {wipeReport.deletedPersonasCount}</li>
                  <li>Fakta profil terhapus: {wipeReport.deletedFactsCount}</li>
                  <li>Sumber bukti terhapus: {wipeReport.deletedSourcesCount}</li>
                  <li>Sesi & pertanyaan terhapus: {wipeReport.deletedSessionsCount + wipeReport.deletedQuestionsCount}</li>
                  <li>Credential OS terhapus: {wipeReport.deletedCredentialsCount}</li>
                  <li>OCR extraction terhapus: {wipeReport.deletedOcrExtractionsCount}</li>
                  <li>Survey recognition terhapus: {wipeReport.deletedRecognitionsCount}</li>
                  <li>NQI transition/prediction terhapus: {wipeReport.deletedBranchTransitionsCount + wipeReport.deletedQuestionPredictionsCount}</li>
                  <li>Diagnostic log terhapus: {wipeReport.deletedDiagnosticLogsCount}</li>
                  <li>Sumber rekomendasi terhapus: {wipeReport.deletedRecommendationSourcesCount}</li>
                  <li>Riwayat health provider terhapus: {wipeReport.deletedProviderHealthCount}</li>
                  <li>File sementara dibersihkan: {wipeReport.deletedTempFilesCount}; gagal dihapus: {wipeReport.failedTempFilesCount}</li>
                  <li>Checkpoint WAL SQLite: {wipeReport.walCheckpointStatus}</li>
                  <li>Status VACUUM SQLite: {wipeReport.vacuumStatus}</li>
                </ul>
                {wipeReport.cleanupWarnings.length > 0 && (
                  <div className="unaffected-items-note">
                    <strong>Peringatan pembersihan:</strong>
                    <ul>
                      {wipeReport.cleanupWarnings.map((warning, idx) => (
                        <li key={idx}>{warning}</li>
                      ))}
                    </ul>
                  </div>
                )}
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
