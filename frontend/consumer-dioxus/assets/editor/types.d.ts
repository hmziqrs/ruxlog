// Type declarations for Editor.js packages without official types

// Window extensions for Rust wasm_bindgen bridge
declare global {
  interface Window {
    editorjs_upload_file?: (file: File) => Promise<{
      success: number;
      file: { url: string };
    }>;
  }
}

declare module 'editorjs-drag-drop' {
  import type EditorJS from '@editorjs/editorjs';

  export default class DragDrop {
    constructor(editor: EditorJS);
  }
}

declare module 'editorjs-undo' {
  import type EditorJS from '@editorjs/editorjs';

  interface UndoConfig {
    editor: EditorJS;
    maxLength?: number;
    onUpdate?: () => void;
  }

  export default class Undo {
    constructor(config: UndoConfig);
    initialize(data: any): void;
    clear(): void;
    count(): number;
  }
}

declare module 'editorjs-button' {
  import type { BlockTool, BlockToolConstructorOptions } from '@editorjs/editorjs';

  export default class Button implements BlockTool {
    constructor(options: BlockToolConstructorOptions);
    static get toolbox(): { icon: string; title: string };
    render(): HTMLElement;
    save(block: HTMLElement): any;
  }
}

declare module 'editorjs-alert' {
  import type { BlockTool, BlockToolConstructorOptions } from '@editorjs/editorjs';

  export default class Alert implements BlockTool {
    constructor(options: BlockToolConstructorOptions);
    static get toolbox(): { icon: string; title: string };
    render(): HTMLElement;
    save(block: HTMLElement): any;
  }
}

declare module 'editorjs-hyperlink' {
  import type { InlineTool, InlineToolConstructorOptions } from '@editorjs/editorjs';

  export default class Hyperlink implements InlineTool {
    constructor(options: InlineToolConstructorOptions);
    static get isInline(): boolean;
    render(): HTMLElement;
    surround(range: Range): void;
    checkState(): boolean;
    renderActions(): HTMLElement;
    save(): void;
    static get sanitize(): any;
  }
}
