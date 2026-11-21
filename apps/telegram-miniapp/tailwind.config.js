/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./app/**/*.{js,ts,jsx,tsx}",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        sprx: {
          50: "#f0f9ff",
          100: "#e0f2fe",
          200: "#bae6fd",
          300: "#7dd3fc",
          400: "#38bdf8",
          500: "#0ea5e9",
          600: "#0284c7",
          700: "#0369a1",
          800: "#075985",
          900: "#0c4a6e",
          950: "#082f49",
        },
        cyber: {
          dark: "#07090e",
          card: "#0d131f",
          border: "rgba(255, 255, 255, 0.08)",
          glow: "#0ea5e9",
          neon: "#00f2fe",
          emerald: "#10b981",
          purple: "#a855f7",
          gold: "#fbbf24",
        },
        tg: {
          bg: "var(--tg-theme-bg-color, #07090e)",
          text: "var(--tg-theme-text-color, #ffffff)",
          hint: "var(--tg-theme-hint-color, #8e9cae)",
          button: "var(--tg-theme-button-color, #0ea5e9)",
          "button-text": "var(--tg-theme-button-text-color, #ffffff)",
        },
      },
      boxShadow: {
        glow: "0 0 25px -5px rgba(14, 165, 233, 0.5)",
        "glow-lg": "0 0 50px -10px rgba(14, 165, 233, 0.6)",
        "glow-gold": "0 0 25px -5px rgba(251, 191, 36, 0.5)",
        "glow-emerald": "0 0 25px -5px rgba(16, 185, 129, 0.5)",
        glass: "0 8px 32px 0 rgba(0, 0, 0, 0.37)",
      },
      keyframes: {
        floatUp: {
          "0%": { opacity: "1", transform: "translateY(0) scale(1)" },
          "100%": { opacity: "0", transform: "translateY(-65px) scale(1.3)" },
        },
        pulseSlow: {
          "0%, 100%": { transform: "scale(1)", opacity: "0.8" },
          "50%": { transform: "scale(1.04)", opacity: "1" },
        },
        shimmer: {
          "0%": { transform: "translateX(-100%)" },
          "100%": { transform: "translateX(100%)" },
        },
      },
      animation: {
        "float-up": "floatUp 0.8s ease-out forwards",
        "pulse-slow": "pulseSlow 3s ease-in-out infinite",
        shimmer: "shimmer 2.5s infinite linear",
      },
      fontFamily: {
        sans: [
          "Inter",
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI",
          "Roboto",
          "sans-serif",
        ],
      },
    },
  },
  plugins: [],
};
