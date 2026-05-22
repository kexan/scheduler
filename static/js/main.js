import { Utils, DOMHelper, ButtonHelper, EventManager } from './utils.js';
import { Api } from './api.js';
import { Calendar } from './calendar.js';
import { Modals } from './modals.js';
import { Templates } from './templates.js';

class Scheduler {
  constructor() {
    this.slots = [];
    this.api = new Api();
    this.calendar = new Calendar(this);
    this.modals = new Modals(this);
    this.isAdminMode = false;
    this.isAuthenticated = false;
  }

  setupEventListeners() {
    EventManager.onClick("prevMonth", () => this.calendar.prevMonth());
    EventManager.onClick("nextMonth", () => this.calendar.nextMonth());
    EventManager.onClick("adminToggle", () => {
      if (this.isAuthenticated) {
        this.toggleAdminMode();
      } else {
        this.modals.openAdminAuthModal();
      }
    });
    EventManager.onClick("reglamentBtn", () => this.modals.openReglamentModal());


    const toggleBtn = DOMHelper.get("toggleCopyPanel");
    if (toggleBtn) {
      EventManager.onClick("toggleCopyPanel", () => {
        const panel = DOMHelper.get("adminCopyPanel");
        const isVisible = panel.style.display !== "none";

        DOMHelper.toggle("adminCopyPanel", !isVisible);

        if (!isVisible) {
          ButtonHelper.setState(
            "toggleCopyPanel",
            "❌ Закрыть копирование",
            "btn-outline-info",
            "btn-outline-danger",
          );
          this.calendar.setCopyMode(true);
          this.calendar.updateCopyButton();
        } else {
          ButtonHelper.setState(
            "toggleCopyPanel",
            "📋 Копирование расписания",
            "btn-outline-danger",
            "btn-outline-info",
          );
          this.calendar.setCopyMode(false);
        }
      });
    }

    EventManager.onChange("sourceDate", () => this.calendar.updateCopyButton());
    EventManager.onClick(
      "confirmCopy",
      async () => await this.calendar.copySchedule(),
    );
    EventManager.onSubmit(
      "copyForm",
      async () => await this.calendar.copySchedule(),
    );

    document.getElementById("copySchedule").addEventListener("click", () => {
      this.calendar.copySchedule();
    });

    document.getElementById("clearSelection").addEventListener("click", () => {
      this.calendar.clearSelection();
    });

    document.addEventListener("keydown", (e) => {
      if (document.querySelector(".modal.show")) return;
      if (
        document.activeElement.tagName === "INPUT" ||
        document.activeElement.tagName === "TEXTAREA"
      )
        return;

      if (e.key === "ArrowLeft") {
        this.calendar.prevMonth();
      } else if (e.key === "ArrowRight") {
        this.calendar.nextMonth();
      }
    });

    const searchInput = document.getElementById("slotSearchInput");
    if (searchInput) {
      let searchDebounceTimeout;
      searchInput.addEventListener("input", (e) => {
        clearTimeout(searchDebounceTimeout);
        const query = e.target.value.trim();
        searchDebounceTimeout = setTimeout(() => {
          this.handleSearch(query);
        }, 300);
      });

      searchInput.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
          searchInput.value = "";
          this.handleSearch("");
        }
      });
    }

    const closeSearchBtn = document.getElementById("closeSearchBtn");
    if (closeSearchBtn) {
      closeSearchBtn.addEventListener("click", () => {
        const searchInput = document.getElementById("slotSearchInput");
        if (searchInput) {
          searchInput.value = "";
        }
        this.handleSearch("");
      });
    }
  }

  async init() {
    this.setupEventListeners();
    Utils.checkTimezoneDifference();

    await this.checkAuthStatus();

    await this.loadSlots();
    this.startMSKClock();
  }

  onAuthSuccess(data) {
    this.isAuthenticated = true;
    this.isAdminMode = true;

    ButtonHelper.setIcon("adminToggle", "👤", "Обычный режим");
    DOMHelper.addClass("adminControls", "show");

    this.calendar.setAdminMode(true);
    Utils.showSuccess(data.message || "Авторизация прошла успешно");
  }

  async checkAuthStatus() {
    try {
      const data = await this.api.checkAuth();
      if (data.authenticated) {
        this.isAuthenticated = true;
        this.isAdminMode = true;

        ButtonHelper.setIcon("adminToggle", "👤", "Обычный режим");
        DOMHelper.addClass("adminControls", "show");
        this.calendar.setAdminMode(true);
        return;
      }

      this.logout();
    } catch (error) {
      console.error("Error checking auth status:", error);
      this.logout();
    }
  }

  logout() {
    this.isAuthenticated = false;
    this.isAdminMode = false;

    ButtonHelper.setIcon("adminToggle", "🗝️", "Режим администратора");
    DOMHelper.removeClass("adminControls", "show");
    DOMHelper.hide("adminCopyPanel");
    ButtonHelper.setState(
      "toggleCopyPanel",
      "📋 Копирование расписания",
      "btn-outline-danger",
      "btn-outline-info",
    );

    const adminPanel = document.getElementById("adminCopyPanel");
    if (adminPanel) {
      adminPanel.style.display = "none";
    }

    this.calendar.setAdminMode(false);
  }

  toggleAdminMode() {
    if (!this.isAuthenticated) {
      this.showAdminAuthModal();
      return;
    }

    this.isAdminMode = !this.isAdminMode;

    if (this.isAdminMode) {
      ButtonHelper.setIcon("adminToggle", "👤", "Обычный режим");
      DOMHelper.addClass("adminControls", "show");
    } else {
      ButtonHelper.setIcon("adminToggle", "🗝️", "Режим администратора");
      DOMHelper.removeClass("adminControls", "show");
      DOMHelper.hide("adminCopyPanel");
      ButtonHelper.setState(
        "toggleCopyPanel",
        "📋 Копирование расписания",
        "btn-outline-danger",
        "btn-outline-info",
      );
      this.calendar.setCopyMode(false);
      this.calendar.clearSelection();
    }

    this.calendar.setAdminMode(this.isAdminMode);
  }

  async loadSlots() {
    try {
      const searchInput = document.getElementById("slotSearchInput");
      const searchQuery = searchInput ? searchInput.value.trim() : "";
      if (searchQuery) {
        await this.handleSearch(searchQuery);
      } else {
        const { from, to } = this.calendar.getVisibleDateRange();
        this.slots = await this.api.getSlots(from, to);
        this.calendar.render(this.slots);
      }
    } catch (error) {
      console.error("Ошибка загрузки слотов:", error);
      Utils.showError("Не удалось загрузить слоты");
    }
  }

  async handleSearch(query) {
    try {
      if (!query) {
        DOMHelper.show("calendarContainer");

        const monthPagination = document.getElementById("monthPagination");
        if (monthPagination) {
          monthPagination.style.display = "flex";
        }

        const adminControls = document.getElementById("adminControls");
        if (adminControls && this.isAdminMode) {
          adminControls.style.display = "flex";
        }

        DOMHelper.hide("searchResultsContainer");

        const calendarTitle = document.getElementById("calendarTitle");
        if (calendarTitle) {
          calendarTitle.textContent = "📅 Календарь слотов";
        }

        const { from, to } = this.calendar.getVisibleDateRange();
        this.slots = await this.api.getSlots(from, to);
        this.calendar.slots = this.slots;
        this.calendar.render(this.slots);
        return;
      }

      this.slots = await this.api.getSlots(null, null, query);
      this.calendar.slots = this.slots;

      DOMHelper.hide("calendarContainer");

      const monthPagination = document.getElementById("monthPagination");
      if (monthPagination) {
        monthPagination.style.display = "none";
      }

      const adminControls = document.getElementById("adminControls");
      if (adminControls) {
        adminControls.style.display = "none";
      }

      const adminCopyPanel = document.getElementById("adminCopyPanel");
      if (adminCopyPanel) {
        adminCopyPanel.style.display = "none";
      }

      DOMHelper.show("searchResultsContainer");

      DOMHelper.text("searchResultsCount", `Найдено: ${this.slots.length}`);

      const resultsList = document.getElementById("searchResultsList");
      if (resultsList) {
        if (this.slots.length === 0) {
          resultsList.innerHTML = `
            <div class="col-12 text-center py-5 text-muted">
              <i class="bi bi-search" style="font-size: 2rem;"></i>
              <p class="mt-2">Слоты по запросу "${Utils.escapeHtml(query)}" не найдены</p>
            </div>
          `;
        } else {
          const sortedSlots = [...this.slots].sort((a, b) => {
            if (a.date !== b.date) return a.date.localeCompare(b.date);
            return a.start_time.localeCompare(b.start_time);
          });

          resultsList.innerHTML = sortedSlots
            .map((slot) => {
              let statusText = "Доступен";
              let slotClass = "";
              const isDateValid = this.calendar.isDateValidForSlot(slot.date);

              if (!slot.is_available) {
                if (slot.completed) {
                  statusText = "Выполнен";
                  slotClass = "completed";
                } else {
                  statusText = "Занят";
                  slotClass = "in-progress";
                }
              } else if (!isDateValid) {
                statusText = "Прошедший";
                slotClass = "past-date-available";
              }

              return Templates.searchResultSlot(slot, slotClass, statusText, !isDateValid);
            })
            .join("");
        }
      }
    } catch (error) {
      console.error("Ошибка при поиске:", error);
      Utils.showError("Не удалось выполнить поиск");
    }
  }

  startMSKClock() {
    Utils.updateCurrentTimeMSK();
    setInterval(() => {
      Utils.updateCurrentTimeMSK();
    }, 1000);
  }
}

let scheduler;

document.addEventListener("DOMContentLoaded", async () => {
  scheduler = new Scheduler();
  await scheduler.init();
});

export { Scheduler, scheduler };

