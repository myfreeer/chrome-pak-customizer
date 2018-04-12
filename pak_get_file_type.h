#pragma once
#ifndef __PAK_GET_FILE_TYPE_H__
#define __PAK_GET_FILE_TYPE_H__
#pragma once
#include "pak_defs.h"
#include <stdint.h>
#include <stdlib.h>
#include <strings.h>

typedef struct FileType {
    char *type;
    char *identifer;
    uint8_t size;
} FileType;

char *pakGetFileType(PakFile file);

#endif // __PAK_GET_FILE_TYPE_H__
