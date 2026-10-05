export type MemoryEvent = {
  event_id: string;
  scope: string;
  kind: string;
  content: string;
  confidence: string;
  authority: string;
};

export class MemoryDurableObject {
  constructor(private readonly state: DurableObjectState) {}

  async fetch(request: Request): Promise<Response> {
    if (request.method === "POST" && new URL(request.url).pathname === "/events") {
      const event = (await request.json()) as MemoryEvent;
      if (!event.event_id || !event.scope || !event.kind || !event.content) {
        return Response.json({ error: "invalid event" }, { status: 400 });
      }
      const key = `event:${event.event_id}`;
      const existing = await this.state.storage.get<MemoryEvent>(key);
      if (!existing) await this.state.storage.put(key, event);
      return Response.json({ accepted: true, duplicate: Boolean(existing) });
    }

    if (request.method === "GET" && new URL(request.url).pathname === "/events") {
      const events = await this.state.storage.list<MemoryEvent>({ prefix: "event:" });
      return Response.json({ events: [...events.values()] });
    }

    return new Response("Not found", { status: 404 });
  }
}
