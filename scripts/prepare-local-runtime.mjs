import { mkdirSync } from 'node:fs'
import { join } from 'node:path'
import { runtimePaths } from './runtime-paths.mjs'

const paths = runtimePaths()

for (const directory of [paths.dataDir, paths.binDir, paths.modelsDir, paths.cacheDir, paths.imageCacheDir]) {
  mkdirSync(directory, { recursive: true })
}

console.log('Odyssée local runtime folders are ready:')
console.log(`- Data: ${paths.dataDir}`)
console.log(`- Binaries: ${paths.binDir}`)
console.log(`- Models: ${paths.modelsDir}`)
console.log(`- Cache: ${paths.cacheDir}`)
console.log(`- Expected llama.cpp binary: ${join(paths.binDir, paths.llamaBinary)}`)
console.log(`- Expected stable-diffusion.cpp binary: ${join(paths.binDir, paths.imageBinary)}`)