#include "pak_file_io.h"

PakFile readFile(const char *fileName) {
    PakFile file;
    /* declare a file pointer */
    FILE *filePtr = fopen(fileName, "rb");
    /* quit if the file does not exist */
    if (filePtr == NULL)
        return NULL_File;

    /* Get the number of bytes */
    fseeko(filePtr, 0, SEEK_END);
    file.size = ftello(filePtr);

    /* reset the file position indicator to
    the beginning of the file */
    fseeko(filePtr, 0, SEEK_SET);

    /* grab sufficient memory for the
    buffer to hold the text */
    file.buffer = calloc(file.size + 1, sizeof(char));

    /* memory error */
    if (file.buffer == NULL)
        return NULL_File;

    /* copy all the text into the buffer */
    fread(file.buffer, sizeof(char), file.size, filePtr);
    fclose(filePtr);

    /* free the memory we used for the buffer */
    // free(buffer);
    return file;
}

bool writeFile(const char *fileName, const PakFile file) {
    FILE *filePtr = fopen(fileName, "wb");
    if (filePtr == NULL)
        return false;
    const size_t result = fwrite(file.buffer, sizeof(char), file.size, filePtr);
    fclose(filePtr);
    return (result == file.size);
}
