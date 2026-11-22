use std::path::Path;
use survey_copilot_lib::db::Database;
use survey_copilot_lib::domain::fact::CreateFactInput;
use survey_copilot_lib::domain::persona::{CreatePersonaInput, UpdatePersonaInput};
use survey_copilot_lib::services::{FactService, PersonaService};

fn setup_memory_db() -> Database {
    Database::initialize(Path::new(":memory:")).expect("memory database should initialize")
}

#[test]
fn test_persona_create_first_is_active_second_is_inactive() {
    let db = setup_memory_db();

    let p1 = PersonaService::create(
        &db,
        CreatePersonaInput::with_description("Rudi Hermanto", Some("Profil Utama".into())),
    )
    .expect("p1 creation");
    assert!(p1.is_active);
    assert_eq!(p1.display_name, "Rudi Hermanto");
    assert_eq!(p1.status, "active");

    let p2 = PersonaService::create(
        &db,
        CreatePersonaInput::with_description("Dewi Lestari", Some("Profil Sekunder".into())),
    )
    .expect("p2 creation");
    assert!(!p2.is_active);
    assert_eq!(p2.display_name, "Dewi Lestari");
    assert_eq!(p2.status, "active");

    let list = PersonaService::list(&db, false).expect("list");
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].id, p1.id);
    assert!(list[0].is_active);
    assert_eq!(list[1].id, p2.id);
    assert!(!list[1].is_active);
}

#[test]
fn test_persona_update_name_and_description() {
    let db = setup_memory_db();
    let p = PersonaService::create(&db, CreatePersonaInput::new("Bambang")).expect("create");

    let updated = PersonaService::update(
        &db,
        &p.id,
        UpdatePersonaInput::new("Bambang Pamungkas", Some("Atlet / Pelatih".into())),
    )
    .expect("update");

    assert_eq!(updated.display_name, "Bambang Pamungkas");
    assert_eq!(updated.description, Some("Atlet / Pelatih".into()));

    let detail = PersonaService::get_detail(&db, &p.id).expect("get detail");
    assert_eq!(detail.display_name, "Bambang Pamungkas");
    assert_eq!(detail.description, Some("Atlet / Pelatih".into()));
}

#[test]
fn test_cannot_update_archived_persona() {
    let db = setup_memory_db();
    let p = PersonaService::create(&db, CreatePersonaInput::new("Eko")).expect("create");
    PersonaService::archive(&db, &p.id).expect("archive");

    let err =
        PersonaService::update(&db, &p.id, UpdatePersonaInput::new("Eko Baru", None)).unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");
}

#[test]
fn test_switch_active_persona_atomic() {
    let db = setup_memory_db();
    let p1 = PersonaService::create(&db, CreatePersonaInput::new("Persona 1")).expect("create p1");
    let p2 = PersonaService::create(&db, CreatePersonaInput::new("Persona 2")).expect("create p2");
    let p3 = PersonaService::create(&db, CreatePersonaInput::new("Persona 3")).expect("create p3");

    assert!(p1.is_active);
    assert!(!p2.is_active);
    assert!(!p3.is_active);

    // Switch to p3
    let switched = PersonaService::set_active(&db, &p3.id).expect("activate p3");
    assert!(switched.is_active);

    let list = PersonaService::list(&db, false).expect("list");
    let active_list: Vec<_> = list.iter().filter(|p| p.is_active).collect();
    assert_eq!(active_list.len(), 1);
    assert_eq!(active_list[0].id, p3.id);

    // Switch to p2
    let switched2 = PersonaService::set_active(&db, &p2.id).expect("activate p2");
    assert!(switched2.is_active);

    let list2 = PersonaService::list(&db, false).expect("list");
    let active_list2: Vec<_> = list2.iter().filter(|p| p.is_active).collect();
    assert_eq!(active_list2.len(), 1);
    assert_eq!(active_list2[0].id, p2.id);
}

#[test]
fn test_database_constraint_prevents_two_active_personas() {
    let db = setup_memory_db();
    let _p1 = PersonaService::create(&db, CreatePersonaInput::new("Orang A")).expect("p1");
    let p2 = PersonaService::create(&db, CreatePersonaInput::new("Orang B")).expect("p2");

    // Both exist, p1 is_active = 1, p2 is_active = 0.
    // Raw SQL direct execution trying to set p2 is_active = 1 without resetting p1
    let connection = db.connection.lock().expect("lock");
    let res = connection.execute("UPDATE personas SET is_active = 1 WHERE id = ?1", [&p2.id]);

    // Must fail due to UNIQUE constraint failed: personas.is_active (idx_one_active_persona)
    assert!(res.is_err());
    let err_str = res.unwrap_err().to_string();
    assert!(
        err_str.contains("UNIQUE constraint failed") || err_str.contains("idx_one_active_persona")
    );
}

