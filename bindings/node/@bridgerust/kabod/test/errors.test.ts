import { describe, it, expect } from "vitest";
import { KabodClient } from "../index";

describe("KabodClient Errors", () => {
  it("should throw error for invalid provider", () => {
    // Requires that KabodClient::new checks provider config
    expect(() => {
      // This might not throw if validation happens lazily.
      // But assuming KabodConfig validation happens in RustClient::new
      new KabodClient("invalid_provider", "http://localhost:6333", null);
    }).toThrow();
  });
});
