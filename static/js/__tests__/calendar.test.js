import { describe, it, expect, beforeEach, vi } from 'vitest';
import { Utils } from '../utils.js';
import { Calendar } from '../calendar.js';

describe('calendar.js', () => {
  let schedulerMock;
  let calendar;

  beforeEach(() => {
    // Mock Utils.getCurrentTimeMSK to return a fixed date (2026-05-21)
    vi.spyOn(Utils, 'getCurrentTimeMSK').mockImplementation(() => {
      return new Date('2026-05-21T12:00:00+03:00'); // Moscow time
    });

    // Setup mock scheduler
    schedulerMock = {
      slots: [],
      loadSlots: vi.fn(),
      api: {
        createSlot: vi.fn(),
        deleteSlot: vi.fn(),
        bookSlot: vi.fn(),
      },
      modals: {
        openBookingModal: vi.fn(),
        openViewSlotModal: vi.fn(),
        openEditModal: vi.fn(),
        openQuickSlotModal: vi.fn(),
      },
    };

    // Mock bootstrap.Modal
    globalThis.bootstrap = {
      Modal: {
        getOrCreateInstance: vi.fn().mockReturnValue({
          show: vi.fn(),
          hide: vi.fn(),
        }),
      },
    };

    // Setup DOM elements needed for calendar rendering
    document.body.innerHTML = `
      <div id="calendarContainer"></div>
      <div id="currentMonth"></div>
      <button id="copySchedule" disabled></button>
      <button id="clearSelection"></button>
      <input id="sourceDate" value="2026-05-21" />
      <div id="adminCopyPanel" style="display: none;"></div>
      <button id="toggleCopyPanel"></button>
      <div id="copyScheduleModal">
        <div id="copySourceDate"></div>
        <div id="copySourceSlots"></div>
        <div id="copyTargetDates"></div>
        <button id="confirmCopy"></button>
      </div>
      <div id="selectedDaysInfo"></div>
    `;

    calendar = new Calendar(schedulerMock);
  });

  describe('Date validity checks with Moscow timezone', () => {
    it('isDateValidForSlot should return true for future dates, false for today and past dates', () => {
      // Mock today as 2026-05-21
      expect(calendar.isDateValidForSlot('2026-05-22')).toBe(true);  // Tomorrow
      expect(calendar.isDateValidForSlot('2026-05-21')).toBe(false); // Today
      expect(calendar.isDateValidForSlot('2026-05-20')).toBe(false); // Yesterday
    });

    it('isPastDate should return true for yesterday, false for today and tomorrow', () => {
      // Mock today as 2026-05-21
      expect(calendar.isPastDate('2026-05-20')).toBe(true);  // Yesterday
      expect(calendar.isPastDate('2026-05-21')).toBe(false); // Today
      expect(calendar.isPastDate('2026-05-22')).toBe(false); // Tomorrow
    });
  });

  describe('getSlotsForDate', () => {
    it('should filter slots by matching date', () => {
      calendar.slots = [
        { id: '1', date: '2026-05-21', start_time: '10:00:00' },
        { id: '2', date: '2026-05-22', start_time: '12:00:00' },
        { id: '3', date: '2026-05-21', start_time: '14:00:00' },
      ];

      const slots = calendar.getSlotsForDate('2026-05-21');
      expect(slots.length).toBe(2);
      expect(slots[0].id).toBe('1');
      expect(slots[1].id).toBe('3');
    });
  });

  describe('Calendar render and listeners', () => {
    it('should render correct number of days and slots', () => {
      // Set current month to May 2026 (Month 4 index, since Jan is 0)
      calendar.currentMonth = 4;
      calendar.currentYear = 2026;

      const slots = [
        { id: 'slot-1', date: '2026-05-22', start_time: '10:00:00', end_time: '11:00:00', is_available: true }
      ];

      calendar.render(slots);

      const container = document.getElementById('calendarContainer');
      expect(document.getElementById('currentMonth').textContent).toBe('Май 2026');
      expect(container.querySelectorAll('.calendar-day').length).toBeGreaterThanOrEqual(31);
      
      // Verify slot is rendered
      const renderedSlot = container.querySelector('[data-slot-id="slot-1"]');
      expect(renderedSlot).not.toBeNull();
      expect(renderedSlot.textContent.trim()).toBe('10:00-11:00');
    });

    it('should trigger correct modals when clicking slots in non-admin mode', () => {
      calendar.isAdminMode = false;
      calendar.slots = [
        { id: 'slot-future-avail', date: '2026-05-22', start_time: '10:00:00', end_time: '11:00:00', is_available: true },
        { id: 'slot-future-booked', date: '2026-05-22', start_time: '11:00:00', end_time: '12:00:00', is_available: false, booking: { company_name: 'Test' } },
        { id: 'slot-past-avail', date: '2026-05-20', start_time: '10:00:00', end_time: '11:00:00', is_available: true },
      ];
      calendar.render(calendar.slots);

      // 1. Click future available slot -> open booking modal
      const slot1 = document.querySelector('[data-slot-id="slot-future-avail"]');
      slot1.click();
      expect(schedulerMock.modals.openBookingModal).toHaveBeenCalledWith('slot-future-avail');

      // 2. Click future booked slot -> open view booking modal
      const slot2 = document.querySelector('[data-slot-id="slot-future-booked"]');
      slot2.click();
      expect(schedulerMock.modals.openViewSlotModal).toHaveBeenCalledWith('slot-future-booked');

      // 3. Click past available slot -> show past date error, do not open booking
      const slot3 = document.querySelector('[data-slot-id="slot-past-avail"]');
      const errSpy = vi.spyOn(Utils, 'showError').mockImplementation(() => {});
      slot3.click();
      expect(errSpy).toHaveBeenCalledWith('Нельзя записываться на слоты в текущие и прошедшие даты');
    });

    it('should trigger openEditModal when clicking slots in admin mode', () => {
      calendar.isAdminMode = true;
      calendar.slots = [
        { id: 'slot-1', date: '2026-05-22', start_time: '10:00:00', end_time: '11:00:00', is_available: true }
      ];
      calendar.render(calendar.slots);

      const slot = document.querySelector('[data-slot-id="slot-1"]');
      slot.click();
      expect(schedulerMock.modals.openEditModal).toHaveBeenCalledWith('slot-1');
    });
  });
});
