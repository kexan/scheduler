class Api {
  constructor() {
    this.baseUrl = '/api';
  }

  async getSlots() {
    const response = await fetch(`${this.baseUrl}/slots`, {
      credentials: 'include'
    });
    return await response.json();
  }

  async createSlot(slotData) {


    const headers = {
      'Content-Type': 'application/json'
    };

    try {
      const response = await fetch(`${this.baseUrl}/slots`, {
        method: 'POST',
        headers,
        credentials: 'include',
        body: JSON.stringify(slotData)
      });



      if (response.ok) {
        const slot = await response.json();

        Utils.showSuccess('Слот создан успешно');
        return slot;
      } else {
        const error = await response.json();

        Utils.showError(error.error || 'Ошибка при создании слота');
        return null;
      }
    } catch (error) {
      Utils.showError('Ошибка сети: ' + error.message);
      return null;
    }
  }

  async updateSlot(slotId, slotData) {


    try {
      const response = await fetch(`${this.baseUrl}/slots/${slotId}/full`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json'
        },
        credentials: 'include',
        body: JSON.stringify(slotData)
      });



      const responseText = await response.text();

      if (response.ok) {
        Utils.showSuccess('Слот обновлен успешно');
        return true;
      } else {
        let error;
        try {
          error = JSON.parse(responseText);
        } catch (e) {
          error = { error: responseText };
        }

        Utils.showError(error.error || 'Ошибка при обновлении слота');
        return false;
      }
    } catch (error) {
      Utils.showError('Ошибка сети: ' + error.message);
      return false;
    }
  }

  async deleteSlot(slotId, skipConfirm = false) {
    if (!skipConfirm && !confirm('Вы уверены, что хотите удалить этот слот?')) {
      return false;
    }

    try {
      const response = await fetch(`${this.baseUrl}/slots/${slotId}`, {
        method: 'DELETE',
        credentials: 'include'
      });

      if (response.ok) {
        Utils.showSuccess('Слот удален успешно');
        return true;
      } else {
        const error = await response.json();
        Utils.showError(error.error || 'Ошибка при удалении слота');
        return false;
      }
    } catch (error) {
      Utils.showError('Ошибка сети: ' + error.message);
      return false;
    }
  }

  async bookSlot(slotId, bookingData) {
    try {
      const response = await fetch(`${this.baseUrl}/slots/${slotId}/book`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        credentials: 'include',
        body: JSON.stringify(bookingData)
      });



      if (response.ok) {
        const slot = await response.json();

        Utils.showSuccess('Слот успешно забронирован');
        return slot;
      } else {
        const error = await response.json();

        Utils.showError(error.error || 'Ошибка при записи');
        return false;
      }
    } catch (error) {
      Utils.showError('Ошибка сети: ' + error.message);
      return false;
    }
  }

  async getYougileSettings() {
    try {
      const response = await fetch(`${this.baseUrl}/yougile/settings`, {
        credentials: 'include'
      });

      if (response.ok) {
        return await response.json();
      } else {
        const error = await response.json();
        throw new Error(error.error || 'Ошибка загрузки настроек');
      }
    } catch (error) {
      Utils.showError('Ошибка сети: ' + error.message);
      throw error;
    }
  }

  async updateYougileSettings(settings) {
    try {
      const response = await fetch(`${this.baseUrl}/yougile/settings`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json'
        },
        credentials: 'include',
        body: JSON.stringify(settings)
      });

      if (response.ok) {
        Utils.showSuccess('Настройки Yougile сохранены');
        return;
      } else {
        const error = await response.json();
        throw new Error(error.error || 'Ошибка сохранения настроек');
      }
    } catch (error) {
      Utils.showError('Ошибка сети: ' + error.message);
      throw error;
    }
  }

  async testYougileConnection() {
    try {
      const response = await fetch(`${this.baseUrl}/yougile/test`, {
        method: 'POST',
        credentials: 'include'
      });

      if (response.ok) {
        return await response.json();
      } else {
        const error = await response.json();
        throw new Error(error.error || 'Ошибка проверки подключения');
      }
    } catch (error) {
      Utils.showError('Ошибка сети: ' + error.message);
      throw error;
    }
  }

}
