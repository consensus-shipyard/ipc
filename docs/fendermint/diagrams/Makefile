MMDS = $(shell find . -type f -name "*.mmd")
PNGS = $(MMDS:.mmd=.png)
MERMAID_CLI_VER=10.6.1

.PHONY: all
all: diagrams

.PHONY: diagrams
diagrams: node_modules $(PNGS)

package.json:
	@echo '{"name": "diagram-builder", "version": "1.0.0", "description": "Mermaid diagram builder", "private": true}' > package.json

node_modules: package.json
	npm cache clean --force 2>/dev/null || true
	npm install @mermaid-js/mermaid-cli@$(MERMAID_CLI_VER)
	touch node_modules

%.png: %.mmd node_modules
	npx mmdc -i $< -o $@ --theme neutral --backgroundColor transparent

.PHONY: clean
clean:
	rm -f *.png
	rm -rf node_modules package.json package-lock.json
