import { prisma } from "@/src/lib/prisma";
import { memoryInputSchema } from "@/src/domain/memory";
import { authenticatedUser } from "@/src/lib/auth";

export async function POST(request: Request) {
  const parsed = memoryInputSchema.safeParse(await request.json());
  if (!parsed.success) return Response.json({ error: parsed.error.flatten() }, { status: 400 });
  let userId: string; try { userId = await authenticatedUser(request); } catch { return Response.json({ error: "unauthorized" }, { status: 401 }); }
  const memory = await prisma.memory.upsert({ where: { userId_eventId: { userId, eventId: parsed.data.eventId } }, update: { content: parsed.data.content, kind: parsed.data.kind }, create: { ...parsed.data, userId } });
  return Response.json({ accepted: true, memory }, { status: 201 });
}
