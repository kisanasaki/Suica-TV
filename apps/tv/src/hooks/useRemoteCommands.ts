import { useEffect, useRef, useState } from 'react';
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
  const callbacks = useRef(options);
  callbacks.current = options;
  const [client] = useState(
    () =>
      new CoreClient({
        onStatus: (status) => callbacks.current.onStatus(status),
        onMessage: (message: ServerMessage) => {
          if (message.type === 'remote.command') callbacks.current.onNavigation(message.action);
          if (message.type === 'system.state') callbacks.current.onMode(message.mode);
          if (message.type === 'error') callbacks.current.onError(message.error.message);
        },
      }),
  );

  useEffect(() => {
    client.start();
    return () => client.stop();
  }, [client]);

  return client;
}
