/**
 * Unit tests for error handling
 */
import { describe, it, expect } from "vitest";
import { KabodClient } from "../../index";

describe("KabodClient - Error Handling", () => {
  it("should handle invalid provider gracefully", () => {
    expect(() => {
      new KabodClient("invalid_provider" as any, "http://localhost:6334", null);
    }).toThrow();
  });

  it("should handle missing URL", () => {
    expect(() => {
      new KabodClient("qdrant", "", null);
    }).not.toThrow(); // May or may not throw depending on validation
  });
});
