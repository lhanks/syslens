# Syslens Brand Identity - Version 1

**Release Date:** December 2025
**Status:** Initial Release

## Overview

This is the first version of the Syslens brand identity, created to establish a professional, modern visual language for a desktop system monitoring application.

## Key Decisions

### Name Selection

**Chosen: Syslens**

After domain availability checks and competitive analysis, "Syslens" was selected because:
- Short and memorable (7 letters)
- Clear meaning: "Sys" (system) + "lens" (clarity/focus)
- Unique in the system monitoring space
- Available domains: .io, .app, .dev, .tech
- Strong metaphor: "viewing your system through a lens"

**Domain Status:**
- syslens.com: Taken (not critical for desktop app)
- syslens.io: Available (recommended)
- syslens.app: Available
- syslens.dev: Available

### Visual Concept

The logo combines two core elements:

1. **Circular Lens Ring** - Represents focus, clarity, and framing
2. **Graph Lines** - Visualizes real-time system metrics (CPU, memory, network)

The design uses:
- Gradient treatment for dynamic, live data feeling
- Clean geometric shapes for precision
- Crosshair overlay for targeting/focus metaphor
- Modern minimal style appropriate for technical tools

### Color Strategy

**Dark Theme Foundation:**
- Primary backgrounds: Deep Space (#0F172A) and Slate 950 (#020617)
- Rationale: Reduces eye strain during extended monitoring sessions
- Aligns with developer/power user preferences

**Accent Colors:**
- Electric Blue (#3B82F6): Primary actions, CPU metrics
- Cyber Purple (#8B5CF6): Secondary accents, memory metrics
- Info Cyan (#06B6D4): Network activity
- Success Green (#10B981): Healthy states
- Warning Amber (#F59E0B): Attention needed
- Critical Red (#EF4444): Errors/critical states

**Gradient Usage:**
Blue to purple gradient for brand elements and highlights, suggesting dynamic data flow.

### Typography

**Display/UI: Inter**
- Rationale: Modern, highly legible, excellent screen rendering
- Wide weight range (400-700) for hierarchy
- Open-source and web-safe

**Data/Monospace: JetBrains Mono**
- Rationale: Designed for developers, excellent number rendering
- Clear distinction between similar characters (0/O, 1/l/I)
- Consistent width for tabular data alignment

## Design Principles

1. **Clarity First** - Remove visual noise, present information clearly
2. **Professional Precision** - Accurate, reliable, trustworthy
3. **Modern Aesthetics** - Contemporary design without trendy gimmicks
4. **Technical Elegance** - Sophisticated without being overcomplicated

## File Structure

```
v1/
├── index.html              # Asset gallery
├── styleguide.html         # Complete design system documentation
├── example-site.html       # Sample landing page
├── assets/
│   ├── logo-icon.svg       # Icon (256x256)
│   ├── logo-full.svg       # Full logo with wordmark (512x160)
│   ├── favicon.svg         # Favicon (32x32)
│   ├── colors.css          # Color system with CSS custom properties
│   └── fonts.css           # Typography system
└── README.md               # This file
```

## Usage Guidelines

### Logo

**Minimum Sizes:**
- Icon: 24px
- Full logo: 120px width

**Clear Space:**
- Minimum: Equal to the diameter of the lens ring
- Recommended: 1.5x the lens diameter for emphasis

**Backgrounds:**
- Optimized for dark backgrounds (primary use)
- Works on Slate 900 (#0F172A) to Black (#000000)
- Can work on light backgrounds with color adjustments

### Colors

**Background Hierarchy:**
1. Deep Space (#0F172A) - Main app background
2. Slate 950 (#020617) - Secondary panels
3. Slate 800 (#1E293B) - Cards, elevated elements

**Text Hierarchy:**
1. Slate 200 (#E2E8F0) - Primary text, headings
2. Slate 400 (#94A3B8) - Secondary text, labels
3. White (#FFFFFF) - Emphasis, data values

**Interactive Elements:**
- Primary: Electric Blue (#3B82F6)
- Hover: Lighter blue (#60A5FA)
- Active: Darker blue (#2563EB)

### Typography

**Headings:**
- Use Inter Bold (700) or Semibold (600)
- Tight letter spacing (-0.02em)
- Tight line height (1.25)

**Body:**
- Use Inter Regular (400) or Medium (500)
- Normal letter spacing
- Relaxed line height (1.625)

**Data/Metrics:**
- Use JetBrains Mono Medium (500)
- Larger sizes for emphasis (2xl-4xl)
- Tight line height for compact display

## Implementation Notes

### Web/Electron

Include both CSS files in your HTML:

```html
<link rel="stylesheet" href="assets/colors.css">
<link rel="stylesheet" href="assets/fonts.css">
```

### Tauri/Desktop

For app icons:
1. Use `logo-icon.svg` as base
2. Generate PNG sizes: 16, 24, 32, 48, 64, 128, 256, 512
3. For Windows: Create .ico with multiple sizes
4. For taskbar: Use 256x256 or higher

### Favicon

Use `favicon.svg` directly in modern browsers:

```html
<link rel="icon" href="favicon.svg" type="image/svg+xml">
```

For legacy support, generate .ico from favicon.svg.

## Future Considerations

### Potential Improvements (v2)

1. **Light Theme Variant** - If user demand requires light mode
2. **Additional Logo Variations** - Monochrome versions for special contexts
3. **Extended Color Palette** - More data visualization colors if needed
4. **Icon Set** - Custom icons for common system components
5. **Animation Guidelines** - Motion design for live data updates

### Known Limitations

1. Logo uses gradients - may need simplified version for some contexts
2. Only optimized for dark backgrounds - light variant not included
3. No print specifications - digital-first design
4. Limited accessibility testing - should verify contrast ratios in production

## Credits

- **Design System:** Based on Tailwind CSS color philosophy
- **Fonts:** Inter by Rasmus Andersson, JetBrains Mono by JetBrains
- **Inspiration:** Modern developer tools (VS Code, GitHub Desktop, etc.)

## License

Brand assets are part of the Syslens project and follow the project's license terms.

## Change Log

### v1.0 (December 2025)

- Initial brand identity creation
- Logo design (icon, full, favicon)
- Color system with 50+ variables
- Typography system with Inter + JetBrains Mono
- Complete style guide documentation
- Example landing page
- Asset gallery
