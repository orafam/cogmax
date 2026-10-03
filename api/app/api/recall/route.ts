import { prisma } from "@/src/lib/prisma";
import { rankMemories } from "@/src/domain/memory";
import { authenticatedUser } from "@/src/lib/auth";

export async function POST(request: Request) {
  const body = await request.json();
  if (!body?.scope || !body?.query) return Response.json({ error: "scope and query are required" }, { status: 400 });
  let userId: string; try { userId = await authenticatedUser(request); } catch { return Response.json({ error: "unauthorized" }, { status: 401 }); }
  const memories = await prisma.memory.findMany({ where: { userId, scope: body.scope, ...(body.project ? { project: body.project } : {}) } });
  return Response.json({ memories: rankMemories(body.query, memories) });
}
