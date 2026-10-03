CREATE TABLE IF NOT EXISTS "Tenant" (
    "id" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "Tenant_pkey" PRIMARY KEY ("id")
);

CREATE TABLE IF NOT EXISTS "TenantMember" (
    "tenantId" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "role" TEXT NOT NULL DEFAULT 'member',
    CONSTRAINT "TenantMember_pkey" PRIMARY KEY ("tenantId", "userId")
);

ALTER TABLE "Memory" ADD COLUMN IF NOT EXISTS "tenantId" TEXT;
INSERT INTO "Tenant" ("id", "name") VALUES ('legacy', 'Legacy tenant') ON CONFLICT ("id") DO NOTHING;
UPDATE "Memory" SET "tenantId" = 'legacy' WHERE "tenantId" IS NULL;
ALTER TABLE "Memory" ALTER COLUMN "tenantId" SET NOT NULL;
ALTER TABLE "Memory" DROP CONSTRAINT IF EXISTS "Memory_userId_eventId_key";
CREATE UNIQUE INDEX IF NOT EXISTS "Memory_tenantId_userId_eventId_key" ON "Memory"("tenantId", "userId", "eventId");
DROP INDEX IF EXISTS "Memory_userId_scope_createdAt_idx";
CREATE INDEX IF NOT EXISTS "Memory_tenantId_scope_createdAt_idx" ON "Memory"("tenantId", "scope", "createdAt");
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'Memory_tenantId_fkey') THEN
    ALTER TABLE "Memory" ADD CONSTRAINT "Memory_tenantId_fkey" FOREIGN KEY ("tenantId") REFERENCES "Tenant"("id") ON DELETE CASCADE ON UPDATE CASCADE;
  END IF;
END $$;
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'TenantMember_tenantId_fkey') THEN
    ALTER TABLE "TenantMember" ADD CONSTRAINT "TenantMember_tenantId_fkey" FOREIGN KEY ("tenantId") REFERENCES "Tenant"("id") ON DELETE CASCADE ON UPDATE CASCADE;
  END IF;
END $$;
