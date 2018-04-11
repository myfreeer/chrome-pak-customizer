CC ?= gcc
CFLAGS += -Wall -Wextra -Os -s -flto \
	-fmerge-all-constants \
	-Wl,--gc-sections,--build-id=none -pipe

OBJS = main.o pak_header.o pak_file.o pak_file_io.o pak_get_file_type.o

all: pakfile

pakfile: $(OBJS)
	$(CC) $(CFLAGS) $(OBJS) -o $@

test: pakfile
	./pakfile
	cmp chrome_100_percent.pak test.pak

clean:
	-@rm -f pakfile *.exe *.o

.PHONY: clean all
.SILENT: clean