/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{html,ts}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // Dark theme colors
        'syslens': {
          'bg': {
            'primary': '#0f0f0f',
            'secondary': '#1a1a1a',
            'tertiary': '#252525',
            'hover': '#2a2a2a',
          },
          'border': {
            'primary': '#333333',
            'secondary': '#404040',
          },
          'text': {
            'primary': '#e5e5e5',
            'secondary': '#a0a0a0',
            'muted': '#666666',
          },
          'accent': {
            'blue': '#3b82f6',
            'green': '#22c55e',
            'yellow': '#eab308',
            'red': '#ef4444',
            'purple': '#8b5cf6',
            'cyan': '#06b6d4',
            'orange': '#f97316',
          }
        }
      },
      fontFamily: {
        'sans': ['Inter', 'system-ui', 'sans-serif'],
        'mono': ['JetBrains Mono', 'Consolas', 'monospace'],
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        'spin-slow': 'spin 3s linear infinite',
      }
    },
  },
  plugins: [],
}
