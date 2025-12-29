# Syslens Marketing Website

Marketing and download page for the Syslens desktop application.

Built with [Next.js 16](https://nextjs.org) and [Tailwind CSS 4](https://tailwindcss.com).

## Development

```bash
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

## Production Build

```bash
npm run build
npm start
```

## Deployment to Vercel

### Option 1: Vercel CLI

```bash
# Install Vercel CLI globally (if not already installed)
npm i -g vercel

# Deploy from this directory
cd projects/website
vercel

# For production deployment
vercel --prod
```

### Option 2: Vercel Dashboard

1. Go to [vercel.com/new](https://vercel.com/new)
2. Import the repository
3. Set **Root Directory** to `projects/website`
4. Click Deploy

### Environment Variables

No environment variables are required for the basic marketing site.

## Project Structure

```
src/
  app/
    globals.css    # Syslens brand colors and styles
    layout.tsx     # Metadata and fonts
    page.tsx       # Landing page
public/
  logo-full.svg    # Full Syslens logo
  logo-icon.svg    # Icon mark
  favicon.svg      # Browser favicon
```

## Brand Assets

Brand colors and styling follow the Syslens brand guidelines. See `projects/ui/brand/BRAND.md` for the full brand identity documentation.
