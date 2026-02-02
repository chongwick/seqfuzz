#!/usr/bin/env bash

# Directory where "good" scripts will be moved
GOOD_DIR="/home/w023dtc/treebugs"
#GOOD_DIR="good"
#mkdir -p "$GOOD_DIR"

# Collect all .php.er files into an array
mapfile -t files < <(find . -maxdepth 1 -type f -name "*.php.er")

# Iterate over the files
for f in "${files[@]}"; do
    echo "=================================================="
    echo "Testing: $f"

    # Step 2.1: run normally
    echo ">>> [1] Running normally"
    /home/w023dtc/nightly_php/php-src/sapi/cli/php "$f"

    # Start triage loop
    while true; do
        echo
        echo "Options: [n]ext mode, [t]rash (delete), [g]ood (move), [s]kip"
        read -r -p "Your choice: " choice

        case "$choice" in
            n)
                if [ -z "$mode" ]; then
                    mode=1
                    echo ">>> [2] Running with USE_ZEND_ALLOC=0"
                    USE_ZEND_ALLOC=0 /home/w023dtc/nightly_php/php-src/sapi/cli/php "$f"
                elif [ "$mode" -eq 1 ]; then
                    mode=2
                    echo ">>> [3] Running with JIT"
                    USE_ZEND_ALLOC=0 /home/w023dtc/nightly_php/php-src/sapi/cli/php -d "memory_limit = -1" -d "zend.assertions = 1" -d "display_errors = On" -d "display_startup_errors = On" -d "opcache.memory_consumption=4096M" -d "opcache.enable=1" -d "opcache.enable_cli=1" -d "opcache.jit=tracing" -d "opcache.validate_timestamps=0" -d "opcache.jit_buffer_size=128M" -d "opcache.file_update_protection=0" -d "opcache.max_accelerated_files=1000000" -d "opcache.interned_strings_buffer=64" -d "opcache.jit_prof_threshold=0.000000001" -d "opcache.jit_max_root_traces=  100000" -d "opcache.jit_max_side_traces=  100000" -d "opcache.jit_max_exit_counters=100000" -d "opcache.jit_hot_loop=1" -d "opcache.jit_hot_func=1" -d "opcache.jit_hot_return=1" -d "opcache.jit_hot_side_exit=1" -d "opcache.jit_blacklist_root_trace=255" -d "opcache.jit_blacklist_side_trace=255" -d "opcache.protect_memory=1" "$f"
                else
                    echo "No more modes to test for $f"
                fi
                ;;
            t)
                echo "Deleting $f"
                yes | rm -f "$f"
                break
                ;;
            g)
                echo "Moving $f -> $GOOD_DIR/"
                mv "$f" "$GOOD_DIR/"
                break
                ;;
            s|"")
                echo "Skipping $f"
                break
                ;;
            *)
                echo "Invalid choice. Please type n / t / g / s"
                ;;
        esac
    done

    unset mode
done

