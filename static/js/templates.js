import { Utils } from './utils.js';

class Templates {
  static calendarHeader(day, isWeekend) {
    return `<div class="calendar-header ${isWeekend ? "weekend" : ""}">${day}</div>`;
  }

  static calendarDay({
    dayNum,
    monthLabel,
    dateStr,
    isOtherMonth,
    isToday,
    isPast,
    isSelectedForCopy,
    showCheckbox,
    isAdminMode,
    isWeekend,
    slotsHtml
  }) {
    const classes = ['calendar-day'];
    if (isOtherMonth) classes.push('other-month');
    if (isToday) classes.push('today');
    if (isPast) classes.push('past-day');
    if (isSelectedForCopy) classes.push('selected-for-copy');
    if (showCheckbox) classes.push('has-checkbox');
    if (isWeekend) classes.push('weekend');

    const checkboxHtml = showCheckbox ? `
      <div class="day-checkbox">
        <input type="checkbox" data-date="${dateStr}" ${isSelectedForCopy ? "checked" : ""}>
      </div>
    ` : "";

    const isCurrentOrPast = isPast || isToday;
    const dayNumberClass = (isCurrentOrPast && !isOtherMonth) ? "disabled-day" : "";

    const adminAddBtn = isAdminMode ? `<button class="admin-add-slot-btn" data-date="${dateStr}" title="Добавить слот">+</button>` : "";

    return `
      <div class="${classes.join(' ')}" data-date="${dateStr}">
        ${checkboxHtml}
        <div class="calendar-day-number ${dayNumberClass}">
          ${dayNum} <span class="month-label">${monthLabel}</span>
          ${adminAddBtn}
        </div>
        <div class="calendar-slots">${slotsHtml}</div>
      </div>
    `;
  }

  static calendarSlotsContainer(slotsHtml, needsScroll) {
    const scrollClass = needsScroll ? "with-scroll" : "no-scroll";
    return `<div class="calendar-slots ${scrollClass}">${slotsHtml}</div>`;
  }

  static calendarEmptySlots(isAdminMode) {
    if (isAdminMode) {
      return '<div class="calendar-slots no-scroll"></div>';
    }
    return '<div class="calendar-slots no-scroll"><div class="text-muted small">Нет слотов</div></div>';
  }

  static calendarSlot(slot, slotClass, content) {
    return `
      <div class="calendar-slot ${slotClass}" data-slot-id="${slot.id}">
        ${content}
      </div>
    `;
  }

  static copyTargetDateBadges(targetDates) {
    return targetDates
      .map(
        (date) =>
          `<span class="badge bg-secondary me-1">${Utils.formatDate(date)}</span>`,
      )
      .join("");
  }

  static viewSlotModalContent(slot, statusText) {
    let html = `
      <div class="mb-3">
        <strong>Дата и время:</strong> ${Utils.formatDateTimeMSK(slot.date, slot.start_time)} - ${Utils.formatTime(slot.end_time)}
      </div>
      <div class="mb-3">
        <strong>Статус:</strong> ${statusText}
      </div>
    `;

    if (slot.booking) {
      html += `
        <div class="card">
          <div class="card-body">
            <h6 class="card-title">Информация о бронировании</h6>
            <div class="row">
              <div class="col-md-6">
                <strong>Компания:</strong> ${slot.booking.company_name}<br>
                <strong>Email администратора:</strong> ${slot.booking.admin_email}<br>
                <strong>ID компании:</strong> ${slot.booking.company_id}
              </div>
              <div class="col-md-6">
                <strong>Email получателя архива:</strong> ${slot.booking.download_email}<br>
                <strong>Дата создания:</strong> ${new Date(slot.booking.created_at).toLocaleString()}
              </div>
            </div>
          </div>
        </div>
      `;
    }
    return html;
  }

  static yougileConnectionSuccess(projectsCount) {
    return `<div class="alert alert-success">Загружено проектов: ${projectsCount}</div>`;
  }

  static yougileConnectionError(errorMessage) {
    return `<div class="alert alert-danger"><strong>❌ Ошибка подключения</strong><br>${errorMessage}</div>`;
  }

  static selectOptions(items, valueField, textField, placeholder) {
    let html = `<option value="">${placeholder}</option>`;
    items.forEach((item) => {
      html += `<option value="${item[valueField]}">${item[textField]}</option>`;
    });
    return html;
  }
}

export { Templates };
