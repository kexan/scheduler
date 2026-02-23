class Calendar {
  constructor(scheduler) {
    this.scheduler = scheduler;
    this.currentMonth = new Date().getMonth();
    this.currentYear = new Date().getFullYear();
    this.slots = [];
    this.isAdminMode = false;
    this.isCopyMode = false;
    this.selectedDaysForCopy = new Set();
    this.sourceDate = null;
    this._resizeTimer = null;
    this._setupResizeObserver();
  }

  _setupResizeObserver() {
    const container = document.getElementById("calendarContainer");
    if (!container || !window.ResizeObserver) return;

    this._resizeObserver = new ResizeObserver(() => {
      clearTimeout(this._resizeTimer);
      this._resizeTimer = setTimeout(() => {
        if (this.slots.length > 0) {
          this.render(this.slots);
        }
      }, 150);
    });

    this._resizeObserver.observe(container);
  }

  render(slots) {
    this.slots = slots;
    const container = document.getElementById("calendarContainer");
    const monthYearDisplay = document.getElementById("currentMonth");

    const monthNames = [
      "Январь",
      "Февраль",
      "Март",
      "Апрель",
      "Май",
      "Июнь",
      "Июль",
      "Август",
      "Сентябрь",
      "Октябрь",
      "Ноябрь",
      "Декабрь",
    ];
    const monthShort = [
      "янв",
      "фев",
      "мар",
      "апр",
      "май",
      "июн",
      "июл",
      "авг",
      "сен",
      "окт",
      "ноя",
      "дек",
    ];
    monthYearDisplay.textContent = `${monthNames[this.currentMonth]} ${this.currentYear}`;

    let firstDay = new Date(this.currentYear, this.currentMonth, 1).getDay();
    const daysInMonth = new Date(
      this.currentYear,
      this.currentMonth + 1,
      0,
    ).getDate();
    const daysInPrevMonth = new Date(
      this.currentYear,
      this.currentMonth,
      0,
    ).getDate();

    firstDay = firstDay === 0 ? 6 : firstDay - 1;

    let html = '<div class="calendar-grid">';

    const dayNames = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];
    dayNames.forEach((day) => {
      html += `<div class="calendar-header">${day}</div>`;
    });

    const prevMonthIdx = this.currentMonth === 0 ? 11 : this.currentMonth - 1;
    for (let i = firstDay - 1; i >= 0; i--) {
      const prevDay = daysInPrevMonth - i;
      const dateStr = this.getDateString(
        prevDay,
        this.currentMonth - 1,
        this.currentYear,
      );
      html += `<div class="calendar-day other-month">
                <div class="calendar-day-number">
                  ${prevDay} <span class="month-label">${monthShort[prevMonthIdx]}</span>
                  ${this.isAdminMode ? `<button class="admin-add-slot-btn" data-date="${dateStr}" title="Добавить слот">+</button>` : ""}
                </div>
                <div class="calendar-slots">${this.renderDaySlots(this.getSlotsForDate(dateStr), dateStr)}</div>
            </div>`;
    }

    const today = new Date();
    for (let day = 1; day <= daysInMonth; day++) {
      const dateStr = this.getDateString(
        day,
        this.currentMonth,
        this.currentYear,
      );
      const daySlots = this.getSlotsForDate(dateStr);
      const isToday =
        today.getDate() === day &&
        today.getMonth() === this.currentMonth &&
        today.getFullYear() === this.currentYear;

      const isPast = this.isPastDate(dateStr);
      const isCurrentOrPast = isPast || isToday;
      const isSelectedForCopy = this.selectedDaysForCopy.has(dateStr);
      const showCheckbox = this.isAdminMode && this.isCopyMode;
      const dayClass = `calendar-day ${isToday ? "today" : ""} ${isPast ? "past-day" : ""} ${isSelectedForCopy ? "selected-for-copy" : ""} ${showCheckbox ? "has-checkbox" : ""}`;

      html += `<div class="${dayClass}" data-date="${dateStr}">
                ${
                  showCheckbox
                    ? `
                  <div class="day-checkbox">
                    <input type="checkbox" data-date="${dateStr}" ${isSelectedForCopy ? "checked" : ""}>
                  </div>
                `
                    : ""
                }
                <div class="calendar-day-number ${isCurrentOrPast ? "disabled-day" : ""}">
                  ${day} <span class="month-label">${monthShort[this.currentMonth]}</span>
                  ${this.isAdminMode ? `<button class="admin-add-slot-btn" data-date="${dateStr}" title="Добавить слот">+</button>` : ""}
                </div>
                <div class="calendar-slots">${this.renderDaySlots(daySlots, dateStr)}</div>
            </div>`;
    }

    const totalCells = firstDay + daysInMonth;
    const nextMonthDays = totalCells % 7 === 0 ? 0 : 7 - (totalCells % 7);
    const nextMonthIdx = this.currentMonth === 11 ? 0 : this.currentMonth + 1;
    for (let day = 1; day <= nextMonthDays; day++) {
      const dateStr = this.getDateString(
        day,
        this.currentMonth + 1,
        this.currentYear,
      );
      html += `<div class="calendar-day other-month">
                <div class="calendar-day-number">
                  ${day} <span class="month-label">${monthShort[nextMonthIdx]}</span>
                  ${this.isAdminMode ? `<button class="admin-add-slot-btn" data-date="${dateStr}" title="Добавить слот">+</button>` : ""}
                </div>
                <div class="calendar-slots">${this.renderDaySlots(this.getSlotsForDate(dateStr), dateStr)}</div>
            </div>`;
    }

    html += "</div>";
    container.innerHTML = html;

    this.attachCalendarSlotListeners();
  }

  renderDaySlots(slots, dateStr) {
    let html = "";

    if (slots.length === 0) {
      if (this.isAdminMode) {
        return '<div class="calendar-slots no-scroll"></div>';
      }
      return '<div class="calendar-slots no-scroll"><div class="text-muted small">Нет слотов</div></div>';
    }

    const needsScroll = slots.length > 4;
    const scrollClass = needsScroll ? "with-scroll" : "no-scroll";

    html = slots
      .map((slot) => {
        let content = "";
        const isPastDate = !this.isDateValidForSlot(dateStr);
        let slotClass = "";

        if (!slot.is_available) {
          if (slot.completed) {
            content += "✓ ";
            slotClass = "completed";
          } else {
            content += "🔒 ";
            slotClass = "in-progress";
          }
        } else if (isPastDate) {
          slotClass = "past-date-available";
        }

        content += `${Utils.formatTime(slot.start_time)}-${Utils.formatTime(slot.end_time)}`;

        if (!slot.is_available && slot.booking && slot.booking.company_name) {
          content += ` ${slot.booking.company_name}`;
        }

        return `
                <div class="calendar-slot ${slotClass}"
                     data-slot-id="${slot.id}">
                    ${content}
                </div>
            `;
      })
      .join("");

    return `<div class="calendar-slots ${scrollClass}">${html}</div>`;
  }

  isDateValidForSlot(dateStr) {
    const slotDate = new Date(dateStr);
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    slotDate.setHours(0, 0, 0, 0);

    return slotDate > today;
  }

  isPastDate(dateStr) {
    const date = new Date(dateStr);
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    date.setHours(0, 0, 0, 0);
    return date < today;
  }

  getVisibleRowCount() {
    let firstDay = new Date(this.currentYear, this.currentMonth, 1).getDay();
    firstDay = firstDay === 0 ? 6 : firstDay - 1;
    const daysInMonth = new Date(
      this.currentYear,
      this.currentMonth + 1,
      0,
    ).getDate();
    return Math.ceil((firstDay + daysInMonth) / 7);
  }

  getSlotsContainerHeight() {
    const container = document.getElementById("calendarContainer");
    if (!container || !container.clientHeight) return 0;

    const GRID_HEADER = 32;
    const rows = this.getVisibleRowCount();
    const cellHeight = (container.clientHeight - GRID_HEADER) / rows;
    const DAY_NUMBER = 31;
    const CELL_PADDING = 16;
    const BUFFER = 8;

    return cellHeight - CELL_PADDING - DAY_NUMBER - BUFFER;
  }

  toggleDayForCopy(dateStr, isSelected) {
    if (isSelected) {
      this.selectedDaysForCopy.add(dateStr);
    } else {
      this.selectedDaysForCopy.delete(dateStr);
    }
    this.updateSelectedDaysInfo();
    this.updateCopyButton();
  }

  updateSelectedDaysInfo() {
    const count = this.selectedDaysForCopy.size;
    if (count === 0) {
      DOMHelper.text("selectedDaysInfo", "Дни не выбраны");
      DOMHelper.removeClass("selectedDaysInfo", "has-selection");
    } else {
      DOMHelper.text("selectedDaysInfo", `Выбрано дней: ${count}`);
      DOMHelper.addClass("selectedDaysInfo", "has-selection");
    }
  }

  updateCopyButton() {
    const copyBtn = DOMHelper.get("copySchedule");
    const sourceDateValue = DOMHelper.get("sourceDate").value;

    if (copyBtn) {
      copyBtn.disabled = !(
        this.selectedDaysForCopy.size > 0 && sourceDateValue
      );
    }
  }

  clearSelection() {
    this.selectedDaysForCopy.clear();
    this.updateSelectedDaysInfo();
    this.updateCopyButton();
    this.render(this.slots);
  }

  getSlotsForDate(dateStr) {
    return this.slots.filter((slot) => slot.date === dateStr);
  }

  async copySchedule() {
    const sourceDate = document.getElementById("sourceDate").value;
    const targetDates = Array.from(this.selectedDaysForCopy).sort();

    const sourceSlots = this.getSlotsForDate(sourceDate);
    if (sourceSlots.length === 0) {
      Utils.showError("В дате-источнике нет слотов для копирования");
      return;
    }

    this.showCopyConfirmation(sourceDate, sourceSlots, targetDates);
  }

  showCopyConfirmation(sourceDate, sourceSlots, targetDates) {
    DOMHelper.text("copySourceDate", Utils.formatDate(sourceDate));
    DOMHelper.text("copySourceSlots", sourceSlots.length);
    DOMHelper.html(
      "copyTargetDates",
      targetDates
        .map(
          (date) =>
            `<span class="badge bg-secondary me-1">${Utils.formatDate(date)}</span>`,
        )
        .join(""),
    );

    const modalElement = DOMHelper.get("copyScheduleModal");
    const existingModal = bootstrap.Modal.getInstance(modalElement);
    if (existingModal) {
      existingModal.dispose();
    }
    const bsModal = new bootstrap.Modal(modalElement);

    const confirmBtn = DOMHelper.get("confirmCopy");
    if (confirmBtn) {
      const newConfirmBtn = confirmBtn.cloneNode(true);
      confirmBtn.parentNode.replaceChild(newConfirmBtn, confirmBtn);

      newConfirmBtn.addEventListener("click", async () => {
        bsModal.hide();
        await this.performCopy(sourceSlots, targetDates);
      });
    }

    bsModal.show();
  }
  async performCopy(sourceSlots, targetDates) {
    for (const targetDate of targetDates) {
      const existingSlots = this.getSlotsForDate(targetDate);
      for (const slot of existingSlots) {
        await this.scheduler.api.deleteSlot(slot.id, true);
      }

      for (const sourceSlot of sourceSlots) {
        const slotData = {
          date: targetDate,
          start_time: sourceSlot.start_time,
          end_time: sourceSlot.end_time,
        };

        await this.scheduler.api.createSlot(slotData);
      }
    }

    Utils.showSuccess(`Расписание скопировано на ${targetDates.length} дней`);
    this.clearSelection();

    const confirmModal = DOMHelper.get("copyScheduleModal");
    if (confirmModal) {
      const modalInstance = bootstrap.Modal.getInstance(confirmModal);
      if (modalInstance) {
        modalInstance.dispose();
      }
    }

    DOMHelper.hide("adminCopyPanel");
    ButtonHelper.setState(
      "toggleCopyPanel",
      "📋 Копирование расписания",
      "btn-outline-danger",
      "btn-outline-info",
    );

    await this.scheduler.loadSlots();

    this.slots = this.scheduler.slots;

    this.setCopyMode(false);
    this.render(this.slots);
  }

  getVisibleDateRange() {
    let firstDay = new Date(this.currentYear, this.currentMonth, 1).getDay();
    firstDay = firstDay === 0 ? 6 : firstDay - 1;

    const daysInMonth = new Date(
      this.currentYear,
      this.currentMonth + 1,
      0,
    ).getDate();
    const daysInPrevMonth = new Date(
      this.currentYear,
      this.currentMonth,
      0,
    ).getDate();

    const fromDay = daysInPrevMonth - firstDay + 1;
    const from = this.getDateString(
      firstDay > 0 ? fromDay : 1,
      firstDay > 0 ? this.currentMonth - 1 : this.currentMonth,
      this.currentYear,
    );

    const totalCells = firstDay + daysInMonth;
    const nextMonthDays = totalCells % 7 === 0 ? 0 : 7 - (totalCells % 7);
    const to = this.getDateString(
      nextMonthDays > 0 ? nextMonthDays : daysInMonth,
      nextMonthDays > 0 ? this.currentMonth + 1 : this.currentMonth,
      this.currentYear,
    );

    return { from, to };
  }

  getDateString(day, month, year) {
    const actualMonth = month < 0 ? 11 : month > 11 ? 0 : month;
    const actualYear = month < 0 ? year - 1 : month > 11 ? year + 1 : year;

    const monthStr = String(actualMonth + 1).padStart(2, "0");
    const dayStr = String(day).padStart(2, "0");
    return `${actualYear}-${monthStr}-${dayStr}`;
  }

  getSlotsForDate(dateStr) {
    return this.slots.filter((slot) => slot.date === dateStr);
  }

  attachCalendarSlotListeners() {
    document
      .querySelectorAll(".calendar-slot:not(.admin-add-slot)")
      .forEach((slot) => {
        slot.addEventListener("click", (e) => {
          const slotId = e.target.dataset.slotId;
          const slot = this.slots.find((s) => s.id === slotId);

          if (this.isAdminMode) {
            this.scheduler.modals.openEditModal(slotId);
          } else if (slot.is_available && this.isDateValidForSlot(slot.date)) {
            this.scheduler.modals.openBookingModal(slotId);
          } else if (slot.is_available) {
            Utils.showError(
              "Нельзя записываться на слоты в текущие и прошедшие даты",
            );
          } else {
            this.scheduler.modals.openViewSlotModal(slotId);
          }
        });
      });

    if (this.isAdminMode && this.isCopyMode) {
      document.querySelectorAll(".day-checkbox input").forEach((checkbox) => {
        checkbox.addEventListener("change", (e) => {
          e.stopPropagation();
          this.toggleDayForCopy(e.target.dataset.date, e.target.checked);
        });

        checkbox.addEventListener("click", (e) => {
          e.stopPropagation();
        });
      });
    }

    document.querySelectorAll(".admin-add-slot-btn").forEach((addBtn) => {
      addBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        const date = e.currentTarget.dataset.date;
        this.scheduler.modals.openQuickSlotModal(date);
      });
    });
  }

  renderSlot(slot) {
    const canBook = slot.is_available && this.isDateValidForSlot(slot.date);
    let statusClass, statusText;

    if (!slot.is_available) {
      statusClass = "slot-booked";
      statusText = "Занят";
    } else if (canBook) {
      statusClass = "slot-available";
      statusText = "Доступен";
    } else {
      statusClass = "slot-past-available";
      statusText = "Доступен (прошедшая дата)";
    }

    const buttonClass = canBook ? "btn-success" : "btn-secondary disabled";
    const buttonText = canBook
      ? "Записаться"
      : slot.is_available
        ? "Недоступно для записи"
        : "Занят";
    const buttonDisabled = canBook ? "" : "disabled";

    let bookingInfo = "";
    if (slot.booking) {
      bookingInfo = `
                    <div class="booking-info mt-2">
                        <strong>Компания:</strong> ${slot.booking.company_name}<br>
                        <strong>ID компании:</strong> ${slot.booking.company_id}<br>
                        <strong>Email администратора:</strong> ${slot.booking.admin_email}<br>
                        <strong>Email для выгрузки:</strong> ${slot.booking.download_email}<br>
                        <small class="text-muted">Запись создана: ${new Date(slot.booking.created_at).toLocaleString()}</small>
                    </div>
                `;
    }

    const adminButton = this.isAdminMode
      ? `<button class="btn btn-outline-secondary btn-sm slot-action-btn ms-2" data-slot-id="${slot.id}">✏️ Редактировать</button>`
      : "";

    return `
                <div class="time-slot ${statusClass}" data-slot-id="${slot.id}">
                    <div class="d-flex justify-content-between align-items-center">
                        <div>
                            <strong>${Utils.formatTime(slot.start_time)} - ${Utils.formatTime(slot.end_time)}</strong>
                            <span class="badge bg-${slot.is_available ? "success" : "danger"} ms-2">${statusText}</span>
                            ${slot.is_available && !canBook ? '<span class="badge bg-warning ms-2">Прошедшая дата</span>' : ""}
                        </div>
                        <div class="d-flex">
                            ${
                              slot.is_available && !this.isAdminMode
                                ? `
                                <button class="btn ${buttonClass} btn-sm slot-action-btn"
                                        data-slot-id="${slot.id}"
                                        ${buttonDisabled}>
                                    ${buttonText}
                                </button>
                            `
                                : ""
                            }
                            ${adminButton}
                        </div>
                    </div>
                    ${bookingInfo}
                </div>
            `;
  }

  async nextMonth() {
    this.currentMonth++;
    if (this.currentMonth > 11) {
      this.currentMonth = 0;
      this.currentYear++;
    }
    await this.scheduler.loadSlots();
  }

  async prevMonth() {
    this.currentMonth--;
    if (this.currentMonth < 0) {
      this.currentMonth = 11;
      this.currentYear--;
    }
    await this.scheduler.loadSlots();
  }

  setAdminMode(isAdmin) {
    this.isAdminMode = isAdmin;
    if (!isAdmin) {
      this.isCopyMode = false;
      this.clearSelection();
    }
    this.render(this.slots);
  }

  setCopyMode(isCopy) {
    this.isCopyMode = isCopy;
    if (!isCopy) {
      this.clearSelection();
    }
    this.render(this.slots);
  }
}
