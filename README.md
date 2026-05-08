# Odyssée Kids

Local-first interactive encyclopedia for children built with Vue 3 + Tauri.

## Current MVP scaffold

- Tauri sidecar-ready runtime profile command (`get_runtime_profile`) with hardware-aware defaults.
- Safety guard and visual style wrapper constants for child-safe prompting.
- Three-level navigation flow:
  - Domain wheel
  - Theme exploration
  - Encyclopedia card with curiosity feed
- i18n-ready frontend with English and French locales.
- Tailwind CSS styling and Lucide icons.

## Development

```bash
npm install
npm run dev
```

## Build frontend

```bash
npm run build
```

## Run Tauri app

```bash
npm run tauri dev
```
