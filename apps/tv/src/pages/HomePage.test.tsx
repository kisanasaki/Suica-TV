import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { HomePage } from './HomePage';

describe('HomePage', () => {
  it('renders all menu tiles and marks only the selected tile', () => {
    render(<HomePage selectedId="youtube" coreConnected={false} modeSwitchPending={false} onFocusItem={vi.fn()} onSelectItem={vi.fn()} />);
    expect(screen.getAllByRole('button')).toHaveLength(4);
    expect(screen.getByTestId('tile-youtube')).toHaveAttribute('aria-current', 'true');
    expect(screen.getByTestId('tile-browser')).not.toHaveAttribute('aria-current');
  });

  it('disables only PC Mode when Core is disconnected', () => {
    render(<HomePage selectedId="youtube" coreConnected={false} modeSwitchPending={false} onFocusItem={vi.fn()} onSelectItem={vi.fn()} />);
    expect(screen.getByTestId('tile-pc-mode')).toBeDisabled();
    expect(screen.getByTestId('tile-youtube')).toBeEnabled();
    expect(screen.getByTestId('tile-browser')).toBeEnabled();
    expect(screen.getByTestId('tile-settings')).toBeEnabled();
  });

  it('selects an enabled item by click', async () => {
    const onSelect = vi.fn();
    render(<HomePage selectedId="youtube" coreConnected modeSwitchPending={false} onFocusItem={vi.fn()} onSelectItem={onSelect} />);
    await userEvent.click(screen.getByTestId('tile-settings'));
    expect(onSelect).toHaveBeenCalledWith('settings');
  });
});
