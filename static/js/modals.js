import { Utils, DOMHelper, FormHelper, EventManager } from './utils.js';
import { Templates } from './templates.js';

class Modals {
  constructor(scheduler) {
    this.scheduler = scheduler;
    this.setupEventListeners();
  }

  hideModal(modalId) {
    const modalElement = DOMHelper.get(modalId);
    if (!modalElement) return;
    const modal = bootstrap.Modal.getOrCreateInstance(modalElement);
    modal.hide();
  }

  setupEventListeners() {
    EventManager.onSubmit("quickSlotForm", () => this.createQuickSlot());
    EventManager.onSubmit("bookingForm", () => this.bookSlot());
    EventManager.onSubmit("editSlotForm", () => this.updateSlot());
    EventManager.onClick("deleteSlotBtn", () => this.deleteSlot());
    EventManager.onSubmit("adminAuthForm", () => this.handleAdminAuth());

    EventManager.onClick("yougileSettingsBtn", () =>
      this.openYougileSettingsModal(),
    );
    EventManager.onClick("saveYougileSettings", () =>
      this.saveYougileSettings(),
    );
    EventManager.onClick("testYougileConnection", () =>
      this.loadProjects().catch(() => {}),
    );

    EventManager.onChange("yougileProjectId", () =>
      this.onProjectChange().catch(() => {}),
    );
    EventManager.onChange("yougileBoardId", () =>
      this.onBoardChange().catch(() => {}),
    );
    EventManager.onChange("yougileAssigneeId", () => this.onAssigneeChange());
  }

  openBookingModal(slotId) {
    const slot = this.scheduler.slots.find((s) => s.id === slotId);
    if (!slot) return;

    DOMHelper.text(
      "slotDateTime",
      Utils.formatDateTimeMSK(slot.date, slot.start_time),
    );

    DOMHelper.set("companyName", "");
    DOMHelper.set("adminEmail", "");
    DOMHelper.set("companyId", "");
    DOMHelper.set("downloadEmail", "");
    Utils.clearValidation("bookingForm");

    const form = DOMHelper.get("bookingForm");
    if (form) {
      form.dataset.slotId = slotId;
    }

    const modalElement = DOMHelper.get("bookingModal");
    if (!modalElement) {
      console.error("Booking modal element not found");
      Utils.showError("Ошибка: модальное окно не найдено");
      return;
    }

    try {
      const modal = bootstrap.Modal.getOrCreateInstance(modalElement);
      modal.show();
    } catch (error) {
      console.error("Error creating booking modal:", error);
      Utils.showError("Ошибка при открытии модального окна");
    }
  }

  openViewSlotModal(slotId) {
    const slot = this.scheduler.slots.find((s) => s.id === slotId);
    if (!slot) return;

    let statusText;
    if (slot.is_available) {
      statusText = "Доступен";
    } else if (slot.completed) {
      statusText = "Выполнен";
    } else {
      statusText = "Забронирован";
    }

    DOMHelper.html("slotDetails", Templates.viewSlotModalContent(slot, statusText));

    const modalElement = DOMHelper.get("viewSlotModal");
    if (!modalElement) {
      console.error("View modal element not found");
      Utils.showError("Ошибка: модальное окно не найдено");
      return;
    }

    try {
      const modal = bootstrap.Modal.getOrCreateInstance(modalElement);
      modal.show();
    } catch (error) {
      console.error("Error creating view modal:", error);
      Utils.showError("Ошибка при открытии модального окна");
    }
  }

