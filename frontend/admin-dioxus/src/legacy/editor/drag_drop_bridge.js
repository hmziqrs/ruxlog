// JavaScript bridge for accessing DataTransfer files in drag-and-drop events
// and clipboard data in paste events
// This bridges the gap between Dioxus event abstraction and native browser APIs

window.editorDragDropBridge = {
    // Store the last drag event for access from Rust
    lastDragEvent: null,
    lastPasteEvent: null,

    // Capture the native drag event
    captureDragEvent: function(editorId) {
        const editor = document.getElementById(editorId);
        if (!editor) return;

        editor.addEventListener('drop', function(e) {
            window.editorDragDropBridge.lastDragEvent = e;
        });

        // Also capture paste events
        editor.addEventListener('paste', function(e) {
            window.editorDragDropBridge.lastPasteEvent = e;
        });
    },

    // Extract files from the last drag event
    getFilesFromLastDrop: function() {
        if (!this.lastDragEvent || !this.lastDragEvent.dataTransfer) {
            return [];
        }

        const files = Array.from(this.lastDragEvent.dataTransfer.files);

        // Clear the stored event
        this.lastDragEvent = null;

        return files;
    },

    // Create blob URLs for files and return metadata
    processFiles: function(files) {
        return files.map(file => ({
            blobUrl: URL.createObjectURL(file),
            name: file.name,
            size: file.size,
            type: file.type,
            file: file  // Keep reference to original file
        }));
    },

    // Get clipboard data from the last paste event
    getClipboardData: function() {
        if (!this.lastPasteEvent) {
            return { type: 'empty', content: '' };
        }

        const clipboardData = this.lastPasteEvent.clipboardData || window.clipboardData;
        if (!clipboardData) {
            return { type: 'empty', content: '' };
        }

        // Try to get HTML first (preserves formatting from Word/GDocs)
        const html = clipboardData.getData('text/html');
        if (html && html.trim()) {
            this.lastPasteEvent = null;
            return { type: 'html', content: html };
        }

        // Fallback to plain text
        const text = clipboardData.getData('text/plain');
        this.lastPasteEvent = null;
        return { type: 'text', content: text || '' };
    }
};
