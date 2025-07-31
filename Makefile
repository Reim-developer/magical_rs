.PHONY: target dist

SHELL := /bin/bash
CHECKS_FILE = "checks"

test-core:
	@$(MAKE) -C scripts test

build-core:
	@$(MAKE) -C scripts build-core-debug

dev-gui:
	@$(MAKE) -C venus_gui run-dev 

run-dev:
	@$(MAKE) build-core
	@$(MAKE) dev-gui

pre-push:
	@$(MAKE) test-core

	@if test ! -x $(CHECK_FILE); then \
		chmod +x checks; \
		./checks; \
	else \
		./checks; \
	fi