  openEditModal(slotId) {
    const slot = this.scheduler.slots.find((s) => s.id === slotId);
    if (!slot) return;

    const modalElement = DOMHelper.get("editSlotModal");
    if (!modalElement) {
      console.error("Edit modal element not found");
      Utils.showError("Ошибка: модальное окно не найдено");
      return;
    }

    // No need to dispose here, getOrCreateInstance handles it safely.

    DOMHelper.text("editSlotIdDisplay", slot.id);
    DOMHelper.set("editSlotId", slot.id);
    DOMHelper.set("editSlotDate", slot.date);
    DOMHelper.set("editSlotStartTime", slot.start_time);
    DOMHelper.set("editSlotEndTime", slot.end_time);
    Utils.clearValidation("editSlotForm");

    if (slot.yougile_task_id) {
      DOMHelper.text("editYougileTaskIdDisplay", slot.yougile_task_id);
      DOMHelper.show("editYougileTaskContainer");
      DOMHelper.show("editCompletedContainer");
      DOMHelper.setChecked("editCompleted", slot.completed || false);
    } else {
      DOMHelper.hide("editYougileTaskContainer");
      DOMHelper.hide("editCompletedContainer");
    }

    if (slot.booking) {
      DOMHelper.set("editCompanyName", slot.booking.company_name || "");
      DOMHelper.set("editAdminEmail", slot.booking.admin_email || "");
      DOMHelper.set("editCompanyId", slot.booking.company_id || "");
      DOMHelper.set("editDownloadEmail", slot.booking.download_email || "");
    } else {
      DOMHelper.set("editCompanyName", "");
      DOMHelper.set("editAdminEmail", "");
      DOMHelper.set("editCompanyId", "");
      DOMHelper.set("editDownloadEmail", "");
    }

    try {
      const modal = bootstrap.Modal.getOrCreateInstance(modalElement);
      modal.show();
    } catch (error) {
      console.error("Error creating modal:", error);
      Utils.showError("Ошибка при открытии модального окна");
    }
  }

  openQuickSlotModal(date) {
    DOMHelper.set("quickSlotDate", date);
    DOMHelper.set("quickSlotDateDisplay", Utils.formatDate(date));
    DOMHelper.set("quickSlotStartTime", "");
    DOMHelper.set("quickSlotEndTime", "");
    Utils.clearValidation("quickSlotForm");

    const modalElement = DOMHelper.get("quickSlotModal");
    if (!modalElement) {
      console.error("Quick slot modal element not found");
      Utils.showError("Ошибка: модальное окно не найдено");
      return;
    }

    try {
      const modal = bootstrap.Modal.getOrCreateInstance(modalElement);
      modal.show();
    } catch (error) {
      console.error("Error creating quick slot modal:", error);
      Utils.showError("Ошибка при открытии модального окна");
    }
  }

  async createQuickSlot() {
    const form = DOMHelper.get("quickSlotForm");
    const submitBtn = form ? form.querySelector('button[type="submit"]') : null;

    const startVal = DOMHelper.get("quickSlotStartTime").value;
    const endVal = DOMHelper.get("quickSlotEndTime").value;

    const startValid = Utils.validateField("quickSlotStartTime", (val) => val.length > 0, "Укажите время начала");
    const endValid = Utils.validateField("quickSlotEndTime", (val) => val.length > 0, "Укажите время окончания");

    let timeRangeValid = true;
    if (startValid && endValid) {
      timeRangeValid = Utils.validateField("quickSlotEndTime", () => startVal < endVal, "Время окончания должно быть позже времени начала");
    }

    if (!startValid || !endValid || !timeRangeValid) {
      return;
    }

    if (submitBtn) submitBtn.disabled = true;

    try {
      const slotData = {
        date: DOMHelper.get("quickSlotDate").value,
        start_time: DOMHelper.get("quickSlotStartTime").value + ":00",
        end_time: DOMHelper.get("quickSlotEndTime").value + ":00",
      };

      const result = await this.scheduler.api.createSlot(slotData);
      if (result) {
        Utils.showSuccess("Слот создан успешно");
        this.hideModal("quickSlotModal");
        FormHelper.reset("quickSlotForm");
        this.scheduler.loadSlots();
      }
    } catch (error) {
      Utils.showError(error.message);
    } finally {
      if (submitBtn) submitBtn.disabled = false;
    }
  }

