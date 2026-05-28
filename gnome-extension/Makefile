# KeyStats GNOME Shell Extension — Build & Install
# Usage:
#   make mo       — compile .po → .mo language packs
#   make zip      — package extension as zip (includes locale)
#   make install  — install extension to local GNOME Shell
#   make pot      — extract translatable strings from source
#   make po       — update .po files with new strings
#   make clean    — remove generated files

EXTENSION_DIR := $(shell pwd)
PO_DIR        := $(EXTENSION_DIR)/po
LOCALE_DIR    := $(EXTENSION_DIR)/locale
DOMAIN        := keystats
POT_FILE      := $(PO_DIR)/$(DOMAIN).pot
SOURCES       := extension.js prefs.js
INSTALL_DIR   := $(HOME)/.local/share/gnome-shell/extensions/keystats@0x5c0f.github.io
ZIP_FILE      := keystats@0x5c0f.github.io.zip
ZIP_SOURCES   := metadata.json extension.js prefs.js stylesheet.css schemas/ locale/

LANGS := zh_CN

.PHONY: all mo zip install pot po clean

all: mo

# ── Language pack build ────────────────────────────────

mo: $(addprefix mo-,$(LANGS))

mo-%: $(PO_DIR)/%.po
	@mkdir -p $(LOCALE_DIR)/$*/LC_MESSAGES
	msgfmt --check --output-file=$(LOCALE_DIR)/$*/LC_MESSAGES/$(DOMAIN).mo $(PO_DIR)/$*.po
	@echo "Compiled $(LOCALE_DIR)/$*/LC_MESSAGES/$(DOMAIN).mo"

pot:
	@mkdir -p $(PO_DIR)
	xgettext \
		--language=JavaScript \
		--keyword=_ \
		--output=$(POT_FILE) \
		--package-name=$(DOMAIN) \
		--package-version=1 \
		--copyright-holder="KeyStats Contributors" \
		--msgid-bugs-address="https://github.com/0x5c0f/KeyStats.Linux/issues" \
		--from-code=UTF-8 \
		$(SOURCES)
	@echo "Generated $(POT_FILE)"

po: $(addprefix po-update-,$(LANGS))

po-update-%: $(POT_FILE)
	msgmerge --update --backup=none $(PO_DIR)/$*.po $(POT_FILE)
	@echo "Updated $(PO_DIR)/$*.po"

# ── Package & Install ─────────────────────────────────

zip: mo
	@rm -f $(ZIP_FILE)
	zip -r $(ZIP_FILE) $(ZIP_SOURCES)
	@echo "Created $(ZIP_FILE)"

install: mo
	@mkdir -p $(INSTALL_DIR)
	cp metadata.json extension.js prefs.js stylesheet.css $(INSTALL_DIR)/
	cp -r schemas $(INSTALL_DIR)/
	cp -r locale $(INSTALL_DIR)/
	glib-compile-schemas $(INSTALL_DIR)/schemas/
	@echo "Installed extension to $(INSTALL_DIR)"

clean:
	rm -f $(PO_DIR)/*.po~
	rm -f $(POT_FILE)
	rm -rf $(LOCALE_DIR)
	rm -f $(ZIP_FILE)
	@echo "Cleaned generated files"
