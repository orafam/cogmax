import { createRemoteJWKSet, jwtVerify } from "jose";

const issuer = process.env.AUTH0_ISSUER_BASE_URL;
const audience = process.env.MCP_AUDIENCE;
const jwks = issuer ? createRemoteJWKSet(new URL(`${issuer}/.well-known/jwks.json`)) : undefined;

export async function authenticatedUser(request: Request) {
  const token = request.headers.get("authorization")?.replace(/^Bearer\s+/i, "");
  if (!token && process.env.NODE_ENV !== "production") return "development-user";
  if (!token || !jwks || !issuer || !audience) throw new Error("unauthorized");
  const { payload } = await jwtVerify(token, jwks, { issuer: `${issuer}/`, audience });
  if (!payload.sub) throw new Error("unauthorized");
  return payload.sub;
}
