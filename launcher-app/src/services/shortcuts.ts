import type { HotkeySettings } from '../types/settings';

/**
 * ShortcutManager handles keyboard shortcut matching and management.
 * It parses shortcut strings (e.g., "Meta+,", "Ctrl+Shift+S") and
 * matches them against keyboard events.
 */
export class ShortcutManager {
    private shortcuts: Record<string, string> = {};

    constructor(hotkeySettings?: HotkeySettings) {
        if (hotkeySettings?.ui_shortcuts) {
            this.shortcuts = { ...hotkeySettings.ui_shortcuts };
        }
    }

    /**
     * Update shortcuts configuration
     */
    updateShortcuts(shortcuts: Record<string, string>) {
        this.shortcuts = { ...shortcuts };
    }

    /**
     * Check if a keyboard event matches a specific action
     */
    matches(event: KeyboardEvent, actionId: string): boolean {
        const shortcut = this.shortcuts[actionId];
        if (!shortcut) {
            return false;
        }

        return this.eventMatchesShortcut(event, shortcut);
    }

    /**
     * Parse and match a keyboard event against a shortcut string
     */
    private eventMatchesShortcut(event: KeyboardEvent, shortcut: string): boolean {
        const parts = shortcut.split('+').map(p => p.trim());

        const requiredModifiers = {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        };

        let key = '';

        for (const part of parts) {
            const lower = part.toLowerCase();
            if (lower === 'ctrl' || lower === 'control') {
                requiredModifiers.ctrl = true;
            } else if (lower === 'alt' || lower === 'option') {
                requiredModifiers.alt = true;
            } else if (lower === 'shift') {
                requiredModifiers.shift = true;
            } else if (lower === 'meta' || lower === 'cmd' || lower === 'command') {
                requiredModifiers.meta = true;
            } else {
                key = part;
            }
        }

        // Check modifiers match exactly
        if (event.ctrlKey !== requiredModifiers.ctrl) return false;
        if (event.altKey !== requiredModifiers.alt) return false;
        if (event.shiftKey !== requiredModifiers.shift) return false;
        if (event.metaKey !== requiredModifiers.meta) return false;

        // Check key matches (case-insensitive for letters, exact for special keys)
        const eventKey = event.key;
        if (key.length === 1 && eventKey.length === 1) {
            // Single character key
            return eventKey.toLowerCase() === key.toLowerCase();
        } else {
            // Special key (Enter, Escape, ArrowDown, etc.)
            return eventKey === key;
        }
    }

    /**
     * Get the shortcut string for an action ID
     */
    getShortcut(actionId: string): string | undefined {
        return this.shortcuts[actionId];
    }

    /**
     * Get all shortcuts
     */
    getAllShortcuts(): Record<string, string> {
        return { ...this.shortcuts };
    }

    /**
     * Format a keyboard event as a shortcut string
     */
    static formatEvent(event: KeyboardEvent): string {
        const parts: string[] = [];

        if (event.metaKey) parts.push('Meta');
        if (event.ctrlKey) parts.push('Ctrl');
        if (event.altKey) parts.push('Alt');
        if (event.shiftKey) parts.push('Shift');

        // Add the main key
        const key = event.key;
        if (key && key !== 'Meta' && key !== 'Control' && key !== 'Alt' && key !== 'Shift') {
            parts.push(key);
        }

        return parts.join('+');
    }
}
