import { Utils, DOMHelper, ButtonHelper, EventManager } from './utils.js';
import { Api } from './api.js';
import { Calendar } from './calendar.js';
import { Modals } from './modals.js';

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
      const { from, to } = this.calendar.getVisibleDateRange();
      this.slots = await this.api.getSlots(from, to);
      this.calendar.render(this.slots);
    } catch (error) {
      console.error("Ошибка загрузки слотов:", error);
      Utils.showError("Не удалось загрузить слоты");
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

