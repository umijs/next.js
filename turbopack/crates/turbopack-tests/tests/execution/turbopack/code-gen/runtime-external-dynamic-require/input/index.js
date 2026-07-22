function load(request) {
  return require(request)
}

it('loads an allowed runtime external', () => {
  expect(load('node:path').basename('/tmp/example.txt')).toBe('example.txt')
})

it('maps the request before invoking the runtime hook', () => {
  const runtimeRequire = Symbol.for('@utoo/pack/runtime-require')
  let resolvedRequest
  globalThis[runtimeRequire] = request => {
    resolvedRequest = request
    return { request }
  }

  try {
    expect(load('path-alias')).toEqual({ request: 'node:path' })
    expect(resolvedRequest).toBe('node:path')
  } finally {
    delete globalThis[runtimeRequire]
  }
})

it('rejects requests outside the CommonJS external map', () => {
  expect(() => load('node:fs')).toThrow(
    'Dynamic require "node:fs" is not declared as a CommonJS external'
  )

  try {
    load('node:fs')
  } catch (error) {
    expect(error.code).toBe('MODULE_NOT_FOUND')
  }
})
