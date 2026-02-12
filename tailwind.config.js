/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./client/src/**/*.rs",      // Scan client Rust files
    "./server/src/**/*.rs",      // Scan server Rust files (for htmx fragments)
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
