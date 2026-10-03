import { auth0 } from "@/src/lib/auth0";
import MemoryClient from "./memory-client";

export default async function MemoriesPage() {
  const session = await auth0.getSession();
  if (!session) return <main><h1>Memórias</h1><a href="/auth/login">Entrar com Auth0</a></main>;
  return <main><h1>Memórias</h1><p>Usuário: {session.user.email ?? session.user.sub}</p><MemoryClient /></main>;
}
