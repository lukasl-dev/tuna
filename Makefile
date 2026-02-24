GO ?= go
BINARY ?= tuna
DIST_DIR ?= dist
LDFLAGS ?= -s -w

PLATFORMS ?= \
	linux/amd64 \
	linux/arm64 \
	darwin/amd64 \
	darwin/arm64 \
	windows/amd64 \
	windows/arm64

.PHONY: release clean

release:
	@mkdir -p "$(DIST_DIR)"
	@for target in $(PLATFORMS); do \
		os=$${target%/*}; \
		arch=$${target#*/}; \
		ext=""; \
		if [ "$$os" = "windows" ]; then ext=".exe"; fi; \
		out="$(DIST_DIR)/$(BINARY)-$$os-$$arch$$ext"; \
		echo "Building $$out"; \
		CGO_ENABLED=0 GOOS=$$os GOARCH=$$arch $(GO) build -trimpath -ldflags "$(LDFLAGS)" -o "$$out" ./cmd/tuna || exit $$?; \
	done

clean:
	rm -rf "$(DIST_DIR)"
