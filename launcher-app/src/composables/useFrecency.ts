/**
 * useFrecency Composable
 *
 * Provides utilities for visualizing and explaining Frecency scores.
 * Frecency = Frequency + Recency (Mozilla's algorithm for scoring visited items)
 *
 * Score calculation factors:
 * - Visit count (frequency)
 * - Time since last visit (recency)
 * - Visit patterns (time decay)
 */

import { computed, type ComputedRef } from 'vue'

export interface FrecencyScore {
  /** Raw frecency score (0-100+) */
  raw: number
  /** Normalized score (0-100) for display */
  normalized: number
  /** Visual level (1-5 stars) */
  level: 1 | 2 | 3 | 4 | 5
  /** Color indicator */
  color: 'low' | 'medium' | 'high' | 'very-high'
  /** Percentage for progress bars (0-100) */
  percentage: number
  /** Human-readable label */
  label: string
  /** Tooltip breakdown */
  breakdown?: FrecencyBreakdown
}

export interface FrecencyBreakdown {
  /** Number of visits */
  visitCount?: number
  /** Days since last visit */
  daysSinceLastVisit?: number
  /** Time decay factor (0-1) */
  timeDecayFactor?: number
  /** Raw frequency score */
  frequencyScore?: number
  /** Raw recency score */
  recencyScore?: number
}

/**
 * Calculate frecency visualization data from raw score
 */
export function useFrecency(rawScore: number, breakdown?: FrecencyBreakdown): ComputedRef<FrecencyScore> {
  return computed(() => {
    // Normalize score to 0-100 range (assuming max score of 200)
    const normalized = Math.min(Math.round((rawScore / 200) * 100), 100)

    // Calculate visual level (1-5 stars)
    let level: 1 | 2 | 3 | 4 | 5
    if (normalized >= 80) level = 5
    else if (normalized >= 60) level = 4
    else if (normalized >= 40) level = 3
    else if (normalized >= 20) level = 2
    else level = 1

    // Determine color indicator
    let color: 'low' | 'medium' | 'high' | 'very-high'
    if (normalized >= 75) color = 'very-high'
    else if (normalized >= 50) color = 'high'
    else if (normalized >= 25) color = 'medium'
    else color = 'low'

    // Generate human-readable label
    let label: string
    if (normalized >= 80) label = 'Very Frequent'
    else if (normalized >= 60) label = 'Frequent'
    else if (normalized >= 40) label = 'Moderate'
    else if (normalized >= 20) label = 'Occasional'
    else label = 'Rare'

    return {
      raw: rawScore,
      normalized,
      level,
      color,
      percentage: normalized,
      label,
      breakdown,
    }
  })
}

/**
 * Format frecency breakdown for tooltip display
 */
export function formatFrecencyBreakdown(breakdown?: FrecencyBreakdown): string {
  if (!breakdown) return 'Score based on usage frequency and recency'

  const parts: string[] = []

  if (breakdown.visitCount !== undefined) {
    parts.push(`${breakdown.visitCount} visit${breakdown.visitCount !== 1 ? 's' : ''}`)
  }

  if (breakdown.daysSinceLastVisit !== undefined) {
    const days = breakdown.daysSinceLastVisit
    if (days === 0) {
      parts.push('used today')
    } else if (days === 1) {
      parts.push('used yesterday')
    } else if (days < 7) {
      parts.push(`used ${days} days ago`)
    } else if (days < 30) {
      parts.push(`used ${Math.floor(days / 7)} weeks ago`)
    } else {
      parts.push(`used ${Math.floor(days / 30)} months ago`)
    }
  }

  if (breakdown.timeDecayFactor !== undefined) {
    const decayPercent = Math.round(breakdown.timeDecayFactor * 100)
    parts.push(`${decayPercent}% relevance`)
  }

  return parts.join(' • ')
}

/**
 * Get star rating emoji based on level
 */
export function getFrecencyStars(level: 1 | 2 | 3 | 4 | 5): string {
  const stars = ['★', '★★', '★★★', '★★★★', '★★★★★']
  return stars[level - 1]
}

/**
 * Get color class for frecency indicator
 */
export function getFrecencyColorClass(color: 'low' | 'medium' | 'high' | 'very-high'): string {
  const colorMap = {
    'low': 'text-gray-400',
    'medium': 'text-amber-500',
    'high': 'text-coral-500',
    'very-high': 'text-coral-600',
  }
  return colorMap[color]
}

/**
 * Get background color class for frecency progress bar
 */
export function getFrecencyBgClass(color: 'low' | 'medium' | 'high' | 'very-high'): string {
  const bgMap = {
    'low': 'bg-gray-400',
    'medium': 'bg-amber-500',
    'high': 'bg-coral-500',
    'very-high': 'bg-coral-600',
  }
  return bgMap[color]
}

/**
 * Mock frecency score generator (for demo purposes)
 * In production, this comes from the backend
 */
export function generateMockFrecencyScore(id: string): number {
  // Use ID hash to generate consistent random scores
  const hash = id.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0)
  return (hash % 150) + 10 // Scores between 10-160
}
