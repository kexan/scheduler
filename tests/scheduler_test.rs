use chrono::{NaiveDate, NaiveTime};
use migration_scheduler::scheduler::{CreateBooking, CreateTimeSlot, Scheduler, UpdateTimeSlot};
use serial_test::serial;
use std::fs;
use std::path::Path;
use uuid::Uuid;

const TEST_DATA_PATH: &str = "data/test_slots.json";

fn clean_storage() {
    let path = Path::new(TEST_DATA_PATH);
    if path.exists() {
        fs::remove_file(path).expect("failed to remove test data file");
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create data directory");
    }
}

async fn create_scheduler() -> Scheduler {
    clean_storage();
    Scheduler::with_path(TEST_DATA_PATH).await.unwrap()
}

fn make_create_slot(date: &str, start: &str, end: &str) -> CreateTimeSlot {
    CreateTimeSlot {
        date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
        start_time: NaiveTime::parse_from_str(start, "%H:%M").unwrap(),
        end_time: NaiveTime::parse_from_str(end, "%H:%M").unwrap(),
    }
}

fn make_create_booking() -> CreateBooking {
    CreateBooking {
        company_name: "TestCorp".to_string(),
        admin_email: "admin@test.com".to_string(),
        company_id: "comp-123".to_string(),
        download_email: "dl@test.com".to_string(),
    }
}

// ── new / slot_count ────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_new_starts_empty() {
    let scheduler = create_scheduler().await;
    assert_eq!(scheduler.slot_count(), 0);
    assert!(scheduler.get_slots_in_range(None, None).is_empty());
}

// ── create_slot ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_create_slot() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    let slot = scheduler.create_slot(ct).await.unwrap();

    assert_eq!(scheduler.slot_count(), 1);
    assert_eq!(slot.date, NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
    assert!(slot.is_available);
    assert!(slot.booking.is_none());
    assert!(slot.yougile_task_id.is_none());
    assert!(!slot.completed);
}

#[tokio::test]
#[serial]
async fn test_create_multiple_slots() {
    let scheduler = create_scheduler().await;

    for (date, start) in &[
        ("2026-06-15", "09:00"),
        ("2026-06-15", "10:00"),
        ("2026-06-16", "09:00"),
    ] {
        let ct = make_create_slot(date, start, "11:00");
        scheduler.create_slot(ct).await.unwrap();
    }

    assert_eq!(scheduler.slot_count(), 3);
}

// ── get_slot ────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_get_slot_by_id() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    let created = scheduler.create_slot(ct).await.unwrap();

    let fetched = scheduler.get_slot(created.id).unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.date, created.date);
    assert_eq!(fetched.start_time, created.start_time);
}

#[tokio::test]
#[serial]
async fn test_get_slot_not_found() {
    let scheduler = create_scheduler().await;
    assert!(scheduler.get_slot(Uuid::new_v4()).is_none());
}

// ── get_slots_in_range ──────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_get_slots_in_range_all() {
    let scheduler = create_scheduler().await;

    for date in &["2026-06-10", "2026-06-15", "2026-06-20"] {
        let ct = make_create_slot(date, "10:00", "11:00");
        scheduler.create_slot(ct).await.unwrap();
    }

    let slots = scheduler.get_slots_in_range(None, None);
    assert_eq!(slots.len(), 3);
}

#[tokio::test]
#[serial]
async fn test_get_slots_in_range_filtered() {
    let scheduler = create_scheduler().await;

    for date in &["2026-06-10", "2026-06-15", "2026-06-20"] {
        let ct = make_create_slot(date, "10:00", "11:00");
        scheduler.create_slot(ct).await.unwrap();
    }

    let from = NaiveDate::from_ymd_opt(2026, 6, 12).unwrap();
    let to = NaiveDate::from_ymd_opt(2026, 6, 18).unwrap();

    let slots = scheduler.get_slots_in_range(Some(from), Some(to));
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].date, NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
}

