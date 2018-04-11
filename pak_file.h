#pragma once
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "pak_defs.h"
#include "pak_header.h"
PakFile pakPackFiles(MyPakHeader* myHeader, PakFile* pakResFile, PakAlias* pakAlias);
PakFile pakGetFile(void* pakBuffer, uint16_t id);
PakFile* pakGetFiles(void* buffer);
