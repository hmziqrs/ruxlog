# Ruxlog Shared

Shared Tailwind CSS v4 configuration for all Ruxlog frontend projects.

## Features

- **Tailwind CSS v4** - Latest version with modern CSS features
- **Unified Theme** - Consistent design tokens across all projects
- **Typography Plugin** - Beautiful typographic defaults
- **Animations** - Smooth animations with tailwindcss-animate
- **Dark Mode** - Built-in dark mode support with custom variants
- **Editor.js Styles** - Pre-configured styles for Editor.js integration
- **Toast/Sonner Styles** - Custom toast notification styles

## Usage

### In Admin/Consumer Dioxus Projects

1. Import the shared Tailwind CSS in your local `tailwind.css`:

```css
@import "../ruxlog-shared/tailwind.css";
```

2. Build with Tailwind CLI:

```bash
bunx --bun @tailwindcss/cli -i ../ruxlog-shared/tailwind.css -o assets/tailwind.css --watch
```

## Theme Tokens

The shared configuration includes comprehensive theme tokens:

- **Colors**: background, foreground, primary, secondary, muted, accent, destructive, border, input, ring
- **Charts**: chart-1 through chart-5
- **Sidebar**: sidebar colors for navigation components
- **Radius**: Configurable border radius (sm, md, lg, xl)

## Customization

To customize the theme for your project, you can override CSS variables in your local styles after importing the shared configuration.

## Dark Mode

Dark mode is automatically supported using the `.dark` class. The theme includes:
- Custom dark mode variants
- Media query based preference detection
- Consistent color tokens that adapt to theme

## Dependencies

- `tailwindcss` (v4.1.4)
- `@tailwindcss/cli` (^4.1.17)
- `@tailwindcss/typography` (^0.5.19)
- `tailwindcss-animate` (^1.0.7)
- `tw-animate-css` (^1.4.0)
