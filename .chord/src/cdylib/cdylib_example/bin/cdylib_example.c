#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char * run(char * req){
    char * result;
    printf("cdylib_example run req: %s\n", req);
    result = malloc(sizeof(char) * 10);
    strcpy(result, "{\"run\": 1}");
    return result;
}

