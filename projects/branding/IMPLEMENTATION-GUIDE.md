# Syslens Brand Implementation Guide

Quick reference for implementing the Syslens brand in the application.

## Quick Start

All brand assets are located in: `projects/branding/syslens/v1/`

### For Angular UI

1. **Import CSS Files**

Add to `projects/ui/src/styles.css`:

```css
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');
@import url('https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500&display=swap');
```

2. **Copy Color Variables to Tailwind Config**

Update `projects/ui/tailwind.config.js`:

```javascript
module.exports = {
  theme: {
    extend: {
      colors: {
        'syslens': {
          'deep-space': '#0F172A',
          'electric-blue': '#3B82F6',
          'cyber-purple': '#8B5CF6',
          'success-green': '#10B981',
          'warning-amber': '#F59E0B',
          'critical-red': '#EF4444',
          'info-cyan': '#06B6D4',
        },
      },
      fontFamily: {
        'sans': ['Inter', 'system-ui', 'sans-serif'],
        'mono': ['JetBrains Mono', 'Consolas', 'Monaco', 'monospace'],
      },
    },
  },
}
```

3. **Use in Components**

```html
<!-- Backgrounds -->
<div class="bg-slate-900">Main background</div>
<div class="bg-slate-800">Card background</div>

<!-- Text -->
<h1 class="text-slate-200 font-bold">Heading</h1>
<p class="text-slate-400">Secondary text</p>

<!-- Data values -->
<span class="font-mono text-2xl">87.5%</span>

<!-- Buttons -->
<button class="bg-blue-500 hover:bg-blue-600">Primary Action</button>

<!-- Status badges -->
<span class="text-green-500">Running</span>
<span class="text-amber-500">Warning</span>
<span class="text-red-500">Error</span>
```

### For Tauri (App Icons)

1. **Copy Logo Files**

```bash
cp projects/branding/syslens/v1/assets/logo-icon.svg projects/backend/icons/icon.svg
```

2. **Generate PNG Icons**

Use online tool or ImageMagick to generate from SVG:
- 16x16, 24x24, 32x32, 48x48, 64x64, 128x128, 256x256, 512x512

3. **Update tauri.conf.json**

```json
{
  "tauri": {
    "bundle": {
      "icon": [
        "icons/icon.ico",
        "icons/icon.png"
      ]
    }
  }
}
```

### For Desktop (Windows .ico)

1. **Install ico-convert or similar tool**

```bash
npm install -g ico-convert
```

2. **Generate .ico from PNG**

```bash
ico-convert projects/backend/icons/icon-256.png projects/backend/icons/icon.ico
```

## Color Usage Reference

### Backgrounds

```
Main App:     #0F172A (Slate 900)
Secondary:    #020617 (Slate 950)
Cards:        #1E293B (Slate 800)
Borders:      #334155 (Slate 700)
```

### Text

```
Primary:      #E2E8F0 (Slate 200)
Secondary:    #94A3B8 (Slate 400)
Disabled:     #64748B (Slate 500)
```

### Interactive

```
Primary:      #3B82F6 (Electric Blue)
Hover:        #60A5FA (Lighter Blue)
Secondary:    #8B5CF6 (Cyber Purple)
```

### Status Colors

```
Success:      #10B981 (Green)
Warning:      #F59E0B (Amber)
Error:        #EF4444 (Red)
Info:         #06B6D4 (Cyan)
```

## Typography Reference

### Font Families

```css
/* UI Text */
font-family: Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;

/* Data Display */
font-family: "JetBrains Mono", "Consolas", "Monaco", monospace;
```

### Font Sizes

```
Headings:     text-4xl (36px), text-3xl (30px), text-2xl (24px)
Body:         text-base (16px)
Small:        text-sm (14px)
Captions:     text-xs (12px)
Metrics:      text-4xl (36px) with mono font
```

### Font Weights

```
Regular:      400
Medium:       500
Semibold:     600
Bold:         700
```

## Component Examples

### Metric Card

```html
<div class="bg-slate-800 p-6 rounded-lg border-l-4 border-blue-500">
  <div class="font-mono text-4xl font-bold text-slate-200 mb-2">87.5%</div>
  <div class="text-sm text-slate-400 uppercase tracking-wide">CPU Usage</div>
</div>
```

