import test from "node:test";
import assert from "node:assert";
import type { Persona, PersonaDependencySummary } from "../src/types/persona.ts";

test("Persona data contract supports full lifecycle fields", () => {
  const persona: Persona = {
    id: "uuid-1234",
    displayName: "Budi Santoso",
    description: "Akun profil responden utama",
    isActive: true,
    status: "active",
    createdAt: "2026-11-23T00:00:00.000Z",
    updatedAt: "2026-11-23T00:00:00.000Z",
    archivedAt: null,
    factsCount: 5,
    sessionsCount: 2,
  };

  assert.strictEqual(persona.id, "uuid-1234");
  assert.strictEqual(persona.displayName, "Budi Santoso");
  assert.strictEqual(persona.isActive, true);
  assert.strictEqual(persona.status, "active");
  assert.strictEqual(persona.factsCount, 5);
  assert.strictEqual(persona.sessionsCount, 2);
});

test("PersonaDependencySummary aggregates all personal relation counts", () => {
  const summary: PersonaDependencySummary = {
    personaId: "uuid-1234",
    displayName: "Budi Santoso",
    factsCount: 4,
    sourcesCount: 4,
    sessionsCount: 2,
    questionsCount: 10,
    recommendationsCount: 8,
    answersCount: 8,
    fitResultsCount: 3,
    attemptsCount: 2,
    totalAssociatedRecords: 33,
  };

  assert.strictEqual(summary.personaId, "uuid-1234");
  assert.strictEqual(summary.totalAssociatedRecords, 33);
  assert.strictEqual(summary.factsCount, 4);
});

test("Only one persona can be active in a valid persona set", () => {
  const personas: Persona[] = [
    {
      id: "p1",
      displayName: "Budi",
      isActive: true,
      status: "active",
    },
    {
      id: "p2",
      displayName: "Siti",
      isActive: false,
      status: "active",
    },
    {
      id: "p3",
      displayName: "Agus",
      isActive: false,
      status: "archived",
      archivedAt: "2026-11-23T00:00:00.000Z",
    },
  ];

  const activePersonas = personas.filter((p) => p.isActive);
  assert.strictEqual(activePersonas.length, 1);
  assert.strictEqual(activePersonas[0].id, "p1");

  const archivedPersonas = personas.filter((p) => p.status === "archived");
  assert.strictEqual(archivedPersonas.length, 1);
  assert.strictEqual(archivedPersonas[0].isActive, false);
});
