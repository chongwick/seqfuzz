#!/bin/bash
# Usage: ./sanitize.sh <target_script> <san_type(0/1)> [options]
# san_type: 0 = general sanitizer, 1 = leak detection mode

set -euo pipefail

# Validate arguments
if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <target_script> <san_type(0/1)> [options]"
    exit 1
fi

script="$1"
san_type="$2"
options="${3:-}"

# Verify script exists
if [[ ! -f "$script" ]]; then
    echo "Error: Script '$script' not found"
    exit 1
fi

# Configure sanitizer environment for leak detection mode
if [[ "$san_type" == "1" ]]; then
    export USE_ZEND_ALLOC=0
    export ASAN_OPTIONS=detect_leaks=0
fi

# PHP JIT/opcache configuration
php_opts=(
    -d "memory_limit=-1"
    -d "zend.assertions=1"
    -d "display_errors=On"
    -d "display_startup_errors=On"
    -d "opcache.memory_consumption=4096M"
    -d "opcache.enable=1"
    -d "opcache.enable_cli=1"
    -d "opcache.jit=tracing"
    -d "opcache.validate_timestamps=0"
    -d "opcache.jit_buffer_size=128M"
    -d "opcache.file_update_protection=0"
    -d "opcache.max_accelerated_files=1000000"
    -d "opcache.interned_strings_buffer=64"
    -d "opcache.jit_prof_threshold=0.000000001"
    -d "opcache.jit_max_root_traces=100000"
    -d "opcache.jit_max_side_traces=100000"
    -d "opcache.jit_max_exit_counters=100000"
    -d "opcache.jit_hot_loop=1"
    -d "opcache.jit_hot_func=1"
    -d "opcache.jit_hot_return=1"
    -d "opcache.jit_hot_side_exit=1"
    -d "opcache.jit_blacklist_root_trace=255"
    -d "opcache.jit_blacklist_side_trace=255"
    -d "opcache.protect_memory=1"
)

# Simplified options for when $options is provided
php_opts_simple=(
    -d "opcache.enable=1"
    -d "opcache.enable_cli=1"
    -d "opcache.jit_buffer_size=256M"
    -d "opcache.jit_hot_func=1"
    -d "opcache.jit=1254"
)

# Run PHP with timeout
if [[ -z "$options" ]]; then
    OUTPUT=$(timeout -s SIGTERM 120 ./php "${php_opts[@]}" "$script" 2>&1) || true
else
    OUTPUT=$(timeout -s SIGTERM 120 ./php "${php_opts_simple[@]}" "$script" 2>&1) || true
fi
RET=$?

# Helper function to mark script as "trivial/ignored" error
mark_trivial() {
    mv "$script" "${script}.tr"
    exit 0
}

# Helper function to mark script as real error
mark_error() {
    mv "$script" "${script}.er"
    chmod 440 "${script}.er"
    echo "$OUTPUT" > san.log
    exit 0
}

# Check for errors
if [[ $RET -ne 0 ]]; then
    # Timeout or signal-related exits (ignore these)
    case $RET in
        124|137|255|153)
            mark_trivial
            ;;
    esac

    # Filter out known non-bug errors
    if echo "$OUTPUT" | grep -q "Allowed memory size of"; then
        mark_trivial
    fi

    if echo "$OUTPUT" | grep -q "zend_variable.c:63"; then
        mark_trivial
    fi

    if echo "$OUTPUT" | grep -qE 'xml.*leak|leak.*xml'; then
        mark_trivial
    fi

    if echo "$OUTPUT" | grep -q "AddressSanitizer failed to allocate"; then
        mark_trivial
    fi

    if echo "$OUTPUT" | grep -q ": Assertion "; then
        mark_trivial
    fi

    # Real error found
    mark_error
fi

# Check for UBSan runtime errors even on successful exit
if echo "$OUTPUT" | grep -q "runtime error:"; then
    mark_error
fi

# Script ran successfully with no errors
exit 0
