import { prisma } from "@/src/lib/prisma";
import { authenticatedUser } from "@/src/lib/auth";
import { rankMemories, memoryInputSchema } from "@/src/domain/memory";

export async function POST(request: Request) {
  let identity; try { identity = await authenticatedUser(request); } catch { return Response.json({ jsonrpc: "2.0", error: { code: -32001, message: "unauthorized" } }, { status: 401 }); }
  const body = await request.json();
  const id = body.id ?? null;
  if (body.method === "initialize") return Response.json({ jsonrpc: "2.0", id, result: { protocolVersion: "2025-03-26", capabilities: { tools: {} }, serverInfo: { name: "cogmax-vault", version: "0.1.0" } } });
  if (body.method === "tools/list") return Response.json({ jsonrpc: "2.0", id, result: { tools: [{ name: "recall", description: "Recall memories", inputSchema: { type: "object", properties: { scope: { type: "string" }, query: { type: "string" } }, required: ["scope", "query"] } }, { name: "learn", description: "Store a memory", inputSchema: { type: "object", properties: { eventId: { type: "string" }, scope: { type: "string" }, kind: { type: "string" }, content: { type: "string" } }, required: ["eventId", "scope", "kind", "content"] } }] } });
  if (body.method !== "tools/call") return Response.json({ jsonrpc: "2.0", id, error: { code: -32601, message: "method not found" } }, { status: 404 });
  const args = body.params?.arguments ?? {};
  if (body.params?.name === "recall") { const memories = await prisma.memory.findMany({ where: { tenantId: identity.tenantId, userId: identity.userId, scope: args.scope } }); return Response.json({ jsonrpc: "2.0", id, result: { content: [{ type: "text", text: JSON.stringify(rankMemories(args.query, memories)) }] } }); }
  if (body.params?.name === "learn") { const parsed = memoryInputSchema.safeParse(args); if (!parsed.success) return Response.json({ jsonrpc: "2.0", id, error: { code: -32602, message: "invalid arguments" } }); await prisma.memory.upsert({ where: { tenantId_userId_eventId: { tenantId: identity.tenantId, userId: identity.userId, eventId: parsed.data.eventId } }, update: { content: parsed.data.content, kind: parsed.data.kind }, create: { ...parsed.data, userId: identity.userId, tenantId: identity.tenantId } }); return Response.json({ jsonrpc: "2.0", id, result: { content: [{ type: "text", text: "accepted" }] } }); }
  return Response.json({ jsonrpc: "2.0", id, error: { code: -32602, message: "unknown tool" } }, { status: 400 });
}
