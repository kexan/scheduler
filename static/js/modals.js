class Modals {
  constructor(scheduler) {
    this.scheduler = scheduler;
    this.setupEventListeners();
  }

  hideModal(modalId) {
    const modalElement = DOMHelper.get(modalId);
    const modal = bootstrap.Modal.getInstance(modalElement);
    if (!modal) {
      document.querySelectorAll(".modal-backdrop").forEach((el) => el.remove());
      document.body.classList.remove("modal-open");
      document.body.style.removeProperty("overflow");
      return;
    }

    document.querySelectorAll(".modal-backdrop").forEach((el) => el.remove());

    const handleHidden = () => {
      setTimeout(() => {
        document
          .querySelectorAll(".modal-backdrop")
          .forEach((el) => el.remove());
        document.body.classList.remove("modal-open");
        document.body.style.removeProperty("overflow");

        if (modalId === "editSlotModal") {
          bootstrap.Modal.getInstance(modalElement)?.dispose();
        }
      }, 50);
    };

    modalElement.addEventListener("hidden.bs.modal", handleHidden);
    modal.hide();
  }

  setupEventListeners() {
    EventManager.onSubmit("quickSlotForm", () => this.createQuickSlot());
    EventManager.onSubmit("bookingForm", () => this.bookSlot());
    EventManager.onSubmit("editSlotForm", () => this.updateSlot());

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
      const modal = new bootstrap.Modal(modalElement);
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

    DOMHelper.html("slotDetails", html);

    const modalElement = DOMHelper.get("viewSlotModal");
    if (!modalElement) {
      console.error("View modal element not found");
      Utils.showError("Ошибка: модальное окно не найдено");
      return;
    }

    try {
      const existingModal = bootstrap.Modal.getInstance(modalElement);
      if (existingModal) {
        existingModal.dispose();
      }
      const modal = new bootstrap.Modal(modalElement);
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

    const existingModal = bootstrap.Modal.getInstance(modalElement);
    if (existingModal) {
      existingModal.dispose();
    }

    DOMHelper.text("editSlotIdDisplay", slot.id);
    DOMHelper.set("editSlotId", slot.id);
    DOMHelper.set("editSlotDate", slot.date);
    DOMHelper.set("editSlotStartTime", slot.start_time);
    DOMHelper.set("editSlotEndTime", slot.end_time);

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
      const modal = new bootstrap.Modal(modalElement);
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

    const modalElement = DOMHelper.get("quickSlotModal");
    if (!modalElement) {
      console.error("Quick slot modal element not found");
      Utils.showError("Ошибка: модальное окно не найдено");
      return;
    }

    try {
      const existingModal = bootstrap.Modal.getInstance(modalElement);
      if (existingModal) {
        existingModal.dispose();
      }
      const modal = new bootstrap.Modal(modalElement);
      modal.show();
    } catch (error) {
      console.error("Error creating quick slot modal:", error);
      Utils.showError("Ошибка при открытии модального окна");
    }
  }

  async createQuickSlot() {
    const slotData = {
      date: DOMHelper.get("quickSlotDate").value,
      start_time: DOMHelper.get("quickSlotStartTime").value + ":00",
      end_time: DOMHelper.get("quickSlotEndTime").value + ":00",
    };

    const result = await this.scheduler.api.createSlot(slotData);
    if (result) {
      this.hideModal("quickSlotModal");
      FormHelper.reset("quickSlotForm");
      this.scheduler.loadSlots();
    }
  }

  async updateSlot() {
    const slotId = DOMHelper.get("editSlotId").value;

    const slotData = {
      date: DOMHelper.get("editSlotDate").value,
      start_time: DOMHelper.get("editSlotStartTime").value,
      end_time: DOMHelper.get("editSlotEndTime").value,
    };

    const companyName = DOMHelper.get("editCompanyName").value.trim();
    const adminEmail = DOMHelper.get("editAdminEmail").value.trim();
    const companyId = DOMHelper.get("editCompanyId").value.trim();
    const downloadEmail = DOMHelper.get("editDownloadEmail").value.trim();

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
      this.hideModal("editSlotModal");
      this.scheduler.loadSlots();
    }
  }

  async bookSlot() {
    const form = DOMHelper.get("bookingForm");
    const slotId = form.dataset.slotId;

    const bookingData = {
      company_name: DOMHelper.get("companyName").value,
      admin_email: DOMHelper.get("adminEmail").value,
      company_id: DOMHelper.get("companyId").value,
      download_email: DOMHelper.get("downloadEmail").value,
    };

    const result = await this.scheduler.api.bookSlot(slotId, bookingData);
    if (result) {
      this.hideModal("bookingModal");
      FormHelper.reset("bookingForm");
      delete form.dataset.slotId;
      this.scheduler.loadSlots();
    }
  }

  async deleteSlot() {
    const slotId = DOMHelper.get("editSlotId").value;
    const result = await this.scheduler.api.deleteSlot(slotId);
    if (result) {
      this.hideModal("editSlotModal");
      this.scheduler.loadSlots();
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
      const modal = new bootstrap.Modal(modalElement);
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
        `<div class="alert alert-success">Загружено проектов: ${projects.length}</div>`,
      );
    } catch (error) {
      projectSelect.innerHTML = '<option value="">Ошибка загрузки</option>';
      DOMHelper.html(
        "connectionResult",
        `<div class="alert alert-danger"><strong>❌ Ошибка подключения</strong><br>${error.message}</div>`,
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
    select.innerHTML = `<option value="">${placeholder}</option>`;

    items.forEach((item) => {
      const option = document.createElement("option");
      option.value = item[valueField];
      option.textContent = item[textField];
      select.appendChild(option);
    });

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

    try {
      await this.scheduler.api.updateYougileSettings(settings);
      apiTokenInput.value = "";
      this.hideModal("yougileSettingsModal");
    } catch (error) {
      Utils.showError("Ошибка сохранения настроек: " + error.message);
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
}
