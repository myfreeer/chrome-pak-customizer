#include "pak_pack.h"
bool pakUnpack(void* buffer, char *outputPath) {
	MyPakHeader myHeader;
	if (!pakParseHeader(buffer, &myHeader)) {
		return false;
	}
	PakFile* files = pakGetFiles(buffer);
	
	char fileNameBuf[FILENAME_MAX];
	memset(fileNameBuf,0,FILENAME_MAX);
	char pathBuf[PATH_MAX];
	memset(pathBuf,0,PATH_MAX);
	
	#ifdef _WIN32
	CreateDirectory(outputPath, NULL);
	#else
	mkdir(outputPath, 0777);
	#endif
	char *pakIndexStr = calloc(PAK_BUFFER_BLOCK_SIZE,sizeof(char));
	uint32_t offset = 0;
	uint32_t length = PAK_BUFFER_BLOCK_SIZE;
	offset += sprintf(pakIndexStr + offset, PAK_INDEX_GLOBAL_TAG "\r\nversion=%u\r\n", myHeader.version);
	offset += sprintf(pakIndexStr + offset, "encoding=%u\r\n\r\n" PAK_INDEX_RES_TAG "\r\n", myHeader.encoding);
    for (uint32_t i = 0; i < myHeader.resource_count; i++) {
       itoa(files->id, fileNameBuf, 10);
       sprintf(fileNameBuf,"%u%s", files->id, pakGetFileType(*files));
       offset += sprintf(pakIndexStr + offset, "%u=%s\r\n", files->id, fileNameBuf);
       if (length - offset < PAK_BUFFER_MIN_FREE_SIZE) {
       	   pakIndexStr = realloc(pakIndexStr, length + PAK_BUFFER_BLOCK_SIZE);
       	   length += PAK_BUFFER_BLOCK_SIZE;
       }
       sprintf(pathBuf, "%s/%s", outputPath, fileNameBuf);
       writeFile(pathBuf, *files);
       files++;
    }
	    PakAlias *aliasBuf=NULL;
    if (myHeader.alias_count > 0) {
		offset += sprintf(pakIndexStr + offset, "\r\n" PAK_INDEX_ALIAS_TAG "\r\n");
		aliasBuf = (PakAlias *) (buffer + myHeader.size + (myHeader.resource_count+1)*PAK_ENTRY_SIZE);
	}
    for (unsigned int i = 0; i < myHeader.alias_count; i++) {
    	offset += sprintf(pakIndexStr + offset, "%u=%u\r\n", aliasBuf->resource_id, aliasBuf->entry_index);
    	aliasBuf++;
       if (length - offset < PAK_BUFFER_MIN_FREE_SIZE) {
       	   pakIndexStr = realloc(pakIndexStr, length + PAK_BUFFER_BLOCK_SIZE);
       	   length += PAK_BUFFER_BLOCK_SIZE;
       }
   }
   PakFile pakIndexBuf;
   //printf(pakIndexStr);
   pakIndexBuf.buffer = pakIndexStr;
   pakIndexBuf.size = offset;
       strcpy(pathBuf, outputPath);
       strcat(pathBuf, "/");
       strcat(pathBuf, "pak_index.ini");
   writeFile(pathBuf,pakIndexBuf);
   freeFile(pakIndexBuf);
   return true;
}

uint32_t countChar(char* string, uint32_t length, char toCount) {
	uint32_t count = 0;
	for (uint32_t i=0; i<length; i++) {
		if (string[i] == toCount) count++;
	}
	return count;
}

