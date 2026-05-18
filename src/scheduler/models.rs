use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use chrono_tz::Europe::Moscow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    pub id: Uuid,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub date: NaiveDate,
    pub is_available: bool,
    pub booking: Option<Booking>,
    pub yougile_task_id: Option<String>,
    #[serde(default)]
    pub completed: bool,
}

impl TimeSlot {
    pub fn to_timestamps_ms(&self) -> (f64, f64) {
        let start = self
            .date
            .and_time(self.start_time)
            .and_local_timezone(Moscow)
            .single()
            .expect("Moscow timezone has no ambiguous times");
        let end = self
            .date
            .and_time(self.end_time)
            .and_local_timezone(Moscow)
            .single()
            .expect("Moscow timezone has no ambiguous times");

        (
            start.timestamp_millis() as f64,
            end.timestamp_millis() as f64,
        )
    }

    pub fn build_task_title(&self) -> String {
        let company = self
            .booking
            .as_ref()
            .map(|b| b.company_name.as_str())
            .unwrap_or("Неизвестная компания");
        format!("Миграция: {}", company)
    }

    pub fn build_task_description(&self) -> String {
        let time_range = format!(
            "{} - {}",
            self.start_time.format("%H:%M"),
            self.end_time.format("%H:%M")
        );

        match &self.booking {
            Some(b) => format!(
                "Информация о бронировании:<br/>\
            • Компания: {}<br/>\
            • Email администратора: {}<br/>\
            • ID компании: {}<br/>\
            • Email получателя архива: {}<br/>\
            • Дата: {}<br/>\
            • Время: {}<br/>\
            • Создано: {}",
                b.company_name,
                b.admin_email,
                b.company_id,
                b.download_email,
                self.date,
                time_range,
                b.created_at.format("%Y-%m-%d %H:%M:%S UTC")
            ),
            None => format!(
                "Информация о слоте:<br/>\
            • Дата: {}<br/>\
            • Время: {}<br/>\
            • Статус: Доступен",
                self.date, time_range
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTimeSlot {
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateTimeSlot {
    pub date: Option<NaiveDate>,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub is_available: Option<bool>,
    pub booking: Option<CreateBooking>,
    pub completed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Booking {
    pub company_name: String,
    pub admin_email: String,
    pub company_id: String,
    pub download_email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBooking {
    pub company_name: String,
    pub admin_email: String,
    pub company_id: String,
    pub download_email: String,
}
