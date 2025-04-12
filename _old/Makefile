BUILD := build

build:
	[ -d $(BUILD) ] && rm -rv $(BUILD)
	jpm build
	$(CC) $(BUILD)/acr2html.c -o $(BUILD)/acr2html -ljanet

run: build
	./build/acr2html test.acr -v -o out/test.html -k katex/

.PHONY: build run
