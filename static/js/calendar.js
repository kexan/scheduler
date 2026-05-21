import { Utils, DOMHelper, ButtonHelper } from './utils.js';
import { Templates } from './templates.js';

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
    this.setupDelegatedEvents();
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
    dayNames.forEach((day, idx) => {
      const isWeekend = idx >= 5;
      html += Templates.calendarHeader(day, isWeekend);
    });

    let cellIdx = 0;

    const prevMonthIdx = this.currentMonth === 0 ? 11 : this.currentMonth - 1;
    for (let i = firstDay - 1; i >= 0; i--) {
      const prevDay = daysInPrevMonth - i;
      const dateStr = this.getDateString(
        prevDay,
        this.currentMonth - 1,
        this.currentYear,
      );
      const isWeekend = (cellIdx % 7) >= 5;
      cellIdx++;
      html += Templates.calendarDay({
        dayNum: prevDay,
        monthLabel: monthShort[prevMonthIdx],
        dateStr,
        isOtherMonth: true,
        isToday: false,
        isPast: false,
        isSelectedForCopy: false,
        showCheckbox: false,
        isAdminMode: this.isAdminMode,
        isWeekend,
        slotsHtml: this.renderDaySlots(this.getSlotsForDate(dateStr), dateStr)
      });
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
      const isSelectedForCopy = this.selectedDaysForCopy.has(dateStr);
      const showCheckbox = this.isAdminMode && this.isCopyMode;
      const isWeekend = (cellIdx % 7) >= 5;
      cellIdx++;
      html += Templates.calendarDay({
        dayNum: day,
        monthLabel: monthShort[this.currentMonth],
        dateStr,
        isOtherMonth: false,
        isToday,
        isPast,
        isSelectedForCopy,
        showCheckbox,
        isAdminMode: this.isAdminMode,
        isWeekend,
        slotsHtml: this.renderDaySlots(daySlots, dateStr)
      });
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
      const isWeekend = (cellIdx % 7) >= 5;
      cellIdx++;
      html += Templates.calendarDay({
        dayNum: day,
        monthLabel: monthShort[nextMonthIdx],
        dateStr,
        isOtherMonth: true,
        isToday: false,
        isPast: false,
        isSelectedForCopy: false,
        showCheckbox: false,
        isAdminMode: this.isAdminMode,
        isWeekend,
        slotsHtml: this.renderDaySlots(this.getSlotsForDate(dateStr), dateStr)
      });
    }

    html += "</div>";
    container.innerHTML = html;
  }

  renderDaySlots(slots, dateStr) {
    if (slots.length === 0) {
      return Templates.calendarEmptySlots(this.isAdminMode);
    }

    const needsScroll = slots.length > 4;

    const slotsHtml = slots
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

        return Templates.calendarSlot(slot, slotClass, content);
      })
      .join("");

    return Templates.calendarSlotsContainer(slotsHtml, needsScroll);
  }

  isDateValidForSlot(dateStr) {
    const parts = dateStr.split('-');
    const slotYear = parseInt(parts[0], 10);
    const slotMonth = parseInt(parts[1], 10) - 1;
    const slotDay = parseInt(parts[2], 10);
    const slotDate = new Date(slotYear, slotMonth, slotDay);

    const todayMsk = Utils.getCurrentTimeMSK();
    todayMsk.setHours(0, 0, 0, 0);

    return slotDate > todayMsk;
  }

  isPastDate(dateStr) {
    const parts = dateStr.split('-');
    const slotYear = parseInt(parts[0], 10);
    const slotMonth = parseInt(parts[1], 10) - 1;
    const slotDay = parseInt(parts[2], 10);
    const date = new Date(slotYear, slotMonth, slotDay);

    const todayMsk = Utils.getCurrentTimeMSK();
    todayMsk.setHours(0, 0, 0, 0);

    return date < todayMsk;
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
      Templates.copyTargetDateBadges(targetDates),
    );

    const modalElement = DOMHelper.get("copyScheduleModal");
    const bsModal = bootstrap.Modal.getOrCreateInstance(modalElement);

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
    try {
      for (const targetDate of targetDates) {
        const existingSlots = this.getSlotsForDate(targetDate);
        // Delete all existing slots for the target date in parallel
        await Promise.all(existingSlots.map((slot) => this.scheduler.api.deleteSlot(slot.id)));

        // Create the new slots in parallel
        await Promise.all(
          sourceSlots.map((sourceSlot) => {
            const slotData = {
              date: targetDate,
              start_time: sourceSlot.start_time,
              end_time: sourceSlot.end_time,
            };
            return this.scheduler.api.createSlot(slotData);
          })
        );
      }

      Utils.showSuccess(`Расписание скопировано на ${targetDates.length} дней`);
      this.clearSelection();

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
    } catch (error) {
      Utils.showError("Ошибка при копировании расписания: " + error.message);
    }
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

  setupDelegatedEvents() {
    const container = document.getElementById("calendarContainer");
    if (!container) return;

    container.addEventListener("click", (e) => {
      // 1. Admin add slot button click
      const addBtn = e.target.closest(".admin-add-slot-btn");
      if (addBtn) {
        e.stopPropagation();
        const date = addBtn.dataset.date;
        this.scheduler.modals.openQuickSlotModal(date);
        return;
      }

      // 2. Calendar slot click
      const slot = e.target.closest(".calendar-slot");
      if (slot && !slot.classList.contains("admin-add-slot")) {
        const slotId = slot.dataset.slotId;
        const slotItem = this.slots.find((s) => s.id === slotId);
        if (!slotItem) return;

        if (this.isAdminMode) {
          this.scheduler.modals.openEditModal(slotId);
        } else if (slotItem.is_available && this.isDateValidForSlot(slotItem.date)) {
          this.scheduler.modals.openBookingModal(slotId);
        } else if (slotItem.is_available) {
          Utils.showError(
            "Нельзя записываться на слоты в текущие и прошедшие даты",
          );
        } else {
          this.scheduler.modals.openViewSlotModal(slotId);
        }
        return;
      }
    });

    container.addEventListener("change", (e) => {
      // 3. Day checkbox change (for copy mode)
      if (e.target.matches(".day-checkbox input")) {
        e.stopPropagation();
        this.toggleDayForCopy(e.target.dataset.date, e.target.checked);
      }
    });
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

export { Calendar };
