declare module 'xterm-addon-fit' {
  import { ITerminalAddon, Terminal } from 'xterm';

  export class FitAddon implements ITerminalAddon {
    activate(terminal: Terminal): void;
    fit(): void;
    dispose(): void;
  }
}