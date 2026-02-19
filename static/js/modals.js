class Modals {
  constructor(scheduler) {
    this.scheduler = scheduler;
    this.setupEventListeners();
  }

  hideModal(modalId) {
    const modalElement = DOMHelper.get(modalId);
    const modal = bootstrap.Modal.getInstance(modalElement);
    if (!modal) {

      document.querySelectorAll('.modal-backdrop').forEach(el => el.remove());
      document.body.classList.remove('modal-open');
      document.body.style.removeProperty('overflow');
      return;
    }


    document.querySelectorAll('.modal-backdrop').forEach(el => el.remove());


    const handleHidden = () => {

      setTimeout(() => {
        document.querySelectorAll('.modal-backdrop').forEach(el => el.remove());
        document.body.classList.remove('modal-open');
        document.body.style.removeProperty('overflow');


        if (modalId === 'editSlotModal') {
          bootstrap.Modal.getInstance(modalElement)?.dispose();
        }
      }, 50);
    };

    modalElement.addEventListener('hidden.bs.modal', handleHidden);
    modal.hide();
  }

  setupEventListeners() {
    EventManager.onSubmit('quickSlotForm', () => this.createQuickSlot());
    EventManager.onSubmit('bookingForm', () => this.bookSlot());
    EventManager.onSubmit('editSlotForm', () => this.updateSlot());

    EventManager.onClick('yougileSettingsBtn', () => this.openYougileSettingsModal());
    EventManager.onClick('saveYougileSettings', () => this.saveYougileSettings());
    EventManager.onClick('testYougileConnection', () => this.testYougileConnection());
    EventManager.onChange('yougileProjectId', () => this.onProjectChange());
    EventManager.onChange('yougileBoardId', () => this.onBoardChange());
  }

  openBookingModal(slotId) {
    const slot = this.scheduler.slots.find(s => s.id === slotId);
    if (!slot) return;

    DOMHelper.text('slotDateTime', Utils.formatDateTimeMSK(slot.date, slot.start_time));


    DOMHelper.set('companyName', '');
    DOMHelper.set('adminEmail', '');
    DOMHelper.set('companyId', '');
    DOMHelper.set('downloadEmail', '');


    const form = DOMHelper.get('bookingForm');
    if (form) {
      form.dataset.slotId = slotId;
    }

    const modalElement = DOMHelper.get('bookingModal');
    if (!modalElement) {
      console.error('Booking modal element not found');
      Utils.showError('Ошибка: модальное окно не найдено');
      return;
    }

    try {
      const modal = new bootstrap.Modal(modalElement);
      modal.show();
    } catch (error) {
      console.error('Error creating booking modal:', error);
      Utils.showError('Ошибка при открытии модального окна');
    }
  }

  openViewSlotModal(slotId) {
    const slot = this.scheduler.slots.find(s => s.id === slotId);
    if (!slot) return;

    let html = `
            <div class="mb-3">
                <strong>ID слота:</strong> ${slot.id}
            </div>
            <div class="mb-3">
                <strong>Дата и время:</strong> ${Utils.formatDateTimeMSK(slot.date, slot.start_time)} - ${Utils.formatTime(slot.end_time)}
            </div>
            <div class="mb-3">
                <strong>Статус:</strong> ${slot.is_available ? 'Доступен' : 'Забронирован'}
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

    DOMHelper.html('slotDetails', html);

    const modalElement = DOMHelper.get('viewSlotModal');
    if (!modalElement) {
      console.error('View modal element not found');
      Utils.showError('Ошибка: модальное окно не найдено');
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
      console.error('Error creating view modal:', error);
      Utils.showError('Ошибка при открытии модального окна');
    }
  }

  openEditModal(slotId) {
    const slot = this.scheduler.slots.find(s => s.id === slotId);
    if (!slot) return;

    const modalElement = DOMHelper.get('editSlotModal');
    if (!modalElement) {
      console.error('Edit modal element not found');
      Utils.showError('Ошибка: модальное окно не найдено');
      return;
    }


    const existingModal = bootstrap.Modal.getInstance(modalElement);
    if (existingModal) {
      existingModal.dispose();
    }


    DOMHelper.text('editSlotIdDisplay', slot.id);
    DOMHelper.set('editSlotId', slot.id);
    DOMHelper.set('editSlotDate', slot.date);
    DOMHelper.set('editSlotStartTime', slot.start_time);
    DOMHelper.set('editSlotEndTime', slot.end_time);


    if (slot.booking) {
      DOMHelper.set('editCompanyName', slot.booking.company_name || '');
      DOMHelper.set('editAdminEmail', slot.booking.admin_email || '');
      DOMHelper.set('editCompanyId', slot.booking.company_id || '');
      DOMHelper.set('editDownloadEmail', slot.booking.download_email || '');
    } else {
      DOMHelper.set('editCompanyName', '');
      DOMHelper.set('editAdminEmail', '');
      DOMHelper.set('editCompanyId', '');
      DOMHelper.set('editDownloadEmail', '');
    }

    try {
      const modal = new bootstrap.Modal(modalElement);
      modal.show();
    } catch (error) {
      console.error('Error creating modal:', error);
      Utils.showError('Ошибка при открытии модального окна');
    }
  }

  openQuickSlotModal(date) {
    DOMHelper.set('quickSlotDate', date);
    DOMHelper.set('quickSlotDateDisplay', Utils.formatDate(date));
    DOMHelper.set('quickSlotStartTime', '');
    DOMHelper.set('quickSlotEndTime', '');

    const modalElement = DOMHelper.get('quickSlotModal');
    if (!modalElement) {
      console.error('Quick slot modal element not found');
      Utils.showError('Ошибка: модальное окно не найдено');
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
      console.error('Error creating quick slot modal:', error);
      Utils.showError('Ошибка при открытии модального окна');
    }
  }

  async createQuickSlot() {
    const slotData = {
      date: DOMHelper.get('quickSlotDate').value,
      start_time: DOMHelper.get('quickSlotStartTime').value + ':00',
      end_time: DOMHelper.get('quickSlotEndTime').value + ':00'
    };

    const result = await this.scheduler.api.createSlot(slotData);
    if (result) {
      this.hideModal('quickSlotModal');
      FormHelper.reset('quickSlotForm');
      this.scheduler.loadSlots();
    }
  }

  async updateSlot() {
    const slotId = DOMHelper.get('editSlotId').value;

    const slotData = {
      date: DOMHelper.get('editSlotDate').value,
      start_time: DOMHelper.get('editSlotStartTime').value,
      end_time: DOMHelper.get('editSlotEndTime').value
    };

    const companyName = DOMHelper.get('editCompanyName').value.trim();
    const adminEmail = DOMHelper.get('editAdminEmail').value.trim();
    const companyId = DOMHelper.get('editCompanyId').value.trim();
    const downloadEmail = DOMHelper.get('editDownloadEmail').value.trim();

    if (companyName || adminEmail || companyId || downloadEmail) {
      slotData.is_available = false;
      slotData.booking = {
        company_name: companyName,
        admin_email: adminEmail,
        company_id: companyId,
        download_email: downloadEmail
      };
    } else {
      slotData.is_available = true;
      slotData.booking = null;
    }

    const result = await this.scheduler.api.updateSlot(slotId, slotData);
    if (result) {
      this.hideModal('editSlotModal');
      this.scheduler.loadSlots();
    }
  }

  async bookSlot() {
    const form = DOMHelper.get('bookingForm');
    const slotId = form.dataset.slotId;

    const bookingData = {
      company_name: DOMHelper.get('companyName').value,
      admin_email: DOMHelper.get('adminEmail').value,
      company_id: DOMHelper.get('companyId').value,
      download_email: DOMHelper.get('downloadEmail').value
    };

    const result = await this.scheduler.api.bookSlot(slotId, bookingData);
    if (result) {
      this.hideModal('bookingModal');
      FormHelper.reset('bookingForm');
      delete form.dataset.slotId;
      this.scheduler.loadSlots();
    }
  }

  async deleteSlot() {
    const slotId = DOMHelper.get('editSlotId').value;
    const result = await this.scheduler.api.deleteSlot(slotId);
    if (result) {
      this.hideModal('editSlotModal');
      this.scheduler.loadSlots();
    }
  }

  async openYougileSettingsModal() {
    try {
      const settings = await this.scheduler.api.getYougileSettings();

      DOMHelper.setChecked('yougileEnabled', settings.enabled);
      DOMHelper.set('yougileApiUrl', settings.api_url);
      if (settings.api_token !== undefined) {
        DOMHelper.set('yougileApiToken', settings.api_token);
      }

      // Store current settings and map
      this.currentYougileSettings = {
        project_id: settings.project_id,
        board_id: settings.board_id,
        column_id: settings.column_id
      };
      this.projectsMap = settings.projects_map || [];

      if (this.projectsMap.length > 0) {
        // Map exists - populate all dropdowns
        this.populateYougileDropdownsFromMap();
        DOMHelper.html('connectionResult', '<div class="alert alert-success">Данные загружены. Нажмите "Проверить подключение" для обновления.</div>');
      } else {
        // No map - show placeholder
        this.resetYougileDropdowns();
        DOMHelper.html('connectionResult', '<div class="alert alert-info">Нажмите "Проверить подключение" для загрузки данных</div>');
      }

      const modalElement = DOMHelper.get('yougileSettingsModal');
      const modal = new bootstrap.Modal(modalElement);
      modal.show();
    } catch (error) {
      Utils.showError('Ошибка загрузки настроек: ' + error.message);
    }
  }

  resetYougileDropdowns() {
    const projectSelect = DOMHelper.get('yougileProjectId');
    const boardSelect = DOMHelper.get('yougileBoardId');
    const columnSelect = DOMHelper.get('yougileColumnId');

    projectSelect.innerHTML = '<option value="">Сначала проверьте подключение</option>';
    boardSelect.innerHTML = '<option value="">Сначала выберите проект</option>';
    columnSelect.innerHTML = '<option value="">Сначала выберите доску</option>';

    projectSelect.disabled = true;
    boardSelect.disabled = true;
    columnSelect.disabled = true;
  }

  populateYougileDropdownsFromMap() {
    // Populate projects
    this.populateSelect('yougileProjectId', this.projectsMap, 'id', 'title', 'Выберите проект');

    // Restore selected project
    if (this.currentYougileSettings.project_id) {
      const projectSelect = DOMHelper.get('yougileProjectId');
      projectSelect.value = this.currentYougileSettings.project_id;

      // Populate boards for selected project
      const project = this.projectsMap.find(p => p.id === this.currentYougileSettings.project_id);
      if (project && project.boards) {
        this.populateSelect('yougileBoardId', project.boards, 'id', 'title', 'Выберите доску');

        // Restore selected board
        if (this.currentYougileSettings.board_id) {
          const boardSelect = DOMHelper.get('yougileBoardId');
          boardSelect.value = this.currentYougileSettings.board_id;

          // Populate columns for selected board
          const board = project.boards.find(b => b.id === this.currentYougileSettings.board_id);
          if (board && board.columns) {
            this.populateSelect('yougileColumnId', board.columns, 'id', 'title', 'Выберите колонку');

            // Restore selected column
            if (this.currentYougileSettings.column_id) {
              const columnSelect = DOMHelper.get('yougileColumnId');
              columnSelect.value = this.currentYougileSettings.column_id;
            }
          }
        }
      }
    }
  }

  populateSelect(selectId, items, valueField, textField, placeholder) {
    const select = DOMHelper.get(selectId);
    select.innerHTML = `<option value="">${placeholder}</option>`;

    items.forEach(item => {
      const option = document.createElement('option');
      option.value = item[valueField];
      option.textContent = item[textField];
      select.appendChild(option);
    });

    select.disabled = false;
  }

  getSelectText(selectId) {
    const select = DOMHelper.get(selectId);
    const selectedOption = select.options[select.selectedIndex];
    return selectedOption ? selectedOption.textContent : '';
  }

  async saveYougileSettings() {
    const apiTokenInput = DOMHelper.get('yougileApiToken');

    const settings = {
      enabled: DOMHelper.isChecked('yougileEnabled'),
      api_url: DOMHelper.get('yougileApiUrl').value,
      project_id: DOMHelper.get('yougileProjectId').value,
      project_title: this.getSelectText('yougileProjectId'),
      board_id: DOMHelper.get('yougileBoardId').value,
      board_title: this.getSelectText('yougileBoardId'),
      column_id: DOMHelper.get('yougileColumnId').value,
      column_title: this.getSelectText('yougileColumnId'),
      projects_map: this.projectsMap || []
    };

    if (apiTokenInput.value) {
      settings.api_token = apiTokenInput.value;
    }

    try {
      await this.scheduler.api.updateYougileSettings(settings);
      apiTokenInput.value = '';
      this.hideModal('yougileSettingsModal');
    } catch (error) {
      Utils.showError('Ошибка сохранения настроек: ' + error.message);
    }
  }

  async testYougileConnection() {
    DOMHelper.html('connectionResult', '<div class="spinner-border spinner-border-sm me-2"></div>Загрузка данных...');

    try {
      const result = await this.scheduler.api.testYougileConnection();

      if (result.success) {
        // Save the map locally
        this.projectsMap = result.projects_map || [];

        // Populate all dropdowns from the new map
        this.populateYougileDropdownsFromMap();

        DOMHelper.html('connectionResult', `
          <div class="alert alert-success">
            <strong>✅ Данные загружены!</strong><br>
            Проектов: ${this.projectsMap.length}
          </div>
        `);
      } else {
        DOMHelper.html('connectionResult', `
          <div class="alert alert-danger">
            <strong>❌ Ошибка подключения</strong><br>
            ${result.message || result.error}
          </div>
        `);
      }
    } catch (error) {
      DOMHelper.html('connectionResult', `
        <div class="alert alert-danger">
          <strong>❌ Ошибка подключения</strong><br>
          ${error.message}
        </div>
      `);
    }
  }

  onProjectChange() {
    const projectId = DOMHelper.get('yougileProjectId').value;
    if (projectId) {
      const project = this.projectsMap.find(p => p.id === projectId);
      if (project && project.boards) {
        this.populateSelect('yougileBoardId', project.boards, 'id', 'title', 'Выберите доску');
        // Reset column dropdown
        const columnSelect = DOMHelper.get('yougileColumnId');
        columnSelect.innerHTML = '<option value="">Выберите колонку</option>';
        columnSelect.disabled = true;
      }
    } else {
      // Reset board and column dropdowns
      const boardSelect = DOMHelper.get('yougileBoardId');
      const columnSelect = DOMHelper.get('yougileColumnId');
      boardSelect.innerHTML = '<option value="">Сначала выберите проект</option>';
      columnSelect.innerHTML = '<option value="">Сначала выберите доску</option>';
      boardSelect.disabled = true;
      columnSelect.disabled = true;
    }
  }

  onBoardChange() {
    const projectId = DOMHelper.get('yougileProjectId').value;
    const boardId = DOMHelper.get('yougileBoardId').value;

    if (projectId && boardId) {
      const project = this.projectsMap.find(p => p.id === projectId);
      if (project) {
        const board = project.boards.find(b => b.id === boardId);
        if (board && board.columns) {
          this.populateSelect('yougileColumnId', board.columns, 'id', 'title', 'Выберите колонку');
        }
      }
    } else {
      // Reset column dropdown
      const columnSelect = DOMHelper.get('yougileColumnId');
      columnSelect.innerHTML = '<option value="">Сначала выберите доску</option>';
      columnSelect.disabled = true;
    }
  }
}
