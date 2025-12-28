import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: [
      "tests/**/*.{test,spec}.{js,ts}",
      "src/tests/**/*.{test,spec}.{js,ts}",
    ],
    exclude: ["node_modules", "dist", ".idea", ".git"],
    testTimeout: 30000,
    hookTimeout: 30000,
  },
});
