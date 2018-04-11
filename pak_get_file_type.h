#pragma once
#include <stdint.h>
#include <stdlib.h>
#include <strings.h>
#include "pak_defs.h"

typedef struct FileType {
    char* type;
    char* identifer;
    uint8_t size;
} FileType;

char* pakGetFileType(PakFile file);
