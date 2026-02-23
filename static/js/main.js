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
        this.showAdminAuthModal();
      }
    });
    EventManager.onClick("reglamentBtn", () => this.openReglamentModal());

    EventManager.onSubmit("adminAuthForm", async () => {
      await this.handleAdminAuth();
    });

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
  }

  async init() {
    this.setupEventListeners();

    await this.checkAuthStatus();

    await this.loadSlots();
    this.startMSKClock();
  }

  showAdminAuthModal() {
    const modalElement = DOMHelper.get("adminAuthModal");
    const modal = new bootstrap.Modal(modalElement);
    modal.show();
  }

  openReglamentModal() {
    const modalElement = DOMHelper.get("reglamentModal");
    const modal = new bootstrap.Modal(modalElement);
    modal.show();
  }

  async handleAdminAuth() {
    const password = document.getElementById("password").value;

    try {
      const response = await fetch("/api/auth/admin", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({ password }),
      });

      if (response.ok) {
        const data = await response.json();

        this.isAuthenticated = true;
        this.isAdminMode = true;

        const authModal = bootstrap.Modal.getInstance(
          document.getElementById("adminAuthModal"),
        );
        authModal.hide();

        ButtonHelper.setIcon("adminToggle", "👤", "Обычный режим");
        DOMHelper.addClass("adminControls", "show");

        this.calendar.setAdminMode(true);
        Utils.showSuccess(data.message || "Авторизация прошла успешно");
      } else {
        const error = await response.json();
        Utils.showError(error.error || "Ошибка авторизации");
        this.shakePasswordField();
      }
    } catch (error) {
      console.error("Auth error:", error);
      Utils.showError("Ошибка сети при авторизации");
      this.shakePasswordField();
    }
  }

  shakePasswordField() {
    document.getElementById("password").value = "";
    document.getElementById("password").focus();

    const passwordField = document.getElementById("password");
    passwordField.classList.add("shake");
    setTimeout(() => {
      passwordField.classList.remove("shake");
    }, 500);
  }

  async checkAuthStatus() {
    try {
      const response = await fetch("/api/auth/check", {
        credentials: "include",
      });

      if (response.ok) {
        const data = await response.json();
        if (data.authenticated) {
          this.isAuthenticated = true;
          this.isAdminMode = true;

          ButtonHelper.setIcon("adminToggle", "👤", "Обычный режим");
          DOMHelper.addClass("adminControls", "show");
          this.calendar.setAdminMode(true);
          return;
        }
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
    if (adminControls) {
      adminControls.classList.remove("show");
    }

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
