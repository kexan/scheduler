import { describe, it, expect, beforeEach } from 'vitest';
import { Utils, DOMHelper, ButtonHelper } from '../utils.js';

describe('utils.js', () => {
  describe('Utils class', () => {
    it('formatDate should format YYYY-MM-DD to DD.MM.YYYY', () => {
      const formatted = Utils.formatDate('2026-05-21');
      expect(formatted).toContain('21.05.2026');
    });

    it('formatTime should format hh:mm:ss to hh:mm', () => {
      expect(Utils.formatTime('12:34:56')).toBe('12:34');
      expect(Utils.formatTime('09:05:00')).toBe('09:05');
    });

    it('getCurrentTimeMSK should return a Date object representing Moscow Time', () => {
      const mskTime = Utils.getCurrentTimeMSK();
      expect(mskTime).toBeInstanceOf(Date);
    });

    it('formatTimeMSK should format time with MSK label', () => {
      expect(Utils.formatTimeMSK('15:30:00')).toBe('15:30 МСК');
    });

    it('formatDateTimeMSK should format date and time', () => {
      expect(Utils.formatDateTimeMSK('2026-05-21', '12:00:00')).toContain('21.05.2026');
      expect(Utils.formatDateTimeMSK('2026-05-21', '12:00:00')).toContain('12:00 МСК');
    });

    it('showNotification should append an alert to document.body', () => {
      document.body.innerHTML = '';
      Utils.showNotification('Hello World', 'success');
      const alerts = document.body.querySelectorAll('.alert');
      expect(alerts.length).toBe(1);
      expect(alerts[0].textContent).toContain('Hello World');
      expect(alerts[0].className).toContain('alert-success');
    });

    it('escapeHtml should escape HTML special characters', () => {
      expect(Utils.escapeHtml('<div>Hello & "World"</div>')).toBe('&lt;div&gt;Hello &amp; &quot;World&quot;&lt;/div&gt;');
      expect(Utils.escapeHtml("o'reilly")).toBe('o&#039;reilly');
      expect(Utils.escapeHtml(null)).toBe('');
      expect(Utils.escapeHtml(undefined)).toBe('');
    });
  });

  describe('DOMHelper class', () => {
    beforeEach(() => {
      document.body.innerHTML = `
        <div id="testDiv" class="existing-class">Hello</div>
        <input type="checkbox" id="testCheckbox" />
        <input type="text" id="testInput" value="initial" />
      `;
    });

    it('get should return the element', () => {
      const el = DOMHelper.get('testDiv');
      expect(el).not.toBeNull();
      expect(el.textContent).toBe('Hello');
    });

    it('set should update input value', () => {
      DOMHelper.set('testInput', 'new-value');
      expect(document.getElementById('testInput').value).toBe('new-value');
    });

    it('text should update textContent', () => {
      DOMHelper.text('testDiv', 'New Text');
      expect(document.getElementById('testDiv').textContent).toBe('New Text');
    });

    it('html should update innerHTML', () => {
      DOMHelper.html('testDiv', '<span>HTML</span>');
      expect(document.getElementById('testDiv').innerHTML).toBe('<span>HTML</span>');
    });

    it('show and hide should set display style', () => {
      DOMHelper.hide('testDiv');
      expect(document.getElementById('testDiv').style.display).toBe('none');
      DOMHelper.show('testDiv');
      expect(document.getElementById('testDiv').style.display).toBe('block');
    });

    it('addClass and removeClass should modify classList', () => {
      DOMHelper.addClass('testDiv', 'new-class');
      expect(document.getElementById('testDiv').classList.contains('new-class')).toBe(true);
      DOMHelper.removeClass('testDiv', 'existing-class');
      expect(document.getElementById('testDiv').classList.contains('existing-class')).toBe(false);
    });

    it('isChecked and setChecked should read and write checkbox state', () => {
      expect(DOMHelper.isChecked('testCheckbox')).toBe(false);
      DOMHelper.setChecked('testCheckbox', true);
      expect(DOMHelper.isChecked('testCheckbox')).toBe(true);
    });
  });

  describe('ButtonHelper class', () => {
    beforeEach(() => {
      document.body.innerHTML = `
        <button id="testBtn" class="btn btn-outline-info">Click</button>
      `;
    });

    it('setState should update text and classes', () => {
      ButtonHelper.setState('testBtn', 'Saving...', 'btn-outline-info', 'btn-outline-danger');
      const btn = document.getElementById('testBtn');
      expect(btn.textContent).toBe('Saving...');
      expect(btn.classList.contains('btn-outline-info')).toBe(false);
      expect(btn.classList.contains('btn-outline-danger')).toBe(true);
    });

    it('setIcon should update content and tooltip title', () => {
      ButtonHelper.setIcon('testBtn', '🔒', 'Locked state');
      const btn = document.getElementById('testBtn');
      expect(btn.textContent).toBe('🔒');
      expect(btn.title).toBe('Locked state');
    });
  });

  describe('Validation and Timezone Helpers', () => {
    beforeEach(() => {
      document.body.innerHTML = `
        <form id="testForm">
          <div>
            <input type="text" id="testEmail" value="" />
          </div>
          <div>
            <input type="text" id="testRequired" value="some value" />
          </div>
        </form>
        <div id="timezoneAlert" style="display: none;">
          <span id="timezoneDiff"></span>
        </div>
      `;
    });

    it('validateField should mark invalid and append feedback', () => {
      const isValid = Utils.validateField('testEmail', (val) => val.includes('@'), 'Invalid email');
      expect(isValid).toBe(false);

      const input = document.getElementById('testEmail');
      expect(input.classList.contains('is-invalid')).toBe(true);
      expect(input.classList.contains('is-valid')).toBe(false);

      const feedback = input.parentElement.querySelector('.invalid-feedback');
      expect(feedback).not.toBeNull();
      expect(feedback.textContent).toBe('Invalid email');
    });

    it('validateField should mark valid and remove feedback', () => {
      Utils.validateField('testEmail', (val) => val.includes('@'), 'Invalid email');

      document.getElementById('testEmail').value = 'test@example.com';
      const isValid = Utils.validateField('testEmail', (val) => val.includes('@'), 'Invalid email');
      expect(isValid).toBe(true);

      const input = document.getElementById('testEmail');
      expect(input.classList.contains('is-valid')).toBe(true);
      expect(input.classList.contains('is-invalid')).toBe(false);

      const feedback = input.parentElement.querySelector('.invalid-feedback');
      expect(feedback).toBeNull();
    });

    it('clearValidation should remove all validation classes and feedback elements', () => {
      Utils.validateField('testEmail', () => false, 'Err');
      Utils.validateField('testRequired', () => true, 'Err');

      expect(document.getElementById('testEmail').classList.contains('is-invalid')).toBe(true);
      expect(document.getElementById('testRequired').classList.contains('is-valid')).toBe(true);
      expect(document.querySelector('.invalid-feedback')).not.toBeNull();

      Utils.clearValidation('testForm');

      expect(document.getElementById('testEmail').classList.contains('is-invalid')).toBe(false);
      expect(document.getElementById('testRequired').classList.contains('is-valid')).toBe(false);
      expect(document.querySelector('.invalid-feedback')).toBeNull();
    });

    it('checkTimezoneDifference should show alert if timezone is not MSK (UTC+3)', () => {
      const originalGetOffset = Date.prototype.getTimezoneOffset;
      Date.prototype.getTimezoneOffset = () => 0;

      try {
        Utils.checkTimezoneDifference();
        expect(document.getElementById('timezoneAlert').style.display).toBe('block');
        expect(document.getElementById('timezoneDiff').textContent).toBe('-3 ч');
      } finally {
        Date.prototype.getTimezoneOffset = originalGetOffset;
      }
    });

    it('checkTimezoneDifference should NOT show alert if timezone is MSK (UTC+3)', () => {
      const originalGetOffset = Date.prototype.getTimezoneOffset;
      Date.prototype.getTimezoneOffset = () => -180;

      try {
        Utils.checkTimezoneDifference();
        expect(document.getElementById('timezoneAlert').style.display).toBe('none');
      } finally {
        Date.prototype.getTimezoneOffset = originalGetOffset;
      }
    });
  });
});
