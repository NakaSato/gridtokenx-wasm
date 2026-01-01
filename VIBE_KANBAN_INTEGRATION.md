# GridTokenX WASM with Vibe Kanban Integration

This project demonstrates the integration of the Vibe Kanban Web Companion into a Vite + React application.

## Installation

The Vibe Kanban Web Companion has been installed using npm:

```bash
npm install vibe-kanban-web-companion
```

## Integration

The `VibeKanbanWebCompanion` component is integrated at the app root in `src/App.jsx`:

```jsx
import { VibeKanbanWebCompanion } from 'vibe-kanban-web-companion';

function App() {
  return (
    <>
      <VibeKanbanWebCompanion />
      {/* Your app content */}
    </>
  );
}
```

## Development

Start the development server:

```bash
npm run dev
```

The application will be available at `http://localhost:3000`.

## Build

Build for production:

```bash
npm run build
```

The built files will be in the `dist/` directory.

## Preview Production Build

Preview the production build locally:

```bash
npm run preview
```

## Features

- ✅ Vibe Kanban Web Companion installed and integrated
- ✅ Renders at the app root
- ✅ No SSR/hydration errors (Vite doesn't use SSR by default)
- ✅ Build and type-check passes
- ✅ Development server runs successfully

## Package Manager

This project uses **npm** (detected from `package-lock.json`).

## Framework

This project uses **Vite + React** for a fast development experience.
