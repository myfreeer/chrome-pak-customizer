CC ?= gcc
CFLAGS += -std=c11 -Wall -Wextra -Os -s -flto \
	-fmerge-all-constants \
	-Wl,--gc-sections,--build-id=none -pipe

PAK_SOURCES = pak_header.c pak_file.c pak_file_io.c pak_get_file_type.c pak_pack.c
PAK_HEADERS = pak_header.h pak_file.h pak_file_io.h pak_get_file_type.h pak_pack.h


all: pak

test: pak
	./pak
	cmp chrome_100_percent.pak test.pak

pak: $(PAK_SOURCES) $(PAK_HEADERS) test.c
	$(CC) $(CFLAGS) $(PAK_SOURCES) test.c -o $@

clean:
	-@rm -f pak *.exe *.o

.PHONY: clean all
.SILENT: clean