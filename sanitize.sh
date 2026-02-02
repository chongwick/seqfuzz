#!/bin/bash
#Usage, ./sanitize.sh [target_script] [san_types(0/1)]
#opcache.jit=1256
#opcache.jit_hot_func=1
#opcache.jit_hot_loop=1
#opcache.jit_hot_return=1
#opcache.jit_hot_side_exit=1

script=$1
san_type=$2 #0 for other, 1 for leak
options=$3
if [ $san_type == "1" ]; then
        export USE_ZEND_ALLOC=0
        export ASAN_OPTIONS=detect_leaks=0
	#export ASAN_OPTIONS=allocator_may_return_null=1
fi
#OUTPUT=$(timeout -s SIGTERM 120 php run-tests.php -p "$php_engine" "$script" 2>&1)
#OUTPUT=$(timeout -s SIGTERM 120 ./run-tests.php -p "$php_engine" "$script" 2>&1)
if [[ -z "$options" ]]; then
        #OUTPUT=$(timeout -s SIGTERM 120 "$php_engine" "$script" 2>&1)
        #OUTPUT=$(timeout -s SIGTERM 120 ./php "$script" 2>&1)
	#OUTPUT=$(timeout -s SIGTERM 120 ./php -d "opcache.enable=1" -d "opcache.enable_cli=1" -d "opcache.jit_buffer_size=256M" -d "opcache.jit_hot_func=1" -d "opcache.jit=1254" "$script" 2>&1)
	#OUTPUT=$(timeout -s SIGTERM 120 ./php -d "opcache.enable=1" -d "opcache.enable_cli=1" -d "opcache.jit_buffer_size=256M" -d "opcache.jit=1254" -d "opcache.jit_hot_func=1" -d "opcache.jit_hot_loop=1" -d "opcache.jit_hot_return=1" -d "opcache.jit_hot_side_exit=1" "$script" 2>&1)
	OUTPUT=$(timeout -s SIGTERM 120 ./php -d "memory_limit = -1" -d "zend.assertions = 1" -d "display_errors = On" -d "display_startup_errors = On" -d "opcache.memory_consumption=4096M" -d "opcache.enable=1" -d "opcache.enable_cli=1" -d "opcache.jit=tracing" -d "opcache.validate_timestamps=0" -d "opcache.jit_buffer_size=128M" -d "opcache.file_update_protection=0" -d "opcache.max_accelerated_files=1000000" -d "opcache.interned_strings_buffer=64" -d "opcache.jit_prof_threshold=0.000000001" -d "opcache.jit_max_root_traces=  100000" -d "opcache.jit_max_side_traces=  100000" -d "opcache.jit_max_exit_counters=100000" -d "opcache.jit_hot_loop=1" -d "opcache.jit_hot_func=1" -d "opcache.jit_hot_return=1" -d "opcache.jit_hot_side_exit=1" -d "opcache.jit_blacklist_root_trace=255" -d "opcache.jit_blacklist_side_trace=255" -d "opcache.protect_memory=1" "$script" 2>&1)
else
        #OUTPUT=$(timeout -s SIGTERM 120 "$php_engine" "$options" "$script" 2>&1)
        #OUTPUT=$(timeout -s SIGTERM 120 ./php "$options" "$script" 2>&1)
	OUTPUT=$(timeout -s SIGTERM 120 ./php -d "opcache.enable=1" -d "opcache.enable_cli=1" -d "opcache.jit_buffer_size=256M" -d "opcache.jit_hot_func=1" -d "opcache.jit=1254" "$script" 2>&1)
fi
RET=$?

if [ $RET -ne 0 ]; then
       if [ $RET -eq 124 ] || [ $RET -eq 137 ] || [ $RET -eq 255 ] || [ $RET -eq 153 ]; then
               mv "$script" "${script}.tr"
               exit 0
       fi
       if [ $(echo "$OUTPUT" | grep "Allowed memory size of" | wc -l) -gt 0 ]; then
               mv "$script" "${script}.tr"
	       exit 0
       fi
       if [ $(echo "$OUTPUT" | grep "zend_variable.c" | wc -l) -gt 0 ]; then
               mv "$script" "${script}.tr"
	       exit 0
       fi
       if echo "$OUTPUT" | grep -q -E 'xml.*leak|leak.*xml'; then
               mv "$script" "${script}.tr"
	       exit 0
       fi
       if [ $(echo "$OUTPUT" | grep "AddressSanitizer failed to allocate" | wc -l) -gt 0 ]; then
               mv "$script" "${script}.tr"
	       exit 0
       fi
       if [ $(echo "$OUTPUT" | grep ": Assertion " | wc -l) -gt 0 ]; then
               mv "$script" "${script}.tr"
	       exit 0
       fi
       mv "$script" "${script}.er"
       chmod 440 "${script}.er"
       echo "$OUTPUT" > san.log
elif [ $(echo "$OUTPUT" | grep "runtime error:" | wc -l) -gt 0 ]; then
       mv "$script" "${script}.er"
       echo "$OUTPUT" > san.log
       exit 0
fi
