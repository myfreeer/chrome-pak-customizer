CC ?= gcc
CFLAGS += -Wall -Wextra -Os -s -flto \
	-fmerge-all-constants \
	-Wl,--gc-sections,--build-id=none -pipe

PAK_SOURCES = pak_header.c pak_file.c pak_file_io.c pak_get_file_type.c pak_pack.c
PAK_HEADERS = pak_header.h pak_file.h pak_file_io.h pak_get_file_type.h pak_pack.h

all: pak

test: clean pak
	./pak -u test.pak unpacked
	./pak -p unpacked/pak_index.ini result.pak
	cmp test.pak result.pak

pak: $(PAK_SOURCES) $(PAK_HEADERS) main.c
	$(CC) $(CFLAGS) $(PAK_SOURCES) main.c -o $@

clean:
	-@rm -f pak *.exe *.o result.pak
	-@rm -f ./unpacked/*
	-@rm -df ./unpacked

.PHONY: clean all
.SILENT: clean