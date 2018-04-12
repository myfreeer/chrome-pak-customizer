#include "pak_get_file_type.h"

static const FileType FILE_TYPES[] = {
    {".png", "\x89\x50\x4E\x47\x0D\x0A\x1A\x0A",
     sizeof("\x89\x50\x4E\x47\x0D\x0A\x1A\x0A") - 1},
    {".html", "<!doctype html>", sizeof("<!doctype html>") - 1},
    {".html", "<html>", sizeof("<html>") - 1},
    {".html", "<link", sizeof("<link") - 1},
    {".js", "// ", sizeof("// ") - 1},
    {".css", "/*", sizeof("/*") - 1},
    {".json", "{", sizeof("{") - 1}};

static const unsigned int FILE_TYPE_COUNT =
    sizeof(FILE_TYPES) / sizeof(FileType);

char *pakGetFileType(PakFile file) {
    for (unsigned int i = 0; i < FILE_TYPE_COUNT; i++)
        if (file.size > FILE_TYPES[i].size &&
            memcmp(file.buffer, FILE_TYPES[i].identifer, FILE_TYPES[i].size) ==
                0)
            return FILE_TYPES[i].type;
    return "";
}
