class MigrationScheduler {
    constructor() {
        this.slots = [];
        this.isAdminMode = false;
        this.init();
    }

    init() {
        this.loadSlots();
        this.setupEventListeners();
    }

    setupEventListeners() {
        // Переключение режима администратора
        document.getElementById('adminToggle').addEventListener('click', () => {
            this.toggleAdminMode();
        });

        // Форма создания слота
        document.getElementById('createSlotForm').addEventListener('submit', (e) => {
            e.preventDefault();
            this.createSlot();
        });

        // Форма бронирования
        document.getElementById('bookingForm').addEventListener('submit', (e) => {
            e.preventDefault();
            this.bookSlot();
        });

        // Форма редактирования слота
        document.getElementById('editSlotForm').addEventListener('submit', (e) => {
            e.preventDefault();
            this.updateSlot();
        });
    }

    toggleAdminMode() {
        this.isAdminMode = !this.isAdminMode;
        const adminPanel = document.getElementById('adminPanel');
        const adminToggle = document.getElementById('adminToggle');
        
        if (this.isAdminMode) {
            adminPanel.classList.add('show');
            adminToggle.textContent = '👤 Обычный режим';
            this.renderAdminSlots();
        } else {
            adminPanel.classList.remove('show');
            adminToggle.textContent = '🔧 Режим администратора';
        }
        
        this.renderSlots();
    }

    async loadSlots() {
        try {
            const response = await fetch('/api/slots');
            this.slots = await response.json();
            this.renderSlots();
            if (this.isAdminMode) {
                this.renderAdminSlots();
            }
        } catch (error) {
            console.error('Ошибка загрузки слотов:', error);
            this.showError('Не удалось загрузить слоты');
        }
    }

    renderSlots() {
        const container = document.getElementById('slotsContainer');
        
        if (this.slots.length === 0) {
            container.innerHTML = '<div class="col-12"><p class="text-muted">Нет доступных слотов</p></div>';
            return;
        }

        // Группировка слотов по дате
        const slotsByDate = this.groupSlotsByDate();
        
        container.innerHTML = Object.entries(slotsByDate).map(([date, slots]) => `
            <div class="col-md-6 col-lg-4 mb-3">
                <div class="card">
                    <div class="card-header">
                        <h6 class="mb-0">${this.formatDate(date)}</h6>
                    </div>
                    <div class="card-body p-2">
                        ${slots.map(slot => this.renderSlot(slot)).join('')}
                    </div>
                </div>
            </div>
        `).join('');

        // Добавляем обработчики событий для слотов
        this.attachSlotListeners();
    }

    renderSlot(slot) {
        const statusClass = slot.is_available ? 'slot-available' : 'slot-booked';
        const statusText = slot.is_available ? 'Доступен' : 'Занят';
        const buttonText = slot.is_available ? 'Записаться' : 'Занят';
        const buttonClass = slot.is_available ? 'btn-success' : 'btn-secondary disabled';
        const buttonDisabled = slot.is_available ? '' : 'disabled';

        let bookingInfo = '';
        if (slot.booking) {
            bookingInfo = `
                <div class="booking-info mt-2">
                    <strong>Компания:</strong> ${slot.booking.company_name}<br>
                    <strong>Email:</strong> ${slot.booking.admin_email}<br>
                    <small>Запись создана: ${new Date(slot.booking.created_at).toLocaleString()}</small>
                </div>
            `;
        }

        return `
            <div class="time-slot ${statusClass}" data-slot-id="${slot.id}">
                <div class="d-flex justify-content-between align-items-center">
                    <div>
                        <strong>${this.formatTime(slot.start_time)} - ${this.formatTime(slot.end_time)}</strong>
                        <span class="badge bg-${slot.is_available ? 'success' : 'danger'} ms-2">${statusText}</span>
                    </div>
                    ${slot.is_available ? `
                        <button class="btn ${buttonClass} btn-sm" 
                                onclick="scheduler.openBookingModal('${slot.id}')"
                                ${buttonDisabled}>
                            ${buttonText}
                        </button>
                    ` : ''}
                </div>
                ${bookingInfo}
            </div>
        `;
    }

    renderAdminSlots() {
        const container = document.getElementById('adminSlotsList');
        
        if (this.slots.length === 0) {
            container.innerHTML = '<p class="text-muted">Нет слотов</p>';
            return;
        }

        container.innerHTML = `
            <div class="card">
                <div class="card-header">
                    <h5 class="mb-0">Управление слотами</h5>
                </div>
                <div class="card-body">
                    ${this.slots.map(slot => `
                        <div class="d-flex justify-content-between align-items-center mb-2 p-2 border rounded">
                            <div>
                                <strong>${this.formatDate(slot.date)} ${this.formatTime(slot.start_time)}-${this.formatTime(slot.end_time)}</strong>
                                <span class="badge bg-${slot.is_available ? 'success' : 'danger'} ms-2">
                                    ${slot.is_available ? 'Свободен' : 'Занят'}
                                </span>
                                ${slot.booking ? `<br><small>Компания: ${slot.booking.company_name}</small>` : ''}
                            </div>
                            <div>
                                <button class="btn btn-outline-primary btn-sm me-2" 
                                        onclick="scheduler.openEditModal('${slot.id}')">
                                    ✏️
                                </button>
                                <button class="btn btn-outline-danger btn-sm" 
                                        onclick="scheduler.deleteSlot('${slot.id}')">
                                    🗑️
                                </button>
                            </div>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    }

