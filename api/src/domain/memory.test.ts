import { describe, expect, it } from "vitest";
import { memoryInputSchema, rankMemories } from "./memory";

describe("memory contract", () => {
  it("rejects an empty memory", () => {
    expect(() => memoryInputSchema.parse({ eventId: "e1", scope: "user:u1", kind: "Decision", content: "" })).toThrow();
  });

  it("ranks exact query matches first", () => {
    const result = rankMemories("vercel", [
      { id: "2", content: "Auth0 setup", kind: "Decision", scope: "user:u1", createdAt: new Date("2026-01-01") },
      { id: "1", content: "Deploy vault on Vercel", kind: "Decision", scope: "user:u1", createdAt: new Date("2026-01-02") },
    ]);
    expect(result[0].id).toBe("1");
  });
});
