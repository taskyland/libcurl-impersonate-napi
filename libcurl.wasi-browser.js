import {
  createOnMessage as __wasmCreateOnMessageForFsProxy,
  getDefaultContext as __emnapiGetDefaultContext,
  instantiateNapiModuleSync as __emnapiInstantiateNapiModuleSync,
  WASI as __WASI,
} from '@napi-rs/wasm-runtime'



const __wasi = new __WASI({
  version: 'preview1',
})

const __wasmUrl = new URL('./libcurl.wasm32-wasi.wasm', import.meta.url).href
const __emnapiContext = __emnapiGetDefaultContext()


const __sharedMemory = new WebAssembly.Memory({
  initial: 4000,
  maximum: 65536,
  shared: true,
})

const __wasmFile = await fetch(__wasmUrl).then((res) => res.arrayBuffer())

const {
  instance: __napiInstance,
  module: __wasiModule,
  napiModule: __napiModule,
} = __emnapiInstantiateNapiModuleSync(__wasmFile, {
  context: __emnapiContext,
  asyncWorkPoolSize: 4,
  wasi: __wasi,
  onCreateWorker() {
    const worker = new Worker(new URL('./wasi-worker-browser.mjs', import.meta.url), {
      type: 'module',
    })

    return worker
  },
  overwriteImports(importObject) {
    importObject.env = {
      ...importObject.env,
      ...importObject.napi,
      ...importObject.emnapi,
      memory: __sharedMemory,
    }
    return importObject
  },
  beforeInit({ instance }) {
    for (const name of Object.keys(instance.exports)) {
      if (name.startsWith('__napi_register__')) {
        instance.exports[name]()
      }
    }
  },
})
export default __napiModule.exports
export const Curl = __napiModule.exports.Curl
export const CurlMulti = __napiModule.exports.CurlMulti
export const CurlError = __napiModule.exports.CurlError
export const CurlHttpVersion = __napiModule.exports.CurlHttpVersion
export const CurlImpersonate = __napiModule.exports.CurlImpersonate
export const CurlInfo = __napiModule.exports.CurlInfo
export const CurlIpResolve = __napiModule.exports.CurlIpResolve
export const CurlMOpt = __napiModule.exports.CurlMOpt
export const CurlOpt = __napiModule.exports.CurlOpt
export const CurlSslVersion = __napiModule.exports.CurlSslVersion
export const CurlWsFlag = __napiModule.exports.CurlWsFlag
export const getDefaultDirName = __napiModule.exports.getDefaultDirName
export const getDefaultLibPath = __napiModule.exports.getDefaultLibPath
export const getLibPath = __napiModule.exports.getLibPath
export const getVersion = __napiModule.exports.getVersion
export const globalCleanup = __napiModule.exports.globalCleanup
export const globalInit = __napiModule.exports.globalInit
export const setLibPath = __napiModule.exports.setLibPath
export const socketIsReadable = __napiModule.exports.socketIsReadable
export const socketIsWritable = __napiModule.exports.socketIsWritable
