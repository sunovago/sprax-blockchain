/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        background: '#0B0F19',
        surface: '#111827',
        'surface-elevated': '#1F2937',
        'border-subtle': '#1E293B',
        'border-focus': '#334155',
        primary: {
          50: '#ecfeff',
          100: '#cffafe',
          400: '#22d3ee',
          500: '#00F0FF',
          600: '#0891b2',
          700: '#0e7490',
        },
        accent: {
          purple: '#8B5CF6',
          emerald: '#10B981',
          rose: '#F43F5E',
          amber: '#F59E0B',
        },
        dark: {
          bg: '#0B0F19',
          card: '#111827',
          elevated: '#1E293B',
          border: '#22304A',
        }
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
    },
  },
  plugins: [],
}
