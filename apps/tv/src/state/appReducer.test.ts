import { describe, expect, it } from 'vitest';
import { appReducer, initialState } from './appReducer';

describe('appReducer', () => {
  it.each([
    ['navigation.right', 'browser'],
    ['navigation.down', 'zaim'],
    ['navigation.left', 'youtube'],
    ['navigation.up', 'youtube'],
  ] as const)('%s moves or remains on the expected tile', (direction, expected) => {
    const state = appReducer(initialState, { type: 'navigate', direction });
    expect(state.selectedId).toBe(expected);
  });

  it('keeps the selection at grid boundaries', () => {
    const browser = { ...initialState, selectedId: 'browser' as const };
    expect(appReducer(browser, { type: 'navigate', direction: 'navigation.right' }).selectedId).toBe('browser');
    const settings = { ...initialState, selectedId: 'settings' as const };
    expect(appReducer(settings, { type: 'navigate', direction: 'navigation.down' }).selectedId).toBe('settings');
  });

  it('navigates through the added Zaim row to settings', () => {
    const zaim = appReducer(initialState, { type: 'navigate', direction: 'navigation.down' });
    expect(zaim.selectedId).toBe('zaim');
    expect(appReducer(zaim, { type: 'navigate', direction: 'navigation.right' }).selectedId).toBe('pc-mode');
    expect(appReducer(zaim, { type: 'navigate', direction: 'navigation.down' }).selectedId).toBe('settings');
  });

  it('home resets the page and selection', () => {
    const state = { ...initialState, page: 'settings' as const, selectedId: 'settings' as const };
    expect(appReducer(state, { type: 'go-home' })).toMatchObject({ page: 'home', selectedId: 'youtube' });
  });

  it('back returns from settings but does nothing on home', () => {
    expect(appReducer({ ...initialState, page: 'settings' }, { type: 'go-back' }).page).toBe('home');
    expect(appReducer(initialState, { type: 'go-back' })).toBe(initialState);
  });

  it('tracks connection, mode and switching state', () => {
    let state = appReducer(initialState, { type: 'connection-changed', status: 'connected' });
    state = appReducer(state, { type: 'mode-switch-started' });
    state = appReducer(state, { type: 'mode-changed', mode: 'pc' });
    expect(state).toMatchObject({ connection: 'connected', modeSwitchPending: true, mode: 'pc' });
    expect(appReducer(state, { type: 'mode-switch-finished' }).modeSwitchPending).toBe(false);
  });
});
