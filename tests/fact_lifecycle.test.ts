import test from "node:test";
import assert from "node:assert/strict";
import type {
  ConflictWarning,
  CreateFactInput,
  DataType,
  FactHistoryEntry,
  FactSource,
  ProfileFact,
  ProfileFactDetail,
  Sensitivity,
  VerificationStatus,
} from "../src/types/fact";

test("ProfileFact data contract supports all typed fact variants", () => {
  const numberFact: ProfileFact = {
    id: "f-1",
    personaId: "p-1",
    category: "Finansial",
    factKey: "monthly_income",
    attribute: "Penghasilan Bulanan",
    dataType: "number",
    value: 15000000,
    normalizedValue: "15000000",
    unit: "IDR",
    verificationStatus: "confirmed",
    sensitivity: "sensitive",
    sourceDescription: "Slip Gaji Resmi",
    validFrom: "2024-01-01",
    validUntil: "2025-01-01",
    isValid: true,
    hasConflict: false,
    sourcesCount: 1,
    createdAt: "2024-01-01T00:00:00Z",
    updatedAt: "2024-01-01T00:00:00Z",
  };

  assert.equal(numberFact.dataType, "number");
  assert.equal(numberFact.value, 15000000);
  assert.equal(numberFact.unit, "IDR");
  assert.equal(numberFact.sensitivity, "sensitive");

  const boolFact: ProfileFact = {
    id: "f-2",
    personaId: "p-1",
    category: "Transportasi",
    factKey: "owns_car",
    attribute: "Kepemilikan Mobil",
    dataType: "boolean",
    value: true,
    normalizedValue: "true",
    verificationStatus: "confirmed",
    sensitivity: "ordinary",
    isValid: true,
    hasConflict: false,
    sourcesCount: 1,
    createdAt: "2024-01-01T00:00:00Z",
    updatedAt: "2024-01-01T00:00:00Z",
  };

  assert.equal(boolFact.dataType, "boolean");
  assert.equal(boolFact.value, true);
});

test("Verification lifecycle covers unverified, confirmed, disputed, and outdated", () => {
  const statuses: VerificationStatus[] = [
    "unverified",
    "confirmed",
    "disputed",
    "outdated",
  ];

  assert.equal(statuses.length, 4);
  assert.ok(statuses.includes("unverified"));
  assert.ok(statuses.includes("confirmed"));
  assert.ok(statuses.includes("disputed"));
  assert.ok(statuses.includes("outdated"));
});

test("Sensitivity classification covers ordinary, sensitive, and highly_sensitive", () => {
  const sensitivities: Sensitivity[] = [
    "ordinary",
    "sensitive",
    "highly_sensitive",
  ];

  assert.equal(sensitivities.length, 3);
  assert.ok(sensitivities.includes("ordinary"));
  assert.ok(sensitivities.includes("sensitive"));
  assert.ok(sensitivities.includes("highly_sensitive"));
});

test("ProfileFactDetail includes sources and audit trail history", () => {
  const source: FactSource = {
    id: "s-1",
    personaId: "p-1",
    factId: "f-1",
    sourceType: "uploaded_document",
    title: "KTP",
    description: "Kartu Tanda Penduduk Republik Indonesia",
    observedAt: "2024-01-01",
    createdAt: "2024-01-01T00:00:00Z",
  };

  const history: FactHistoryEntry = {
    id: "h-1",
    personaId: "p-1",
    factId: "f-1",
    previousValueJson: '"Staff"',
    newValueJson: '"Manager"',
    previousStatus: "unverified",
    newStatus: "confirmed",
    changeReason: "Promosi jabatan tahun 2024",
    sourceId: "s-1",
    changedAt: "2024-06-01T00:00:00Z",
  };

  const detail: ProfileFactDetail = {
    id: "f-1",
    personaId: "p-1",
    category: "Pekerjaan",
    factKey: "job_title",
    attribute: "Jabatan",
    dataType: "string",
    value: "Manager",
    normalizedValue: "manager",
    verificationStatus: "confirmed",
    sensitivity: "ordinary",
    isValid: true,
    hasConflict: false,
    sourcesCount: 1,
    createdAt: "2024-01-01T00:00:00Z",
    updatedAt: "2024-06-01T00:00:00Z",
    sources: [source],
    history: [history],
  };

  assert.equal(detail.sources.length, 1);
  assert.equal(detail.sources[0].sourceType, "uploaded_document");
  assert.equal(detail.history.length, 1);
  assert.equal(detail.history[0].changeReason, "Promosi jabatan tahun 2024");
});

