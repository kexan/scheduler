class Utils {
  static formatDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleDateString('ru-RU', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric'
    });
  }

  static formatTime(timeString) {
    return timeString.substring(0, 5);
  }

  static getCurrentTimeMSK() {
    const now = new Date();
    return new Date(now.toLocaleString("en-US", { timeZone: "Europe/Moscow" }));
  }

  static formatTimeMSK(timeString) {

    return `${timeString.substring(0, 5)} МСК`;
  }

  static formatDateTimeMSK(dateString, timeString) {
    const date = new Date(dateString);
    const formattedDate = date.toLocaleDateString('ru-RU', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric'
    });
    return `${formattedDate}, ${this.formatTimeMSK(timeString)}`;
  }

  static updateCurrentTimeMSK() {
    const mskTime = this.getCurrentTimeMSK();
    const timeString = mskTime.toLocaleTimeString('ru-RU', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
    const timeElement = document.getElementById('currentTimeMSK');
    if (timeElement) {
      timeElement.textContent = timeString;
    }
  }

  static showSuccess(message) {
    this.showNotification(message, 'success');
  }

  static showError(message) {
    this.showNotification(message, 'danger');
  }

  static showNotification(message, type = 'info') {
    const notification = document.createElement('div');
    notification.className = `alert alert-${type} alert-dismissible fade show position-fixed`;
    notification.style.cssText = 'top: 20px; right: 20px; z-index: 9999; min-width: 300px;';
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

  static validateField(id, validationFn, errorMsg) {
    const element = DOMHelper.get(id);
    if (!element) return true;
    const val = element.value.trim();
    const isValid = validationFn(val);

    element.classList.remove("is-valid", "is-invalid");
    const parent = element.parentElement;
    const existingFeedback = parent.querySelector(".invalid-feedback");
    if (existingFeedback) {
      existingFeedback.remove();
    }

    if (!isValid) {
      element.classList.add("is-invalid");
      const feedback = document.createElement("div");
      feedback.className = "invalid-feedback";
      feedback.textContent = errorMsg;
      parent.appendChild(feedback);
      return false;
    } else {
      element.classList.add("is-valid");
      return true;
    }
  }

  static clearValidation(formId) {
    const form = DOMHelper.get(formId);
    if (!form) return;
    form.querySelectorAll(".is-valid, .is-invalid").forEach((el) => {
      el.classList.remove("is-valid", "is-invalid");
    });
    form.querySelectorAll(".invalid-feedback").forEach((el) => {
      el.remove();
    });
  }

  static checkTimezoneDifference() {
    const userOffset = new Date().getTimezoneOffset();
    const mskOffset = -180;
    if (userOffset !== mskOffset) {
      const diffMinutes = mskOffset - userOffset;
      const diffHours = diffMinutes / 60;
      const sign = diffHours > 0 ? "+" : "";
      const text = `${sign}${diffHours} ч`;
      const diffSpan = DOMHelper.get("timezoneDiff");
      if (diffSpan) diffSpan.textContent = text;
      DOMHelper.show("timezoneAlert");
    }
  }

  static escapeHtml(str) {
    if (!str) return "";
    return String(str)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }
}


class DOMHelper {
  static get(id) {
    return document.getElementById(id);
  }

  static set(id, value) {
    const element = this.get(id);
    if (element) element.value = value;
  }

  static text(id, text) {
    const element = this.get(id);
    if (element) element.textContent = text;
  }

  static html(id, html) {
    const element = this.get(id);
    if (element) element.innerHTML = html;
  }

  static show(id) {
    const element = this.get(id);
    if (element) element.style.display = 'block';
  }

  static hide(id) {
    const element = this.get(id);
    if (element) element.style.display = 'none';
  }

  static toggle(id, show = null) {
    const element = this.get(id);
    if (element) element.style.display = show !== null ? (show ? 'block' : 'none') :
      (element.style.display === 'none' ? 'block' : 'none');
  }

  static addClass(id, className) {
    const element = this.get(id);
    if (element) element.classList.add(className);
  }

  static removeClass(id, className) {
    const element = this.get(id);
    if (element) element.classList.remove(className);
  }

  static toggleClass(id, className, add = null) {
    const element = this.get(id);
    if (element) element.classList[add !== null ? (add ? 'add' : 'remove') : 'toggle'](className);
  }

  static setChecked(id, checked) {
    const element = this.get(id);
    if (element) element.checked = checked;
  }

  static isChecked(id) {
    const element = this.get(id);
    return element ? element.checked : false;
  }
}


class FormHelper {
  static reset(formId) {
    const form = DOMHelper.get(formId);
    if (form) form.reset();
  }
}


class EventManager {
  static onClick(id, handler) {
    const element = DOMHelper.get(id);
    if (element) element.addEventListener('click', handler);
  }

  static onSubmit(id, handler) {
    const element = DOMHelper.get(id);
    if (element) {
      element.addEventListener('submit', (e) => {
        e.preventDefault();
        handler();
      });
    }
  }

  static onChange(id, handler) {
    const element = DOMHelper.get(id);
    if (element) element.addEventListener('change', handler);
  }

  static onInput(id, handler) {
    const element = DOMHelper.get(id);
    if (element) element.addEventListener('input', handler);
  }
}


class ButtonHelper {
  static setState(id, text, removeClass, addClass) {
    const button = DOMHelper.get(id);
    if (button) {
      if (text) button.textContent = text;
      if (removeClass) button.classList.remove(removeClass);
      if (addClass) button.classList.add(addClass);
    }
  }

  static setIcon(id, icon, tooltip = null) {
    const button = DOMHelper.get(id);
    if (button) {
      button.textContent = icon;
      if (tooltip) button.title = tooltip;
    }
  }
}

export { Utils, DOMHelper, FormHelper, EventManager, ButtonHelper };

