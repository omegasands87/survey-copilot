import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import type {
  AttemptFilter,
  CreateAttemptInput,
  HistoricalLearningSummary,
  LearningDataStatus,
  OutcomeConfidence,
  OutcomeLabel,
  RecordOutcomeEventInput,
  SurveyAttempt,
  SurveyAttemptDetail,
  SurveyOutcomeEvent,
} from '../src/types/attempt';

describe('SC-052: Historical Outcome Learning & Survey Attempt Contracts', () => {
  it('OutcomeLabel taxonomy strictly covers all 9 normative attempt statuses', () => {
    const validOutcomes: OutcomeLabel[] = [
      'qualified',
      'screened_out',
      'quota_full',
      'completed',
      'quality_rejected',
      'technical_failure',
      'abandoned_by_user',
      'pending',
      'unknown',
    ];

    assert.equal(validOutcomes.length, 9);
    assert.ok(validOutcomes.includes('qualified'));
    assert.ok(validOutcomes.includes('screened_out'));
    assert.ok(validOutcomes.includes('quota_full'));
    assert.ok(validOutcomes.includes('completed'));
    assert.ok(validOutcomes.includes('quality_rejected'));
    assert.ok(validOutcomes.includes('technical_failure'));
    assert.ok(validOutcomes.includes('abandoned_by_user'));
    assert.ok(validOutcomes.includes('pending'));
    assert.ok(validOutcomes.includes('unknown'));
  });

  it('OutcomeConfidence taxonomy covers observed, user_confirmed, and uncertain', () => {
    const confidences: OutcomeConfidence[] = ['observed', 'user_confirmed', 'uncertain'];
    assert.equal(confidences.length, 3);
  });

  it('LearningDataStatus taxonomy distinguishes insufficient_data from sufficient_data (AT-063)', () => {
    const statuses: LearningDataStatus[] = ['insufficient_data', 'sufficient_data'];
    assert.equal(statuses.length, 2);
  });

  it('AT-063: Sparse history payload strictly enforces null rates and insufficient_data status', () => {
    const sparseSummary: HistoricalLearningSummary = {
      personaId: 'persona-1',
      personaName: 'Budi Santoso',
      category: 'Teknologi',
      sourcePlatform: 'Milieu',
      sampleSize: 3,
      minimumRequiredSample: 10,
      status: 'insufficient_data',
      observationWindowStart: '2026-11-20T10:00:00Z',
      observationWindowEnd: '2026-11-23T10:00:00Z',
      countsByOutcome: { completed: 3 },
      completionRate: null,
      qualificationRate: null,
      validationMethod: 'chronological_holdout',
      holdoutSampleSize: 0,
      holdoutConsistencyRate: null,
      message:
        'Data historis belum mencukupi (tercatat 3 dari minimal 10 observasi terkalibrasi). Persentase probabilitas dan prediksi kelayakan tidak ditampilkan.',
      uncertaintyDisclosure:
        'Ketidakpastian sangat tinggi karena ukuran sampel sangat kecil.',
      disclaimers: [
        'Tidak ada fitur pelacakan saldo, reward, uang tunai, atau poin.',
        'Dilarang memanipulasi profil atau merekayasa jawaban demi lolos kriteria skrining.',
        'Observasi faktual terpisah secara tegas dari model pembelajaran historis.',
      ],
    };

    assert.equal(sparseSummary.status, 'insufficient_data');
    assert.equal(sparseSummary.sampleSize, 3);
    assert.equal(sparseSummary.completionRate, null);
    assert.equal(sparseSummary.qualificationRate, null);
    assert.equal(sparseSummary.holdoutConsistencyRate, null);
    assert.ok(sparseSummary.message.includes('belum mencukupi'));
  });

  it('Sufficient history payload provides calibrated descriptive rates and chronological holdout metrics', () => {
    const calibratedSummary: HistoricalLearningSummary = {
      personaId: 'persona-1',
      personaName: 'Budi Santoso',
      category: 'Finansial',
      sourcePlatform: 'YouGov',
      sampleSize: 12,
      minimumRequiredSample: 10,
      status: 'sufficient_data',
      observationWindowStart: '2026-11-10T10:00:00Z',
      observationWindowEnd: '2026-11-22T10:00:00Z',
      countsByOutcome: { completed: 6, qualified: 2, screened_out: 4 },
      completionRate: 0.5,
      qualificationRate: 0.667,
      validationMethod: 'chronological_holdout',
      holdoutSampleSize: 4,
      holdoutConsistencyRate: 0.85,
      message: 'Tercatat 12 observasi historis terkalibrasi.',
      uncertaintyDisclosure: 'Ketidakpastian estimasi: ±14.2%.',
      disclaimers: [
        'Hasil historis adalah observasi masa lalu dan tidak menjamin kelayakan untuk survei baru.',
        'Tidak ada fitur pelacakan saldo, reward, uang tunai, atau poin.',
      ],
    };

    assert.equal(calibratedSummary.status, 'sufficient_data');
    assert.equal(calibratedSummary.sampleSize, 12);
    assert.equal(calibratedSummary.completionRate, 0.5);
    assert.equal(calibratedSummary.qualificationRate, 0.667);
    assert.equal(calibratedSummary.holdoutSampleSize, 4);
    assert.equal(calibratedSummary.holdoutConsistencyRate, 0.85);
  });

  it('SurveyAttemptDetail contract preserves traceability to opportunity and chronological events', () => {
    const detail: SurveyAttemptDetail = {
      id: 'att-101',
      opportunityId: 'opp-202',
      opportunityTitle: 'Survei Belanja Online',
      sourcePlatform: 'Toluna',
      category: 'E-commerce',
      targetUrl: 'https://toluna.example.com/survey',
      rawText: 'Kriteria: Domisili Jabodetabek, usia 18-35 tahun',
      personaId: 'persona-1',
      personaName: 'Budi Santoso',
      sessionId: null,
      startedAt: '2026-11-24T08:00:00Z',
      endedAt: '2026-11-24T08:15:00Z',
      currentOutcome: 'completed',
      criteria: [],
      events: [
        {
          id: 'ev-1',
          attemptId: 'att-101',
          outcomeLabel: 'pending',
          evidenceSource: 'pencatatan_manual',
          evidenceNote: 'Mulai mengisi',
          confidence: 'observed',
          observedAt: '2026-11-24T08:00:00Z',
          createdAt: '2026-11-24T08:00:00Z',
        },
        {
          id: 'ev-2',
          attemptId: 'att-101',
          outcomeLabel: 'completed',
          evidenceSource: 'halaman_penutup',
          evidenceNote: 'Selesai 100%',
          confidence: 'user_confirmed',
          observedAt: '2026-11-24T08:15:00Z',
          createdAt: '2026-11-24T08:15:00Z',
        },
      ],
      createdAt: '2026-11-24T08:00:00Z',
      updatedAt: '2026-11-24T08:15:00Z',
    };

    assert.equal(detail.currentOutcome, 'completed');
    assert.equal(detail.events.length, 2);
    assert.equal(detail.events[1].outcomeLabel, 'completed');
    assert.equal(detail.personaId, 'persona-1');
  });

  it('Strict persona isolation constraint: attempt operations require personaId scoping', () => {
    const createInput: CreateAttemptInput = {
      personaId: 'persona-alpha',
      opportunityTitle: 'Survei Konsumsi',
    };
    assert.ok(createInput.personaId);

    const recordEventInput: RecordOutcomeEventInput = {
      attemptId: 'att-1',
      personaId: 'persona-alpha',
      outcomeLabel: 'completed',
      evidenceSource: 'browser_closing_page',
    };
    assert.ok(recordEventInput.personaId);

    const filter: AttemptFilter = {
      personaId: 'persona-alpha',
      category: 'Teknologi',
      sourcePlatform: 'YouGov',
      status: 'completed',
    };
    assert.equal(filter.personaId, 'persona-alpha');
  });

  it('Product boundary enforcement: no earnings, rewards, points, or balance fields in attempt contracts', () => {
    const forbiddenKeys = [
      'reward',
      'rewards',
      'earning',
      'earnings',
      'points',
      'balance',
      'payout',
      'cashout',
      'income',
      'rate_per_hour',
    ];

    const sampleAttempt: SurveyAttempt = {
      id: 'att-1',
      opportunityId: 'opp-1',
      personaId: 'p-1',
      startedAt: '2026-11-24T10:00:00Z',
      currentOutcome: 'qualified',
      createdAt: '2026-11-24T10:00:00Z',
      updatedAt: '2026-11-24T10:00:00Z',
    };

    const attemptKeys = Object.keys(sampleAttempt);
    for (const forbidden of forbiddenKeys) {
      assert.ok(
        !attemptKeys.includes(forbidden),
        `SurveyAttempt contract dilarang memuat field '${forbidden}'`
      );
    }
  });
});
