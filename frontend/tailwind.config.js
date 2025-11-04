/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Catppuccin Mocha theme colors
        base: {
          DEFAULT: '#1e1e2e',
          darker: '#181825',
          darkest: '#11111b',
        },
        surface: {
          0: '#313244',
          1: '#45475a',
          2: '#585b70',
        },
        overlay: {
          0: '#6c7086',
          1: '#7f849c',
          2: '#9399b2',
        },
        text: {
          DEFAULT: '#cdd6f4',
          muted: '#bac2de',
          subtle: '#a6adc8',
        },
        // Catppuccin accent colors
        rosewater: '#f5e0dc',
        flamingo: '#f2cdcd',
        pink: '#f5c2e7',
        mauve: '#cba6f7',
        red: '#f38ba8',
        maroon: '#eba0ac',
        peach: '#fab387',
        yellow: '#f9e2af',
        green: '#a6e3a1',
        teal: '#94e2d5',
        sky: '#89dceb',
        sapphire: '#74c7ec',
        blue: '#89b4fa',
        lavender: '#b4befe',
      },
    },
  },
  plugins: [],
}