  async updateSlot() {
    const form = DOMHelper.get("editSlotForm");
    const submitBtn = form ? form.querySelector('button[type="submit"]') : null;
    const deleteBtn = form ? form.querySelector('button.btn-danger') : null;

    const startVal = DOMHelper.get("editSlotStartTime").value;
    const endVal = DOMHelper.get("editSlotEndTime").value;

    const startValid = Utils.validateField("editSlotStartTime", (val) => val.length > 0, "Укажите время начала");
    const endValid = Utils.validateField("editSlotEndTime", (val) => val.length > 0, "Укажите время окончания");

    let timeRangeValid = true;
    if (startValid && endValid) {
      timeRangeValid = Utils.validateField("editSlotEndTime", () => startVal < endVal, "Время окончания должно быть позже времени начала");
    }

    const companyName = DOMHelper.get("editCompanyName").value.trim();
    const adminEmail = DOMHelper.get("editAdminEmail").value.trim();
    const companyId = DOMHelper.get("editCompanyId").value.trim();
    const downloadEmail = DOMHelper.get("editDownloadEmail").value.trim();

    let bookingFieldsValid = true;
    if (companyName || adminEmail || companyId || downloadEmail) {
      const companyNameValid = Utils.validateField("editCompanyName", (val) => val.length > 0, "Название компании обязательно");
      const adminEmailValid = Utils.validateField("editAdminEmail", (val) => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(val), "Введите корректный email");
      const companyIdValid = Utils.validateField("editCompanyId", (val) => val.length > 0, "ID компании обязателен");
      const downloadEmailValid = Utils.validateField("editDownloadEmail", (val) => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(val), "Введите корректный email");

      bookingFieldsValid = companyNameValid && adminEmailValid && companyIdValid && downloadEmailValid;
    } else {
      DOMHelper.get("editCompanyName").classList.remove("is-valid", "is-invalid");
      DOMHelper.get("editAdminEmail").classList.remove("is-valid", "is-invalid");
      DOMHelper.get("editCompanyId").classList.remove("is-valid", "is-invalid");
      DOMHelper.get("editDownloadEmail").classList.remove("is-valid", "is-invalid");
    }

    if (!startValid || !endValid || !timeRangeValid || !bookingFieldsValid) {
      return;
    }

    if (submitBtn) submitBtn.disabled = true;
    if (deleteBtn) deleteBtn.disabled = true;

    try {
      const slotId = DOMHelper.get("editSlotId").value;

      const slotData = {
        date: DOMHelper.get("editSlotDate").value,
        start_time: DOMHelper.get("editSlotStartTime").value,
        end_time: DOMHelper.get("editSlotEndTime").value,
      };

      if (companyName || adminEmail || companyId || downloadEmail) {
        slotData.is_available = false;
        slotData.booking = {
          company_name: companyName,
          admin_email: adminEmail,
          company_id: companyId,
          download_email: downloadEmail,
        };
      } else {
        slotData.is_available = true;
        slotData.booking = null;
      }

      const completedContainer = DOMHelper.get("editCompletedContainer");
      if (completedContainer && completedContainer.style.display !== "none") {
        slotData.completed = DOMHelper.isChecked("editCompleted");
      }

      const result = await this.scheduler.api.updateSlot(slotId, slotData);
      if (result) {
        Utils.showSuccess("Слот обновлен успешно");
        this.hideModal("editSlotModal");
        this.scheduler.loadSlots();
      }
    } catch (error) {
      Utils.showError(error.message);
    } finally {
      if (submitBtn) submitBtn.disabled = false;
      if (deleteBtn) deleteBtn.disabled = false;
    }
  }

