def collect_cov(self, fuzztime):
        def run_coverage_collection():
            #os.system("python3 bot.py")
            os.chdir(self.php_root)
            cmd = f"gcovr -sr . -o /tmp/gcovr-{fuzztime}.xml --xml --exclude-directories 'ext/date/lib$$' -e 'ext/bcmath/libbcmath/.*' -e 'ext/date/lib/.*' -e 'ext/fileinfo/libmagic/.*' -e 'ext/gd/libgd/.*' -e 'ext/hash/sha3/.*' -e 'ext/mbstring/libmbfl/.*' -e 'ext/pcre/pcre2lib/.*' > /dev/null"
            os.system(cmd)
            os.chdir(self.test_root)
            with open(f"/tmp/gcovr-{fuzztime}.xml", "r") as f:
                x = f.read()
            self.coverage = float(x.split('line-rate="')[1].split('"')[0])
            print(f"Coverage: {self.coverage:.2%}")

        # Create a new thread for running coverage collection
        coverage_thread = threading.Thread(target=run_coverage_collection)
        coverage_thread.start()
