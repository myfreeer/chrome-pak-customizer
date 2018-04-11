#include "pak_defs.h"
#include "pak_header.h"
#include "pak_file.h"
#include "pak_file_io.h"
#include "pak_get_file_type.h"
#define DEMO_FILE "chrome_100_percent.pak"
int main(){
	PakFile pakFile = readFile(DEMO_FILE);
	if (pakFile.buffer == NULL) {
		return 1;
	}
	MyPakHeader myHeader;
	if (!pakParseHeader(pakFile.buffer, &myHeader)) {
		return -1;
	}
	if (!pakCheckFormat(pakFile.buffer, pakFile.size)) {
		return -2;
	}
	PakFile* pakResFile = pakGetFiles(pakFile.buffer);
	if (pakResFile == NULL) {
		return -4;
	}
	PakAlias* pakAlias = (PakAlias*)(pakFile.buffer + myHeader.size + (myHeader.resource_count+1)*PAK_ENTRY_SIZE);
	PakFile pakFileBuffer = pakPackFiles(&myHeader, pakResFile, pakAlias);
	if (pakFileBuffer.buffer == NULL) {
		return -8;
	}
	writeFile("test.pak",pakFileBuffer);
	freeFile(pakFile);
	free(pakResFile);
	freeFile(pakFileBuffer);
	return 0;
}
