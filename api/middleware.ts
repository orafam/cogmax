import type { NextRequest } from "next/server";
import { NextResponse } from "next/server";
import { auth0 } from "./src/lib/auth0";

export async function middleware(request: NextRequest) {
  if (process.env.NODE_ENV !== "production") return NextResponse.next();
  return auth0.middleware(request);
}

export const config = {
  matcher: ["/((?!_next/static|_next/image|favicon.ico|sitemap.xml|robots.txt).*)"],
};
