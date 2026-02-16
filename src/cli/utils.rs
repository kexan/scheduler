use crate::error::{AppError, Result};
use crate::scheduler::TimeSlot;
use chrono::NaiveTime;
use tracing::error;
use uuid::Uuid;

pub fn parse_time(time_str: &str) -> Result<NaiveTime> {
    chrono::NaiveTime::parse_from_str(time_str, "%H:%M")
        .map_err(|_| AppError::InvalidTimeFormat(time_str.to_string()))
}

pub fn validate_time_range(start: NaiveTime, end: NaiveTime) -> Result<()> {
    if start >= end {
        error!("Invalid time range: start {} >= end {}", start, end);
        return Err(AppError::InvalidTimeRange);
    }
    Ok(())
}

pub fn parse_uuid(id_str: &str) -> Result<Uuid> {
    Uuid::parse_str(id_str).map_err(|_| AppError::InvalidUuid(id_str.to_string()))
}

pub fn print_slot_created(slot: &TimeSlot) {
    println!("Слот успешно создан:");
    println!("   Дата: {}", slot.date);
    println!(
        "   Время: {} - {}",
        slot.start_time.format("%H:%M"),
        slot.end_time.format("%H:%M")
    );
    println!("   ID: {}", slot.id);
}

pub fn print_slot_booked(slot: &TimeSlot, company: &str, company_id: &str) {
    println!("Слот успешно забронирован:");
    println!("   Компания: {}", company);
    println!("   ID компании: {}", company_id);
    println!("   Дата: {}", slot.date);
    println!(
        "   Время: {} - {}",
        slot.start_time.format("%H:%M"),
        slot.end_time.format("%H:%M")
    );
}

pub fn print_slot_info(slot: &TimeSlot) {
    println!("Информация о слоте:");
    println!("   ID: {}", slot.id);
    println!("   Дата: {}", slot.date);
    println!(
        "   Время: {} - {}",
        slot.start_time.format("%H:%M"),
        slot.end_time.format("%H:%M")
    );
    println!(
        "   Статус: {}",
        if slot.is_available {
            "Доступен"
        } else {
            "Занят"
        }
    );

    if let Some(booking) = &slot.booking {
        println!("   Компания: {}", booking.company_name);
        println!("   Email админа: {}", booking.admin_email);
        println!("   ID компании: {}", booking.company_id);
        println!("   Email для скачивания: {}", booking.download_email);
        println!(
            "   Создано: {}",
            booking.created_at.format("%Y-%m-%d %H:%M:%S")
        );
    }
}