#[tokio::test]
#[serial]
async fn test_get_slots_in_range_inclusive_bounds() {
    let scheduler = create_scheduler().await;

    for date in &["2026-06-10", "2026-06-15", "2026-06-20"] {
        let ct = make_create_slot(date, "10:00", "11:00");
        scheduler.create_slot(ct).await.unwrap();
    }

    let from = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
    let to = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();

    let slots = scheduler.get_slots_in_range(Some(from), Some(to));
    assert_eq!(slots.len(), 2);
}

#[tokio::test]
#[serial]
async fn test_get_slots_sorted() {
    let scheduler = create_scheduler().await;

    for (date, start, end) in &[
        ("2026-06-15", "14:00", "15:00"),
        ("2026-06-15", "09:00", "11:00"),
        ("2026-06-10", "10:00", "11:00"),
    ] {
        let ct = make_create_slot(date, start, end);
        scheduler.create_slot(ct).await.unwrap();
    }

    let slots = scheduler.get_slots_in_range(None, None);
    assert_eq!(slots.len(), 3);

    assert_eq!(slots[0].date, NaiveDate::from_ymd_opt(2026, 6, 10).unwrap());
    assert_eq!(slots[1].date, NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
    assert_eq!(
        slots[1].start_time,
        NaiveTime::from_hms_opt(9, 0, 0).unwrap()
    );
    assert_eq!(
        slots[2].start_time,
        NaiveTime::from_hms_opt(14, 0, 0).unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_get_slots_in_range_empty_result() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    scheduler.create_slot(ct).await.unwrap();

    let from = NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();
    let slots = scheduler.get_slots_in_range(Some(from), None);
    assert!(slots.is_empty());
}

// ── book_slot ───────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_book_slot_success() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    let slot = scheduler.create_slot(ct).await.unwrap();

    let booking = make_create_booking();
    let booked = scheduler.book_slot(slot.id, booking).await.unwrap();

    assert!(!booked.is_available);
    assert!(booked.booking.is_some());
    let b = booked.booking.as_ref().unwrap();
    assert_eq!(b.company_name, "TestCorp");

    let fetched = scheduler.get_slot(slot.id).unwrap();
    assert!(!fetched.is_available);
}

#[tokio::test]
#[serial]
async fn test_book_slot_already_booked() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    let slot = scheduler.create_slot(ct).await.unwrap();

    let booking = make_create_booking();
    scheduler
        .book_slot(slot.id, booking.clone())

        .await
        .unwrap();

    let err = scheduler
        .book_slot(slot.id, booking)
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        migration_scheduler::AppError::SlotAlreadyBooked
    ));
}

#[tokio::test]
#[serial]
async fn test_book_slot_past_date_rejected() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2020-01-01", "10:00", "11:00");
    let slot = scheduler.create_slot(ct).await.unwrap();

    let booking = make_create_booking();
    let err = scheduler
        .book_slot(slot.id, booking)
        .await
        .unwrap_err();

    assert!(matches!(err, migration_scheduler::AppError::Other(_)));
}

#[tokio::test]
#[serial]
async fn test_book_slot_not_found() {
    let scheduler = create_scheduler().await;

    let booking = make_create_booking();
    let err = scheduler
        .book_slot(Uuid::new_v4(), booking)
        .await
        .unwrap_err();

    assert!(matches!(err, migration_scheduler::AppError::SlotNotFound));
}

// ── delete_slot ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_delete_slot_success() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    let slot = scheduler.create_slot(ct).await.unwrap();

    assert_eq!(scheduler.slot_count(), 1);

    let deleted = scheduler.delete_slot(slot.id).await.unwrap();
    assert!(deleted);
    assert_eq!(scheduler.slot_count(), 0);
    assert!(scheduler.get_slot(slot.id).is_none());
}

#[tokio::test]
#[serial]
async fn test_delete_slot_not_found() {
    let scheduler = create_scheduler().await;

    let err = scheduler.delete_slot(Uuid::new_v4()).await.unwrap_err();
    assert!(matches!(err, migration_scheduler::AppError::SlotNotFound));
}

#[tokio::test]
#[serial]
async fn test_delete_slot_persists() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    let slot = scheduler.create_slot(ct).await.unwrap();

    scheduler.delete_slot(slot.id).await.unwrap();

    let scheduler2 = Scheduler::with_path(TEST_DATA_PATH).await.unwrap();
    assert_eq!(scheduler2.slot_count(), 0);
}

