.PHONY: fuzz
fuzz:
	(cd fuzz && cargo fuzz run unpickle -- -max_len=4096 -timeout=10 -max_total_time=1200)