  async bookSlot() {
    const form = DOMHelper.get("bookingForm");
    const submitBtn = form ? form.querySelector('button[type="submit"]') : null;

    const companyNameValid = Utils.validateField("companyName", (val) => val.length > 0, "Название компании обязательно");
    const adminEmailValid = Utils.validateField("adminEmail", (val) => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(val), "Введите корректный email");
    const companyIdValid = Utils.validateField("companyId", (val) => val.length > 0, "ID компании обязателен");
    const downloadEmailValid = Utils.validateField("downloadEmail", (val) => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(val), "Введите корректный email");

    if (!companyNameValid || !adminEmailValid || !companyIdValid || !downloadEmailValid) {
      return;
    }

    if (submitBtn) submitBtn.disabled = true;

    try {
      const slotId = form.dataset.slotId;

      const bookingData = {
        company_name: DOMHelper.get("companyName").value,
        admin_email: DOMHelper.get("adminEmail").value,
        company_id: DOMHelper.get("companyId").value,
        download_email: DOMHelper.get("downloadEmail").value,
      };

      const result = await this.scheduler.api.bookSlot(slotId, bookingData);
      if (result) {
        Utils.showSuccess("Слот успешно забронирован");
        this.hideModal("bookingModal");
        FormHelper.reset("bookingForm");
        delete form.dataset.slotId;
        this.scheduler.loadSlots();
      }
    } catch (error) {
      Utils.showError(error.message);
    } finally {
      if (submitBtn) submitBtn.disabled = false;
    }
  }

  async deleteSlot() {
    if (!confirm("Вы уверены, что хотите удалить этот слот?")) {
      return;
    }

    const form = DOMHelper.get("editSlotForm");
    const submitBtn = form ? form.querySelector('button[type="submit"]') : null;
    const deleteBtn = form ? form.querySelector('button.btn-danger') : null;
    if (submitBtn) submitBtn.disabled = true;
    if (deleteBtn) deleteBtn.disabled = true;

    try {
      const slotId = DOMHelper.get("editSlotId").value;
      await this.scheduler.api.deleteSlot(slotId);
      Utils.showSuccess("Слот удален успешно");
      this.hideModal("editSlotModal");
      this.scheduler.loadSlots();
    } catch (error) {
      Utils.showError(error.message);
    } finally {
      if (submitBtn) submitBtn.disabled = false;
      if (deleteBtn) deleteBtn.disabled = false;
    }
  }

  async openYougileSettingsModal() {
    try {
      const settings = await this.scheduler.api.getYougileSettings();

      DOMHelper.setChecked("yougileEnabled", settings.enabled);
      DOMHelper.set("yougileApiUrl", settings.api_url);

      this.currentYougileSettings = {
        project_id: settings.project_id,
        board_id: settings.board_id,
        column_id: settings.column_id,
        assignee_id: settings.assignee_id,
      };

      this.resetYougileDropdowns();
      DOMHelper.html("connectionResult", "");

      const modalElement = DOMHelper.get("yougileSettingsModal");
      const modal = bootstrap.Modal.getOrCreateInstance(modalElement);
      modal.show();

      if (settings.enabled) {
        await this.loadProjects();
      }
    } catch (error) {
      Utils.showError("Ошибка загрузки настроек: " + error.message);
    }
  }

  resetYougileDropdowns() {
    const boardSelect = DOMHelper.get("yougileBoardId");
    const columnSelect = DOMHelper.get("yougileColumnId");

    DOMHelper.get("yougileProjectId").innerHTML =
      '<option value="">Нажмите "Проверить подключение"</option>';
    boardSelect.innerHTML = '<option value="">Сначала выберите проект</option>';
    columnSelect.innerHTML = '<option value="">Сначала выберите доску</option>';

    DOMHelper.get("yougileProjectId").disabled = true;
    boardSelect.disabled = true;
    columnSelect.disabled = true;

    DOMHelper.get("loadUsersSection").style.display = "none";
    DOMHelper.get("assigneeSection").style.display = "none";
  }

