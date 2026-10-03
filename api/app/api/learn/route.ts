import { prisma } from "@/src/lib/prisma";
import { memoryInputSchema } from "@/src/domain/memory";
import { authenticatedUser } from "@/src/lib/auth";

export async function POST(request: Request) {
  const parsed = memoryInputSchema.safeParse(await request.json());
  if (!parsed.success) return Response.json({ error: parsed.error.flatten() }, { status: 400 });
  let identity; try { identity = await authenticatedUser(request); } catch { return Response.json({ error: "unauthorized" }, { status: 401 }); }
  const member = identity.tenantId ? await prisma.tenantMember.findUnique({ where: { tenantId_userId: { tenantId: identity.tenantId, userId: identity.userId } } }).catch(() => null) : null;
  if (identity.tenantId && !member && process.env.NODE_ENV === "production") return Response.json({ error: "tenant membership required" }, { status: 403 });
  const existing = await prisma.memory.findFirst({ where: { tenantId: identity.tenantId ?? null, userId: identity.userId, eventId: parsed.data.eventId } });
  const memory = existing
    ? await prisma.memory.update({ where: { id: existing.id }, data: { content: parsed.data.content, kind: parsed.data.kind } })
    : await prisma.memory.create({ data: { ...parsed.data, userId: identity.userId, tenantId: identity.tenantId } });
  return Response.json({ accepted: true, memory }, { status: 201 });
}
