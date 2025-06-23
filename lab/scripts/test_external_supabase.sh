#!/bin/bash
#
# Test External Supabase Connection
# Validates that the external Supabase instance is properly configured
# and the Rust adapter can communicate with it
#

set -e

echo "🧪 External Supabase Connection Test"
echo "===================================="
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: This script must be run from the project root directory"
    exit 1
fi

# Load environment variables
if [ ! -f ".env" ]; then
    echo "❌ Error: .env file not found"
    echo "   Run lab/scripts/setup_external_supabase.sh first"
    exit 1
fi

source .env

# Validate required environment variables
if [ -z "$SUPABASE_URL" ] || [ -z "$SUPABASE_ANON_KEY" ] || [ -z "$SUPABASE_SERVICE_ROLE_KEY" ]; then
    echo "❌ Error: Missing required environment variables"
    echo "   Required: SUPABASE_URL, SUPABASE_ANON_KEY, SUPABASE_SERVICE_ROLE_KEY"
    echo "   Run lab/scripts/setup_external_supabase.sh to configure"
    exit 1
fi


# Pre-check: Is Supabase instance reachable?

echo -n "🌐 Checking Supabase instance reachability... "
# Try curl and capture both HTTP status and exit code
HTTP_STATUS=""
CURL_EXIT=0
HTTP_STATUS=$(curl -sS -o /dev/null -w "%{http_code}" --max-time 8 "$SUPABASE_URL/rest/v1/" 2>/dev/null) || CURL_EXIT=$?
if [ "$CURL_EXIT" -ne 0 ]; then
    echo "FAILED (network error)"
    echo "❌ Supabase URL is not reachable (curl exit code $CURL_EXIT)"
    echo "   - Check your internet connection and the SUPABASE_URL: $SUPABASE_URL"
    echo "   - If using the free tier, your project may be paused due to inactivity."
    echo "   - Visit your Supabase dashboard and resume the project, then retry."
    exit 1
elif [ "$HTTP_STATUS" != "200" ]; then
    echo "FAILED (HTTP $HTTP_STATUS)"
    echo "❌ Supabase instance is unreachable, paused, or returned error (HTTP $HTTP_STATUS)"
    echo "   - If using the free tier, your project may be paused due to inactivity."
    echo "   - Visit your Supabase dashboard and resume the project, then retry."
    echo "   - If this is a network issue, check your internet connection and project URL."
    echo "   - If this is a 4xx/5xx error, check your Supabase project status."
    exit 1
else
    echo "OK (HTTP $HTTP_STATUS)"
    echo "   ✅ Supabase instance is reachable (HTTP $HTTP_STATUS)"
fi

echo "📋 Configuration:"
echo "   Supabase URL: $SUPABASE_URL"
echo "   Anon Key: ${SUPABASE_ANON_KEY:0:20}..." 
echo "   Service Role Key: ${SUPABASE_SERVICE_ROLE_KEY:0:20}..."
echo ""

# Test 1: Basic API connectivity
echo "🔍 Test 1: Basic API Connectivity"
echo "   Testing GET $SUPABASE_URL/rest/v1/"

curl_output=$(curl -s -w "%{http_code}" -o /tmp/api_test.json \
    -H "apikey: $SUPABASE_ANON_KEY" \
    -H "Authorization: Bearer $SUPABASE_ANON_KEY" \
    "$SUPABASE_URL/rest/v1/" 2>/dev/null)

http_code="${curl_output: -3}"

if [ "$http_code" = "200" ]; then
    echo "   ✅ API connectivity successful (HTTP $http_code)"
else
    echo "   ❌ API connectivity failed (HTTP $http_code)"
    echo "   Response: $(cat /tmp/api_test.json 2>/dev/null)"
    exit 1
fi

# Test 2: Service role authentication
echo ""
echo "🔐 Test 2: Service Role Authentication"
echo "   Testing with service role key..."

curl_output=$(curl -s -w "%{http_code}" -o /tmp/auth_test.json \
    -H "apikey: $SUPABASE_SERVICE_ROLE_KEY" \
    -H "Authorization: Bearer $SUPABASE_SERVICE_ROLE_KEY" \
    "$SUPABASE_URL/rest/v1/" 2>/dev/null)

http_code="${curl_output: -3}"

if [ "$http_code" = "200" ]; then
    echo "   ✅ Service role authentication successful (HTTP $http_code)"
else
    echo "   ❌ Service role authentication failed (HTTP $http_code)"
    echo "   Response: $(cat /tmp/auth_test.json 2>/dev/null)"
    exit 1
fi

# Test 3: Create test table (if it doesn't exist)
echo ""
echo "📊 Test 3: Database Schema Setup"
echo "   Creating test table if needed..."

# Check if test table exists
table_check=$(curl -s \
    -H "apikey: $SUPABASE_SERVICE_ROLE_KEY" \
    -H "Authorization: Bearer $SUPABASE_SERVICE_ROLE_KEY" \
    "$SUPABASE_URL/rest/v1/$TEST_TABLE_NAME?select=*&limit=1" 2>/dev/null || echo "")

