import { describe, expect, it } from "vitest";
import { tenantClaim } from "./auth";

describe("tenant claim", () => {
  it("uses the namespaced Auth0 tenant claim", () => {
    expect(tenantClaim({ "https://cogmax.dev/tenant_id": "tenant-a" })).toBe("tenant-a");
  });
});