    groupSlotsByDate() {
        return this.slots.reduce((groups, slot) => {
            const date = slot.date;
            if (!groups[date]) {
                groups[date] = [];
            }
            groups[date].push(slot);
            return groups;
        }, {});
    }

    attachSlotListeners() {
        // Дополнительные обработчики можно добавить здесь
    }

    openBookingModal(slotId) {
        const modal = new bootstrap.Modal(document.getElementById('bookingModal'));
        document.getElementById('bookingForm').dataset.slotId = slotId;
        modal.show();
    }

    openEditModal(slotId) {
        const slot = this.slots.find(s => s.id === slotId);
        if (!slot) return;

        document.getElementById('editSlotId').value = slot.id;
        document.getElementById('editSlotDate').value = slot.date;
        document.getElementById('editSlotStartTime').value = slot.start_time;
        document.getElementById('editSlotEndTime').value = slot.end_time;

        const modal = new bootstrap.Modal(document.getElementById('editSlotModal'));
        modal.show();
    }

    async createSlot() {
        const formData = new FormData(document.getElementById('createSlotForm'));
        
        const slotData = {
            date: formData.get('slotDate') || document.getElementById('slotDate').value,
            start_time: formData.get('slotStartTime') || document.getElementById('slotStartTime').value,
            end_time: formData.get('slotEndTime') || document.getElementById('slotEndTime').value
        };

        try {
            const response = await fetch('/api/slots', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(slotData)
            });

            if (response.ok) {
                this.showSuccess('Слот создан успешно');
                bootstrap.Modal.getInstance(document.getElementById('createSlotModal')).hide();
                document.getElementById('createSlotForm').reset();
                this.loadSlots();
            } else {
                const error = await response.json();
                this.showError(error.error || 'Ошибка при создании слота');
            }
        } catch (error) {
            console.error('Ошибка:', error);
            this.showError('Ошибка при создании слота');
        }
    }

    async bookSlot() {
        const form = document.getElementById('bookingForm');
        const slotId = form.dataset.slotId;
        
        const bookingData = {
            company_name: document.getElementById('companyName').value,
            admin_email: document.getElementById('adminEmail').value,
            company_id: document.getElementById('companyId').value,
            download_email: document.getElementById('downloadEmail').value
        };

        try {
            const response = await fetch(`/api/slots/${slotId}/book`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(bookingData)
            });

            if (response.ok) {
                this.showSuccess('Запись успешно создана!');
                bootstrap.Modal.getInstance(document.getElementById('bookingModal')).hide();
                form.reset();
                delete form.dataset.slotId;
                this.loadSlots();
            } else {
                const error = await response.json();
                this.showError(error.error || 'Ошибка при записи');
            }
        } catch (error) {
            console.error('Ошибка:', error);
            this.showError('Ошибка при записи');
        }
    }

    async updateSlot() {
        const slotId = document.getElementById('editSlotId').value;
        
        const slotData = {
            date: document.getElementById('editSlotDate').value,
            start_time: document.getElementById('editSlotStartTime').value,
            end_time: document.getElementById('editSlotEndTime').value
        };

        try {
            const response = await fetch(`/api/slots/${slotId}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(slotData)
            });

            if (response.ok) {
                this.showSuccess('Слот обновлен успешно');
                bootstrap.Modal.getInstance(document.getElementById('editSlotModal')).hide();
                this.loadSlots();
            } else {
                const error = await response.json();
                this.showError(error.error || 'Ошибка при обновлении слота');
            }
        } catch (error) {
            console.error('Ошибка:', error);
            this.showError('Ошибка при обновлении слота');
        }
    }

    async deleteSlot(slotId) {
        if (!confirm('Вы уверены, что хотите удалить этот слот?')) {
            return;
        }

        try {
            const response = await fetch(`/api/slots/${slotId}`, {
                method: 'DELETE'
            });

            if (response.ok) {
                this.showSuccess('Слот удален успешно');
                this.loadSlots();
            } else {
                const error = await response.json();
                this.showError(error.error || 'Ошибка при удалении слота');
            }
        } catch (error) {
            console.error('Ошибка:', error);
            this.showError('Ошибка при удалении слота');
        }
    }

    formatDate(dateString) {
        const date = new Date(dateString);
        return date.toLocaleDateString('ru-RU', {
            day: 'numeric',
            month: 'long',
            year: 'numeric'
        });
    }

    formatTime(timeString) {
        return timeString.substring(0, 5);
    }

    showSuccess(message) {
        this.showNotification(message, 'success');
    }

    showError(message) {
        this.showNotification(message, 'danger');
    }

    showNotification(message, type) {
        const notification = document.createElement('div');
        notification.className = `alert alert-${type} alert-dismissible fade show position-fixed top-0 end-0 m-3`;
        notification.style.zIndex = '9999';
        notification.innerHTML = `
            ${message}
            <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
        `;
        
        document.body.appendChild(notification);
        
        setTimeout(() => {
            if (notification.parentNode) {
                notification.parentNode.removeChild(notification);
            }
        }, 5000);
    }
}

// Инициализация приложения
let scheduler;
document.addEventListener('DOMContentLoaded', () => {
    scheduler = new MigrationScheduler();
});