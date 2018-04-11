#pragma once
#ifndef __PAK_FILE_IO_H__
#define __PAK_FILE_IO_H__

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include "pak_defs.h"


PakFile readFile(const char *fileName);
bool writeFile(const char *fileName, const PakFile file);

#endif // __PAK_FILE_IO_H__
