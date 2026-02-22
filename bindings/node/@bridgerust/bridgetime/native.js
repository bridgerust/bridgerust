/* eslint-disable */
const { readFileSync } = require('node:fs')

let nativeBinding = null
const loadErrors = []

const isMusl = () => {
  if (process.platform !== 'linux') {
    return false
  }

  try {
    return readFileSync('/usr/bin/ldd', 'utf-8').includes('musl')
  } catch {
    try {
      return require('child_process')
        .execSync('ldd --version', { encoding: 'utf8' })
        .includes('musl')
    } catch {
      return false
    }
  }
}

function tryLoad(localFile, pkgName) {
  try {
    return require(localFile)
  } catch (e) {
    loadErrors.push(e)
  }

  if (pkgName) {
    try {
      return require(pkgName)
    } catch (e) {
      loadErrors.push(e)
    }
  }

  return null
}

function requireNative() {
  if (process.env.NAPI_RS_NATIVE_LIBRARY_PATH) {
    try {
      return require(process.env.NAPI_RS_NATIVE_LIBRARY_PATH)
    } catch (e) {
      loadErrors.push(e)
    }
  }

  if (process.platform === 'darwin') {
    if (process.arch === 'arm64') {
      return tryLoad('./bridgetime.darwin-arm64.node', '@bridgerust/bridgetime-darwin-arm64')
    }
    if (process.arch === 'x64') {
      return tryLoad('./bridgetime.darwin-x64.node', '@bridgerust/bridgetime-darwin-x64')
    }
  }

  if (process.platform === 'linux') {
    if (process.arch === 'x64') {
      if (isMusl()) {
        return (
          tryLoad('./bridgetime.linux-x64-musl.node', '@bridgerust/bridgetime-linux-x64-musl') ||
          tryLoad('./bridgetime.linux-x64-gnu.node', '@bridgerust/bridgetime-linux-x64-gnu')
        )
      }
      return tryLoad('./bridgetime.linux-x64-gnu.node', '@bridgerust/bridgetime-linux-x64-gnu')
    }
  }

  if (process.platform === 'win32') {
    if (process.arch === 'x64') {
      return tryLoad('./bridgetime.win32-x64-msvc.node', '@bridgerust/bridgetime-win32-x64-msvc')
    }
  }

  return null
}

nativeBinding = requireNative()

if (!nativeBinding) {
  throw new Error(`Failed to load native binding for ${process.platform}/${process.arch}\n${loadErrors.map((e) => `- ${e.message}`).join('\n')}`)
}

module.exports = nativeBinding
