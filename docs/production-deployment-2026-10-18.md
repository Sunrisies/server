# Production backend deployment — 2026-10-18

## Active release

- Image: `web-server:0.1.20-20261018-r2`
- Build: `production-20261018-linux-x86_64`
- Build timestamp: `2026-10-18T00:23:24+08:00`
- Artifact SHA-256: `cea6f5cd97c2f313cacf7960e866f26438bbaf9a08c2b073ea53c8795bf24c76`
- Verification endpoint: `GET /api/v1/version`

## Deployment safeguards

- The runtime image does not contain production environment values. The server passes its protected runtime environment file when starting the container.
- A PostgreSQL backup was taken before the schema and data compatibility migrations.
- The previous image and stopped rollback container are retained as `web-server-rollback-20261018`.
- The release was first started against a restored production database clone before the public port was switched.

## Applied compatibility migrations

- Added the `uploads` table required by managed image uploads. This is additive and safe for the prior service version.
- Migrated 96 historical posts from legacy status `0` to published status `1`. Every migrated row already had a publication timestamp; this preserves the previous public content behaviour while keeping future draft posts private.

## Acceptance checks

- Public version endpoint returns HTTP 200 and the active build identity.
- Public post list returns 96 published records; post detail resolves successfully.
- Default WebP avatar and generated category SVG cover return successfully.
- Cross-origin access from `https://sunrise1024.top` is allowed.
