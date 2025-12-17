/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'zots-primary': '#6366f1',
        'zots-secondary': '#8b5cf6',
        'zots-dark': '#0f172a',
        'zots-darker': '#020617',
        'zots-accent': '#22c55e',
        'zots-warning': '#f59e0b',
        'zots-error': '#ef4444',
      },
      fontFamily: {
        'mono': ['JetBrains Mono', 'Fira Code', 'monospace'],
      },
    },
  },
  plugins: [],
}
