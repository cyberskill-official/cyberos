SHELL := /bin/bash

.PHONY: test package

test:
	@bash tests/authbridge_shape_test.sh

package:
	@mkdir -p dist
	@tar --sort=name --mtime='2000-01-01' --owner=0 --group=0 --numeric-owner \
		-czf dist/cyberos-authbridge.tar.gz plugin.json main.go jwks_cache.go jit_provision.go
	@echo "wrote dist/cyberos-authbridge.tar.gz"
