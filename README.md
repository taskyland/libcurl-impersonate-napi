# libcurl-impersonate-napi

A Node.js binding for libcurl-impersonate that allows HTTP requests with browser fingerprint spoofing. Built with Rust and NAPI-RS for high performance and cross-platform compatibility.

## Features

- **Browser Impersonation**: Mimic popular browsers (Chrome, Firefox, Safari, Edge) with accurate TLS fingerprints
- **High Performance**: Built with Rust for optimal speed and memory safety
- **Async/Sync Support**: Choose between asynchronous and synchronous request execution
- **Full HTTP Support**: GET, POST, headers, cookies, redirects, compression
- **Multi-handle Support**: Concurrent request processing with curl multi interface
- **Cross-platform**: Works on Windows, macOS, Linux (including musl), and ARM architectures

## Installation

```bash
npm install @taskylizard/libcurl
```

## Quick Start

```javascript
const { setLibPath, Curl, CurlOpt, CurlInfo, globalInit } = require('@taskylizard/libcurl')
const path = require('path')

// Initialize libcurl globally
globalInit(3)

// Create a new curl instance
const curl = new Curl()

// Configure the request
curl.setOption(CurlOpt.Url, 'https://httpbin.org/get')
curl.setOption(CurlOpt.SslVerifyPeer, false)
curl.setOption(CurlOpt.FollowLocation, true)
curl.setOption(CurlOpt.AcceptEncoding, '') // Enable automatic decompression

// Impersonate Chrome browser
curl.impersonate('chrome136', true)

// Set cookies
curl.setCookies('session=abc123; theme=dark')

// Execute the request
async function makeRequest() {
  try {
    await curl.perform()

    // Get response information
    console.log('Status:', curl.status())
    console.log('Response Code:', curl.getInfoNumber(CurlInfo.ResponseCode))

    // Get headers and body
    const headers = curl.getRespHeaders().toString('utf8')
    const body = curl.getRespBody().toString('utf8')

    console.log('Headers:', headers)
    console.log('Body:', body)

    // Get cookies set by server
    const cookies = curl.getCookies()
    console.log('Cookies:', cookies)
  } catch (error) {
    console.error('Request failed:', error)
  } finally {
    curl.close()
  }
}

makeRequest()
```

## POST Request Example

```javascript
const curl = new Curl()

curl.setOption(CurlOpt.Url, 'https://httpbin.org/post')
curl.setOption(CurlOpt.Post, true)

// Set JSON payload
const data = JSON.stringify({ name: 'John', age: 30 })
curl.setBody(data)

// Set headers
curl.setHeadersRaw(['Content-Type: application/json', 'User-Agent: MyApp/1.0'])

// Impersonate Firefox
curl.impersonate('firefox135', true)

await curl.perform()
console.log('Response:', curl.getRespBody().toString())
curl.close()
```

## Browser Impersonation

Supported browser targets:

- Chrome: `chrome99`, `chrome100`, `chrome120`, `chrome136`, etc.
- Firefox: `firefox133`, `firefox135`
- Safari: `safari153`, `safari180`, `safari184`
- Edge: `edge99`, `edge101`
- Android Chrome: `chrome99android`, `chrome131android`

## Multi-handle for Concurrent Requests

```javascript
const { CurlMulti } = require('@taskylizard/libcurl')

const multi = new CurlMulti()
const curl1 = new Curl()
const curl2 = new Curl()

// Configure both curl instances
curl1.setOption(CurlOpt.Url, 'https://httpbin.org/delay/1')
curl2.setOption(CurlOpt.Url, 'https://httpbin.org/delay/2')

// Add to multi handle
multi.addHandle(curl1)
multi.addHandle(curl2)

// Execute concurrently
let running = multi.perform()
while (running > 0) {
  await multi.poll(1000)
  running = multi.getRunningHandles()
}

// Process results
const msg1 = multi.infoRead()
const msg2 = multi.infoRead()

curl1.close()
curl2.close()
multi.close()
```

## API Reference

### Curl Class

- `new Curl()` - Create new curl instance
- `setOption(option, value)` - Set curl option
- `setBody(data)` - Set request body (string or Buffer)
- `setHeadersRaw(headers)` - Set request headers array
- `impersonate(browser, defaultHeaders)` - Impersonate browser
- `perform()` - Execute request (async)
- `performSync()` - Execute request (sync)
- `getRespHeaders()` - Get response headers as Buffer
- `getRespBody()` - Get response body as Buffer
- `status()` - Get HTTP status code
- `close()` - Clean up resources

### Global Functions

- `globalInit(flags)` - Initialize libcurl globally
- `globalCleanup()` - Cleanup libcurl globally
- `setLibPath(path)` - Set path to libcurl library
- `getLibPath()` - Get current library path

## Platform Support

- Windows (x64, x86, ARM64)
- macOS (x64, ARM64)
- Linux (x64, ARM64, ARM, musl)
- FreeBSD (x64)
- Android (ARM)

## License

MIT
