import { useEffect, useMemo } from 'react';
import { CoreClient } from '../services/coreClient';
import type { ServerMessage } from '../services/messages';
import type { ConnectionStatus, NavigationAction } from '../state/types';

interface Options {
  onStatus: (status: ConnectionStatus) => void;
  onNavigation: (action: NavigationAction) => void;
  onMode: (mode: 'tv' | 'pc') => void;
  onError: (message: string) => void;
}

export function useRemoteCommands(options: Options) {
  const client = useMemo(
    () =>
      new CoreClient({
        onStatus: options.onStatus,
        onMessage: (message: ServerMessage) => {
          if (message.type === 'remote.command') options.onNavigation(message.action);
          if (message.type === 'system.state') options.onMode(message.mode);
          if (message.type === 'error') options.onError(message.error.message);
        },
      }),
    [options.onError, options.onMode, options.onNavigation, options.onStatus],
  );

  useEffect(() => {
    client.start();
    return () => client.stop();
  }, [client]);

  return client;
}
