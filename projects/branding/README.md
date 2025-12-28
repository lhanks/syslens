# Syslens Branding

Complete brand identity and design system for Syslens system monitoring application.

## Overview

This directory contains all brand assets, guidelines, and design resources for Syslens. The brand combines professional precision with modern aesthetics, using a lens metaphor to represent clarity and system visibility.

## Quick Navigation

### View Brand Assets
- **Asset Gallery:** [syslens/v1/index.html](syslens/v1/index.html) - Download logos and design files
- **Style Guide:** [syslens/v1/styleguide.html](syslens/v1/styleguide.html) - Interactive design system documentation
- **Example Site:** [syslens/v1/example-site.html](syslens/v1/example-site.html) - Sample landing page
- **Version History:** [syslens/history.html](syslens/history.html) - All brand versions

### Documentation
- **Master Brand Doc:** [syslens/BRAND.md](syslens/BRAND.md) - Complete brand guidelines
- **Implementation Guide:** [IMPLEMENTATION-GUIDE.md](IMPLEMENTATION-GUIDE.md) - Quick reference for developers
- **Version Notes:** [syslens/v1/README.md](syslens/v1/README.md) - Current version details

## Brand Summary

### Name
**Syslens** - "Sys" (system) + "lens" (clarity/focus)

**Tagline:** "Clarity for your system"

**Domain Availability:**
- syslens.io (available - recommended)
- syslens.app (available)
- syslens.dev (available)
- syslens.com (taken, but not critical)

### Visual Identity

**Logo Concept:**
- Circular lens frame representing focus and clarity
- Embedded graph lines showing real-time system metrics
- Modern geometric design with gradient treatment
- Works at all sizes from 16px favicon to large displays

**Color Palette:**
- Primary: Electric Blue (#3B82F6), Cyber Purple (#8B5CF6)
- Background: Deep Space (#0F172A), Slate tones
- Status: Success Green, Warning Amber, Critical Red, Info Cyan
- 50+ color variables for complete design system

**Typography:**
- **UI/Display:** Inter (400, 500, 600, 700)
- **Data/Metrics:** JetBrains Mono (400, 500)
- Complete type scale from 12px to 48px

## File Structure

```
projects/branding/
├── README.md                          # This file
├── IMPLEMENTATION-GUIDE.md            # Developer quick reference
└── syslens/
    ├── BRAND.md                       # Master brand document
    ├── history.html                   # Version comparison
    └── v1/                            # Current version
        ├── index.html                 # Asset gallery
        ├── styleguide.html            # Design system docs
        ├── example-site.html          # Landing page example
        ├── README.md                  # Version notes
        └── assets/
            ├── logo-icon.svg          # Icon only (256x256)
            ├── logo-full.svg          # Full logo + wordmark
            ├── favicon.svg            # Favicon (32x32)
            ├── colors.css             # Color system CSS
            └── fonts.css              # Typography CSS
```

## Quick Start for Developers

### 1. View the Brand

Open in browser:
```
projects/branding/syslens/v1/styleguide.html
```

### 2. Copy Assets

Logo files are in:
```
projects/branding/syslens/v1/assets/
```

### 3. Implement in Angular

Add to `projects/ui/tailwind.config.js`:
```javascript
colors: {
  'syslens': {
    'electric-blue': '#3B82F6',
    'cyber-purple': '#8B5CF6',
    // ... see IMPLEMENTATION-GUIDE.md
  }
}
```

### 4. Use in Components

```html
<div class="bg-slate-900 text-slate-200">
  <h1 class="text-4xl font-bold">Syslens</h1>
  <span class="font-mono text-2xl text-blue-500">87.5%</span>
</div>
```

See [IMPLEMENTATION-GUIDE.md](IMPLEMENTATION-GUIDE.md) for complete usage examples.

## Key Design Decisions

### Why "Syslens"?
- Memorable and pronounceable (7 letters)
- Clear meaning without being generic
- Available domains (.io, .app, .dev)
- Strong verb/noun flexibility ("lens into your system")

### Why Dark Theme?
- Reduces eye strain during extended monitoring sessions
- Preferred by target audience (developers, sysadmins)
- Better contrast for data visualization
- Modern, professional appearance

### Why Lens Metaphor?
- Represents clarity and focus (core value proposition)
- Visual connection to "viewing" system details
- Creates distinctive icon that stands out
- Works at all sizes from favicon to splash screen

### Why These Colors?
- Electric Blue: High visibility, tech-forward, primary actions
- Cyber Purple: Secondary accent, complements blue
- Data colors: Clear status indication (green/amber/red/cyan)
- Slate neutrals: Professional, not too harsh on eyes

### Why These Fonts?
- **Inter:** Modern, highly legible UI font with excellent screen rendering
- **JetBrains Mono:** Designed for developers, perfect numeric clarity
- Both are open-source and web-safe

## Usage Guidelines

### Logo
- Minimum size: 24px for icon, 120px for full logo
- Clear space: Equal to lens diameter
- Optimized for dark backgrounds
- Never distort, stretch, or add effects

### Colors
- Use Electric Blue for primary actions
- Use data colors consistently (green=good, red=critical)
- Maintain WCAG AA contrast ratios minimum
- Dark backgrounds: Slate 900-950
- Light text: Slate 200-400

### Typography
- Use Inter for all UI text
- Use JetBrains Mono for all numeric data
- Headings: Bold (700) or Semibold (600)
- Body: Regular (400) or Medium (500)
- Data: Medium (500) mono

## Resources

### Design Files
All assets are SVG format for infinite scalability:
- `logo-icon.svg` - App icon, taskbar icon
- `logo-full.svg` - Headers, splash screens
- `favicon.svg` - Browser tabs, bookmarks

### CSS Files
Ready-to-use stylesheets:
- `colors.css` - 50+ CSS custom properties
- `fonts.css` - Complete typography system

### Documentation
- `styleguide.html` - Interactive component library
- `example-site.html` - Real-world application example
- `BRAND.md` - Complete brand guidelines
- `IMPLEMENTATION-GUIDE.md` - Developer quick reference

## Next Steps

### To Use in Application

1. **Review the style guide:**
   ```
   Open: projects/branding/syslens/v1/styleguide.html
   ```

2. **Copy logo to app:**
   ```bash
   cp projects/branding/syslens/v1/assets/logo-icon.svg projects/backend/icons/
   ```

3. **Update Tailwind config:**
   See [IMPLEMENTATION-GUIDE.md](IMPLEMENTATION-GUIDE.md) for color variables

4. **Apply typography:**
   Use Inter for UI, JetBrains Mono for data

### To Generate App Icons

1. Convert logo-icon.svg to PNG at multiple sizes
2. Create .ico file for Windows
3. Update tauri.conf.json with icon paths

See [IMPLEMENTATION-GUIDE.md](IMPLEMENTATION-GUIDE.md) for detailed steps.

## Future Improvements (v2)

Potential enhancements for next version:
- Light theme variant (if needed)
- Additional icon sizes/formats
- Animation guidelines for live data
- Extended color palette for more chart types
- Print specifications
- Accessibility testing results

## Credits

- **Design System:** Based on Tailwind CSS color philosophy
- **Fonts:** Inter by Rasmus Andersson, JetBrains Mono by JetBrains
- **Created:** December 2025
- **Version:** 1.0 (Initial Release)

## License

Brand assets are part of the Syslens project and follow the project's license terms.

---

**Questions?** Review the [full brand guidelines](syslens/BRAND.md) or [implementation guide](IMPLEMENTATION-GUIDE.md).
