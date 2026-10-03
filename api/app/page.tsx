import { auth0 } from "@/src/lib/auth0";

export default async function Home() {
  const session = await auth0.getSession();
  return <main><h1>Cogmax Vault API</h1><p>Webservice de memórias ativo.</p>{session ? <p>Autenticado como {session.user.email ?? session.user.sub}</p> : <a href="/auth/login">Entrar com Auth0</a>}</main>;
}
