import { describe, it, expect, beforeEach, vi } from 'vitest';
import { Scheduler } from '../main.js';
import { Utils } from '../utils.js';

describe('main.js Scheduler search coordination', () => {
  let scheduler;

  beforeEach(() => {
    // Mock fetch and bootstrap
    globalThis.fetch = vi.fn();
    globalThis.bootstrap = {
      Modal: {
        getOrCreateInstance: vi.fn().mockReturnValue({
          show: vi.fn(),
          hide: vi.fn(),
        }),
      },
    };

    // Setup DOM elements needed for search and scheduler rendering
    document.body.innerHTML = `
      <input type="text" id="slotSearchInput" />
      <h5 id="calendarTitle">📅 Календарь слотов</h5>
      <div id="calendarContainer"></div>
      <div id="monthPagination" style="display: flex;"></div>
      <div id="adminControls" style="display: none;"></div>
      <div id="adminCopyPanel" style="display: none;"></div>
      
      <div id="searchResultsContainer" style="display: none;">
        <span id="searchResultsCount">Найдено: 0</span>
        <button id="closeSearchBtn"></button>
        <div id="searchResultsList"></div>
      </div>

      <div id="currentMonth"></div>
      <button id="copySchedule" disabled></button>
      <button id="clearSelection"></button>
      <input id="sourceDate" value="2026-05-21" />
      <div id="selectedDaysInfo"></div>
    `;

    // Spy on Utils.getCurrentTimeMSK to return a fixed date
    vi.spyOn(Utils, 'getCurrentTimeMSK').mockImplementation(() => {
      return new Date('2026-05-21T12:00:00+03:00');
    });

    scheduler = new Scheduler();
  });

  it('should initialize scheduler with helper components', () => {
    expect(scheduler.api).toBeDefined();
    expect(scheduler.calendar).toBeDefined();
    expect(scheduler.modals).toBeDefined();
    expect(scheduler.isAuthenticated).toBe(false);
  });

  describe('handleSearch', () => {
    it('should restore calendar view when search query is empty', async () => {
      // Mock API call to getSlots
      const mockSlots = [
        { id: '1', date: '2026-05-21', start_time: '10:00:00', end_time: '11:00:00', is_available: true }
      ];
      vi.spyOn(scheduler.api, 'getSlots').mockResolvedValue(mockSlots);

      // Perform search with empty string
      await scheduler.handleSearch('');

      // Verify DOM changes
      expect(document.getElementById('calendarContainer').style.display).toBe('block');
      expect(document.getElementById('monthPagination').style.display).toBe('flex');
      expect(document.getElementById('searchResultsContainer').style.display).toBe('none');
      expect(document.getElementById('calendarTitle').textContent).toBe('📅 Календарь слотов');
      
      // Verify API was called with date range
      expect(scheduler.api.getSlots).toHaveBeenCalled();
    });

    it('should show search results when search query is active', async () => {
      const mockSlots = [
        { id: '1', date: '2026-05-22', start_time: '10:00:00', end_time: '11:00:00', is_available: true }
      ];
      vi.spyOn(scheduler.api, 'getSlots').mockResolvedValue(mockSlots);

      await scheduler.handleSearch('Google');

      // Verify DOM changes
      expect(document.getElementById('calendarContainer').style.display).toBe('none');
      expect(document.getElementById('monthPagination').style.display).toBe('none');
      expect(document.getElementById('searchResultsContainer').style.display).toBe('block');
      expect(document.getElementById('calendarTitle').textContent).toBe('📅 Календарь слотов');
      expect(document.getElementById('searchResultsCount').textContent).toBe('Найдено: 1');

      // Verify search results are rendered inside #searchResultsList
      const list = document.getElementById('searchResultsList');
      expect(list.innerHTML).toContain('10:00 - 11:00');
      expect(list.innerHTML).toContain('search-slot-item');

      // Verify API was called with the search term
      expect(scheduler.api.getSlots).toHaveBeenCalledWith(null, null, 'Google');
    });

    it('should show "no results" message when search returns empty list', async () => {
      vi.spyOn(scheduler.api, 'getSlots').mockResolvedValue([]);

      await scheduler.handleSearch('NonexistentCompany');

      expect(document.getElementById('searchResultsCount').textContent).toBe('Найдено: 0');
      const list = document.getElementById('searchResultsList');
      expect(list.innerHTML).toContain('не найдены');
      expect(list.innerHTML).toContain('NonexistentCompany');
    });

    it('should clear input and search results when closeSearchBtn is clicked', async () => {
      const mockSlots = [
        { id: '1', date: '2026-05-22', start_time: '10:00:00', end_time: '11:00:00', is_available: true }
      ];
      vi.spyOn(scheduler.api, 'getSlots').mockResolvedValue(mockSlots);

      scheduler.setupEventListeners();

      const searchInput = document.getElementById('slotSearchInput');
      searchInput.value = 'Google';

      const handleSearchSpy = vi.spyOn(scheduler, 'handleSearch').mockResolvedValue();

      const closeSearchBtn = document.getElementById('closeSearchBtn');
      closeSearchBtn.click();

      expect(searchInput.value).toBe('');
      expect(handleSearchSpy).toHaveBeenCalledWith('');
    });
  });
});