// ── update_slot ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_update_slot_time() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    let slot = scheduler.create_slot(ct).await.unwrap();

    let update = UpdateTimeSlot {
        start_time: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
        end_time: Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
        ..Default::default()
    };

    let updated = scheduler.update_slot(slot.id, update).await.unwrap();
    assert_eq!(
        updated.start_time,
        NaiveTime::from_hms_opt(12, 0, 0).unwrap()
    );
    assert_eq!(updated.end_time, NaiveTime::from_hms_opt(13, 0, 0).unwrap());
}

#[tokio::test]
#[serial]
async fn test_update_slot_date_reindexes() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    let slot = scheduler.create_slot(ct).await.unwrap();

    let update = UpdateTimeSlot {
        date: Some(NaiveDate::from_ymd_opt(2026, 7, 20).unwrap()),
        ..Default::default()
    };

    let updated = scheduler.update_slot(slot.id, update).await.unwrap();
    assert_eq!(updated.date, NaiveDate::from_ymd_opt(2026, 7, 20).unwrap());

    let slots = scheduler.get_slots_in_range(None, None);
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].date, NaiveDate::from_ymd_opt(2026, 7, 20).unwrap());
}

#[tokio::test]
#[serial]
async fn test_update_slot_set_completed() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    let slot = scheduler.create_slot(ct).await.unwrap();

    let update = UpdateTimeSlot {
        completed: Some(true),
        ..Default::default()
    };

    let updated = scheduler.update_slot(slot.id, update).await.unwrap();
    assert!(updated.completed);
}

#[tokio::test]
#[serial]
async fn test_update_slot_open_clears_booking() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    let slot = scheduler.create_slot(ct).await.unwrap();

    let booking = make_create_booking();
    scheduler.book_slot(slot.id, booking).await.unwrap();

    let update = UpdateTimeSlot {
        is_available: Some(true),
        ..Default::default()
    };

    let updated = scheduler.update_slot(slot.id, update).await.unwrap();
    assert!(updated.is_available);
    assert!(updated.booking.is_none());
}

#[tokio::test]
#[serial]
async fn test_update_slot_set_booking() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    let slot = scheduler.create_slot(ct).await.unwrap();

    let booking = make_create_booking();
    let update = UpdateTimeSlot {
        booking: Some(booking),
        is_available: Some(false),
        ..Default::default()
    };

    let updated = scheduler.update_slot(slot.id, update).await.unwrap();
    assert!(!updated.is_available);
    assert!(updated.booking.is_some());
    assert_eq!(updated.booking.as_ref().unwrap().company_name, "TestCorp");
}

#[tokio::test]
#[serial]
async fn test_update_slot_not_found() {
    let scheduler = create_scheduler().await;

    let update = UpdateTimeSlot::default();
    let err = scheduler
        .update_slot(Uuid::new_v4(), update)
        .await
        .unwrap_err();

    assert!(matches!(err, migration_scheduler::AppError::SlotNotFound));
}

// ── set_yougile_task_id ─────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_set_yougile_task_id() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    let slot = scheduler.create_slot(ct).await.unwrap();

    let result = scheduler
        .set_yougile_task_id(slot.id, "yg-task-123".to_string())
        .await
        .unwrap();

    assert_eq!(result.yougile_task_id, Some("yg-task-123".to_string()));

    let fetched = scheduler.get_slot(slot.id).unwrap();
    assert_eq!(fetched.yougile_task_id, Some("yg-task-123".to_string()));
}

#[tokio::test]
#[serial]
async fn test_set_yougile_task_id_not_found() {
    let scheduler = create_scheduler().await;

    let err = scheduler
        .set_yougile_task_id(Uuid::new_v4(), "task".to_string())
        .await
        .unwrap_err();

    assert!(matches!(err, migration_scheduler::AppError::SlotNotFound));
}