  async loadProjects() {
    const projectSelect = DOMHelper.get("yougileProjectId");
    projectSelect.innerHTML = '<option value="">Загрузка...</option>';
    projectSelect.disabled = true;
    DOMHelper.html("connectionResult", "");

    try {
      const projects = await this.scheduler.api.getProjects();
      this.populateSelect(
        "yougileProjectId",
        projects,
        "id",
        "title",
        "Выберите проект",
      );

      if (this.currentYougileSettings.project_id) {
        projectSelect.value = this.currentYougileSettings.project_id;
        if (projectSelect.value) {
          await this.loadBoards(this.currentYougileSettings.project_id);
        }
      }

      if (this.currentYougileSettings.project_id) {
        await this.loadUsers(this.currentYougileSettings.project_id);
      }

      DOMHelper.html(
        "connectionResult",
        Templates.yougileConnectionSuccess(projects.length),
      );
    } catch (error) {
      projectSelect.innerHTML = '<option value="">Ошибка загрузки</option>';
      DOMHelper.html(
        "connectionResult",
        Templates.yougileConnectionError(error.message),
      );
      throw error;
    }
  }

  async loadBoards(projectId) {
    const boardSelect = DOMHelper.get("yougileBoardId");
    const columnSelect = DOMHelper.get("yougileColumnId");

    boardSelect.innerHTML = '<option value="">Загрузка...</option>';
    boardSelect.disabled = true;
    columnSelect.innerHTML = '<option value="">Сначала выберите доску</option>';
    columnSelect.disabled = true;

    try {
      const boards = await this.scheduler.api.getBoards(projectId);
      this.populateSelect(
        "yougileBoardId",
        boards,
        "id",
        "title",
        "Выберите доску",
      );

      if (this.currentYougileSettings.board_id) {
        boardSelect.value = this.currentYougileSettings.board_id;
        if (boardSelect.value) {
          await this.loadColumns(this.currentYougileSettings.board_id);
        }
      }
    } catch (error) {
      boardSelect.innerHTML = '<option value="">Ошибка загрузки</option>';
      throw error;
    }
  }

  async loadColumns(boardId) {
    const columnSelect = DOMHelper.get("yougileColumnId");
    columnSelect.innerHTML = '<option value="">Загрузка...</option>';
    columnSelect.disabled = true;

    try {
      const columns = await this.scheduler.api.getColumns(boardId);
      this.populateSelect(
        "yougileColumnId",
        columns,
        "id",
        "title",
        "Выберите колонку",
      );

      if (this.currentYougileSettings.column_id) {
        columnSelect.value = this.currentYougileSettings.column_id;
      }
    } catch (error) {
      columnSelect.innerHTML = '<option value="">Ошибка загрузки</option>';
      throw error;
    }
  }

  async loadUsers(projectId) {
    const assigneeSection = DOMHelper.get("assigneeSection");
    assigneeSection.style.display = "none";

    try {
      const users = await this.scheduler.api.loadYougileUsers(projectId);

      if (users && users.length > 0) {
        assigneeSection.style.display = "block";
        this.populateSelect(
          "yougileAssigneeId",
          users,
          "id",
          "name",
          "Выберите исполнителя",
        );

        if (this.currentYougileSettings.assignee_id) {
          DOMHelper.get("yougileAssigneeId").value =
            this.currentYougileSettings.assignee_id;
        }
      }
    } catch (error) {
      // non-critical, silently ignore
    }
  }

  onAssigneeChange() {
    this.currentYougileSettings.assignee_id =
      DOMHelper.get("yougileAssigneeId").value;
  }

  populateSelect(selectId, items, valueField, textField, placeholder) {
    const select = DOMHelper.get(selectId);
    select.innerHTML = Templates.selectOptions(items, valueField, textField, placeholder);
    select.disabled = false;
  }

