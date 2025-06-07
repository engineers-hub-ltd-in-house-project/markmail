import "@testing-library/jest-dom";
import { vi } from "vitest";

// SVGのmock
vi.mock("svelte/transition", () => {
  return {
    fade: () => ({}),
    fly: () => ({}),
    slide: () => ({}),
  };
});

// localStorage mock if not available
if (typeof window.localStorage === "undefined") {
  const localStorageMock = (() => {
    let store: Record<string, string> = {};
    return {
      getItem: vi.fn((key: string): string | null => {
        return store[key] || null;
      }),
      setItem: vi.fn((key: string, value: string): void => {
        store[key] = value.toString();
      }),
      removeItem: vi.fn((key: string): void => {
        delete store[key];
      }),
      clear: vi.fn((): void => {
        store = {};
      }),
      key: vi.fn((index: number): string | null => {
        return Object.keys(store)[index] || null;
      }),
      length: 0,
    };
  })();
  Object.defineProperty(window, "localStorage", {
    value: localStorageMock,
  });
}

// Fetch API mock if not available
if (typeof window.fetch === "undefined") {
  global.fetch = vi.fn().mockImplementation(() =>
    Promise.resolve({
      ok: true,
      json: () => Promise.resolve({}),
    }),
  );
}

// Match media mock
if (typeof window.matchMedia === "undefined") {
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    value: vi.fn().mockImplementation((query) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(), // Deprecated
      removeListener: vi.fn(), // Deprecated
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  });
}

// ResizeObserver mock
if (typeof window.ResizeObserver === "undefined") {
  window.ResizeObserver = class ResizeObserver {
    observe = vi.fn();
    unobserve = vi.fn();
    disconnect = vi.fn();
  };
}

// Clean up mock implementations after tests
afterEach(() => {
  vi.clearAllMocks();
});
