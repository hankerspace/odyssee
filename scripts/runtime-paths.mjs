import { homedir, platform } from 'node:os'
import { join } from 'node:path'

export function runtimePaths() {
  const isWindows = platform() === 'win32'
  const dataDir = isWindows
    ? join(process.env.APPDATA ?? join(homedir(), 'AppData', 'Roaming'), 'Odyssee')
    : join(homedir(), '.local', 'share', 'Odyssee')

  return {
    dataDir,
    binDir: join(dataDir, 'bin'),
    modelsDir: join(dataDir, 'models'),
    cacheDir: join(dataDir, 'cache'),
    imageCacheDir: join(dataDir, 'cache', 'images'),
    llamaBinary: isWindows ? 'llama-cli.exe' : 'llama-cli',
    imageBinary: isWindows ? 'sd.exe' : 'sd',
  }
}