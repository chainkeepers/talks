help:
	@echo "Usage:"
	@echo "    prepare    downloads 3rd party libraries"
	@echo "    build      builds Data Server"
	@echo "    help       see help for more information"

prepare:
	cargo fetch

build: prepare
	cargo build --locked

.PHONY: help prepare build

