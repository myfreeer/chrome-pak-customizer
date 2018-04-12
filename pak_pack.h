#pragma once
#ifdef WIN32
#include <windows.h>
#endif
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>

#include "pak_defs.h"
#include "pak_header.h"
#include "pak_file.h"
#include "pak_get_file_type.h"
#include "pak_file_io.h"
bool pakUnpack(void* buffer, char *outputPath);
PakFile pakPack(PakFile pakIndex, char* path);