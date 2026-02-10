#!/usr/bin/env bash
#
# Calls the Verifier API to verify a TLSNotary presentation.
# Reads a JSON file exported by the browser extension containing:
#   { "transcriptRecv": "...", "transcriptSent": "...", "tlsn_presentation": "<hex>" }
#
# Usage:
#   ./scripts/verify.sh <presentation-json-file> --group <credential_group_id> [options]
#
# Examples:
#   ./scripts/verify.sh apple_devices.json --group 11
#   ./scripts/verify.sh uber_rides.json --group 12
#   ./scripts/verify.sh presentation.json --group 12 --registry 0x1234... --app-id 1

set -euo pipefail

# Defaults
VERIFIER_URL="http://localhost:3000"
REGISTRY="0x0000000000000000000000000000000000000000"
APP_ID="0"
COMMITMENT="0"
CREDENTIAL_GROUP_ID=""
INPUT_FILE=""

usage() {
    echo "Usage: $0 <presentation-json-file> --group <credential_group_id> [options]"
    echo ""
    echo "Arguments:"
    echo "  <presentation-json-file>  JSON file with \"tlsn_presentation\" field"
    echo ""
    echo "Required:"
    echo "  --group <id>              Credential group ID (e.g. 11 for Apple Devices, 12 for Uber Rides)"
    echo ""
    echo "Options:"
    echo "  --registry <address>      Ethereum registry address (default: $REGISTRY)"
    echo "  --app-id <uint256>        Application ID (default: $APP_ID)"
    echo "  --commitment <uint256>    Semaphore identity commitment (default: $COMMITMENT)"
    echo "  --url <base_url>          Verifier base URL (default: $VERIFIER_URL)"
    echo "  -h, --help                Show this help message"
    exit 1
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --group)
            CREDENTIAL_GROUP_ID="$2"
            shift 2
            ;;
        --registry)
            REGISTRY="$2"
            shift 2
            ;;
        --app-id)
            APP_ID="$2"
            shift 2
            ;;
        --commitment)
            COMMITMENT="$2"
            shift 2
            ;;
        --url)
            VERIFIER_URL="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        -*)
            echo "Error: unknown option '$1'" >&2
            usage
            ;;
        *)
            if [[ -z "$INPUT_FILE" ]]; then
                INPUT_FILE="$1"
            else
                echo "Error: unexpected argument '$1'" >&2
                usage
            fi
            shift
            ;;
    esac
done

if [[ -z "$INPUT_FILE" ]]; then
    echo "Error: presentation JSON file is required" >&2
    usage
fi

if [[ -z "$CREDENTIAL_GROUP_ID" ]]; then
    echo "Error: --group <credential_group_id> is required" >&2
    usage
fi

if [[ ! -f "$INPUT_FILE" ]]; then
    echo "Error: file not found: $INPUT_FILE" >&2
    exit 1
fi

if ! command -v jq &>/dev/null; then
    echo "Error: jq is required (brew install jq)" >&2
    exit 1
fi

# Extract presentation hex from JSON
PRESENTATION=$(jq -r '.tlsn_presentation // empty' "$INPUT_FILE")

if [[ -z "$PRESENTATION" ]]; then
    echo "Error: \"tlsn_presentation\" field not found in $INPUT_FILE" >&2
    echo "The JSON file must contain a \"tlsn_presentation\" field with the hex-encoded TLSNotary presentation." >&2
    exit 1
fi

echo "Verifier URL:  $VERIFIER_URL"
echo "Group ID:      $CREDENTIAL_GROUP_ID"
echo "Registry:      $REGISTRY"
echo "App ID:        $APP_ID"
echo "Commitment:    $COMMITMENT"
echo "Presentation:  ${#PRESENTATION} hex chars"
echo ""
echo "Sending POST $VERIFIER_URL/verify ..."
echo ""

RESPONSE=$(curl -s -w "\n%{http_code}" \
    -X POST "$VERIFIER_URL/verify" \
    -H "Content-Type: application/json" \
    -d "$(jq -n \
        --arg p "$PRESENTATION" \
        --arg r "$REGISTRY" \
        --arg g "$CREDENTIAL_GROUP_ID" \
        --arg a "$APP_ID" \
        --arg c "$COMMITMENT" \
        '{
            tlsn_presentation: $p,
            registry: $r,
            credential_group_id: $g,
            app_id: $a,
            semaphore_identity_commitment: $c
        }')")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

echo "HTTP Status: $HTTP_CODE"
echo ""

if [[ "$HTTP_CODE" -ge 200 && "$HTTP_CODE" -lt 300 ]]; then
    echo "Attestation:"
    echo "$BODY" | jq '.attestation'
    echo ""
    echo "Verifier Hash:"
    echo "$BODY" | jq -r '.verifier_hash'
    echo ""
    echo "Signature:"
    echo "$BODY" | jq -r '.signature'
else
    echo "Error response:"
    echo "$BODY"
    exit 1
fi