  getSelectText(selectId) {
    const select = DOMHelper.get(selectId);
    const selectedOption = select.options[select.selectedIndex];
    return selectedOption ? selectedOption.textContent : "";
  }

  async saveYougileSettings() {
    const apiTokenInput = DOMHelper.get("yougileApiToken");
    const assigneeId = DOMHelper.get("yougileAssigneeId").value;

    const settings = {
      enabled: DOMHelper.isChecked("yougileEnabled"),
      api_url: DOMHelper.get("yougileApiUrl").value,
      project_id: DOMHelper.get("yougileProjectId").value || null,
      board_id: DOMHelper.get("yougileBoardId").value || null,
      column_id: DOMHelper.get("yougileColumnId").value || null,
      assignee_id: assigneeId || null,
    };

    if (apiTokenInput.value) {
      settings.api_token = apiTokenInput.value;
    }

    const submitBtn = DOMHelper.get("saveYougileSettings");
    if (submitBtn) submitBtn.disabled = true;

    try {
      await this.scheduler.api.updateYougileSettings(settings);
      Utils.showSuccess("Настройки Yougile сохранены");
      apiTokenInput.value = "";
      this.hideModal("yougileSettingsModal");
    } catch (error) {
      Utils.showError("Ошибка сохранения настроек: " + error.message);
    } finally {
      if (submitBtn) submitBtn.disabled = false;
    }
  }

  async onProjectChange() {
    const projectId = DOMHelper.get("yougileProjectId").value;

    DOMHelper.get("yougileBoardId").innerHTML =
      '<option value="">Сначала выберите проект</option>';
    DOMHelper.get("yougileBoardId").disabled = true;
    DOMHelper.get("yougileColumnId").innerHTML =
      '<option value="">Сначала выберите доску</option>';
    DOMHelper.get("yougileColumnId").disabled = true;
    DOMHelper.get("assigneeSection").style.display = "none";

    this.currentYougileSettings.board_id = null;
    this.currentYougileSettings.column_id = null;

    if (projectId) {
      await Promise.all([
        this.loadBoards(projectId),
        this.loadUsers(projectId),
      ]);
    }
  }

  async onBoardChange() {
    const boardId = DOMHelper.get("yougileBoardId").value;

    DOMHelper.get("yougileColumnId").innerHTML =
      '<option value="">Сначала выберите доску</option>';
    DOMHelper.get("yougileColumnId").disabled = true;

    this.currentYougileSettings.column_id = null;

    if (boardId) {
      await this.loadColumns(boardId);
    }
  }

  openAdminAuthModal() {
    const modalElement = DOMHelper.get("adminAuthModal");
    if (!modalElement) return;
    const modal = bootstrap.Modal.getOrCreateInstance(modalElement);
    modal.show();
  }

  openReglamentModal() {
    const modalElement = DOMHelper.get("reglamentModal");
    if (!modalElement) return;
    const modal = bootstrap.Modal.getOrCreateInstance(modalElement);
    modal.show();
  }

  async handleAdminAuth() {
    const form = DOMHelper.get("adminAuthForm");
    const submitBtn = form ? form.querySelector('button[type="submit"]') : null;
    if (submitBtn) submitBtn.disabled = true;

    const password = document.getElementById("password").value;

    try {
      const data = await this.scheduler.api.adminAuth(password);
      this.hideModal("adminAuthModal");
      this.scheduler.onAuthSuccess(data);
    } catch (error) {
      console.error("Auth error:", error);
      Utils.showError(error.message || "Ошибка авторизации");
      this.shakePasswordField();
    } finally {
      if (submitBtn) submitBtn.disabled = false;
    }
  }

  shakePasswordField() {
    const passwordField = document.getElementById("password");
    if (!passwordField) return;
    passwordField.value = "";
    passwordField.focus();
    passwordField.classList.add("shake");
    setTimeout(() => {
      passwordField.classList.remove("shake");
    }, 500);
  }
}

export { Modals };
