import { computed, type Ref } from 'vue';
import type { SearchResult } from '../types/search';

export interface ResultGroup {
  type: 'bookmark' | 'file' | 'plugin';
  label: string;
  icon: string;
  results: SearchResult[];
  collapsed: boolean;
}

export interface GroupedResults {
  groups: ResultGroup[];
  totalCount: number;
  hasMultipleGroups: boolean;
}

/**
 * Groups search results by type (bookmark/file)
 * @param results - Reactive ref to search results array
 * @returns Computed grouped results with metadata
 */
export function useGroupedResults(results: Ref<SearchResult[]>) {
  const groupConfig: Record<string, { label: string; icon: string; order: number }> = {
    bookmark: {
      label: 'Bookmarks',
      icon: '🔖',
      order: 1,
    },
    file: {
      label: 'Files',
      icon: '📄',
      order: 2,
    },
    plugin: {
      label: 'Plugins',
      icon: '🧩',
      order: 3,
    },
  };

  const groupedResults = computed<GroupedResults>(() => {
    const groups: Map<string, SearchResult[]> = new Map();

    // Group results by type
    for (const result of results.value) {
      const type = result.type;
      if (!groups.has(type)) {
        groups.set(type, []);
      }
      groups.get(type)!.push(result);
    }

    // Convert to array and sort by config order
    const groupArray: ResultGroup[] = Array.from(groups.entries())
      .map(([type, items]) => ({
        type: type as 'bookmark' | 'file' | 'plugin',
        label: groupConfig[type]?.label ?? type,
        icon: groupConfig[type]?.icon ?? '📁',
        results: items,
        collapsed: false,
      }))
      .sort((a, b) => {
        const orderA = groupConfig[a.type]?.order ?? 99;
        const orderB = groupConfig[b.type]?.order ?? 99;
        return orderA - orderB;
      });

    return {
      groups: groupArray,
      totalCount: results.value.length,
      hasMultipleGroups: groupArray.length > 1,
    };
  });

  return {
    groupedResults,
    groupConfig,
  };
}

/**
 * Flattens grouped results back to a single array with group headers
 * Useful for virtual scrolling with mixed content
 */
export interface FlattenedItem {
  type: 'header' | 'result';
  data: ResultGroup | SearchResult;
  groupType?: 'bookmark' | 'file' | 'plugin';
}

export function useFlattenedGroups(groupedResults: Ref<GroupedResults>) {
  const flattenedItems = computed<FlattenedItem[]>(() => {
    const items: FlattenedItem[] = [];

    for (const group of groupedResults.value.groups) {
      if (group.collapsed) {
        // Only add header for collapsed groups
        items.push({
          type: 'header',
          data: group,
          groupType: group.type,
        });
      } else {
        // Add header
        items.push({
          type: 'header',
          data: group,
          groupType: group.type,
        });
        // Add all results in group
        for (const result of group.results) {
          items.push({
            type: 'result',
            data: result,
            groupType: group.type,
          });
        }
      }
    }

    return items;
  });

  return {
    flattenedItems,
  };
}