// ── index consistency ───────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_index_consistency_after_operations() {
    let scheduler = create_scheduler().await;

    let mut ids = Vec::new();
    for i in 0..5 {
        let hour = 8 + i / 2;
        let minute = (i % 2) * 30;
        let ct = CreateTimeSlot {
            date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            start_time: NaiveTime::from_hms_opt(hour as u32, minute as u32, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(hour as u32 + 1, minute as u32, 0).unwrap(),
        };
        let slot = scheduler.create_slot(ct).await.unwrap();
        ids.push(slot.id);
    }

    assert_eq!(scheduler.slot_count(), 5);

    for id in &ids {
        assert!(scheduler.get_slot(*id).is_some());
    }

    scheduler.delete_slot(ids[0]).await.unwrap();
    scheduler.delete_slot(ids[2]).await.unwrap();

    assert_eq!(scheduler.slot_count(), 3);
    assert!(scheduler.get_slot(ids[0]).is_none());
    assert!(scheduler.get_slot(ids[2]).is_none());
    assert!(scheduler.get_slot(ids[1]).is_some());
    assert!(scheduler.get_slot(ids[3]).is_some());
    assert!(scheduler.get_slot(ids[4]).is_some());
}

#[tokio::test]
#[serial]
async fn test_index_consistency_after_update_date() {
    let scheduler = create_scheduler().await;

    let ct = make_create_slot("2026-06-15", "10:00", "11:00");
    let slot = scheduler.create_slot(ct).await.unwrap();

    let update = UpdateTimeSlot {
        date: Some(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap()),
        start_time: Some(NaiveTime::from_hms_opt(14, 0, 0).unwrap()),
        end_time: Some(NaiveTime::from_hms_opt(15, 0, 0).unwrap()),
        ..Default::default()
    };
    scheduler.update_slot(slot.id, update).await.unwrap();

    assert_eq!(scheduler.slot_count(), 1);
    let fetched = scheduler.get_slot(slot.id).unwrap();
    assert_eq!(fetched.date, NaiveDate::from_ymd_opt(2026, 8, 1).unwrap());
    assert_eq!(
        fetched.start_time,
        NaiveTime::from_hms_opt(14, 0, 0).unwrap()
    );

    let slots = scheduler.get_slots_in_range(None, None);
    assert_eq!(slots.len(), 1);
}

// ── persistence ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_persistence_across_restart() {
    let scheduler = create_scheduler().await;

    let ct1 = make_create_slot("2026-06-15", "10:00", "11:00");
    let ct2 = make_create_slot("2026-06-16", "14:00", "15:00");
    let slot1 = scheduler.create_slot(ct1).await.unwrap();
    let slot2 = scheduler.create_slot(ct2).await.unwrap();

    let booking = make_create_booking();
    scheduler.book_slot(slot1.id, booking).await.unwrap();

    let scheduler2 = Scheduler::with_path(TEST_DATA_PATH).await.unwrap();
    assert_eq!(scheduler2.slot_count(), 2);

    let s1 = scheduler2.get_slot(slot1.id).unwrap();
    assert!(!s1.is_available);
    assert!(s1.booking.is_some());

    let s2 = scheduler2.get_slot(slot2.id).unwrap();
    assert!(s2.is_available);
}

#[tokio::test]
#[serial]
async fn test_create_and_update_slot_invalid_time_range() {
    let scheduler = create_scheduler().await;

    let ct_equal = make_create_slot("2026-06-15", "10:00", "10:00");
    let err_equal = scheduler.create_slot(ct_equal).await.unwrap_err();
    assert!(matches!(err_equal, migration_scheduler::AppError::InvalidTimeRange));

    let ct_greater = make_create_slot("2026-06-15", "11:00", "10:00");
    let err_greater = scheduler.create_slot(ct_greater).await.unwrap_err();
    assert!(matches!(err_greater, migration_scheduler::AppError::InvalidTimeRange));

    let ct_valid = make_create_slot("2026-06-15", "10:00", "11:00");
    let slot = scheduler.create_slot(ct_valid).await.unwrap();

    let update_equal = UpdateTimeSlot {
        start_time: Some(NaiveTime::from_hms_opt(11, 0, 0).unwrap()),
        ..Default::default()
    };
    let err_update = scheduler.update_slot(slot.id, update_equal).await.unwrap_err();
    assert!(matches!(err_update, migration_scheduler::AppError::InvalidTimeRange));
}
