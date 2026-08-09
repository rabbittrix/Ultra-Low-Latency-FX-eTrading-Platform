/** @type {import('tailwindcss').Config} */
export default {
  safelist: ['bg-orange-600', 'bg-red-600', 'bg-blue-600', 'bg-yellow-500'],
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        background: 'var(--background)',
        foreground: 'var(--foreground)',
      },
    },
  },
  plugins: [],
};