test("ConflictWarning accurately surfaces contradictory facts", () => {
  const warning: ConflictWarning = {
    factKey: "status_pekerjaan",
    category: "Pekerjaan",
    existingFactId: "f-1",
    existingDisplayName: "Status Pekerjaan",
    existingValue: "Karyawan Swasta",
    existingStatus: "confirmed",
    conflictingFactId: "f-2",
    conflictingDisplayName: "Status Pekerjaan",
    conflictingValue: "Wirausaha",
    conflictingStatus: "confirmed",
    message: "Konflik fakta terdeteksi: Karyawan Swasta bertentangan dengan Wirausaha",
  };

  assert.equal(warning.factKey, "status_pekerjaan");
  assert.notEqual(warning.existingValue, warning.conflictingValue);
  assert.ok(warning.message.includes("Konflik fakta terdeteksi"));
});

test("AI fact eligibility rule: only confirmed, non-disputed, valid facts eligible", () => {
  const facts: ProfileFact[] = [
    {
      id: "1",
      personaId: "p1",
      category: "Demografi",
      factKey: "age",
      attribute: "Usia",
      dataType: "number",
      value: 28,
      normalizedValue: "28",
      verificationStatus: "confirmed",
      sensitivity: "ordinary",
      isValid: true,
      hasConflict: false,
      sourcesCount: 1,
      createdAt: "2024-01-01T00:00:00Z",
      updatedAt: "2024-01-01T00:00:00Z",
    },
    {
      id: "2",
      personaId: "p1",
      category: "Pekerjaan",
      factKey: "job",
      attribute: "Pekerjaan",
      dataType: "string",
      value: "Dokter",
      normalizedValue: "dokter",
      verificationStatus: "unverified", // Unverified -> Ineligible
      sensitivity: "ordinary",
      isValid: true,
      hasConflict: false,
      sourcesCount: 1,
      createdAt: "2024-01-01T00:00:00Z",
      updatedAt: "2024-01-01T00:00:00Z",
    },
    {
      id: "3",
      personaId: "p1",
      category: "Kesehatan",
      factKey: "allergy",
      attribute: "Alergi",
      dataType: "string",
      value: "Debu",
      normalizedValue: "debu",
      verificationStatus: "disputed", // Disputed -> Ineligible
      sensitivity: "ordinary",
      isValid: true,
      hasConflict: false,
      sourcesCount: 1,
      createdAt: "2024-01-01T00:00:00Z",
      updatedAt: "2024-01-01T00:00:00Z",
    },
    {
      id: "4",
      personaId: "p1",
      category: "Finansial",
      factKey: "salary",
      attribute: "Gaji",
      dataType: "number",
      value: 10000000,
      normalizedValue: "10000000",
      verificationStatus: "confirmed",
      sensitivity: "sensitive", // Sensitive -> Requires consent
      isValid: true,
      hasConflict: false,
      sourcesCount: 1,
      createdAt: "2024-01-01T00:00:00Z",
      updatedAt: "2024-01-01T00:00:00Z",
    },
  ];

  function filterEligible(items: ProfileFact[], consentForSensitive: boolean) {
    return items.filter((f) => {
      if (f.verificationStatus !== "confirmed") return false;
      if (!f.isValid) return false;
      if (f.hasConflict) return false;
      if (f.sensitivity !== "ordinary" && !consentForSensitive) return false;
      return true;
    });
  }

  // Without sensitive consent: only #1 (Usia)
  const noConsent = filterEligible(facts, false);
  assert.equal(noConsent.length, 1);
  assert.equal(noConsent[0].attribute, "Usia");

  // With sensitive consent: #1 and #4
  const withConsent = filterEligible(facts, true);
  assert.equal(withConsent.length, 2);
  assert.ok(withConsent.some((f) => f.attribute === "Usia"));
  assert.ok(withConsent.some((f) => f.attribute === "Gaji"));
});
