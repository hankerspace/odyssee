import { join } from 'node:path'
import { runtimePaths } from './runtime-paths.mjs'

const paths = runtimePaths()

console.log(JSON.stringify({
  dataDirectory: paths.dataDir,
  binariesDirectory: paths.binDir,
  modelsDirectory: paths.modelsDir,
  cacheDirectory: paths.cacheDir,
  imageCacheDirectory: paths.imageCacheDir,
  databasePath: join(paths.dataDir, 'odyssee.sqlite'),
  llmBinary: join(paths.binDir, paths.llamaBinary),
  imageBinary: join(paths.binDir, paths.imageBinary),
  llmModel: join(paths.modelsDir, 'phi-4-mini-instruct-q4_k_m.gguf'),
  imageModel: join(paths.modelsDir, 'sd_turbo.safetensors'),
}, null, 2))