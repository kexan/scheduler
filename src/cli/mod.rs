pub mod utils;

use crate::cli::utils::*;
use crate::error::Result;
use crate::scheduler::Scheduler;
use crate::{api::handlers::slots::CreateSlotRequest, scheduler::BookingRequest};
use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "migration-scheduler")]
#[command(about = "Планировщик миграций компаний", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Server {
        #[arg(short, long, default_value = "3030")]
        port: u16,
    },
    List {
        #[arg(short, long)]
        available: bool,
    },
    Create {
        #[arg(short, long)]
        date: NaiveDate,
        #[arg(short, long)]
        start: String,
        #[arg(short, long)]
        end: String,
    },
    Delete {
        id: String,
    },
    Show {
        id: String,
    },
    Book {
        id: String,
        #[arg(long)]
        company: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        company_id: String,
        #[arg(long)]
        download_email: String,
    },
}

pub async fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { port } => {
            crate::web::start_server(port).await?;
        }
        Commands::List { available } => {
            list_slots(available).await?;
        }
        Commands::Create { date, start, end } => {
            create_slot(date, start, end).await?;
        }
        Commands::Delete { id } => {
            delete_slot(id).await?;
        }
        Commands::Show { id } => {
            show_slot(id).await?;
        }
        Commands::Book {
            id,
            company,
            email,
            company_id,
            download_email,
        } => {
            book_slot(id, company, email, company_id, download_email).await?;
        }
    }

    Ok(())
}

async fn list_slots(available_only: bool) -> Result<()> {
    let scheduler = Scheduler::load().await?;
    let slots = scheduler.get_slots();

    if slots.is_empty() {
        println!("📅 Слотов не найдено");
        return Ok(());
    }

    let filtered_slots: Vec<_> = if available_only {
        slots.into_iter().filter(|s| s.is_available).collect()
    } else {
        slots
    };

    if filtered_slots.is_empty() {
        println!("📅 Доступных слотов не найдено");
        return Ok(());
    }

    println!("📋 Список слотов:");
    println!();

    use tabled::{Table, Tabled, settings::Style};

    #[derive(Tabled)]
    struct SlotRow {
        #[tabled(rename = "ID")]
        id: String,
        #[tabled(rename = "Дата")]
        date: String,
        #[tabled(rename = "Время")]
        time: String,
        #[tabled(rename = "Статус")]
        status: String,
        #[tabled(rename = "Компания")]
        company: String,
    }

    let rows: Vec<SlotRow> = filtered_slots
        .iter()
        .map(|slot| {
            let (status, company) = if slot.is_available {
                ("✅ Доступен", "-".to_string())
            } else if let Some(booking) = &slot.booking {
                ("🔒 Занят", booking.company_name.clone())
            } else {
                ("❓ Неизвестно", "-".to_string())
            };

            SlotRow {
                id: slot.id.to_string()[..8].to_string(),
                date: slot.date.to_string(),
                time: format!(
                    "{}-{}",
                    slot.start_time.format("%H:%M"),
                    slot.end_time.format("%H:%M")
                ),
                status: status.to_string(),
                company,
            }
        })
        .collect();

    let table = Table::new(&rows).with(Style::modern()).to_string();

    println!("{}", table);

    println!();
    println!("📊 Всего слотов: {}", filtered_slots.len());
    println!(
        "✅ Доступно: {}",
        filtered_slots.iter().filter(|s| s.is_available).count()
    );
    println!(
        "🔒 Занято: {}",
        filtered_slots.iter().filter(|s| !s.is_available).count()
    );

    Ok(())
}

async fn create_slot(date: NaiveDate, start: String, end: String) -> Result<()> {
    let start_time = parse_time(&start)?;
    let end_time = parse_time(&end)?;
    validate_time_range(start_time, end_time)?;

    let mut scheduler = Scheduler::load().await?;

    let request = CreateSlotRequest {
        date,
        start_time,
        end_time,
    };

    let slot = scheduler.create_slot(request);
    scheduler.save().await?;
    print_slot_created(&slot);

    Ok(())
}

async fn delete_slot(id: String) -> Result<()> {
    let slot_uuid = parse_uuid(&id)?;
    let mut scheduler = Scheduler::load().await?;

    if scheduler.delete_slot(slot_uuid) {
        scheduler.save().await?;
        info!("🗑️  Slot {} deleted successfully", &id[..8.min(id.len())]);
        println!("✅ Слот {} успешно удален", &id[..8.min(id.len())]);
    } else {
        warn!("❌ Attempted to delete non-existent slot: {}", id);
        println!("❌ Слот с ID {} не найден", &id[..8.min(id.len())]);
    }

    Ok(())
}

async fn show_slot(id: String) -> Result<()> {
    let slot_uuid = parse_uuid(&id)?;
    let scheduler = Scheduler::load().await?;

    if let Some(slot) = scheduler.get_slot(slot_uuid) {
        print_slot_info(slot);
    } else {
        println!("❌ Слот с ID {} не найден", &id[..8.min(id.len())]);
    }

    Ok(())
}

async fn book_slot(
    id: String,
    company: String,
    email: String,
    company_id: String,
    download_email: String,
) -> Result<()> {
    let slot_uuid = parse_uuid(&id)?;
    let mut scheduler = Scheduler::load().await?;

    let request = BookingRequest {
        company_name: company,
        admin_email: email,
        company_id,
        download_email,
    };

    match scheduler.book_slot(slot_uuid, request) {
        Ok(slot) => {
            scheduler.save().await?;
            print_slot_booked(
                &slot,
                &slot.booking.as_ref().unwrap().company_name,
                &slot.booking.as_ref().unwrap().company_id,
            );
        }
        Err(e) => {
            println!("❌ Ошибка бронирования: {}", e);
        }
    }

    Ok(())
}
