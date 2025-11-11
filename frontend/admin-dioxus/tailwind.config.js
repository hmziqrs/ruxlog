/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: 'all',
  content: ['./src/**/*.{rs,html,css}', './dist/**/*.html', './index.html'],
  theme: {
    extend: {},
  },
  theme: {
    extend: {
      screens: {
        xs: '410px',
      },
      colors: {
        background: 'var(--background)',
        foreground: 'var(--foreground)',
      },
      keyframes: {
        keyframes: {
          'accordion-down': {
            from: { height: '0' },
            to: { height: 'var(--radix-accordion-content-height)' },
          },
          'accordion-up': {
            from: { height: 'var(--radix-accordion-content-height)' },
            to: { height: '0' },
          },
        },
        animation: {
          'accordion-down': 'accordion-down 0.2s ease-out',
          'accordion-up': 'accordion-up 0.2s ease-out',
        },
      },
    },
  },
  plugins: [require('tailwindcss-animate'), require('@tailwindcss/typography')],
};


