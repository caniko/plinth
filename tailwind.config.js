/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./crates/client/src/**/*.rs",
    "./crates/server/src/**/*.rs",
    "./index.html",
  ],
  theme: {
    extend: {
      // Custom colors, fonts, spacing can be added here
    },
  },
  plugins: [],
  darkMode: 'class',      // Enable dark mode via class
}
