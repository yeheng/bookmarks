/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        dark: {
          bg: '#1a1a1a',
          'input-bg': '#2a2a2a',
          'text-primary': '#e0e0e0',
          'text-secondary': '#9a9a9a',
          accent: '#ff6b6b',
          border: '#3a3a3a',
        },
        light: {
          bg: '#ffffff',
          'input-bg': '#f5f5f5',
          'text-primary': '#1a1a1a',
          'text-secondary': '#6a6a6a',
          accent: '#ff6b6b',
          border: '#e0e0e0',
        }
      },
      fontSize: {
        'search-input': '18px',
        'result-title': '14px',
        'result-subtitle': '12px',
      },
      spacing: {
        'window': '16px',
        'input': '12px 16px',
        'result-item': '12px 16px',
      },
      borderRadius: {
        'input': '8px',
        'result': '6px',
      },
      transitionDuration: {
        'window': '150ms',
        'highlight': '100ms',
      }
    },
  },
  plugins: [],
}
