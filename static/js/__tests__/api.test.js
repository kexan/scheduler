import { describe, it, expect, beforeEach, vi } from 'vitest';
import { Api } from '../api.js';

describe('api.js', () => {
  let api;

  beforeEach(() => {
    api = new Api();
    globalThis.fetch = vi.fn();
  });

  describe('createSlot', () => {
    it('should successfully post slot data', async () => {
      const mockSlot = { id: 'slot-1', date: '2026-05-21', start_time: '12:00:00', end_time: '13:00:00' };
      
      globalThis.fetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'Content-Type': 'application/json' }),
        json: async () => mockSlot,
      });

      const slotData = { date: '2026-05-21', start_time: '12:00:00', end_time: '13:00:00' };
      const result = await api.createSlot(slotData);

      expect(globalThis.fetch).toHaveBeenCalledWith('/api/slots', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify(slotData),
      });
      expect(result).toEqual(mockSlot);
    });

    it('should throw API validation/server errors', async () => {
      globalThis.fetch.mockResolvedValueOnce({
        ok: false,
        status: 400,
        text: async () => JSON.stringify({ error: 'Неверный интервал времени' }),
        headers: new Headers(),
      });

      await expect(api.createSlot({})).rejects.toThrow('Неверный интервал времени');
    });

    it('should throw HTML or non-JSON raw text errors', async () => {
      globalThis.fetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        text: async () => 'Внутренняя ошибка сервера',
        headers: new Headers(),
      });

      await expect(api.createSlot({})).rejects.toThrow('Внутренняя ошибка сервера');
    });

    it('should throw network exceptions', async () => {
      globalThis.fetch.mockRejectedValueOnce(new Error('Network Fail'));

      await expect(api.createSlot({})).rejects.toThrow('Network Fail');
    });
  });

  describe('bookSlot', () => {
    it('should book a slot successfully', async () => {
      const mockSlot = { id: 'slot-1', is_available: false, booking: { company_name: 'Test' } };
      globalThis.fetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'Content-Type': 'application/json' }),
        json: async () => mockSlot,
      });

      const bookingData = { company_name: 'Test', admin_email: 'admin@test.com' };
      const result = await api.bookSlot('slot-1', bookingData);

      expect(globalThis.fetch).toHaveBeenCalledWith('/api/slots/slot-1/book', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify(bookingData),
      });
      expect(result).toEqual(mockSlot);
    });
  });

  describe('getYougileSettings', () => {
    it('should fetch and return settings', async () => {
      const mockSettings = { enabled: true, api_url: 'http://test' };
      globalThis.fetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'Content-Type': 'application/json' }),
        json: async () => mockSettings,
      });

      const result = await api.getYougileSettings();
      expect(result).toEqual(mockSettings);
    });

    it('should throw on failure', async () => {
      globalThis.fetch.mockResolvedValueOnce({
        ok: false,
        status: 401,
        text: async () => 'Unauthorized',
        headers: new Headers(),
      });

      await expect(api.getYougileSettings()).rejects.toThrow('Unauthorized');
    });
  });

  describe('getSlots', () => {
    it('should request correct URL with from and to parameters', async () => {
      globalThis.fetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'Content-Type': 'application/json' }),
        json: async () => [],
      });

      await api.getSlots('2026-05-21', '2026-05-22');

      expect(globalThis.fetch).toHaveBeenCalledWith('/api/slots?from=2026-05-21&to=2026-05-22', expect.any(Object));
    });

    it('should request correct URL with only search query parameter', async () => {
      globalThis.fetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'Content-Type': 'application/json' }),
        json: async () => [],
      });

      await api.getSlots(null, null, 'Google');

      expect(globalThis.fetch).toHaveBeenCalledWith('/api/slots?search=Google', expect.any(Object));
    });

    it('should encode URL parameters correctly', async () => {
      globalThis.fetch.mockResolvedValueOnce({
        ok: true,
        headers: new Headers({ 'Content-Type': 'application/json' }),
        json: async () => [],
      });

      await api.getSlots(null, null, 'Test & Co');

      expect(globalThis.fetch).toHaveBeenCalledWith('/api/slots?search=Test%20%26%20Co', expect.any(Object));
    });
  });
});
