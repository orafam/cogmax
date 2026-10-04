import { MemoryDurableObject } from "./durable-object";

export { MemoryDurableObject };

export type Env = {
  MEMORY: DurableObjectNamespace;
  COGMAX_API_TOKEN: string;
};

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method === "GET" && new URL(request.url).pathname === "/health") {
      return Response.json({ status: "ok" });
    }
    const authorization = request.headers.get("authorization");
    if (authorization !== `Bearer ${env.COGMAX_API_TOKEN}`) {
      return Response.json({ error: "unauthorized" }, { status: 401 });
    }
    const url = new URL(request.url);
    const body = request.method === "POST"
      ? await request.clone().json().catch(() => ({})) as { scope?: string }
      : {};
    const scope = body.scope ?? url.searchParams.get("scope") ?? undefined;
    if (!scope || !scope.startsWith("user:")) {
      return Response.json({ error: "invalid scope" }, { status: 400 });
    }
    const id = env.MEMORY.idFromName(scope.split("/", 1)[0]!);
    return env.MEMORY.get(id).fetch(new Request(new URL(request.url).pathname.replace("/v1", ""), request));
  },
};