### Status Badge

```html
<span class="inline-block px-3 py-1 bg-green-500/20 text-green-500 border border-green-500 rounded-full text-xs font-medium uppercase tracking-wide">
  Running
</span>
```

### Button

```html
<button class="px-6 py-3 bg-blue-500 hover:bg-blue-600 text-white font-medium rounded-lg transition-colors">
  Primary Action
</button>
```

### Data Table Row

```html
<tr class="border-b border-slate-700 hover:bg-slate-800">
  <td class="py-3 px-4 font-mono text-sm">12345</td>
  <td class="py-3 px-4 text-slate-200">chrome.exe</td>
  <td class="py-3 px-4 font-mono text-blue-500">42.8%</td>
  <td class="py-3 px-4 font-mono text-purple-500">1.2 GB</td>
</tr>
```

## Logo Usage

### App Title Bar

```html
<div class="flex items-center gap-3 p-4">
  <img src="assets/logo-icon.svg" alt="Syslens" width="32" height="32">
  <span class="text-xl font-bold tracking-tight">SYSLENS</span>
</div>
```

### Splash Screen

```html
<div class="flex flex-col items-center justify-center h-screen bg-slate-950">
  <img src="assets/logo-icon.svg" alt="Syslens" width="120" height="120">
  <h1 class="mt-6 text-4xl font-bold">SYSLENS</h1>
  <p class="mt-2 text-slate-400 text-lg">Clarity for your system</p>
</div>
```

### About Dialog

```html
<div class="p-8 bg-slate-800 rounded-lg">
  <div class="flex items-center gap-4 mb-6">
    <img src="assets/logo-icon.svg" alt="Syslens" width="64" height="64">
    <div>
      <h2 class="text-2xl font-bold">Syslens</h2>
      <p class="text-slate-400">Version 1.0.0</p>
    </div>
  </div>
  <p class="text-slate-300 leading-relaxed">
    Real-time system monitoring with clarity and precision.
  </p>
</div>
```

## Spacing System

Use Tailwind's spacing scale:

```
Tight:     gap-1, p-1   (0.25rem)
Small:     gap-2, p-2   (0.5rem)
Medium:    gap-4, p-4   (1rem)
Large:     gap-6, p-6   (1.5rem)
XL:        gap-8, p-8   (2rem)
```

## Border Radius

```
Small:     rounded-sm    (0.125rem)
Medium:    rounded-md    (0.375rem)
Large:     rounded-lg    (0.5rem)
XL:        rounded-xl    (0.75rem)
Full:      rounded-full  (9999px)
```

## Gradients

### Primary Gradient

```css
background: linear-gradient(135deg, #3B82F6 0%, #8B5CF6 100%);
```

```html
<div class="bg-gradient-to-br from-blue-500 to-purple-600">
  Gradient background
</div>
```

## Transitions

Use Tailwind's transition utilities:

```html
<button class="transition-colors duration-150">Fast</button>
<button class="transition-all duration-250">Normal</button>
<button class="transition-transform duration-350">Slow</button>
```

## Resources

- **Full Brand Document:** `projects/branding/syslens/BRAND.md`
- **Style Guide:** `projects/branding/syslens/v1/styleguide.html`
- **Asset Gallery:** `projects/branding/syslens/v1/index.html`
- **Example Site:** `projects/branding/syslens/v1/example-site.html`
- **Version Notes:** `projects/branding/syslens/v1/README.md`

## Quick Links

- Inter Font: https://fonts.google.com/specimen/Inter
- JetBrains Mono: https://fonts.google.com/specimen/JetBrains+Mono
- Tailwind CSS Docs: https://tailwindcss.com/docs
- SVG to ICO Converter: https://convertio.co/svg-ico/

## Notes

- All colors are from Tailwind's Slate palette for consistency
- Use `font-mono` class for all numeric data (percentages, sizes, counts)
- Prefer SVG logos when possible for crisp rendering at any size
- Test color contrast for accessibility (WCAG AA minimum)
- Keep clear space around logos equal to the lens diameter
