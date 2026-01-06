# Supabase Setup Summary

## ✅ Migration Complete

The project has been successfully migrated from PostgREST to the official Supabase Stack:

### Changes Made:
1. **Replaced docker-compose.yml** with the official Supabase docker-compose.yml from the main repository
2. **Removed PostgREST references** - Now using the complete Supabase stack instead
3. **Updated environment configuration** to use environment variables for all settings
4. **Fixed test behavior** to fail when Supabase is configured but not available (no silent fallback)
5. **Updated URLs** to point to the correct Kong gateway port (8000) instead of CLI port (54321)
6. **Fixed JWT key consistency** - Synchronized ANON_KEY and SERVICE_ROLE_KEY between .env files
7. **Corrected port references** - All configurations now use port 8000 consistently

### Current Setup:
- **File**: `/lab/docker/docker-compose.yml` - Official Supabase stack
- **File**: `/lab/docker/.env` - Environment variables for all Supabase services
- **File**: `/lab/config/supabase/local.env` - Local development environment variables
- **Tests**: Now correctly fail when Supabase is unavailable

### Environment Variables:
All configuration is managed via environment variables in `.env` file:
- `POSTGRES_PASSWORD` - Database password
- `JWT_SECRET` - JWT signing secret  
- `ANON_KEY` - Public API key
- `SERVICE_ROLE_KEY` - Service role API key
- `KONG_HTTP_PORT=8000` - API gateway port
- Plus many more for full Supabase functionality

## 🚀 How to Start Supabase

### Option 1: Start Full Supabase Stack (Recommended)
```bash
cd lab/docker
docker compose up -d
```

This will start all Supabase services:
- PostgreSQL database (supabase-db)
- Kong API Gateway (supabase-kong) - Port 8000
- Storage API (supabase-storage)
- Auth service (supabase-auth) 
- Studio UI (supabase-studio)
- Realtime (realtime-dev.supabase-realtime)
- Functions (supabase-edge-functions)
- Analytics (supabase-analytics)
- And more...

### Option 2: Use Supabase CLI (Alternative)
```bash
# Install Supabase CLI first
npm install -g @supabase/cli

# Start local instance
supabase start
```

## 🧪 Running Tests

### Test Storage Performance (Should Fail if Supabase Not Running)
```bash
# Source environment variables
source lab/config/supabase/local.env

# Run storage tests
cargo test test_supabase_storage_performance --features storage-supabase
```

### Expected Behavior:
- ✅ **Pass**: When Supabase is running and accessible
- ❌ **Fail**: When Supabase is configured but not running (prevents silent fallback)
- ⏭️ **Skip**: When Supabase environment variables are not set

## 📍 Service URLs

When Supabase is running:
- **API Gateway**: http://localhost:8000
- **Storage API**: http://localhost:8000/storage/v1/
- **Auth API**: http://localhost:8000/auth/v1/
- **Studio UI**: http://localhost:3000 (if exposed)
- **Database**: localhost:5432

## 🔧 Configuration Files

### `/lab/docker/.env`
Main configuration file with all Supabase settings. Contains defaults for local development.

### `/lab/config/supabase/local.env`  
Environment variables for the application to connect to local Supabase instance.

### `/lab/docker/docker-compose.yml`
Official Supabase docker-compose setup with all services.

## 🐳 Docker Containers

When running, you should see these containers:
```bash
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

Expected containers:
- supabase-db
- supabase-kong  
- supabase-storage
- supabase-auth
- supabase-studio
- realtime-dev.supabase-realtime
- supabase-edge-functions
- supabase-analytics
- supabase-meta
- supabase-imgproxy
- supabase-vector
- supabase-pooler

## ✨ Key Improvements

1. **No More PostgREST Confusion**: Using official Supabase stack
2. **Environment-Driven**: All config via environment variables
3. **Proper Test Behavior**: Tests fail when dependencies unavailable
4. **Official Setup**: Matches Supabase documentation exactly
5. **Better Error Messages**: Clear guidance when services are down
6. **JWT Key Consistency**: Fixed authentication issues by synchronizing keys between config files

## 🔧 Troubleshooting

### Authentication Issues
If you get authentication errors:
1. **Verify JWT keys match**: The `ANON_KEY` and `SERVICE_ROLE_KEY` in `/lab/docker/.env` and `/lab/config/supabase/local.env` must be identical
2. **Check port configuration**: Ensure all services use port 8000 (Kong gateway), not 54321 (CLI port)
3. **Restart containers**: After changing environment variables, restart: `cd lab/docker && docker compose down && docker compose up -d`

### Connection Issues
If tests fail with connection errors:
1. **Check containers are running**: `docker ps` should show all Supabase containers
2. **Verify URL accessibility**: `curl http://localhost:8000/health` should return 200
3. **Check environment variables**: Ensure `SUPABASE_URL=http://localhost:8000` is set correctly

The system is now properly configured to use the official Supabase Storage API with proper fallback behavior and error handling.
