build:
	# Right now this is uh... glitchy.
	jpm build || true
	gcc build/acr2html.c -o build/acr2html -ljanet

run: build
	./build/acr2html test.acr -v -o out/test.html -k katex/

.PHONY: build run
