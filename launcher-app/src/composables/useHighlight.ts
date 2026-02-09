/**
 * useHighlight - Search keyword highlighting composable
 *
 * Returns structured text parts instead of HTML strings,
 * eliminating the need for v-html and preventing XSS by design.
 */

export interface HighlightPart {
  text: string;
  highlight: boolean;
}

/**
 * Escape special regex characters in user input
 */
function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Split text into highlighted and non-highlighted parts
 *
 * @param text - The source text to highlight within
 * @param query - The search query to match
 * @returns Array of text parts with highlight flags, safe for v-for rendering
 */
export function highlightParts(text: string, query: string): HighlightPart[] {
  if (!text) return [];
  if (!query) return [{ text, highlight: false }];

  const escapedQuery = escapeRegex(query.trim());
  if (!escapedQuery) return [{ text, highlight: false }];

  // Split query into individual words for multi-word matching
  const words = escapedQuery.split(/\s+/).filter(w => w.length > 0);
  if (words.length === 0) return [{ text, highlight: false }];

  const pattern = new RegExp(`(${words.join('|')})`, 'gi');
  const parts = text.split(pattern);

  return parts
    .filter(part => part.length > 0)
    .map(part => {
      const isMatch = words.some(w => new RegExp(`^${w}$`, 'i').test(part));
      return { text: part, highlight: isMatch };
    });
}

/**
 * Composable wrapper for reactive highlight usage
 */
export function useHighlight() {
  return { highlightParts };
}