PakFile pakPack(PakFile pakIndex, char* path) { //TODO
	MyPakHeader myHeader;
	memset(&myHeader, 0 ,sizeof(myHeader));
	char* pakIndexBuf = pakIndex.buffer;
	
	//if (!sscanf(pakIndexBuf, "version=%u", &myHeader.version))
	//	return NULL_File;
	uint32_t count = 0;
	uint32_t offset = sizeof(PAK_INDEX_GLOBAL_TAG) -1;
	sscanf(pakIndexBuf + offset, " version=%u%n", &myHeader.version, &count);
	offset += count;
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wformat"
#pragma GCC diagnostic ignored "-Wformat-extra-args"
	sscanf(pakIndexBuf + offset, " encoding=%hhu%n", &myHeader.encoding, &count);
#pragma GCC diagnostic pop
//	printf("version=%u\nencoding=%u\n", myHeader.version, myHeader.encoding);
	if (myHeader.version == 5) {
		myHeader.size = PAK_HEADER_SIZE_V5;
	} else if (myHeader.version == 4) {
		myHeader.size = PAK_HEADER_SIZE_V4;
	} else {
        printf(PAK_ERROR_UNKNOWN_VER);
        return NULL_File;
    }
	char *pakIndexEnd = pakIndexBuf + pakIndex.size - 1;
	char* pakEntryIndex = strstr(pakIndexBuf,PAK_INDEX_RES_TAG) + sizeof(PAK_INDEX_RES_TAG) -1;
	char* pakAliasIndex = strstr(pakIndexBuf,PAK_INDEX_ALIAS_TAG);
	if (myHeader.version == 4 || pakAliasIndex == NULL) {
		myHeader.alias_count = 0;
		pakAliasIndex = pakIndexEnd;
	} else {
		pakAliasIndex += sizeof(PAK_INDEX_ALIAS_TAG) -1;
		myHeader.alias_count = countChar(pakAliasIndex, pakIndexEnd-pakAliasIndex, '=');
	}
	myHeader.resource_count = countChar(pakEntryIndex, pakAliasIndex-pakEntryIndex, '=');
	
	// printf("resource_count=%u\nalias_count=%u\n", myHeader.resource_count, myHeader.alias_count);
	
	char fileNameBuf[FILENAME_MAX];
	memset(fileNameBuf,0,FILENAME_MAX);
	char pathBuf[PATH_MAX];
	memset(pathBuf,0,PATH_MAX);
	PakFile* resFiles = calloc(myHeader.resource_count, sizeof(PakFile));
	// TODO: null handling
	uint32_t totalFileSize = 0;
	offset = 0;
	for (uint32_t i=0;i<myHeader.resource_count;i++) {
		uint32_t id=0;
		sscanf(pakEntryIndex + offset, " %u=%s%n ", &id, fileNameBuf, &count);//TODO
		sprintf(pathBuf,"%s%s",path,fileNameBuf);//TODO
		offset += count;
		resFiles[i] = readFile(pathBuf);//TODO
		resFiles[i].id = id;
		totalFileSize += 	resFiles[i].size;
	//	printf("id=%u\tfile_name=%s\tpath=%s\tsize=%u\n",id, fileNameBuf, pathBuf, resFiles[i].size);
	}
	PakAlias* pakAlias = NULL;
	if (myHeader.alias_count > 0) {
		offset = 0;
		pakAlias = calloc(myHeader.alias_count, sizeof(PakAlias)); //TODO
		for (uint32_t i=0;i<myHeader.alias_count;i++) {
			sscanf(pakAliasIndex + offset, " %hu=%hu%n ", &pakAlias[i].resource_id, &pakAlias[i].entry_index, &count);//TODO
			offset += count;
		//	printf("resource_id=%hu\tentry_index=%hu\n",pakAlias[i].resource_id, pakAlias[i].entry_index);
		}
	}
	PakFile pakFile = pakPackFiles(&myHeader, resFiles, pakAlias);
	printf("\nresource_count=%u\nalias_count=%u\n", myHeader.resource_count, myHeader.alias_count);
	printf("version=%u\nencoding=%u\n", myHeader.version, myHeader.encoding);
	printf ("\npak size: %u\n",pakFile.size);
	free(resFiles);
	if (pakAlias != NULL)
		free(pakAlias);
    return pakFile;
}
