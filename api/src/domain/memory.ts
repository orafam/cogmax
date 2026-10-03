import { z } from "zod";

export const memoryInputSchema = z.object({
  eventId: z.string().min(1),
  scope: z.string().min(1),
  kind: z.enum(["Fact", "Decision", "Preference", "Note"]),
  content: z.string().trim().min(1),
  project: z.string().optional(),
});

export type MemoryRecord = { id: string; content: string; kind: string; scope: string; createdAt: Date };

export function rankMemories(query: string, memories: MemoryRecord[]) {
  const needle = query.trim().toLowerCase();
  return [...memories].sort((a, b) => {
    const score = (m: MemoryRecord) => (m.content.toLowerCase().includes(needle) ? 1 : 0);
    return score(b) - score(a) || b.createdAt.getTime() - a.createdAt.getTime();
  });
}