#[test]
fn test_cannot_activate_archived_persona() {
    let db = setup_memory_db();
    let p1 = PersonaService::create(&db, CreatePersonaInput::new("P1")).expect("p1");
    let p2 = PersonaService::create(&db, CreatePersonaInput::new("P2")).expect("p2");

    PersonaService::archive(&db, &p2.id).expect("archive p2");

    let err = PersonaService::set_active(&db, &p2.id).unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");

    // p1 should still remain active
    let p1_after = PersonaService::get_detail(&db, &p1.id).expect("get p1");
    assert!(p1_after.is_active);
}

#[test]
fn test_archive_active_persona_safely_transfers_active_status() {
    let db = setup_memory_db();
    let p1 = PersonaService::create(&db, CreatePersonaInput::new("P1")).expect("p1");
    let p2 = PersonaService::create(&db, CreatePersonaInput::new("P2")).expect("p2");

    assert!(p1.is_active);

    // Archiving p1 (the active one) must transfer active to p2
    let archived_p1 = PersonaService::archive(&db, &p1.id).expect("archive p1");
    assert_eq!(archived_p1.status, "archived");
    assert!(!archived_p1.is_active);

    let p2_after = PersonaService::get_detail(&db, &p2.id).expect("p2 after");
    assert!(p2_after.is_active);
    assert_eq!(p2_after.status, "active");
}

#[test]
fn test_archive_only_persona_leaves_zero_active() {
    let db = setup_memory_db();
    let p1 = PersonaService::create(&db, CreatePersonaInput::new("Sole Persona")).expect("p1");
    assert!(p1.is_active);

    let archived = PersonaService::archive(&db, &p1.id).expect("archive p1");
    assert_eq!(archived.status, "archived");
    assert!(!archived.is_active);

    let active_list = PersonaService::list(&db, false).expect("list active");
    assert_eq!(active_list.len(), 0);

    let all_list = PersonaService::list(&db, true).expect("list all");
    assert_eq!(all_list.len(), 1);
    assert_eq!(all_list[0].status, "archived");
}

#[test]
fn test_restore_archived_persona() {
    let db = setup_memory_db();
    let p1 = PersonaService::create(&db, CreatePersonaInput::new("P1")).expect("p1");
    let p2 = PersonaService::create(&db, CreatePersonaInput::new("P2")).expect("p2");

    PersonaService::archive(&db, &p2.id).expect("archive p2");

    let restored = PersonaService::restore(&db, &p2.id).expect("restore p2");
    assert_eq!(restored.status, "active");
    assert!(restored.archived_at.is_none());
    assert!(!restored.is_active); // p1 is already active

    // Now test restoring when NO persona is active
    PersonaService::archive(&db, &p1.id).expect("archive p1");
    PersonaService::archive(&db, &p2.id).expect("archive p2");

    let restored_when_empty = PersonaService::restore(&db, &p2.id).expect("restore p2 solo");
    assert!(restored_when_empty.is_active); // Automatically activated because no active persona existed
}

#[test]
fn test_dependency_summary_counts_all_associated_records() {
    let db = setup_memory_db();
    let p = PersonaService::create(&db, CreatePersonaInput::new("Sari")).expect("create");

    // Add facts
    let fact1 = FactService::create(
        &db,
        CreateFactInput::new(
            &p.id,
            "Demografi",
            "Kota Domisili",
            "Surabaya",
            "KTP Jawa Timur",
        ),
    )
    .expect("create fact 1");

    let _fact2 = FactService::create(
        &db,
        CreateFactInput::new(
            &p.id,
            "Pekerjaan",
            "Profesi",
            "Software Engineer",
            "Slip Gaji",
        ),
    )
    .expect("create fact 2");

    let summary = PersonaService::get_dependency_summary(&db, &p.id).expect("summary");
    assert_eq!(summary.persona_id, p.id);
    assert_eq!(summary.display_name, "Sari");
    assert_eq!(summary.facts_count, 2);
    assert_eq!(summary.sources_count, 2);
    assert!(summary.total_associated_records >= 4);

    let _ = fact1;
}

