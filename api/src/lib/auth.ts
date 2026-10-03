import { createRemoteJWKSet, jwtVerify } from "jose";

const issuer = process.env.AUTH0_ISSUER_BASE_URL;
const audience = process.env.MCP_AUDIENCE;
const jwks = issuer ? createRemoteJWKSet(new URL(`${issuer}/.well-known/jwks.json`)) : undefined;
const TENANT_CLAIM = "https://cogmax.dev/tenant_id";

export function tenantClaim(payload: Record<string, unknown>) {
  return typeof payload[TENANT_CLAIM] === "string" ? payload[TENANT_CLAIM] : undefined;
}

export async function authenticatedUser(request: Request) {
  const token = request.headers.get("authorization")?.replace(/^Bearer\s+/i, "");
  if (!token && process.env.NODE_ENV !== "production") return { userId: "development-user", tenantId: "development-tenant" };
  if (!token || !jwks || !issuer || !audience) throw new Error("unauthorized");
  const { payload } = await jwtVerify(token, jwks, { issuer: `${issuer}/`, audience });
  const tenantId = tenantClaim(payload as Record<string, unknown>);
  if (!payload.sub || !tenantId) throw new Error("tenant claim missing");
  return { userId: payload.sub, tenantId };
}
