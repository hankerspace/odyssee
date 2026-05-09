# Odyssée Kids

Odyssée Kids is a local-first MVP for an interactive children encyclopedia. A Vue 3 interface drives a Tauri/Rust backend that initializes a SQLite knowledge tree, detects the local hardware profile, resolves model paths, and runs `llama.cpp` / `stable-diffusion.cpp` sidecars when they are installed.

## MVP Features

- Three-level visual navigation: domain wheel, themed exploration, and encyclopedia article page.
- Fixed SQLite catalog seeded at startup: 6 domains, 18 sections, and 18 bilingual FR/EN articles.
- Text generation through a `llama.cpp` sidecar with a child-safe prompt guard and local cache.
- Image generation through a `stable-diffusion.cpp` sidecar with an educational style wrapper and local cache.
- Generation stops clearly when required sidecars or model files are missing.
- Runtime profile panel with CUDA, Metal, Vulkan, or CPU detection plus model, cache, and database paths.

## Architecture

- Frontend: `Vue 3`, `Tailwind CSS`, `Lucide Icons`, and `Vite`.
- Desktop wrapper: `Tauri 2` with Rust commands.
- Local storage: `SQLite` through `rusqlite`.
- Inference sidecars: command-line `llama.cpp` and `stable-diffusion.cpp` binaries.
- Backend modules:
  - `commands.rs`: Tauri command orchestration.
  - `constants.rs`: prompt, storage, and model constants.
  - `generation.rs`: prompt builders for local generation.
  - `hardware.rs`: hardware detection and accelerator flags.
  - `models.rs`: serialized DTO contracts shared with the frontend.
  - `paths.rs`: runtime path resolution and folder bootstrap.
  - `seeds.rs`: deterministic catalog seed data.
  - `sidecars.rs`: process adapters for external inference binaries.
  - `storage.rs`: SQLite schema, seed loading, catalog queries, and cache access.

## IntelliJ IDEA Setup

1. Install the recommended JetBrains plugins: Vue.js, Rust, TOML, and Node.js.
2. Install a Node.js version compatible with `package.json`: `20.19+` or `22.12+`.
3. Install stable Rust with Cargo: <https://rustup.rs/>.
4. On Windows, install the Tauri prerequisites if needed: Microsoft C++ Build Tools and WebView2 Runtime.
5. Open the project directory in IntelliJ IDEA and run commands from the integrated terminal.

## Developer Installation

```powershell
npm install
```

Prepare the local runtime directory tree used by the sidecars:

```powershell
npm run setup:runtime
```

Print the resolved runtime paths as JSON:

```powershell
npm run paths:runtime
```

## Run the Web UI Only

```powershell
npm run dev
```

This mode is useful for browser-only UI work. Tauri commands are unavailable, so backend-driven catalog and generation actions are not available.

## Run the Full Tauri Application

```powershell
npm run tauri:dev
```

At Tauri startup, the backend creates local folders, initializes SQLite, and exposes these commands:

- `detect_hardware`
- `get_runtime_profile`
- `get_model_status`
- `prepare_models`
- `get_catalog`
- `generate_article`
- `generate_image`

## Local Runtime Layout

On Windows, Odyssée uses `%APPDATA%\Odyssee`:

```text
%APPDATA%\Odyssee\
  bin\
    llama-cli.exe
    sd.exe
  models\
    phi-4-mini-instruct-q4_k_m.gguf
    sd_turbo.safetensors
  cache\
    images\
  odyssee.sqlite
```

On macOS and Linux, the equivalent directory is `~/.local/share/Odyssee`.

## Install Sidecars

### Text: `llama.cpp`

1. Build or download a `llama.cpp` CLI binary for your hardware.
2. Rename the CLI to `llama-cli.exe` on Windows or `llama-cli` elsewhere.
3. Copy the binary to the local `bin` directory printed by `npm run paths:runtime`.
4. The app downloads Phi-4 Mini Instruct `Q4_K_M` automatically on first Tauri startup if `phi-4-mini-instruct-q4_k_m.gguf` is missing. You can still copy the file manually to the local `models` directory to skip the download.

### Image: `stable-diffusion.cpp`

1. Build or download a `stable-diffusion.cpp` CLI binary for your hardware.
2. Rename the CLI to `sd.exe` on Windows or `sd` elsewhere.
3. Copy the binary to the local `bin` directory printed by `npm run paths:runtime`.
4. The app downloads SD-Turbo automatically on first Tauri startup if `sd_turbo.safetensors` is missing. You can still copy the file manually to the local `models` directory to skip the download.

The automatic model download needs network access only for missing model files. If it fails or the sidecar binaries are absent, generation commands return an explicit error. As soon as the expected binaries and models exist, the Rust commands use them and cache successful results locally.

## Scripts

- `npm run dev`: start the Vite web dev server.
- `npm run tauri:dev`: start the full desktop app in development mode.
- `npm run build` / `npm run build:web`: generate the production web bundle.
- `npm run build:desktop` / `npm run tauri:build`: build the packaged Tauri application.
- `npm run test:frontend`: run the frontend production build check.
- `npm run test:rust`: run Rust unit tests with Cargo.
- `npm run check`: run the web build and Rust tests.
- `npm run generate`: generate the production web assets.
- `npm run package`: package the desktop application.
- `npm run deploy:local`: local deployment alias for the packaged desktop build.
- `npm run setup:runtime`: create the local runtime folders for binaries, models, and cache.
- `npm run paths:runtime`: print all runtime paths as JSON.

## Build and Test

Frontend production build:

```powershell
npm run build:web
```

Rust backend tests:

```powershell
npm run test:rust
```

Full local check:

```powershell
npm run check
```

Packaged desktop application:

```powershell
npm run package
```

## Parental Safety Notes

- Application code does not send prompts or generated history to the cloud; first-start model bootstrap only downloads the public GGUF files when they are missing.
- Text prompts always include a child-safe instruction: factual, kind, and free from inappropriate content.
- Image prompts are wrapped in an educational illustration style: sticker style, clean lines, bright colors, and a white background.
- Generated history remains in the local SQLite and file cache.

## MVP Limitations

- First-start model download is implemented for the default GGUF files, but the sidecar binaries still need to be installed manually for the target hardware.
- LanceDB/vector RAG is not integrated in this first MVP; SQLite currently stores the navigation tree and cache.
- Real performance depends on the selected sidecar binaries and whether they were compiled for CUDA, Metal, Vulkan, or CPU.
