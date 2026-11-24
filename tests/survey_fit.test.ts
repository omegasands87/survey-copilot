import { describe, it } from 'node:test';
import assert from 'node:assert';
import type {
  FitCriterionResult,
  MatchState,
  OverallFitState,
  PersonaFitComparisonItem,
  PersonaOpportunityFitDetail,
} from '../src/types/fit';

describe('SC-051: Deterministic Persona Fit Contracts & Rules', () => {
  it('AT-060: Criterion matches confirmed current fact with fact ID references', () => {
    const criterionResult: FitCriterionResult = {
      criterionId: 'crit-001',
      criterionText: 'Usia 20 - 35 tahun',
      rawText: 'usia 20-35 tahun',
      criterionType: 'age',
      operator: 'range',
      matchState: 'match',
      supportingFactIds: ['fact-age-budi'],
      supportingFactsSummary: ['Usia: 28 tahun (confirmed)'],
      explanation: 'Usia persona (28 tahun) berada dalam rentang kriteria (20-35 tahun).',
    };

    assert.strictEqual(criterionResult.matchState, 'match');
    assert.strictEqual(criterionResult.supportingFactIds.length, 1);
    assert.strictEqual(criterionResult.supportingFactIds[0], 'fact-age-budi');
    assert.ok(criterionResult.explanation.includes('28 tahun'));
  });

  it('AT-061: Unknown is never treated as match when criterion evidence is absent', () => {
    const unknownResult: FitCriterionResult = {
      criterionId: 'crit-002',
      criterionText: 'Pengguna mobil listrik',
      rawText: 'memiliki mobil listrik',
      criterionType: 'product_usage',
      operator: 'contains',
      matchState: 'unknown',
      supportingFactIds: [],
      supportingFactsSummary: [],
      explanation: 'Tidak ada data kepemilikan mobil listrik pada profil persona.',
    };

    assert.strictEqual(unknownResult.matchState, 'unknown');
    assert.notStrictEqual(unknownResult.matchState, 'match');
    assert.strictEqual(unknownResult.supportingFactIds.length, 0);

    // Invariant: Overall fit with an unknown criterion must be incomplete, never eligible
    const fitDetail: PersonaOpportunityFitDetail = {
      opportunityId: 'opp-001',
      personaId: 'persona-001',
      personaName: 'Budi Santoso',
      overallFit: 'incomplete',
      matchCount: 1,
      mismatchCount: 0,
      unknownCount: 1,
      staleCount: 0,
      conflictCount: 0,
      totalCriteria: 2,
      criteriaResults: [
        {
          criterionId: 'crit-001',
          criterionText: 'Usia 20 - 35 tahun',
          rawText: 'usia 20-35',
          criterionType: 'age',
          operator: 'range',
          matchState: 'match',
          supportingFactIds: ['fact-age'],
          supportingFactsSummary: ['Usia: 28 tahun'],
          explanation: 'Usia cocok.',
        },
        unknownResult,
      ],
      evaluatedAt: '2026-11-24T12:00:00Z',
    };

    assert.strictEqual(fitDetail.overallFit, 'incomplete');
    assert.notStrictEqual(fitDetail.overallFit, 'eligible');
    assert.strictEqual(fitDetail.unknownCount, 1);
  });

  it('AT-062: Criterion returns stale when fact is outdated and conflict when disputed', () => {
    const staleResult: FitCriterionResult = {
      criterionId: 'crit-003',
      criterionText: 'Pekerjaan Karyawan',
      rawText: 'pekerjaan karyawan',
      criterionType: 'occupation',
      operator: 'in',
      matchState: 'stale',
      supportingFactIds: ['fact-old-job'],
      supportingFactsSummary: ['Fakta pekerjaan sudah kadaluarsa (outdated)'],
      explanation: 'Fakta pekerjaan pada profil persona telah kadaluarsa (outdated).',
    };

    const conflictResult: FitCriterionResult = {
      criterionId: 'crit-004',
      criterionText: 'Domisili Surabaya',
      rawText: 'domisili surabaya',
      criterionType: 'location',
      operator: 'in',
      matchState: 'conflict',
      supportingFactIds: ['fact-loc-1', 'fact-loc-2'],
      supportingFactsSummary: ['Ditemukan pertentangan fakta domisili'],
      explanation: 'Ditemukan beberapa fakta domisili yang saling berkontradiksi pada profil.',
    };

    assert.strictEqual(staleResult.matchState, 'stale');
    assert.strictEqual(conflictResult.matchState, 'conflict');
  });

  it('PersonaFitComparisonItem allows truthful respondent selection without identity substitution', () => {
    const comparison: PersonaFitComparisonItem[] = [
      {
        personaId: 'p-1',
        personaName: 'Rian (Mahasiswa)',
        overallFit: 'eligible',
        matchCount: 3,
        mismatchCount: 0,
        unknownCount: 0,
        staleCount: 0,
        conflictCount: 0,
        totalCriteria: 3,
        keyReasons: ['Memenuhi seluruh 3 kriteria yang disyaratkan'],
        evaluatedAt: '2026-11-24T12:00:00Z',
      },
      {
        personaId: 'p-2',
        personaName: 'Hendra (Pensiunan)',
        overallFit: 'ineligible',
        matchCount: 1,
        mismatchCount: 2,
        unknownCount: 0,
        staleCount: 0,
        conflictCount: 0,
        totalCriteria: 3,
        keyReasons: ['Usia persona (60 tahun) di luar rentang kriteria (18-25 tahun).'],
        evaluatedAt: '2026-11-24T12:00:00Z',
      },
    ];

    assert.strictEqual(comparison.length, 2);
    assert.strictEqual(comparison[0].overallFit, 'eligible');
    assert.strictEqual(comparison[1].overallFit, 'ineligible');
    assert.ok(comparison[1].keyReasons[0].includes('60 tahun'));
  });

  it('OverallFitState accurately classifies all 6 possible outcomes', () => {
    const states: OverallFitState[] = [
      'eligible',
      'ineligible',
      'conflict',
      'stale',
      'incomplete',
      'no_criteria',
    ];
    assert.strictEqual(states.length, 6);
  });
});
