// Editor.js Bundle - Package-managed setup
// This replaces the CDN-based loading from assets/editor.js

// Core Editor.js
import EditorJS, {
  type EditorConfig,
  type OutputData,
} from "@editorjs/editorjs";

// Block Tools
import Header from "@editorjs/header";
import List from "@editorjs/list";
import NestedList from "@editorjs/nested-list";
import Quote from "@editorjs/quote";
import Delimiter from "@editorjs/delimiter";
import Image from "@editorjs/image";
import Embed from "@editorjs/embed";
import Link from "@editorjs/link";
import Attaches from "@editorjs/attaches";
import Code from "@editorjs/code";
import RawTool from "@editorjs/raw";
import Table from "@editorjs/table";
import Checklist from "@editorjs/checklist";
import Warning from "@editorjs/warning";
import Button from "editorjs-button";
import Alert from "editorjs-alert";

// Inline Tools
import InlineCode from "@editorjs/inline-code";
import Marker from "@editorjs/marker";
import Underline from "@editorjs/underline";
import Hyperlink from "editorjs-hyperlink";

// Plugins
import DragDrop from "editorjs-drag-drop";
import Undo from "editorjs-undo";

// Constants
const READY_EVENT = "DOMContentLoaded";
const CHANGE_EVENT = "editor:change";
const SAVE_EVENT = "editor:save";
const HOLDER_ID = "editorjs";

// Types
interface EditorWindow extends Window {
  __EDITOR_INITIAL_DATA_RAW?: string;
  __EDITORJS_INSTANCE__?: EditorJS;
  editor?: {
    save: () => Promise<OutputData>;
    clear: () => Promise<void>;
    render: (data: OutputData) => Promise<void>;
    destroy: () => void;
    instance: EditorJS;
  };
}

declare const window: EditorWindow;

// Default seed data
const defaultSeed = (): OutputData => ({
  time: Date.now(),
  blocks: [
    {
      type: "header",
      data: {
        text: "Welcome to Editor.js!",
        level: 2,
      },
    },
    {
      type: "paragraph",
      data: {
        text: "Start writing your blog post here…",
      },
    },
  ],
  version: "2.30.7",
});

// DOM ready helper
const ready = (fn: () => void): void => {
  if (document.readyState === "loading") {
    document.addEventListener(READY_EVENT, fn, { once: true });
  } else {
    fn();
  }
};

// Parse initial data from Rust
const parseInitialData = (): OutputData => {
  const raw = window.__EDITOR_INITIAL_DATA_RAW;
  if (!raw) {
    return defaultSeed();
  }

  try {
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === "object") {
      return parsed as OutputData;
    }
  } catch (err) {
    console.error(
      "[EditorJS] Failed to parse initial data, falling back to default",
      err,
    );
  }

  return defaultSeed();
};

// Emit custom event
const emitEvent = (name: string, detail: string | null): void => {
  try {
    window.dispatchEvent(new CustomEvent(name, { detail }));
  } catch (err) {
    console.error("[EditorJS] Failed to emit event", name, err);
  }
};

// Stringify with error handling
const withStringDetail = (data: OutputData): string | null => {
  try {
    return JSON.stringify(data);
  } catch (err) {
    console.error("[EditorJS] Failed to stringify data", err);
    return null;
  }
};

