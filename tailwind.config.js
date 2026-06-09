/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'daw-bg': '#0a0a0a',
        'daw-panel': '#121518',
        'daw-track': '#1a1d22',
        'daw-header': '#0d0f12',
        'daw-blue': '#00a8ff',
        'daw-green': '#00ff88',
        'daw-red': '#ff3366',
        'daw-border': '#2a2f36',
      },
      height: {
        'track': '48px',
      }
    },
  },
  plugins: [],
}