#[test]
fn test_delete_persona_cascades_and_cleans_records() {
    let db = setup_memory_db();
    let p1 = PersonaService::create(&db, CreatePersonaInput::new("P1")).expect("p1");
    let p2 = PersonaService::create(&db, CreatePersonaInput::new("P2")).expect("p2");

    // Add facts to p2
    FactService::create(
        &db,
        CreateFactInput::new(&p2.id, "Umum", "Umur", "28", "Input manual"),
    )
    .expect("fact");

    let summary_before = PersonaService::get_dependency_summary(&db, &p2.id).expect("summary");
    assert_eq!(summary_before.facts_count, 1);
    assert_eq!(summary_before.sources_count, 1);

    // Delete p2
    let deleted_summary = PersonaService::delete(&db, &p2.id).expect("delete p2");
    assert_eq!(deleted_summary.persona_id, p2.id);
    assert_eq!(deleted_summary.facts_count, 1);

    // Verify p2 no longer exists
    assert!(PersonaService::get_detail(&db, &p2.id).is_err());

    // Verify facts of p2 are completely removed
    let facts_p2 = FactService::list(&db, &p2.id, false).expect("list facts");
    assert_eq!(facts_p2.len(), 0);

    // Verify p1 still intact
    let p1_after = PersonaService::get_detail(&db, &p1.id).expect("p1 after");
    assert_eq!(p1_after.display_name, "P1");
}

#[test]
fn test_delete_active_persona_safely_transfers_active_status() {
    let db = setup_memory_db();
    let p1 = PersonaService::create(&db, CreatePersonaInput::new("Active P1")).expect("p1");
    let p2 = PersonaService::create(&db, CreatePersonaInput::new("Candidate P2")).expect("p2");

    assert!(p1.is_active);
    assert!(!p2.is_active);

    // Delete p1 (active)
    PersonaService::delete(&db, &p1.id).expect("delete active p1");

    // p2 must now be active
    let p2_after = PersonaService::get_detail(&db, &p2.id).expect("p2 detail");
    assert!(p2_after.is_active);
}

#[test]
fn test_strict_cross_persona_data_isolation() {
    let db = setup_memory_db();
    let persona_a = PersonaService::create(&db, CreatePersonaInput::new("Persona A")).expect("pA");
    let persona_b = PersonaService::create(&db, CreatePersonaInput::new("Persona B")).expect("pB");

    // Persona A adds facts
    let fact_a = FactService::create(
        &db,
        CreateFactInput::new(
            &persona_a.id,
            "Finansial",
            "Penghasilan Bulanan",
            "15.000.000 IDR",
            "SPT Pajak",
        ),
    )
    .expect("fact A");

    // 1. Persona B queries facts -> must NOT see Persona A's facts
    let facts_b = FactService::list(&db, &persona_b.id, false).expect("facts B");
    assert_eq!(facts_b.len(), 0);

    // 2. Persona B tries to update Persona A's fact status -> MUST fail with NOT_FOUND
    let update_res = FactService::set_status(&db, &persona_b.id, &fact_a.id, "confirmed");
    assert!(update_res.is_err());
    assert_eq!(update_res.unwrap_err().code, "NOT_FOUND");

    // 3. Persona B tries to delete Persona A's fact -> MUST fail with NOT_FOUND
    let delete_res = FactService::delete(&db, &persona_b.id, &fact_a.id);
    assert!(delete_res.is_err());
    assert_eq!(delete_res.unwrap_err().code, "NOT_FOUND");

    // 4. Persona A's fact must still be intact
    let facts_a = FactService::list(&db, &persona_a.id, false).expect("facts A");
    assert_eq!(facts_a.len(), 1);
    assert_eq!(facts_a[0].id, fact_a.id);
    assert_eq!(facts_a[0].attribute, "Penghasilan Bulanan");
}

#[test]
fn test_persona_persists_across_database_reopen() {
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join(format!("survey_test_{}.sqlite3", uuid::Uuid::new_v4()));

    // Scope 1: Create personas and set active
    let p2_id = {
        let db = Database::initialize(&db_path).expect("init db");
        let _p1 = PersonaService::create(&db, CreatePersonaInput::new("Toko A")).expect("p1");
        let p2 = PersonaService::create(&db, CreatePersonaInput::new("Toko B")).expect("p2");
        PersonaService::set_active(&db, &p2.id).expect("set active p2");
        p2.id
    };

    // Scope 2: Reopen from file
    {
        let db_reopened = Database::initialize(&db_path).expect("reopen db");
        let personas = PersonaService::list(&db_reopened, false).expect("list reopened");
        assert_eq!(personas.len(), 2);

        let active = personas
            .iter()
            .find(|p| p.is_active)
            .expect("active persona");
        assert_eq!(active.id, p2_id);
        assert_eq!(active.display_name, "Toko B");
    }

    // Cleanup
    let _ = std::fs::remove_file(db_path);
}
