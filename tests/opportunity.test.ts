import { describe, it } from 'node:test';
import assert from 'node:assert';
import type {
  CriterionInput,
  CriterionOperator,
  CriterionType,
  OpportunityCriterion,
  OpportunityDetail,
  OpportunityProvenance,
  OpportunityStatus,
  SaveOpportunityWithCriteriaInput,
  SurveyOpportunity,
  UpdateCriteriaInput,
  UpdateOpportunityInput,
} from '../src/types/opportunity';

describe('SC-050: Opportunity Capture and Criteria Extraction Contracts', () => {
  it('SurveyOpportunity contract accurately represents captured opportunity with provenance and capture timestamp', () => {
    const opp: SurveyOpportunity = {
      id: 'opp-uuid-001',
      title: 'Survei Kepuasan Konsumen Kopi Kekinian',
      platformLabel: 'YouGov',
      topic: 'Minuman & Konsumsi',
      description: 'Survei preferensi konsumsi kopi',
      rawText: 'Survei Kepuasan Konsumen Kopi Kekinian\nPlatform: YouGov\nKriteria: Pria dan Wanita usia 20-35 tahun',
      provenance: 'manual_entry',
      status: 'confirmed',
      sourceCaptureId: null,
      sourceOcrId: null,
      capturedAt: '2026-11-24T00:00:00Z',
      createdAt: '2026-11-24T00:00:00Z',
      updatedAt: '2026-11-24T00:00:00Z',
    };

    assert.strictEqual(opp.id, 'opp-uuid-001');
    assert.strictEqual(opp.title, 'Survei Kepuasan Konsumen Kopi Kekinian');
    assert.strictEqual(opp.platformLabel, 'YouGov');
    assert.strictEqual(opp.provenance, 'manual_entry');
    assert.strictEqual(opp.status, 'confirmed');
    assert.ok(opp.rawText && opp.rawText.includes('YouGov'));
  });

  it('OpportunityCriterion contract preserves exact rawText for traceability and structured value', () => {
    const criterion: OpportunityCriterion = {
      id: 'crit-uuid-001',
      opportunityId: 'opp-uuid-001',
      criterionText: 'Usia 20 - 35 tahun',
      rawText: 'Pria dan Wanita usia 20-35 tahun',
      criterionType: 'age',
      normalizedAttribute: 'age',
      expectedValueJson: { min: 20, max: 35 },
      operator: 'range',
      criterionOrder: 0,
    };

    assert.strictEqual(criterion.id, 'crit-uuid-001');
    assert.strictEqual(criterion.criterionType, 'age');
    assert.strictEqual(criterion.operator, 'range');
    assert.strictEqual(criterion.criterionOrder, 0);
    assert.strictEqual(criterion.rawText, 'Pria dan Wanita usia 20-35 tahun');
    assert.deepStrictEqual(criterion.expectedValueJson, { min: 20, max: 35 });
  });

  it('CriterionOperator accurately supports equality, ranges, bounds, exclusions and sets', () => {
    const operators: CriterionOperator[] = [
      'equals',
      'range',
      'greater_than_or_equal',
      'less_than_or_equal',
      'in',
      'contains',
      'not_equals',
    ];

    assert.strictEqual(operators.length, 7);
    assert.ok(operators.includes('not_equals'));
    assert.ok(operators.includes('range'));
    assert.ok(operators.includes('contains'));
  });

  it('CriterionType supports all standard survey screening categories', () => {
    const types: CriterionType[] = [
      'age',
      'gender',
      'location',
      'occupation',
      'product_usage',
      'education',
      'income',
      'marital_status',
      'general',
    ];

    assert.strictEqual(types.length, 9);
    assert.ok(types.includes('age'));
    assert.ok(types.includes('location'));
    assert.ok(types.includes('occupation'));
    assert.ok(types.includes('product_usage'));
  });

  it('OpportunityDetail contract encapsulates opportunity entity along with ordered criteria list', () => {
    const detail: OpportunityDetail = {
      opportunity: {
        id: 'opp-test-01',
        title: 'Riset Gaya Hidup',
        platformLabel: 'Milieu',
        topic: 'Gaya Hidup',
        description: null,
        rawText: 'Riset Gaya Hidup...',
        provenance: 'ocr_detected',
        status: 'criteria_extracted',
        sourceCaptureId: 'cap-123',
        sourceOcrId: 'ocr-456',
        capturedAt: '2026-11-24T00:00:00Z',
        createdAt: '2026-11-24T00:00:00Z',
        updatedAt: '2026-11-24T00:00:00Z',
      },
      criteria: [
        {
          id: 'c-1',
          opportunityId: 'opp-test-01',
          criterionText: 'Domisili: Surabaya',
          rawText: 'Domisili di Surabaya',
          criterionType: 'location',
          normalizedAttribute: 'location',
          expectedValueJson: { locations: ['Surabaya'] },
          operator: 'in',
          criterionOrder: 0,
        },
        {
          id: 'c-2',
          opportunityId: 'opp-test-01',
          criterionText: 'Pekerjaan Karyawan Swasta',
          rawText: 'Pekerjaan: Karyawan swasta',
          criterionType: 'occupation',
          normalizedAttribute: 'occupation',
          expectedValueJson: { allowed: ['karyawan swasta'] },
          operator: 'in',
          criterionOrder: 1,
        },
      ],
    };

    assert.strictEqual(detail.opportunity.provenance, 'ocr_detected');
    assert.strictEqual(detail.opportunity.sourceCaptureId, 'cap-123');
    assert.strictEqual(detail.criteria.length, 2);
    assert.strictEqual(detail.criteria[0].criterionOrder, 0);
    assert.strictEqual(detail.criteria[1].criterionOrder, 1);
  });

  it('SaveOpportunityWithCriteriaInput input DTO contract validates payload structure', () => {
    const payload: SaveOpportunityWithCriteriaInput = {
      opportunity: {
        title: 'Survei Smartphone 2026',
        platformLabel: 'Toluna',
        topic: 'Teknologi',
        description: 'Pengguna Android',
        rawText: 'Survei Smartphone Toluna',
        provenance: 'manual_entry',
      },
      criteria: [
        {
          criterionText: 'Pengguna Android',
          rawText: 'Memiliki smartphone Android',
          criterionType: 'product_usage',
          operator: 'contains',
          criterionOrder: 0,
        },
      ],
    };

    assert.strictEqual(payload.opportunity.title, 'Survei Smartphone 2026');
    assert.strictEqual(payload.opportunity.platformLabel, 'Toluna');
    assert.strictEqual(payload.criteria.length, 1);
    assert.strictEqual(payload.criteria[0].criterionType, 'product_usage');
  });
});
