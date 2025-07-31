.PHONY: target dist

SHELL := /bin/bash

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