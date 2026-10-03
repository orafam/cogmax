import { prisma } from "@/src/lib/prisma";
import { rankMemories } from "@/src/domain/memory";
import { authenticatedUser } from "@/src/lib/auth";

export async function POST(request: Request) {
  const body = await request.json();
  if (!body?.scope || !body?.query) return Response.json({ error: "scope and query are required" }, { status: 400 });
  let identity; try { identity = await authenticatedUser(request); } catch { return Response.json({ error: "unauthorized" }, { status: 401 }); }
  const member = await prisma.tenantMember.findUnique({ where: { tenantId_userId: { tenantId: identity.tenantId, userId: identity.userId } } }).catch(() => null);
  if (!member && process.env.NODE_ENV === "production") return Response.json({ error: "tenant membership required" }, { status: 403 });
  const memories = await prisma.memory.findMany({ where: { tenantId: identity.tenantId, userId: identity.userId, scope: body.scope, ...(body.project ? { project: body.project } : {}) } });
  return Response.json({ memories: rankMemories(body.query, memories) });
}