// Mount the editor
const mountEditor = async (): Promise<void> => {
  // Destroy existing instance if present
  if (
    window.__EDITORJS_INSTANCE__ &&
    typeof window.__EDITORJS_INSTANCE__.destroy === "function"
  ) {
    window.__EDITORJS_INSTANCE__.destroy();
  }

  const config: EditorConfig = {
    holder: HOLDER_ID,
    autofocus: false,
    logLevel: "ERROR" as any,
    placeholder: "Start writing your blog post…",
    tools: {
      // Block Tools
      header: {
        class: Header,
        inlineToolbar: true,
      },
      // Use regular list instead of nested list for simplicity
      // Switch to NestedList if you need nested functionality
      list: {
        class: List,
        inlineToolbar: true,
      },
      quote: {
        class: Quote,
        inlineToolbar: true,
      },
      delimiter: Delimiter,
      image: {
        class: Image,
        config: {
          uploader: {
            // Upload by URL
            uploadByUrl: async (url: string) => ({
              success: 1,
              file: { url },
            }),
            // Upload by file - integrated with Rust media store
            uploadByFile: async (file: File) => {
              try {
                // @ts-ignore - editorjs_upload_file is exposed by Rust wasm_bindgen
                if (typeof window.editorjs_upload_file !== 'function') {
                  console.error('[EditorJS] Upload bridge not available');
                  throw new Error('Upload functionality not available');
                }

                console.log('[EditorJS] Uploading file:', file.name);
                // @ts-ignore
                const response = await window.editorjs_upload_file(file);
                console.log('[EditorJS] Upload response:', response);
                return response;
              } catch (err) {
                console.error('[EditorJS] Upload failed:', err);
                throw err;
              }
            },
          },
        },
      },
      embed: {
        class: Embed,
        inlineToolbar: true,
      },
      // Renamed from "link" to "linktool" to avoid conflict with inline hyperlink
      linktool: {
        class: Link,
        config: {
          // Link metadata fetching endpoint (optional)
          // Can be added later if needed: endpoint: '/api/link-preview'
        },
      },
      attaches: {
        class: Attaches,
        config: {
          uploader: {
            // Upload file attachments via Rust media store
            uploadByFile: async (file: File) => {
              try {
                // @ts-ignore - editorjs_upload_file is exposed by Rust wasm_bindgen
                if (typeof window.editorjs_upload_file !== 'function') {
                  console.error('[EditorJS] Upload bridge not available');
                  throw new Error('Upload functionality not available');
                }

                console.log('[EditorJS] Uploading attachment:', file.name);
                // @ts-ignore
                const response = await window.editorjs_upload_file(file);
                console.log('[EditorJS] Attachment upload response:', response);

                // Attaches tool expects additional metadata
                return {
                  ...response,
                  title: file.name,
                  size: file.size,
                };
              } catch (err) {
                console.error('[EditorJS] Attachment upload failed:', err);
                throw err;
              }
            },
          },
        },
      },
      code: {
        class: Code,
      },
      raw: RawTool,
      table: {
        class: Table,
        inlineToolbar: true,
      },
      checklist: {
        class: Checklist,
        inlineToolbar: true,
      },
      warning: {
        class: Warning,
        inlineToolbar: true,
      },
      button: Button,
      alert: Alert,

      // Inline Tools
      inlineCode: {
        class: InlineCode,
      },
      marker: {
        class: Marker,
      },
      underline: Underline,
      hyperlink: {
        class: Hyperlink,
        config: {
          shortcut: "CMD+L",
          target: "_blank",
          rel: "nofollow",
          availableTargets: ["_blank", "_self"],
          availableRels: ["author", "noreferrer", "nofollow"],
          validate: false,
        },
      },
    },
    data: parseInitialData(),
    onReady: async () => {
      // Initialize plugins
      const editor = window.__EDITORJS_INSTANCE__;
      if (editor) {
        // @ts-ignore - DragDrop doesn't have types
        new DragDrop(editor);
        // @ts-ignore - Undo doesn't have proper types
        new Undo({ editor });
      }

      emitEvent("editor:ready", null);

      // Emit initial change event
      try {
        if (editor) {
          const data = await editor.save();
          const detail = withStringDetail(data);
          if (detail) {
            emitEvent(CHANGE_EVENT, detail);
          }
        }
      } catch (err) {
        console.error("[EditorJS] Initial save failed", err);
      }
    },
    onChange: async (api) => {
      try {
        const data = await api.saver.save();
        const detail = withStringDetail(data);
        if (detail) {
          emitEvent(CHANGE_EVENT, detail);
        }
      } catch (err) {
        console.error("[EditorJS] onChange save failed", err);
      }
    },
  };

  const editor = new EditorJS(config);
  window.__EDITORJS_INSTANCE__ = editor;

  // Expose API for Rust to call
  window.editor = {
    save: async () => editor.save(),
    clear: async () => editor.clear(),
    render: async (data: OutputData) => editor.render(data),
    destroy: () => editor.destroy(),
    instance: editor,
  };

  // Listen for save requests from Rust
  window.addEventListener("editor:request-save", async () => {
    try {
      const data = await editor.save();
      const detail = withStringDetail(data);
      if (detail) {
        emitEvent(SAVE_EVENT, detail);
      }
    } catch (err) {
      console.error("[EditorJS] Forced save failed", err);
    }
  });
};

// Initialize on DOM ready
ready(() => {
  mountEditor().catch((err) => {
    console.error("[EditorJS] Mount failed:", err);
  });
});
