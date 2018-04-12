#pragma once
#ifndef __PAK_PACKER_UNPACKER_H__
#define __PAK_PACKER_UNPACKER_H__

#ifdef _WIN32
#include <windows.h>
#else
#include <sys/stat.h>
#include <sys/types.h>
#endif
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "pak_defs.h"
#include "pak_file.h"
#include "pak_file_io.h"
#include "pak_get_file_type.h"
#include "pak_header.h"
bool pakUnpack(void *buffer, char *outputPath);
PakFile pakPack(PakFile pakIndex, char *path);
uint32_t countChar(char *string, uint32_t length, char toCount);

#endif // __PAK_PACKER_UNPACKER_H__
