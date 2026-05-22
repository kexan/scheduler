class Api {
  constructor() {
    this.baseUrl = "/api";
  }

  async _getError(response) {
    try {
      const text = await response.text();
      try {
        const data = JSON.parse(text);
        return data.error || "Произошла ошибка";
      } catch (e) {
        return text || `Ошибка со статусом ${response.status}`;
      }
    } catch (e) {
      return "Не удалось прочитать ответ сервера";
    }
  }

  async _request(path, options = {}) {
    const url = `${this.baseUrl}${path}`;
    const defaultHeaders = {};
    if (options.body && !(options.body instanceof FormData)) {
      defaultHeaders['Content-Type'] = 'application/json';
    }

    const mergedOptions = {
      credentials: 'include',
      ...options,
      headers: {
        ...defaultHeaders,
        ...options.headers
      }
    };

    try {
      const response = await fetch(url, mergedOptions);
      if (!response.ok) {
        const errorMsg = await this._getError(response);
        throw new Error(errorMsg);
      }

      const contentType = response.headers.get("content-type");
      if (contentType && contentType.includes("application/json")) {
        return await response.json();
      }
      return response;
    } catch (error) {
      throw error;
    }
  }

  async getSlots(from, to, search) {
    const params = [];
    if (from) params.push(`from=${from}`);
    if (to) params.push(`to=${to}`);
    if (search) params.push(`search=${encodeURIComponent(search)}`);
    const queryString = params.length > 0 ? `?${params.join("&")}` : "";
    return this._request(`/slots${queryString}`);
  }

  async createSlot(slotData) {
    return this._request(`/slots`, {
      method: "POST",
      body: JSON.stringify(slotData)
    });
  }

  async updateSlot(slotId, slotData) {
    return this._request(`/slots/${slotId}`, {
      method: "PUT",
      body: JSON.stringify(slotData)
    });
  }

  async deleteSlot(slotId) {
    return this._request(`/slots/${slotId}`, {
      method: "DELETE"
    });
  }

  async bookSlot(slotId, bookingData) {
    return this._request(`/slots/${slotId}/book`, {
      method: "POST",
      body: JSON.stringify(bookingData)
    });
  }

  async getYougileSettings() {
    return this._request(`/yougile/settings`);
  }

  async updateYougileSettings(settings) {
    return this._request(`/yougile/settings`, {
      method: "PUT",
      body: JSON.stringify(settings)
    });
  }

  async getProjects() {
    return this._request(`/yougile/projects`);
  }

  async getBoards(projectId) {
    return this._request(`/yougile/boards?project_id=${projectId}`);
  }

  async getColumns(boardId) {
    return this._request(`/yougile/columns?board_id=${boardId}`);
  }

  async loadYougileUsers(projectId) {
    return this._request(`/yougile/users?project_id=${projectId}`);
  }

  async adminAuth(password) {
    return this._request(`/auth/admin`, {
      method: "POST",
      body: JSON.stringify({ password })
    });
  }

  async checkAuth() {
    return this._request(`/auth/check`);
  }
}

export { Api };
