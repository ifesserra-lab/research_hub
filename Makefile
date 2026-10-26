CARGO := $(HOME)/.cargo/bin/cargo

# Terminals launched from a snap (VS Code, some emulators) export an
# LD_LIBRARY_PATH, XDG_DATA_DIRS and GSETTINGS_SCHEMA_DIR pointing at
# /snap/... libraries and schemas, which collide with the system glibc and GTK:
#   - symbol lookup error: ... undefined symbol: __libc_pthread_init
#   - GLib-GIO-ERROR: Settings schema 'org.gnome.settings-daemon.plugins.xsettings'
#     does not contain a key named 'antialiasing'
# Dropping/resetting these variables before `cargo tauri dev` starts prevents both.
RUN := env -u LD_LIBRARY_PATH -u LD_PRELOAD -u SNAP -u SNAP_NAME \
  -u GSETTINGS_SCHEMA_DIR -u GTK_PATH -u GIO_MODULE_DIR \
  -u GDK_PIXBUF_MODULE_FILE -u GDK_PIXBUF_MODULEDIR \
  -u GTK_EXE_PREFIX -u GTK_DATA_PREFIX \
  XDG_DATA_DIRS=/usr/local/share:/usr/share:/var/lib/snapd/desktop

.PHONY: build dev run test bundle docker-bundle clean

## Install frontend dependencies (Rust deps are fetched by cargo on demand).
build:
	cd frontend && npm install

## Run the desktop app with hot reload.
dev run:
	cd src-tauri && $(RUN) $(CARGO) tauri dev

## Full suite: Rust core + frontend bridge.
test:
	cd src-tauri && $(RUN) $(CARGO) test
	cd frontend && npm test

## Native installers for the current platform.
bundle:
	cd src-tauri && $(RUN) $(CARGO) tauri build

## Reproducible Linux installers, built in a container (no host toolchain).
docker-bundle:
	docker build -t research-hub-builder .
	@mkdir -p dist
	@id=$$(docker create research-hub-builder) \
	  && docker cp $$id:/out/. dist/ \
	  && docker rm -v $$id >/dev/null
	@ls -lh dist/

clean:
	rm -rf frontend/node_modules frontend/dist src-tauri/target dist
