/**
 * useHighlight - Search keyword highlighting composable
 *
 * Safely highlights matching text segments in search results.
 * Escapes HTML and regex special characters to prevent XSS.
 */

/**
 * Escape special regex characters in user input
 */
function escapeRegex(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Escape HTML entities to prevent XSS when using v-html
 */
function escapeHtml(str: string): string {
  const div = document.createElement('div');
  div.appendChild(document.createTextNode(str));
  return div.innerHTML;
}

/**
 * Highlight matching text with <mark> tags
 *
 * @param text - The source text to highlight within
 * @param query - The search query to match
 * @returns HTML string with highlighted matches, safe for v-html
 */
export function highlightText(text: string, query: string): string {
  if (!query || !text) return escapeHtml(text || '');

  const escapedQuery = escapeRegex(query.trim());
  if (!escapedQuery) return escapeHtml(text);

  // Split query into individual words for multi-word matching
  const words = escapedQuery.split(/\s+/).filter(w => w.length > 0);
  if (words.length === 0) return escapeHtml(text);

  const pattern = new RegExp(`(${words.join('|')})`, 'gi');
  const parts = text.split(pattern);

  return parts
    .map(part => {
      const isMatch = words.some(w => new RegExp(`^${w}$`, 'i').test(part));
      if (isMatch) {
        return `<mark class="search-highlight">${escapeHtml(part)}</mark>`;
      }
      return escapeHtml(part);
    })
    .join('');
}

/**
 * Composable wrapper for reactive highlight usage
 */
export function useHighlight() {
  return { highlightText };
}
