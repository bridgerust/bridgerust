/**
 * Unit tests for error handling
 */
import { describe, it, expect } from "vitest";
import { EmbexClient } from "../../index";

describe("EmbexClient - Error Handling", () => {
  it("should handle invalid provider gracefully", () => {
    expect(() => {
      new EmbexClient("invalid_provider" as any, "http://localhost:6334", null);
    }).toThrow();
  });

  it("should handle missing URL", () => {
    expect(() => {
      new EmbexClient("qdrant", "", null);
    }).not.toThrow(); // May or may not throw depending on validation
  });
});