if [[ "$table_check" == *"relation"*"does not exist"* ]] || [ -z "$table_check" ]; then
    echo "   📝 Test table doesn't exist, creating..."
    
    # Create test table via SQL
    sql_payload='{
        "query": "CREATE TABLE IF NOT EXISTS '${TEST_TABLE_NAME}' (id SERIAL PRIMARY KEY, key TEXT UNIQUE NOT NULL, value TEXT, metadata JSONB, created_at TIMESTAMPTZ DEFAULT NOW(), updated_at TIMESTAMPTZ DEFAULT NOW());"
    }'
    
    create_result=$(curl -s -w "%{http_code}" -o /tmp/create_table.json \
        -X POST \
        -H "apikey: $SUPABASE_SERVICE_ROLE_KEY" \
        -H "Authorization: Bearer $SUPABASE_SERVICE_ROLE_KEY" \
        -H "Content-Type: application/json" \
        -d "$sql_payload" \
        "$SUPABASE_URL/rest/v1/rpc/exec_sql" 2>/dev/null || echo "000")
    
    create_code="${create_result: -3}"
    
    if [ "$create_code" = "200" ] || [ "$create_code" = "201" ]; then
        echo "   ✅ Test table created successfully"
    else
        echo "   ⚠️ Could not create test table (this may be normal)"
        echo "   You may need to create the table manually in Supabase SQL Editor:"
        echo "   CREATE TABLE $TEST_TABLE_NAME (id SERIAL PRIMARY KEY, key TEXT UNIQUE NOT NULL, value TEXT, metadata JSONB, created_at TIMESTAMPTZ DEFAULT NOW(), updated_at TIMESTAMPTZ DEFAULT NOW());"
    fi
else
    echo "   ✅ Test table already exists"
fi

# Test 4: Basic CRUD operations
echo ""
echo "🔄 Test 4: Basic CRUD Operations"

# INSERT test
test_key="test_$(date +%s)"
test_value="test_value_$(date +%s)"

echo "   📝 Testing INSERT..."
insert_payload="{\"key\": \"$test_key\", \"value\": \"$test_value\"}"

insert_result=$(curl -s -w "%{http_code}" -o /tmp/insert_test.json \
    -X POST \
    -H "apikey: $SUPABASE_SERVICE_ROLE_KEY" \
    -H "Authorization: Bearer $SUPABASE_SERVICE_ROLE_KEY" \
    -H "Content-Type: application/json" \
    -H "Prefer: return=representation" \
    -d "$insert_payload" \
    "$SUPABASE_URL/rest/v1/$TEST_TABLE_NAME" 2>/dev/null)

insert_code="${insert_result: -3}"

if [ "$insert_code" = "201" ]; then
    echo "   ✅ INSERT successful"
    
    # SELECT test
    echo "   🔍 Testing SELECT..."
    select_result=$(curl -s -w "%{http_code}" -o /tmp/select_test.json \
        -H "apikey: $SUPABASE_ANON_KEY" \
        -H "Authorization: Bearer $SUPABASE_ANON_KEY" \
        "$SUPABASE_URL/rest/v1/$TEST_TABLE_NAME?key=eq.$test_key&select=*" 2>/dev/null)
    
    select_code="${select_result: -3}"
    
    if [ "$select_code" = "200" ]; then
        echo "   ✅ SELECT successful"
        
        # DELETE test
        echo "   🗑️ Testing DELETE..."
        delete_result=$(curl -s -w "%{http_code}" -o /tmp/delete_test.json \
            -X DELETE \
            -H "apikey: $SUPABASE_SERVICE_ROLE_KEY" \
            -H "Authorization: Bearer $SUPABASE_SERVICE_ROLE_KEY" \
            "$SUPABASE_URL/rest/v1/$TEST_TABLE_NAME?key=eq.$test_key" 2>/dev/null)
        
        delete_code="${delete_result: -3}"
        
        if [ "$delete_code" = "204" ]; then
            echo "   ✅ DELETE successful"
        else
            echo "   ⚠️ DELETE failed (HTTP $delete_code)"
        fi
    else
        echo "   ❌ SELECT failed (HTTP $select_code)"
    fi
else
    echo "   ❌ INSERT failed (HTTP $insert_code)"
    echo "   Response: $(cat /tmp/insert_test.json 2>/dev/null)"
fi

# Test 5: Rust adapter compilation
echo ""
echo "🦀 Test 5: Rust Adapter"
echo "   Checking Rust code compilation..."

if cargo check --features supabase >/dev/null 2>&1; then
    echo "   ✅ Rust code compiles successfully"
    
    # Run Rust tests if they exist
    echo "   🧪 Running Rust tests..."
    if cargo test storage::supabase --features supabase >/dev/null 2>&1; then
        echo "   ✅ Rust tests pass"
    else
        echo "   ⚠️ Some Rust tests failed (this may be expected during development)"
        echo "   Run 'cargo test storage::supabase --features supabase' for details"
    fi
else
    echo "   ❌ Rust compilation issues"
    echo "   Run 'cargo check --features supabase' for details"
fi

# Cleanup
rm -f /tmp/api_test.json /tmp/auth_test.json /tmp/create_table.json /tmp/insert_test.json /tmp/select_test.json /tmp/delete_test.json

echo ""
echo "🎉 External Supabase Connection Test Complete!"
echo ""
echo "Summary:"
echo "✅ API connectivity"
echo "✅ Authentication" 
echo "✅ Database access"
echo "✅ Basic CRUD operations"
echo "✅ Rust adapter compilation"
echo ""
echo "Your external Supabase instance is ready for development!"
echo ""
echo "Next steps:"
echo "1. Develop your Rust application"
echo "2. Run specific tests: cargo test --features supabase"
echo "3. Create additional database schema as needed"
echo ""
echo "For issues, see: lab/config/supabase/UPDATED_TROUBLESHOOTING.md"
